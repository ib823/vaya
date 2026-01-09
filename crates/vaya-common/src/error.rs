//! Error types for VAYA
//!
//! Centralized error handling with error codes for API responses.

use std::fmt;

/// Result type alias using VayaError
pub type Result<T> = std::result::Result<T, VayaError>;

/// Main error type for VAYA
#[derive(Debug)]
pub struct VayaError {
    /// Error code (for API responses)
    pub code: ErrorCode,
    /// Human-readable message
    pub message: String,
    /// Additional context
    pub context: Option<String>,
    /// Source error (if wrapping another error)
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl VayaError {
    /// Create a new error
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: None,
            source: None,
        }
    }

    /// Add context to the error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Add a source error
    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(mut self, source: E) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Get the HTTP status code for this error
    pub fn http_status(&self) -> u16 {
        self.code.http_status()
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        let status = self.http_status();
        (400..500).contains(&status)
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        let status = self.http_status();
        (500..600).contains(&status)
    }

    // Convenience constructors

    /// Creates a NotFound error for the specified resource.
    pub fn not_found(resource: &str) -> Self {
        Self::new(ErrorCode::NotFound, format!("{} not found", resource))
    }

    /// Creates a validation error with the given message.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ValidationFailed, message)
    }

    /// Creates an unauthorized error with the given message.
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unauthorized, message)
    }

    /// Creates a forbidden error with the given message.
    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Forbidden, message)
    }

    /// Creates an internal server error with the given message.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, message)
    }

    /// Creates a rate limited error with a default message.
    pub fn rate_limited() -> Self {
        Self::new(
            ErrorCode::RateLimited,
            "Too many requests, please try again later",
        )
    }

    /// Creates a conflict error with the given message.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Conflict, message)
    }

    /// Creates a bad request error with the given message.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, message)
    }
}

impl fmt::Display for VayaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code.as_str(), self.message)?;
        if let Some(ctx) = &self.context {
            write!(f, " ({})", ctx)?;
        }
        Ok(())
    }
}

impl std::error::Error for VayaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn std::error::Error + 'static))
    }
}

/// Error codes for API responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ErrorCode {
    // 400 Bad Request
    /// Generic bad request error
    BadRequest = 4000,
    /// Validation failed for one or more fields
    ValidationFailed = 4001,
    /// Input value is invalid
    InvalidInput = 4002,
    /// Required field is missing
    MissingField = 4003,
    /// Value format is invalid
    InvalidFormat = 4004,
    /// Date range is invalid (e.g., end before start)
    InvalidDateRange = 4005,
    /// Route is invalid (e.g., same origin/destination)
    InvalidRoute = 4006,
    /// Currency code is invalid
    InvalidCurrency = 4007,
    /// Price value is invalid
    InvalidPrice = 4008,

    // 401 Unauthorized
    /// Authentication required
    Unauthorized = 4010,
    /// Access token is invalid
    InvalidToken = 4011,
    /// Access token has expired
    TokenExpired = 4012,
    /// Username/password incorrect
    InvalidCredentials = 4013,
    /// Multi-factor authentication required
    MfaRequired = 4014,
    /// MFA code is invalid
    InvalidMfaCode = 4015,
    /// Session has expired
    SessionExpired = 4016,

    // 403 Forbidden
    /// Action is not allowed
    Forbidden = 4030,
    /// User lacks required permissions
    InsufficientPermissions = 4031,
    /// User account is suspended
    AccountSuspended = 4032,
    /// Feature is not available
    FeatureNotAvailable = 4033,
    /// Feature requires higher subscription tier
    TierRestricted = 4034,

    // 404 Not Found
    /// Generic resource not found
    NotFound = 4040,
    /// User not found
    UserNotFound = 4041,
    /// Booking not found
    BookingNotFound = 4042,
    /// Pool not found
    PoolNotFound = 4043,
    /// Alert not found
    AlertNotFound = 4044,
    /// Flight not found
    FlightNotFound = 4045,
    /// Offer not found
    OfferNotFound = 4046,

    // 409 Conflict
    /// Generic conflict error
    Conflict = 4090,
    /// Email address already registered
    DuplicateEmail = 4091,
    /// Booking already exists for this offer
    BookingAlreadyExists = 4092,
    /// Pool is already closed
    PoolAlreadyClosed = 4093,
    /// Alert limit reached for tier
    AlertLimitReached = 4094,
    /// Search limit reached for tier
    SearchLimitReached = 4095,

    // 422 Unprocessable Entity
    /// Request understood but cannot be processed
    UnprocessableEntity = 4220,
    /// Pool is not accepting new members
    PoolNotJoinable = 4221,
    /// Booking cannot be cancelled
    BookingNotCancellable = 4222,
    /// Payment was declined
    PaymentDeclined = 4223,
    /// Offer has expired
    OfferExpired = 4224,
    /// Not enough seats available
    InsufficientSeats = 4225,

    // 429 Too Many Requests
    /// Generic rate limit exceeded
    RateLimited = 4290,
    /// Search rate limit exceeded
    SearchRateLimited = 4291,
    /// API rate limit exceeded
    ApiRateLimited = 4292,

    // 500 Internal Server Error
    /// Generic internal server error
    InternalError = 5000,
    /// Database operation failed
    DatabaseError = 5001,
    /// Cache operation failed
    CacheError = 5002,
    /// Cryptographic operation failed
    CryptoError = 5003,
    /// Serialization/deserialization failed
    SerializationError = 5004,
    /// I/O operation failed
    IoError = 5005,

    // 502 Bad Gateway
    /// Upstream service error
    UpstreamError = 5020,
    /// Flight supplier error
    SupplierError = 5021,
    /// Payment gateway error
    PaymentGatewayError = 5022,

    // 503 Service Unavailable
    /// Service is temporarily unavailable
    ServiceUnavailable = 5030,
    /// System is in maintenance mode
    MaintenanceMode = 5031,
    /// Feature is temporarily disabled
    TemporarilyDisabled = 5032,

    // 504 Gateway Timeout
    /// Request timed out
    Timeout = 5040,
    /// Supplier request timed out
    SupplierTimeout = 5041,
}

