//! GDS Error types

use thiserror::Error;

/// Result type for GDS operations
pub type GdsResult<T> = Result<T, GdsError>;

/// GDS error type
#[derive(Error, Debug)]
pub enum GdsError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Token expired
    #[error("Access token expired")]
    TokenExpired,

    /// Rate limited by GDS
    #[error("Rate limited, retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds to wait before retry
        retry_after_secs: u64,
    },

    /// Flight not available
    #[error("Flight unavailable: {0}")]
    FlightUnavailable(String),

    /// Price changed since search
    #[error("Price changed from {old} to {new}")]
    PriceChanged {
        /// Old price
        old: String,
        /// New price
        new: String,
    },

    /// Offer expired
    #[error("Offer expired: {offer_id}")]
    OfferExpired {
        /// The expired offer ID
        offer_id: String,
    },

    /// Booking failed
    #[error("Booking failed: {code} - {message}")]
    BookingFailed {
        /// Error code from GDS
        code: String,
        /// Error message
        message: String,
    },

    /// Ticketing failed
    #[error("Ticketing failed: {0}")]
    TicketingFailed(String),

    /// Cancellation failed
    #[error("Cancellation failed: {0}")]
    CancellationFailed(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Invalid response from GDS
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Timeout
    #[error("Request timed out after {timeout_secs} seconds")]
    Timeout {
        /// Timeout duration
        timeout_secs: u64,
    },

    /// GDS service unavailable
    #[error("GDS service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Not found
    #[error("{resource} not found: {id}")]
    NotFound {
        /// Resource type
        resource: String,
        /// Resource ID
        id: String,
    },
}

impl GdsError {
    /// Check if error is retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimited { .. }
                | Self::Timeout { .. }
                | Self::NetworkError(_)
                | Self::ServiceUnavailable(_)
                | Self::TokenExpired
        )
    }

    /// Get retry delay in seconds (if applicable)
    #[must_use]
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after_secs } => Some(*retry_after_secs),
            Self::Timeout { .. } | Self::NetworkError(_) => Some(5),
            Self::ServiceUnavailable(_) => Some(30),
            Self::TokenExpired => Some(0), // Immediate retry after re-auth
            _ => None,
        }
    }

    /// Get HTTP-like status code for this error
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::Configuration(_) | Self::InvalidRequest(_) => 400,
            Self::AuthenticationFailed(_) | Self::TokenExpired => 401,
            Self::FlightUnavailable(_) | Self::OfferExpired { .. } => 404,
            Self::PriceChanged { .. } => 409,
            Self::RateLimited { .. } => 429,
            Self::Timeout { .. } => 408,
            Self::ServiceUnavailable(_) => 503,
            Self::NetworkError(_)
            | Self::Internal(_)
            | Self::Serialization(_)
            | Self::InvalidResponse(_) => 500,
            Self::BookingFailed { .. }
            | Self::TicketingFailed(_)
            | Self::CancellationFailed(_) => 422,
            Self::NotFound { .. } => 404,
        }
    }

    /// Create from reqwest error
    pub fn from_reqwest(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout { timeout_secs: 30 }
        } else if err.is_connect() {
            Self::NetworkError(format!("Connection failed: {err}"))
        } else if err.is_decode() {
            Self::InvalidResponse(format!("Failed to decode response: {err}"))
        } else {
            Self::NetworkError(err.to_string())
        }
    }
}

impl From<reqwest::Error> for GdsError {
    fn from(err: reqwest::Error) -> Self {
        Self::from_reqwest(err)
    }
}

impl From<serde_json::Error> for GdsError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        assert!(GdsError::RateLimited { retry_after_secs: 60 }.is_retryable());
        assert!(GdsError::TokenExpired.is_retryable());
        assert!(GdsError::NetworkError("test".to_string()).is_retryable());
        assert!(!GdsError::InvalidRequest("test".to_string()).is_retryable());
        assert!(!GdsError::BookingFailed {
            code: "ERR".to_string(),
            message: "test".to_string()
        }
        .is_retryable());
    }

    #[test]
    fn test_error_retry_after() {
        assert_eq!(
            GdsError::RateLimited { retry_after_secs: 60 }.retry_after(),
            Some(60)
        );
        assert_eq!(GdsError::TokenExpired.retry_after(), Some(0));
        assert!(GdsError::InvalidRequest("test".to_string())
            .retry_after()
            .is_none());
    }

    #[test]
    fn test_error_status_codes() {
        assert_eq!(
            GdsError::AuthenticationFailed("test".to_string()).status_code(),
            401
        );
        assert_eq!(
            GdsError::RateLimited { retry_after_secs: 60 }.status_code(),
            429
        );
        assert_eq!(
            GdsError::NotFound {
                resource: "booking".to_string(),
                id: "123".to_string()
            }
            .status_code(),
            404
        );
    }
}
