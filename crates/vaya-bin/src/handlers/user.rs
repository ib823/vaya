//! User profile handlers

use vaya_api::{ApiError, ApiResult, FieldError, JsonSerialize, Request, Response};

/// Get user profile
pub fn get_profile(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let user_id = req.user_id.as_ref().unwrap();

    // TODO: Fetch user from database
    let profile = UserProfile {
        id: user_id.clone(),
        email: "user@example.com".into(),
        name: None,
        phone: None,
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: "2026-01-01T00:00:00Z".into(),
    };

    let mut response = Response::ok();
    response.set_json_body(&profile);
    Ok(response)
}

/// Update user profile
pub fn update_profile(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let body = req.body_string().ok_or(ApiError::BadRequest("Missing request body".into()))?;

    // Parse update fields
    let name = extract_field(&body, "name");
    let phone = extract_field(&body, "phone");

    // Validate phone if provided
    if let Some(ref p) = phone {
        if !is_valid_phone(p) {
            return Err(ApiError::ValidationError(vec![FieldError::invalid(
                "phone",
                "Invalid phone number format",
            )]));
        }
    }

    // TODO: Update user in database
    let user_id = req.user_id.as_ref().unwrap();
    let profile = UserProfile {
        id: user_id.clone(),
        email: "user@example.com".into(),
        name,
        phone,
        created_at: "2026-01-01T00:00:00Z".into(),
        updated_at: current_timestamp(),
    };

    let mut response = Response::ok();
    response.set_json_body(&profile);
    Ok(response)
}

/// Change password
pub fn change_password(req: &Request) -> ApiResult<Response> {
    if !req.is_authenticated() {
        return Err(ApiError::Unauthorized("Authentication required".into()));
    }

    let body = req.body_string().ok_or(ApiError::BadRequest("Missing request body".into()))?;

    let current_password = extract_field(&body, "current_password")
        .ok_or(ApiError::ValidationError(vec![FieldError::required("current_password")]))?;

    let new_password = extract_field(&body, "new_password")
        .ok_or(ApiError::ValidationError(vec![FieldError::required("new_password")]))?;

    // Validate new password
    if new_password.len() < 12 {
        return Err(ApiError::ValidationError(vec![FieldError::invalid(
            "new_password",
            "Password must be at least 12 characters",
        )]));
    }

    // TODO: Verify current password and update
    if current_password.is_empty() {
        return Err(ApiError::Unauthorized("Invalid current password".into()));
    }

    let mut response = Response::ok();
    response.body = b"{\"message\":\"Password updated successfully\"}".to_vec();
    Ok(response)
}

/// User profile
#[derive(Debug, Clone)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl JsonSerialize for UserProfile {
    fn to_json(&self) -> String {
        let name = match &self.name {
            Some(n) => format!(r#""{}""#, escape_json(n)),
            None => "null".into(),
        };
        let phone = match &self.phone {
            Some(p) => format!(r#""{}""#, escape_json(p)),
            None => "null".into(),
        };
        format!(
            r#"{{"id":"{}","email":"{}","name":{},"phone":{},"created_at":"{}","updated_at":"{}"}}"#,
            self.id,
            escape_json(&self.email),
            name,
            phone,
            self.created_at,
            self.updated_at
        )
    }
}

/// Extract field from JSON body
fn extract_field(body: &str, field: &str) -> Option<String> {
    let pattern = format!(r#""{}"#, field);
    let start = body.find(&pattern)?;
    let rest = &body[start + pattern.len()..];
    let rest = rest.trim_start_matches(|c: char| c == '"' || c == ':' || c.is_whitespace());

    if rest.starts_with("null") {
        return None;
    }

    if rest.starts_with('"') {
        let rest = &rest[1..];
        let end = rest.find('"')?;
        Some(rest[..end].to_string())
    } else {
        let end = rest.find(|c: char| c == ',' || c == '}').unwrap_or(rest.len());
        Some(rest[..end].trim().to_string())
    }
}

/// Validate phone number (basic)
fn is_valid_phone(phone: &str) -> bool {
    if phone.len() < 7 || phone.len() > 20 {
        return false;
    }

    let chars: Vec<char> = phone.chars().collect();
    let start = if chars.first() == Some(&'+') { 1 } else { 0 };

    chars[start..].iter().all(|c| c.is_ascii_digit() || *c == ' ' || *c == '-')
}

/// Escape JSON string
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Get current timestamp
fn current_timestamp() -> String {
    use time::OffsetDateTime;
    let now = OffsetDateTime::now_utc();
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        now.year(),
        now.month() as u8,
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_profile_requires_auth() {
        let req = Request::new("GET", "/users/me");
        let result = get_profile(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_update_profile_requires_auth() {
        let req = Request::new("PUT", "/users/me");
        let result = update_profile(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_change_password_requires_auth() {
        let req = Request::new("PUT", "/users/me/password");
        let result = change_password(&req);
        assert!(matches!(result, Err(ApiError::Unauthorized(_))));
    }

    #[test]
    fn test_phone_validation() {
        assert!(is_valid_phone("+1234567890"));
        assert!(is_valid_phone("123-456-7890"));
        assert!(is_valid_phone("123 456 7890"));
        assert!(!is_valid_phone("12345")); // Too short
        assert!(!is_valid_phone("abcdefghij")); // Letters
    }

    #[test]
    fn test_user_profile_json() {
        let profile = UserProfile {
            id: "usr-123".into(),
            email: "test@example.com".into(),
            name: Some("John Doe".into()),
            phone: None,
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = profile.to_json();
        assert!(json.contains(r#""email":"test@example.com""#));
        assert!(json.contains(r#""name":"John Doe""#));
        assert!(json.contains(r#""phone":null"#));
    }

    #[test]
    fn test_user_profile_null_name() {
        let profile = UserProfile {
            id: "usr-123".into(),
            email: "test@example.com".into(),
            name: None,
            phone: Some("+1234567890".into()),
            created_at: "2026-01-01T00:00:00Z".into(),
            updated_at: "2026-01-01T00:00:00Z".into(),
        };
        let json = profile.to_json();
        assert!(json.contains(r#""name":null"#));
        assert!(json.contains(r#""phone":"+1234567890""#));
    }
}
