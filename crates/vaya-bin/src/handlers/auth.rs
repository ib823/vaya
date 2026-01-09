//! Authentication handlers

use vaya_api::{ApiError, ApiResult, FieldError, JsonSerialize, Request, Response};

/// Register a new user
pub fn register(req: &Request) -> ApiResult<Response> {
    let body = req
        .body_string()
        .ok_or(ApiError::BadRequest("Missing request body".into()))?;

    if body.trim().is_empty() {
        return Err(ApiError::BadRequest("Missing request body".into()));
    }

    // Parse registration data (simplified - would use proper JSON parsing)
    let email = extract_field(&body, "email").ok_or(ApiError::ValidationError(vec![
        FieldError::required("email"),
    ]))?;

    let password = extract_field(&body, "password").ok_or(ApiError::ValidationError(vec![
        FieldError::required("password"),
    ]))?;

    // Validate email format
    if !is_valid_email(&email) {
        return Err(ApiError::ValidationError(vec![FieldError::invalid(
            "email",
            "Invalid email format",
        )]));
    }

    // Validate password strength
    if password.len() < 12 {
        return Err(ApiError::ValidationError(vec![FieldError::invalid(
            "password",
            "Password must be at least 12 characters",
        )]));
    }

    // TODO: Create user in database
    let response = AuthResponse {
        user_id: generate_user_id(),
        email: email.clone(),
        access_token: "mock-access-token".into(),
        refresh_token: "mock-refresh-token".into(),
        expires_in: 900,
    };

    let mut resp = Response::created();
    resp.set_json_body(&response);
    Ok(resp)
}

/// Login
pub fn login(req: &Request) -> ApiResult<Response> {
    let body = req
        .body_string()
        .ok_or(ApiError::BadRequest("Missing request body".into()))?;

    if body.trim().is_empty() {
        return Err(ApiError::BadRequest("Missing request body".into()));
    }

    let email = extract_field(&body, "email").ok_or(ApiError::ValidationError(vec![
        FieldError::required("email"),
    ]))?;

    let password = extract_field(&body, "password").ok_or(ApiError::ValidationError(vec![
        FieldError::required("password"),
    ]))?;

    // TODO: Verify credentials
    // For now, just check they're not empty
    if email.is_empty() || password.is_empty() {
        return Err(ApiError::Unauthorized("Invalid credentials".into()));
    }

    let response = AuthResponse {
        user_id: "user-123".into(),
        email,
        access_token: "mock-access-token".into(),
        refresh_token: "mock-refresh-token".into(),
        expires_in: 900,
    };

    let mut resp = Response::ok();
    resp.set_json_body(&response);
    Ok(resp)
}

/// Logout
pub fn logout(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Not logged in".into()));
    }

    // TODO: Invalidate session/token
    Ok(Response::no_content())
}

/// Refresh token
pub fn refresh_token(req: &Request) -> ApiResult<Response> {
    let body = req
        .body_string()
        .ok_or(ApiError::BadRequest("Missing request body".into()))?;

    let refresh_token =
        extract_field(&body, "refresh_token").ok_or(ApiError::ValidationError(vec![
            FieldError::required("refresh_token"),
        ]))?;

    if refresh_token.is_empty() {
        return Err(ApiError::Unauthorized("Invalid refresh token".into()));
    }

    // TODO: Validate and refresh token
    let response = TokenRefreshResponse {
        access_token: "new-access-token".into(),
        expires_in: 900,
    };

    let mut resp = Response::ok();
    resp.set_json_body(&response);
    Ok(resp)
}

/// Auth response
#[derive(Debug, Clone)]
pub struct AuthResponse {
    pub user_id: String,
    pub email: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

impl JsonSerialize for AuthResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"user_id":"{}","email":"{}","access_token":"{}","refresh_token":"{}","expires_in":{}}}"#,
            self.user_id,
            escape_json(&self.email),
            self.access_token,
            self.refresh_token,
            self.expires_in
        )
    }
}

/// Token refresh response
#[derive(Debug, Clone)]
pub struct TokenRefreshResponse {
    pub access_token: String,
    pub expires_in: u64,
}

impl JsonSerialize for TokenRefreshResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"access_token":"{}","expires_in":{}}}"#,
            self.access_token, self.expires_in
        )
    }
}

/// Extract field from JSON body (simplified parsing)
fn extract_field(body: &str, field: &str) -> Option<String> {
    // Look for "field":
    let pattern = format!(r#""{}":"#, field);
    let start = body.find(&pattern)?;
    let rest = &body[start + pattern.len()..];

    // Skip whitespace
    let rest = rest.trim_start();

    // Find the value
    if let Some(rest) = rest.strip_prefix('"') {
        // String value - find closing quote
        // Find unescaped closing quote
        let mut end = 0;
        let chars: Vec<char> = rest.chars().collect();
        while end < chars.len() {
            if chars[end] == '"' {
                break;
            }
            if chars[end] == '\\' && end + 1 < chars.len() {
                end += 2; // Skip escaped character
            } else {
                end += 1;
            }
        }
        if end <= rest.len() {
            Some(rest[..end].to_string())
        } else {
            None
        }
    } else {
        // Non-string value
        let end = rest.find([',', '}']).unwrap_or(rest.len());
        Some(rest[..end].trim().to_string())
    }
}

/// Validate email format (basic)
fn is_valid_email(email: &str) -> bool {
    if email.len() < 5 || email.len() > 254 {
        return false;
    }

    let at_pos = match email.find('@') {
        Some(pos) => pos,
        None => return false,
    };

    let local = &email[..at_pos];
    let domain = &email[at_pos + 1..];

    // Basic validation
    !local.is_empty()
        && !domain.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

/// Generate user ID
fn generate_user_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("usr-{:x}", timestamp)
}

/// Escape JSON string
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_missing_body() {
        let req = Request::new("POST", "/auth/register");
        let result = register(&req);
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_login_missing_body() {
        let req = Request::new("POST", "/auth/login");
        let result = login(&req);
        assert!(matches!(result, Err(ApiError::BadRequest(_))));
    }

    #[test]
    fn test_logout_requires_auth() {
        let req = Request::new("POST", "/auth/logout");
        let result = logout(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_email_validation() {
        assert!(is_valid_email("test@example.com"));
        assert!(is_valid_email("user.name@sub.domain.com"));
        assert!(!is_valid_email("invalid"));
        assert!(!is_valid_email("@nodomain.com"));
        assert!(!is_valid_email("noat.com"));
    }

    #[test]
    fn test_extract_field() {
        let body = r#"{"email":"test@example.com","password":"secret123"}"#;
        assert_eq!(
            extract_field(body, "email"),
            Some("test@example.com".into())
        );
        assert_eq!(extract_field(body, "password"), Some("secret123".into()));
        assert_eq!(extract_field(body, "nonexistent"), None);
    }

    #[test]
    fn test_auth_response_json() {
        let response = AuthResponse {
            user_id: "usr-123".into(),
            email: "test@example.com".into(),
            access_token: "abc".into(),
            refresh_token: "xyz".into(),
            expires_in: 900,
        };
        let json = response.to_json();
        assert!(json.contains(r#""user_id":"usr-123""#));
        assert!(json.contains(r#""expires_in":900"#));
    }
}
