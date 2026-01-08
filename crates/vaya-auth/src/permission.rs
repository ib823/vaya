//! Role-based access control (RBAC)

use std::collections::{HashMap, HashSet};

use crate::{AuthError, AuthResult};

/// A permission string (e.g., "users:read", "bookings:write")
pub type Permission = String;

/// A role name (e.g., "admin", "user", "moderator")
pub type RoleName = String;

/// A role with associated permissions
#[derive(Debug, Clone)]
pub struct Role {
    /// Role name
    pub name: RoleName,
    /// Permissions granted to this role
    pub permissions: HashSet<Permission>,
    /// Description
    pub description: Option<String>,
}

impl Role {
    /// Create a new role
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            permissions: HashSet::new(),
            description: None,
        }
    }

    /// Add a permission
    pub fn with_permission(mut self, permission: impl Into<String>) -> Self {
        self.permissions.insert(permission.into());
        self
    }

    /// Add multiple permissions
    pub fn with_permissions(mut self, permissions: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for perm in permissions {
            self.permissions.insert(perm.into());
        }
        self
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Check if role has a permission
    pub fn has_permission(&self, permission: &str) -> bool {
        // Direct match
        if self.permissions.contains(permission) {
            return true;
        }

        // Wildcard match (e.g., "users:*" matches "users:read")
        let parts: Vec<&str> = permission.split(':').collect();
        if parts.len() >= 2 {
            let wildcard = format!("{}:*", parts[0]);
            if self.permissions.contains(&wildcard) {
                return true;
            }
        }

        // Global wildcard
        self.permissions.contains("*")
    }
}

/// RBAC manager
pub struct RbacManager {
    /// All defined roles
    roles: HashMap<RoleName, Role>,
    /// User to roles mapping
    user_roles: HashMap<String, HashSet<RoleName>>,
}

