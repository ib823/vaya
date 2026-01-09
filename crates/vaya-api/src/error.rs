//! API Error types

use std::fmt;

use crate::Response;

/// Result type for API operations
pub type ApiResult<T> = Result<T, ApiError>;

/// API errors
#[derive(Debug, Clone)]
pub enum ApiError {
    // === Client Errors (4xx) ===
    /// Bad request
    BadRequest(String),
    /// Validation error
    ValidationError(Vec<FieldError>),
    /// Unauthorized
    Unauthorized(String),
    /// Forbidden
    Forbidden(String),
    /// Not found
    NotFound(String),
    /// Method not allowed
    MethodNotAllowed(String),
    /// Conflict
    Conflict(String),
    /// Rate limited
    RateLimited { retry_after: u32 },

    // === Server Errors (5xx) ===
    /// Internal server error
    Internal(String),
    /// Service unavailable
    ServiceUnavailable(String),

    // === Domain Errors ===
    /// Search error
    SearchError(String),
    /// Booking error
    BookingError(String),
    /// Payment error
    PaymentError(String),
    /// Pool error
    PoolError(String),
}

#[derive(Debug, Clone)]
pub struct FieldError {
    pub field: String,
    pub code: String,
    pub message: String,
}

impl FieldError {
    pub fn required(field: &str) -> Self {
        Self {
            field: field.into(),
            code: "required".into(),
            message: format!("{} is required", field),
        }
    }

    pub fn invalid(field: &str, message: &str) -> Self {
        Self {
            field: field.into(),
            code: "invalid".into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ApiError::ValidationError(errors) => {
                let msgs: Vec<String> = errors.iter().map(|e| e.message.clone()).collect();
                write!(f, "Validation error: {}", msgs.join(", "))
            }
            ApiError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ApiError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            ApiError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ApiError::MethodNotAllowed(method) => write!(f, "Method not allowed: {}", method),
            ApiError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            ApiError::RateLimited { retry_after } => {
                write!(f, "Rate limited, retry after {} seconds", retry_after)
            }
            ApiError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ApiError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
            ApiError::SearchError(msg) => write!(f, "Search error: {}", msg),
            ApiError::BookingError(msg) => write!(f, "Booking error: {}", msg),
            ApiError::PaymentError(msg) => write!(f, "Payment error: {}", msg),
            ApiError::PoolError(msg) => write!(f, "Pool error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl ApiError {
    /// Create bad request error
    pub fn bad_request(msg: impl Into<String>) -> Self {
        ApiError::BadRequest(msg.into())
    }

    /// Create unauthorized error
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        ApiError::Unauthorized(msg.into())
    }

    /// Create forbidden error
    pub fn forbidden(msg: impl Into<String>) -> Self {
        ApiError::Forbidden(msg.into())
    }

    /// Create not found error
    pub fn not_found(msg: impl Into<String>) -> Self {
        ApiError::NotFound(msg.into())
    }

    /// Create internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        ApiError::Internal(msg.into())
    }

    /// Get HTTP status code
    pub fn status_code(&self) -> u16 {
        match self {
            ApiError::BadRequest(_) => 400,
            ApiError::ValidationError(_) => 400,
            ApiError::Unauthorized(_) => 401,
            ApiError::Forbidden(_) => 403,
            ApiError::NotFound(_) => 404,
            ApiError::MethodNotAllowed(_) => 405,
            ApiError::Conflict(_) => 409,
            ApiError::RateLimited { .. } => 429,
            ApiError::Internal(_) => 500,
            ApiError::ServiceUnavailable(_) => 503,
            ApiError::SearchError(_) => 400,
            ApiError::BookingError(_) => 400,
            ApiError::PaymentError(_) => 400,
            ApiError::PoolError(_) => 400,
        }
    }

    /// Get error code string
    pub fn error_code(&self) -> &'static str {
        match self {
            ApiError::BadRequest(_) => "bad_request",
            ApiError::ValidationError(_) => "validation_error",
            ApiError::Unauthorized(_) => "unauthorized",
            ApiError::Forbidden(_) => "forbidden",
            ApiError::NotFound(_) => "not_found",
            ApiError::MethodNotAllowed(_) => "method_not_allowed",
            ApiError::Conflict(_) => "conflict",
            ApiError::RateLimited { .. } => "rate_limited",
            ApiError::Internal(_) => "internal_error",
            ApiError::ServiceUnavailable(_) => "service_unavailable",
            ApiError::SearchError(_) => "search_error",
            ApiError::BookingError(_) => "booking_error",
            ApiError::PaymentError(_) => "payment_error",
            ApiError::PoolError(_) => "pool_error",
        }
    }

