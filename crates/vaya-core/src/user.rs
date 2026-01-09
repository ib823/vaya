//! User management service

use std::collections::HashMap;
use tracing::{debug, info};

use vaya_auth::{Claims, JwtTokenizer, PasswordHasher};
use vaya_common::{Timestamp, Uuid};

use crate::error::{CoreError, CoreResult};

/// User profile
#[derive(Debug, Clone)]
pub struct User {
    /// Unique user ID
    pub id: String,
    /// Email address
    pub email: String,
    /// First name
    pub first_name: String,
    /// Last name
    pub last_name: String,
    /// Phone number
    pub phone: Option<String>,
    /// Date of birth
    pub date_of_birth: Option<String>,
    /// Nationality
    pub nationality: Option<String>,
    /// Passport number
    pub passport_number: Option<String>,
    /// Passport expiry
    pub passport_expiry: Option<String>,
    /// Preferred currency
    pub preferred_currency: String,
    /// Preferred language
    pub preferred_language: String,
    /// Frequent flyer accounts
    pub frequent_flyer_accounts: Vec<FrequentFlyerAccount>,
    /// Marketing opt-in
    pub marketing_opt_in: bool,
    /// Created at
    pub created_at: Timestamp,
    /// Updated at
    pub updated_at: Timestamp,
    /// Email verified
    pub email_verified: bool,
    /// Phone verified
    pub phone_verified: bool,
    /// Account status
    pub status: UserStatus,
}

/// User status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserStatus {
    /// Active account
    Active,
    /// Suspended
    Suspended,
    /// Pending verification
    PendingVerification,
    /// Deleted
    Deleted,
}

/// Frequent flyer account
#[derive(Debug, Clone)]
pub struct FrequentFlyerAccount {
    /// Airline code
    pub airline: String,
    /// Membership number
    pub number: String,
    /// Tier level
    pub tier: Option<String>,
}

/// User registration request
#[derive(Debug, Clone)]
pub struct RegisterRequest {
    /// Email
    pub email: String,
    /// Password
    pub password: String,
    /// First name
    pub first_name: String,
    /// Last name
    pub last_name: String,
    /// Phone (optional)
    pub phone: Option<String>,
    /// Marketing opt-in
    pub marketing_opt_in: bool,
}

impl RegisterRequest {
    /// Validate registration request
    pub fn validate(&self) -> CoreResult<()> {
        if self.email.is_empty() || !self.email.contains('@') {
            return Err(CoreError::InvalidUserData("Invalid email address".to_string()));
        }
        if self.password.len() < 8 {
            return Err(CoreError::InvalidUserData(
                "Password must be at least 8 characters".to_string(),
            ));
        }
        if self.first_name.is_empty() {
            return Err(CoreError::MissingField("first_name".to_string()));
        }
        if self.last_name.is_empty() {
            return Err(CoreError::MissingField("last_name".to_string()));
        }
        Ok(())
    }
}

/// Login request
#[derive(Debug, Clone)]
pub struct LoginRequest {
    /// Email
    pub email: String,
    /// Password
    pub password: String,
}

/// Authentication response
#[derive(Debug, Clone)]
pub struct AuthResponse {
    /// Access token
    pub access_token: String,
    /// Refresh token
    pub refresh_token: String,
    /// Token type (Bearer)
    pub token_type: String,
    /// Expiry in seconds
    pub expires_in: u64,
    /// User profile
    pub user: User,
}

/// Auth configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT secret key
    pub jwt_secret: String,
    /// Token issuer
    pub issuer: String,
    /// Access token expiry in seconds
    pub access_token_expiry_secs: u64,
    /// Refresh token expiry in seconds
    pub refresh_token_expiry_secs: u64,
}

impl AuthConfig {
    /// Create new auth config with secret
    pub fn new(jwt_secret: &str) -> Self {
        Self {
            jwt_secret: jwt_secret.to_string(),
            issuer: "vaya".to_string(),
            access_token_expiry_secs: 3600,
            refresh_token_expiry_secs: 86400 * 7,
        }
    }
}

/// User service
pub struct UserService {
    /// JWT tokenizer
    tokenizer: JwtTokenizer,
    /// Password hasher
    hasher: PasswordHasher,
    /// In-memory user store (would be database in production)
    users: std::sync::RwLock<HashMap<String, StoredUser>>,
}

/// Stored user with password hash
struct StoredUser {
    user: User,
    password_hash: String,
}

