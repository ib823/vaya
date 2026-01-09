//! Error types for vaya-core

use std::fmt;

/// Result type for core operations
pub type CoreResult<T> = Result<T, CoreError>;

/// Errors that can occur in core business logic
#[derive(Debug)]
pub enum CoreError {
    // === Search Errors ===
    /// No flights found for search criteria
    NoFlightsFound { origin: String, destination: String },
    /// Search timeout
    SearchTimeout,
    /// Invalid search parameters
    InvalidSearchParams(String),

    // === Booking Errors ===
    /// Booking not found
    BookingNotFound(String),
    /// Booking already exists
    BookingAlreadyExists(String),
    /// Booking cannot be modified
    BookingNotModifiable(String),
    /// Booking expired
    BookingExpired(String),
    /// Fare no longer available
    FareNotAvailable(String),
    /// Price changed
    PriceChanged { expected: i64, actual: i64 },
    /// Insufficient seats
    InsufficientSeats { requested: u8, available: u8 },

    // === User Errors ===
    /// User not found
    UserNotFound(String),
    /// User not authenticated
    NotAuthenticated,
    /// User not authorized
    NotAuthorized(String),
    /// Invalid user data
    InvalidUserData(String),

    // === Payment Errors ===
    /// Payment failed
    PaymentFailed(String),
    /// Payment not found
    PaymentNotFound(String),
    /// Refund failed
    RefundFailed(String),

    // === Notification Errors ===
    /// Notification failed
    NotificationFailed(String),

    // === Validation Errors ===
    /// Validation error
    ValidationError(String),
    /// Missing required field
    MissingField(String),

    // === System Errors ===
    /// Database error
    Database(String),
    /// Cache error
    Cache(String),
    /// GDS provider error
    GdsError(String),
    /// Internal error
    Internal(String),
    /// Service unavailable
    ServiceUnavailable(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Search
            CoreError::NoFlightsFound {
                origin,
                destination,
            } => {
                write!(f, "No flights found from {} to {}", origin, destination)
            }
            CoreError::SearchTimeout => write!(f, "Search timed out"),
            CoreError::InvalidSearchParams(msg) => write!(f, "Invalid search parameters: {}", msg),

            // Booking
            CoreError::BookingNotFound(id) => write!(f, "Booking not found: {}", id),
            CoreError::BookingAlreadyExists(id) => write!(f, "Booking already exists: {}", id),
            CoreError::BookingNotModifiable(id) => write!(f, "Booking cannot be modified: {}", id),
            CoreError::BookingExpired(id) => write!(f, "Booking has expired: {}", id),
            CoreError::FareNotAvailable(msg) => write!(f, "Fare no longer available: {}", msg),
            CoreError::PriceChanged { expected, actual } => {
                write!(f, "Price changed from {} to {}", expected, actual)
            }
            CoreError::InsufficientSeats {
                requested,
                available,
            } => {
                write!(
                    f,
                    "Requested {} seats but only {} available",
                    requested, available
                )
            }

            // User
            CoreError::UserNotFound(id) => write!(f, "User not found: {}", id),
            CoreError::NotAuthenticated => write!(f, "Authentication required"),
            CoreError::NotAuthorized(msg) => write!(f, "Not authorized: {}", msg),
            CoreError::InvalidUserData(msg) => write!(f, "Invalid user data: {}", msg),

            // Payment
            CoreError::PaymentFailed(msg) => write!(f, "Payment failed: {}", msg),
            CoreError::PaymentNotFound(id) => write!(f, "Payment not found: {}", id),
            CoreError::RefundFailed(msg) => write!(f, "Refund failed: {}", msg),

            // Notification
            CoreError::NotificationFailed(msg) => write!(f, "Notification failed: {}", msg),

            // Validation
            CoreError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            CoreError::MissingField(field) => write!(f, "Missing required field: {}", field),

            // System
            CoreError::Database(msg) => write!(f, "Database error: {}", msg),
            CoreError::Cache(msg) => write!(f, "Cache error: {}", msg),
            CoreError::GdsError(msg) => write!(f, "GDS error: {}", msg),
            CoreError::Internal(msg) => write!(f, "Internal error: {}", msg),
            CoreError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
        }
    }
}

impl std::error::Error for CoreError {}

impl CoreError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CoreError::SearchTimeout | CoreError::ServiceUnavailable(_) | CoreError::GdsError(_)
        )
    }

    /// Check if error is user-facing (should show detailed message)
    pub fn is_user_facing(&self) -> bool {
        matches!(
            self,
            CoreError::NoFlightsFound { .. }
                | CoreError::FareNotAvailable(_)
                | CoreError::PriceChanged { .. }
                | CoreError::InsufficientSeats { .. }
                | CoreError::ValidationError(_)
                | CoreError::MissingField(_)
                | CoreError::NotAuthenticated
                | CoreError::NotAuthorized(_)
        )
    }

    /// Get HTTP status code for API responses
    pub fn http_status_code(&self) -> u16 {
        match self {
            CoreError::NotAuthenticated => 401,
            CoreError::NotAuthorized(_) => 403,
            CoreError::BookingNotFound(_)
            | CoreError::UserNotFound(_)
            | CoreError::PaymentNotFound(_)
            | CoreError::NoFlightsFound { .. } => 404,
            CoreError::BookingAlreadyExists(_) => 409,
            CoreError::ValidationError(_)
            | CoreError::MissingField(_)
            | CoreError::InvalidSearchParams(_)
            | CoreError::InvalidUserData(_) => 400,
            CoreError::PriceChanged { .. }
            | CoreError::FareNotAvailable(_)
            | CoreError::InsufficientSeats { .. } => 409,
            CoreError::ServiceUnavailable(_) | CoreError::SearchTimeout => 503,
            _ => 500,
        }
    }
}

// Error conversions from underlying crates
impl From<vaya_gds::GdsError> for CoreError {
    fn from(e: vaya_gds::GdsError) -> Self {
        CoreError::GdsError(e.to_string())
    }
}

impl From<vaya_payment::PaymentError> for CoreError {
    fn from(e: vaya_payment::PaymentError) -> Self {
        CoreError::PaymentFailed(e.to_string())
    }
}

impl From<vaya_notification::NotificationError> for CoreError {
    fn from(e: vaya_notification::NotificationError) -> Self {
        CoreError::NotificationFailed(e.to_string())
    }
}

impl From<vaya_auth::AuthError> for CoreError {
    fn from(e: vaya_auth::AuthError) -> Self {
        match e {
            vaya_auth::AuthError::InvalidToken(_)
            | vaya_auth::AuthError::TokenExpired
            | vaya_auth::AuthError::SignatureInvalid
            | vaya_auth::AuthError::InvalidCredentials => CoreError::NotAuthenticated,
            vaya_auth::AuthError::PermissionDenied
            | vaya_auth::AuthError::MissingPermission(_)
            | vaya_auth::AuthError::AccountSuspended => CoreError::NotAuthorized(e.to_string()),
            _ => CoreError::Internal(e.to_string()),
        }
    }
}