impl ErrorCode {
    /// Returns the string representation of the error code.
    ///
    /// Used for API responses and logging.
    pub fn as_str(&self) -> &'static str {
        match self {
            // 400
            Self::BadRequest => "BAD_REQUEST",
            Self::ValidationFailed => "VALIDATION_FAILED",
            Self::InvalidInput => "INVALID_INPUT",
            Self::MissingField => "MISSING_FIELD",
            Self::InvalidFormat => "INVALID_FORMAT",
            Self::InvalidDateRange => "INVALID_DATE_RANGE",
            Self::InvalidRoute => "INVALID_ROUTE",
            Self::InvalidCurrency => "INVALID_CURRENCY",
            Self::InvalidPrice => "INVALID_PRICE",

            // 401
            Self::Unauthorized => "UNAUTHORIZED",
            Self::InvalidToken => "INVALID_TOKEN",
            Self::TokenExpired => "TOKEN_EXPIRED",
            Self::InvalidCredentials => "INVALID_CREDENTIALS",
            Self::MfaRequired => "MFA_REQUIRED",
            Self::InvalidMfaCode => "INVALID_MFA_CODE",
            Self::SessionExpired => "SESSION_EXPIRED",

            // 403
            Self::Forbidden => "FORBIDDEN",
            Self::InsufficientPermissions => "INSUFFICIENT_PERMISSIONS",
            Self::AccountSuspended => "ACCOUNT_SUSPENDED",
            Self::FeatureNotAvailable => "FEATURE_NOT_AVAILABLE",
            Self::TierRestricted => "TIER_RESTRICTED",

            // 404
            Self::NotFound => "NOT_FOUND",
            Self::UserNotFound => "USER_NOT_FOUND",
            Self::BookingNotFound => "BOOKING_NOT_FOUND",
            Self::PoolNotFound => "POOL_NOT_FOUND",
            Self::AlertNotFound => "ALERT_NOT_FOUND",
            Self::FlightNotFound => "FLIGHT_NOT_FOUND",
            Self::OfferNotFound => "OFFER_NOT_FOUND",

            // 409
            Self::Conflict => "CONFLICT",
            Self::DuplicateEmail => "DUPLICATE_EMAIL",
            Self::BookingAlreadyExists => "BOOKING_ALREADY_EXISTS",
            Self::PoolAlreadyClosed => "POOL_ALREADY_CLOSED",
            Self::AlertLimitReached => "ALERT_LIMIT_REACHED",
            Self::SearchLimitReached => "SEARCH_LIMIT_REACHED",

            // 422
            Self::UnprocessableEntity => "UNPROCESSABLE_ENTITY",
            Self::PoolNotJoinable => "POOL_NOT_JOINABLE",
            Self::BookingNotCancellable => "BOOKING_NOT_CANCELLABLE",
            Self::PaymentDeclined => "PAYMENT_DECLINED",
            Self::OfferExpired => "OFFER_EXPIRED",
            Self::InsufficientSeats => "INSUFFICIENT_SEATS",

            // 429
            Self::RateLimited => "RATE_LIMITED",
            Self::SearchRateLimited => "SEARCH_RATE_LIMITED",
            Self::ApiRateLimited => "API_RATE_LIMITED",

            // 500
            Self::InternalError => "INTERNAL_ERROR",
            Self::DatabaseError => "DATABASE_ERROR",
            Self::CacheError => "CACHE_ERROR",
            Self::CryptoError => "CRYPTO_ERROR",
            Self::SerializationError => "SERIALIZATION_ERROR",
            Self::IoError => "IO_ERROR",

            // 502
            Self::UpstreamError => "UPSTREAM_ERROR",
            Self::SupplierError => "SUPPLIER_ERROR",
            Self::PaymentGatewayError => "PAYMENT_GATEWAY_ERROR",

            // 503
            Self::ServiceUnavailable => "SERVICE_UNAVAILABLE",
            Self::MaintenanceMode => "MAINTENANCE_MODE",
            Self::TemporarilyDisabled => "TEMPORARILY_DISABLED",

            // 504
            Self::Timeout => "TIMEOUT",
            Self::SupplierTimeout => "SUPPLIER_TIMEOUT",
        }
    }

    /// Returns the HTTP status code for this error code.
    ///
    /// Maps error codes to their corresponding HTTP status (400-504).
    pub fn http_status(&self) -> u16 {
        match self {
            // 400
            Self::BadRequest
            | Self::ValidationFailed
            | Self::InvalidInput
            | Self::MissingField
            | Self::InvalidFormat
            | Self::InvalidDateRange
            | Self::InvalidRoute
            | Self::InvalidCurrency
            | Self::InvalidPrice => 400,

            // 401
            Self::Unauthorized
            | Self::InvalidToken
            | Self::TokenExpired
            | Self::InvalidCredentials
            | Self::MfaRequired
            | Self::InvalidMfaCode
            | Self::SessionExpired => 401,

            // 403
            Self::Forbidden
            | Self::InsufficientPermissions
            | Self::AccountSuspended
            | Self::FeatureNotAvailable
            | Self::TierRestricted => 403,

            // 404
            Self::NotFound
            | Self::UserNotFound
            | Self::BookingNotFound
            | Self::PoolNotFound
            | Self::AlertNotFound
            | Self::FlightNotFound
            | Self::OfferNotFound => 404,

            // 409
            Self::Conflict
            | Self::DuplicateEmail
            | Self::BookingAlreadyExists
            | Self::PoolAlreadyClosed
            | Self::AlertLimitReached
            | Self::SearchLimitReached => 409,

            // 422
            Self::UnprocessableEntity
            | Self::PoolNotJoinable
            | Self::BookingNotCancellable
            | Self::PaymentDeclined
            | Self::OfferExpired
            | Self::InsufficientSeats => 422,

            // 429
            Self::RateLimited | Self::SearchRateLimited | Self::ApiRateLimited => 429,

            // 500
            Self::InternalError
            | Self::DatabaseError
            | Self::CacheError
            | Self::CryptoError
            | Self::SerializationError
            | Self::IoError => 500,

            // 502
            Self::UpstreamError | Self::SupplierError | Self::PaymentGatewayError => 502,

            // 503
            Self::ServiceUnavailable | Self::MaintenanceMode | Self::TemporarilyDisabled => 503,

            // 504
            Self::Timeout | Self::SupplierTimeout => 504,
        }
    }

    /// Returns the raw u16 value of the error code.
    ///
    /// Used for serialization and database storage.
    pub fn as_u16(&self) -> u16 {
        *self as u16
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Implement From for common error types

impl From<std::io::Error> for VayaError {
    fn from(err: std::io::Error) -> Self {
        VayaError::new(ErrorCode::IoError, err.to_string()).with_source(err)
    }
}

impl<C, T> From<rkyv::validation::CheckArchiveError<T, C>> for VayaError
where
    C: std::fmt::Display,
    T: std::fmt::Display,
{
    fn from(err: rkyv::validation::CheckArchiveError<T, C>) -> Self {
        VayaError::new(ErrorCode::SerializationError, err.to_string())
    }
}

/// Field validation error (for form validation)
#[derive(Debug, Clone)]
pub struct FieldError {
    /// Name of the field with the error
    pub field: String,
    /// Error code (e.g., "required", "too_short")
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

impl FieldError {
    /// Creates a new field validation error.
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }

    /// Creates a "required field" error.
    pub fn required(field: &str) -> Self {
        Self::new(field, "required", format!("{} is required", field))
    }

    /// Creates an "invalid field" error with a custom message.
    pub fn invalid(field: &str, message: &str) -> Self {
        Self::new(field, "invalid", message)
    }

    /// Creates a "too short" error for minimum length validation.
    pub fn too_short(field: &str, min: usize) -> Self {
        Self::new(
            field,
            "too_short",
            format!("{} must be at least {} characters", field, min),
        )
    }

    /// Creates a "too long" error for maximum length validation.
    pub fn too_long(field: &str, max: usize) -> Self {
        Self::new(
            field,
            "too_long",
            format!("{} must be at most {} characters", field, max),
        )
    }
}

impl fmt::Display for FieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Validation error with multiple field errors
#[derive(Debug)]
pub struct ValidationError {
    /// Collection of field-level validation errors
    pub errors: Vec<FieldError>,
}

impl ValidationError {
    /// Creates a new empty ValidationError.
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Adds a field error to this validation error.
    pub fn add(&mut self, error: FieldError) {
        self.errors.push(error);
    }

    /// Returns true if there are no validation errors.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Converts this ValidationError into a Result.
    ///
    /// Returns `Ok(value)` if no errors, or `Err(VayaError)` if there are errors.
    pub fn into_result<T>(self, value: T) -> Result<T> {
        if self.is_empty() {
            Ok(value)
        } else {
            Err(self.into())
        }
    }
}

impl Default for ValidationError {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ValidationError> for VayaError {
    fn from(err: ValidationError) -> Self {
        let messages: Vec<String> = err.errors.iter().map(|e| e.message.clone()).collect();
        VayaError::new(ErrorCode::ValidationFailed, messages.join("; "))
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let messages: Vec<String> = self.errors.iter().map(|e| e.to_string()).collect();
        write!(f, "Validation failed: {}", messages.join(", "))
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err =
            VayaError::new(ErrorCode::NotFound, "User not found").with_context("user_id: 123");

        assert_eq!(err.code, ErrorCode::NotFound);
        assert_eq!(err.http_status(), 404);
        assert!(err.message.contains("User not found"));
    }

    #[test]
    fn test_error_code_status() {
        assert_eq!(ErrorCode::BadRequest.http_status(), 400);
        assert_eq!(ErrorCode::Unauthorized.http_status(), 401);
        assert_eq!(ErrorCode::NotFound.http_status(), 404);
        assert_eq!(ErrorCode::InternalError.http_status(), 500);
    }

    #[test]
    fn test_validation_error() {
        let mut validation = ValidationError::new();
        validation.add(FieldError::required("email"));
        validation.add(FieldError::too_short("password", 8));

        assert!(!validation.is_empty());
        assert_eq!(validation.errors.len(), 2);

        let err: VayaError = validation.into();
        assert_eq!(err.code, ErrorCode::ValidationFailed);
    }
}
