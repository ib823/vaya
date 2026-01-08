//! Authentication, authorization, and session management
//!
//! This crate provides:
//! - Password hashing with PBKDF2-SHA256
//! - JWT token generation and validation
//! - Session management with in-memory store
//! - Role-based access control (RBAC)
//!
//! # Example
//!
//! ```
//! use vaya_auth::{PasswordHasher, JwtTokenizer, RbacManager};
//! use time::Duration;
//!
//! // Hash passwords
//! let hasher = PasswordHasher::new();
//! let hash = hasher.hash("SecurePassword123").unwrap();
//! assert!(hasher.verify("SecurePassword123", &hash).unwrap());
//!
//! // Generate tokens
//! let tokenizer = JwtTokenizer::new(b"secret-key", "vaya");
//! let token = tokenizer.generate("user-123").unwrap();
//! let claims = tokenizer.validate(&token).unwrap();
//!
//! // Check permissions
//! let mut rbac = RbacManager::with_default_roles();
//! rbac.assign_role("user-123", "user").unwrap();
//! assert!(rbac.has_permission("user-123", "profile:read"));
//! ```

pub mod error;
pub mod password;
pub mod permission;
pub mod session;
pub mod token;

pub use error::{AuthError, AuthResult};
pub use password::PasswordHasher;
pub use permission::{Permission, PermissionGuard, RbacManager, Role, RoleName};
pub use session::{Session, SessionConfig, SessionStore};
pub use token::{Claims, JwtTokenizer};