    /// Convert to HTTP response
    pub fn to_response(&self) -> Response {
        let status = self.status_code();
        let error_code = self.error_code();
        let message = self.to_string();

        let mut response = Response::new(status, status_text(status));

        let body = match self {
            ApiError::ValidationError(errors) => {
                let field_errors: Vec<String> = errors
                    .iter()
                    .map(|e| {
                        format!(
                            r#"{{"field":"{}","code":"{}","message":"{}"}}"#,
                            e.field,
                            e.code,
                            escape_json(&e.message)
                        )
                    })
                    .collect();

                format!(
                    r#"{{"error":"{}","message":"{}","errors":[{}]}}"#,
                    error_code,
                    escape_json(&message),
                    field_errors.join(",")
                )
            }
            ApiError::RateLimited { retry_after } => {
                response = response.with_header("Retry-After", retry_after.to_string());
                format!(
                    r#"{{"error":"{}","message":"{}","retry_after":{}}}"#,
                    error_code,
                    escape_json(&message),
                    retry_after
                )
            }
            _ => {
                format!(
                    r#"{{"error":"{}","message":"{}"}}"#,
                    error_code,
                    escape_json(&message)
                )
            }
        };

        response.body = body.into_bytes();
        response
    }

    /// Check if error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            ApiError::Internal(_) | ApiError::ServiceUnavailable(_) | ApiError::RateLimited { .. }
        )
    }
}

/// Get status text for code
fn status_text(code: u16) -> &'static str {
    match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        _ => "Unknown",
    }
}

/// Escape JSON string
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// Conversions from domain errors
impl From<vaya_search::SearchError> for ApiError {
    fn from(e: vaya_search::SearchError) -> Self {
        ApiError::SearchError(e.to_string())
    }
}

impl From<vaya_book::BookError> for ApiError {
    fn from(e: vaya_book::BookError) -> Self {
        ApiError::BookingError(e.to_string())
    }
}

impl From<vaya_pool::PoolError> for ApiError {
    fn from(e: vaya_pool::PoolError) -> Self {
        ApiError::PoolError(e.to_string())
    }
}

impl From<vaya_oracle::OracleError> for ApiError {
    fn from(e: vaya_oracle::OracleError) -> Self {
        ApiError::SearchError(e.to_string())
    }
}

impl From<vaya_auth::AuthError> for ApiError {
    fn from(e: vaya_auth::AuthError) -> Self {
        match e {
            vaya_auth::AuthError::InvalidCredentials => {
                ApiError::Unauthorized("Invalid credentials".into())
            }
            vaya_auth::AuthError::TokenExpired => ApiError::Unauthorized("Token expired".into()),
            vaya_auth::AuthError::PermissionDenied => {
                ApiError::Forbidden("Permission denied".into())
            }
            vaya_auth::AuthError::MissingPermission(perm) => {
                ApiError::Forbidden(format!("Missing permission: {}", perm))
            }
            _ => ApiError::Unauthorized(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(ApiError::BadRequest("test".into()).status_code(), 400);
        assert_eq!(ApiError::Unauthorized("test".into()).status_code(), 401);
        assert_eq!(ApiError::NotFound("test".into()).status_code(), 404);
        assert_eq!(ApiError::Internal("test".into()).status_code(), 500);
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(
            ApiError::BadRequest("test".into()).error_code(),
            "bad_request"
        );
        assert_eq!(
            ApiError::Unauthorized("test".into()).error_code(),
            "unauthorized"
        );
    }

    #[test]
    fn test_error_to_response() {
        let error = ApiError::NotFound("User not found".into());
        let response = error.to_response();

        assert_eq!(response.status, 404);
        let body = response.body_string().unwrap();
        assert!(body.contains("not_found"));
        assert!(body.contains("User not found"));
    }

    #[test]
    fn test_validation_error() {
        let errors = vec![
            FieldError::required("email"),
            FieldError::invalid("password", "Too short"),
        ];
        let error = ApiError::ValidationError(errors);
        let response = error.to_response();

        assert_eq!(response.status, 400);
        let body = response.body_string().unwrap();
        assert!(body.contains("errors"));
        assert!(body.contains("email"));
    }

    #[test]
    fn test_rate_limited_response() {
        let error = ApiError::RateLimited { retry_after: 60 };
        let response = error.to_response();

        assert_eq!(response.status, 429);
        assert_eq!(response.headers.get("retry-after"), Some(&"60".to_string()));
    }

    #[test]
    fn test_is_retriable() {
        assert!(ApiError::Internal("test".into()).is_retriable());
        assert!(ApiError::RateLimited { retry_after: 60 }.is_retriable());
        assert!(!ApiError::NotFound("test".into()).is_retriable());
    }

    #[test]
    fn test_field_error() {
        let err = FieldError::required("email");
        assert_eq!(err.field, "email");
        assert_eq!(err.code, "required");
    }
}