impl RbacManager {
    /// Create a new RBAC manager
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
            user_roles: HashMap::new(),
        }
    }

    /// Create with predefined roles
    pub fn with_default_roles() -> Self {
        let mut manager = Self::new();

        // Admin role - full access
        manager.add_role(
            Role::new("admin")
                .with_permission("*")
                .with_description("Full administrative access"),
        );

        // User role - basic access
        manager.add_role(
            Role::new("user")
                .with_permissions([
                    "profile:read",
                    "profile:write",
                    "bookings:read",
                    "bookings:write",
                    "alerts:read",
                    "alerts:write",
                    "search:read",
                ])
                .with_description("Standard user access"),
        );

        // Premium user role - enhanced access
        manager.add_role(
            Role::new("premium")
                .with_permissions([
                    "profile:read",
                    "profile:write",
                    "bookings:*",
                    "alerts:*",
                    "search:*",
                    "pools:*",
                    "analytics:read",
                ])
                .with_description("Premium user access"),
        );

        // Moderator role
        manager.add_role(
            Role::new("moderator")
                .with_permissions([
                    "users:read",
                    "bookings:read",
                    "reports:read",
                    "reports:write",
                ])
                .with_description("Content moderator access"),
        );

        manager
    }

    /// Add a role
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.name.clone(), role);
    }

    /// Get a role by name
    pub fn get_role(&self, name: &str) -> Option<&Role> {
        self.roles.get(name)
    }

    /// Assign a role to a user
    pub fn assign_role(&mut self, user_id: &str, role_name: &str) -> AuthResult<()> {
        if !self.roles.contains_key(role_name) {
            return Err(AuthError::Internal(format!("Role not found: {}", role_name)));
        }

        self.user_roles
            .entry(user_id.to_string())
            .or_insert_with(HashSet::new)
            .insert(role_name.to_string());

        Ok(())
    }

    /// Remove a role from a user
    pub fn revoke_role(&mut self, user_id: &str, role_name: &str) {
        if let Some(roles) = self.user_roles.get_mut(user_id) {
            roles.remove(role_name);
        }
    }

    /// Get all roles for a user
    pub fn get_user_roles(&self, user_id: &str) -> Vec<&Role> {
        self.user_roles
            .get(user_id)
            .map(|role_names| {
                role_names
                    .iter()
                    .filter_map(|name| self.roles.get(name))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a user has a specific permission
    pub fn has_permission(&self, user_id: &str, permission: &str) -> bool {
        self.get_user_roles(user_id)
            .iter()
            .any(|role| role.has_permission(permission))
    }

    /// Check if a user has any of the specified permissions
    pub fn has_any_permission(&self, user_id: &str, permissions: &[&str]) -> bool {
        permissions.iter().any(|p| self.has_permission(user_id, p))
    }

    /// Check if a user has all of the specified permissions
    pub fn has_all_permissions(&self, user_id: &str, permissions: &[&str]) -> bool {
        permissions.iter().all(|p| self.has_permission(user_id, p))
    }

    /// Require a permission (returns error if not authorized)
    pub fn require_permission(&self, user_id: &str, permission: &str) -> AuthResult<()> {
        if self.has_permission(user_id, permission) {
            Ok(())
        } else {
            Err(AuthError::MissingPermission(permission.to_string()))
        }
    }

    /// Get all permissions for a user (flattened from all roles)
    pub fn get_user_permissions(&self, user_id: &str) -> HashSet<&Permission> {
        self.get_user_roles(user_id)
            .iter()
            .flat_map(|role| role.permissions.iter())
            .collect()
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission guard for request handling
pub struct PermissionGuard<'a> {
    manager: &'a RbacManager,
    user_id: String,
}

impl<'a> PermissionGuard<'a> {
    /// Create a new permission guard
    pub fn new(manager: &'a RbacManager, user_id: impl Into<String>) -> Self {
        Self {
            manager,
            user_id: user_id.into(),
        }
    }

    /// Check a single permission
    pub fn can(&self, permission: &str) -> bool {
        self.manager.has_permission(&self.user_id, permission)
    }

    /// Check any of multiple permissions
    pub fn can_any(&self, permissions: &[&str]) -> bool {
        self.manager.has_any_permission(&self.user_id, permissions)
    }

    /// Check all of multiple permissions
    pub fn can_all(&self, permissions: &[&str]) -> bool {
        self.manager.has_all_permissions(&self.user_id, permissions)
    }

    /// Require a permission
    pub fn require(&self, permission: &str) -> AuthResult<()> {
        self.manager.require_permission(&self.user_id, permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_permissions() {
        let role = Role::new("user")
            .with_permission("profile:read")
            .with_permission("profile:write");

        assert!(role.has_permission("profile:read"));
        assert!(role.has_permission("profile:write"));
        assert!(!role.has_permission("admin:delete"));
    }

    #[test]
    fn test_wildcard_permissions() {
        let role = Role::new("admin").with_permission("users:*");

        assert!(role.has_permission("users:read"));
        assert!(role.has_permission("users:write"));
        assert!(role.has_permission("users:delete"));
        assert!(!role.has_permission("bookings:read"));
    }

    #[test]
    fn test_global_wildcard() {
        let role = Role::new("superadmin").with_permission("*");

        assert!(role.has_permission("anything:here"));
        assert!(role.has_permission("deeply:nested:permission"));
    }

    #[test]
    fn test_rbac_manager() {
        let mut manager = RbacManager::with_default_roles();

        manager.assign_role("user-123", "user").unwrap();
        manager.assign_role("admin-456", "admin").unwrap();

        assert!(manager.has_permission("user-123", "profile:read"));
        assert!(!manager.has_permission("user-123", "users:delete"));

        assert!(manager.has_permission("admin-456", "users:delete"));
        assert!(manager.has_permission("admin-456", "anything:at:all"));
    }

    #[test]
    fn test_require_permission() {
        let mut manager = RbacManager::with_default_roles();
        manager.assign_role("user-123", "user").unwrap();

        assert!(manager.require_permission("user-123", "profile:read").is_ok());
        assert!(manager.require_permission("user-123", "admin:delete").is_err());
    }

    #[test]
    fn test_multiple_roles() {
        let mut manager = RbacManager::with_default_roles();

        manager.assign_role("user-123", "user").unwrap();
        manager.assign_role("user-123", "premium").unwrap();

        // Has permissions from both roles
        assert!(manager.has_permission("user-123", "profile:read")); // from user
        assert!(manager.has_permission("user-123", "analytics:read")); // from premium
    }

    #[test]
    fn test_permission_guard() {
        let mut manager = RbacManager::with_default_roles();
        manager.assign_role("user-123", "user").unwrap();

        let guard = PermissionGuard::new(&manager, "user-123");

        assert!(guard.can("profile:read"));
        assert!(!guard.can("admin:delete"));
        assert!(guard.can_any(&["admin:delete", "profile:read"]));
        assert!(!guard.can_all(&["profile:read", "admin:delete"]));
    }
}