impl UserService {
    /// Create new user service
    pub fn new(config: AuthConfig) -> Self {
        Self {
            tokenizer: JwtTokenizer::new(config.jwt_secret.as_bytes(), &config.issuer),
            hasher: PasswordHasher::new(),
            users: std::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Register a new user
    pub async fn register(&self, request: RegisterRequest) -> CoreResult<AuthResponse> {
        request.validate()?;

        info!("Registering new user: {}", request.email);

        // Check if email already exists
        {
            let users = self.users.read().unwrap();
            if users.values().any(|u| u.user.email == request.email) {
                return Err(CoreError::InvalidUserData(
                    "Email already registered".to_string(),
                ));
            }
        }

        // Hash password
        let password_hash = self
            .hasher
            .hash(&request.password)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        // Create user
        let user_id = Uuid::new_v4().to_string();
        let now = Timestamp::now();

        let user = User {
            id: user_id.clone(),
            email: request.email.clone(),
            first_name: request.first_name,
            last_name: request.last_name,
            phone: request.phone,
            date_of_birth: None,
            nationality: None,
            passport_number: None,
            passport_expiry: None,
            preferred_currency: "MYR".to_string(),
            preferred_language: "en".to_string(),
            frequent_flyer_accounts: vec![],
            marketing_opt_in: request.marketing_opt_in,
            created_at: now,
            updated_at: now,
            email_verified: false,
            phone_verified: false,
            status: UserStatus::PendingVerification,
        };

        // Store user
        {
            let mut users = self.users.write().unwrap();
            users.insert(
                user_id.clone(),
                StoredUser {
                    user: user.clone(),
                    password_hash,
                },
            );
        }

        // Generate tokens
        let (access_token, refresh_token) = self.generate_tokens(&user)?;

        debug!("User registered: {}", user_id);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user,
        })
    }

    /// Login user
    pub async fn login(&self, request: LoginRequest) -> CoreResult<AuthResponse> {
        info!("Login attempt: {}", request.email);

        // Find user by email
        let stored = {
            let users = self.users.read().unwrap();
            users
                .values()
                .find(|u| u.user.email == request.email)
                .cloned()
        };

        let stored = stored.ok_or(CoreError::NotAuthenticated)?;

        // Verify password
        let valid = self
            .hasher
            .verify(&request.password, &stored.password_hash)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        if !valid {
            return Err(CoreError::NotAuthenticated);
        }

        // Check account status
        if stored.user.status != UserStatus::Active
            && stored.user.status != UserStatus::PendingVerification
        {
            return Err(CoreError::NotAuthorized(
                "Account is suspended".to_string(),
            ));
        }

        // Generate tokens
        let (access_token, refresh_token) = self.generate_tokens(&stored.user)?;

        debug!("User logged in: {}", stored.user.id);

        Ok(AuthResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user: stored.user,
        })
    }

    /// Refresh access token
    pub async fn refresh_token(&self, refresh_token: &str) -> CoreResult<AuthResponse> {
        // Validate refresh token
        let claims = self
            .tokenizer
            .validate(refresh_token)
            .map_err(|_| CoreError::NotAuthenticated)?;

        // Get user
        let user = self.get_user(&claims.sub).await?;

        // Generate new tokens
        let (access_token, new_refresh) = self.generate_tokens(&user)?;

        Ok(AuthResponse {
            access_token,
            refresh_token: new_refresh,
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            user,
        })
    }

    /// Get user by ID
    pub async fn get_user(&self, user_id: &str) -> CoreResult<User> {
        let users = self.users.read().unwrap();
        users
            .get(user_id)
            .map(|u| u.user.clone())
            .ok_or_else(|| CoreError::UserNotFound(user_id.to_string()))
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: &str,
        updates: ProfileUpdate,
    ) -> CoreResult<User> {
        let mut users = self.users.write().unwrap();
        let stored = users
            .get_mut(user_id)
            .ok_or_else(|| CoreError::UserNotFound(user_id.to_string()))?;

        // Apply updates
        if let Some(first_name) = updates.first_name {
            stored.user.first_name = first_name;
        }
        if let Some(last_name) = updates.last_name {
            stored.user.last_name = last_name;
        }
        if let Some(phone) = updates.phone {
            stored.user.phone = Some(phone);
        }
        if let Some(dob) = updates.date_of_birth {
            stored.user.date_of_birth = Some(dob);
        }
        if let Some(nationality) = updates.nationality {
            stored.user.nationality = Some(nationality);
        }
        if let Some(currency) = updates.preferred_currency {
            stored.user.preferred_currency = currency;
        }
        if let Some(lang) = updates.preferred_language {
            stored.user.preferred_language = lang;
        }
        if let Some(opt_in) = updates.marketing_opt_in {
            stored.user.marketing_opt_in = opt_in;
        }

        stored.user.updated_at = Timestamp::now();

        debug!("Updated profile for user {}", user_id);

        Ok(stored.user.clone())
    }

    /// Change password
    pub async fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> CoreResult<()> {
        if new_password.len() < 8 {
            return Err(CoreError::InvalidUserData(
                "Password must be at least 8 characters".to_string(),
            ));
        }

        let mut users = self.users.write().unwrap();
        let stored = users
            .get_mut(user_id)
            .ok_or_else(|| CoreError::UserNotFound(user_id.to_string()))?;

        // Verify old password
        let valid = self
            .hasher
            .verify(old_password, &stored.password_hash)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        if !valid {
            return Err(CoreError::NotAuthenticated);
        }

        // Update password
        stored.password_hash = self
            .hasher
            .hash(new_password)
            .map_err(|e| CoreError::Internal(e.to_string()))?;
        stored.user.updated_at = Timestamp::now();

        info!("Password changed for user {}", user_id);

        Ok(())
    }

    /// Verify token and get claims
    pub fn verify_token(&self, token: &str) -> CoreResult<Claims> {
        self.tokenizer
            .validate(token)
            .map_err(|_| CoreError::NotAuthenticated)
    }

    /// Generate access and refresh tokens
    fn generate_tokens(&self, user: &User) -> CoreResult<(String, String)> {
        let access_token = self
            .tokenizer
            .generate(&user.id)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        let refresh_token = self
            .tokenizer
            .generate(&user.id)
            .map_err(|e| CoreError::Internal(e.to_string()))?;

        Ok((access_token, refresh_token))
    }
}

/// Profile update request
#[derive(Debug, Clone)]
pub struct ProfileUpdate {
    /// First name
    pub first_name: Option<String>,
    /// Last name
    pub last_name: Option<String>,
    /// Phone
    pub phone: Option<String>,
    /// Date of birth
    pub date_of_birth: Option<String>,
    /// Nationality
    pub nationality: Option<String>,
    /// Preferred currency
    pub preferred_currency: Option<String>,
    /// Preferred language
    pub preferred_language: Option<String>,
    /// Marketing opt-in
    pub marketing_opt_in: Option<bool>,
}

// Clone for StoredUser
impl Clone for StoredUser {
    fn clone(&self) -> Self {
        Self {
            user: self.user.clone(),
            password_hash: self.password_hash.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_auth_config() -> AuthConfig {
        AuthConfig::new("test-secret-key-32-bytes-long!!")
    }

    #[test]
    fn test_register_validation() {
        let invalid_email = RegisterRequest {
            email: "invalid".to_string(),
            password: "StrongP@ssw0rd!123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            phone: None,
            marketing_opt_in: false,
        };
        assert!(invalid_email.validate().is_err());

        let short_password = RegisterRequest {
            email: "john@example.com".to_string(),
            password: "short".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            phone: None,
            marketing_opt_in: false,
        };
        assert!(short_password.validate().is_err());

        let valid = RegisterRequest {
            email: "john@example.com".to_string(),
            password: "StrongP@ssw0rd!123".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            phone: None,
            marketing_opt_in: false,
        };
        assert!(valid.validate().is_ok());
    }

    #[tokio::test]
    async fn test_register_and_login() {
        let service = UserService::new(test_auth_config());

        // Register
        let register = RegisterRequest {
            email: "test@example.com".to_string(),
            password: "StrongP@ssw0rd!123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            phone: None,
            marketing_opt_in: true,
        };

        let response = service.register(register).await.unwrap();
        assert!(!response.access_token.is_empty());
        assert_eq!(response.user.email, "test@example.com");

        // Login
        let login = LoginRequest {
            email: "test@example.com".to_string(),
            password: "StrongP@ssw0rd!123".to_string(),
        };

        let login_response = service.login(login).await.unwrap();
        assert!(!login_response.access_token.is_empty());
    }

    #[tokio::test]
    async fn test_update_profile() {
        let service = UserService::new(test_auth_config());

        // Register user first
        let register = RegisterRequest {
            email: "update@example.com".to_string(),
            password: "StrongP@ssw0rd!123".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            phone: None,
            marketing_opt_in: false,
        };

        let response = service.register(register).await.unwrap();
        let user_id = response.user.id;

        // Update profile
        let updates = ProfileUpdate {
            first_name: Some("Updated".to_string()),
            last_name: None,
            phone: Some("+60123456789".to_string()),
            date_of_birth: None,
            nationality: None,
            preferred_currency: Some("SGD".to_string()),
            preferred_language: None,
            marketing_opt_in: Some(true),
        };

        let updated = service.update_profile(&user_id, updates).await.unwrap();
        assert_eq!(updated.first_name, "Updated");
        assert_eq!(updated.phone, Some("+60123456789".to_string()));
        assert_eq!(updated.preferred_currency, "SGD");
        assert!(updated.marketing_opt_in);
    }
}
