//! Payment error types

use thiserror::Error;

/// Payment result type
pub type PaymentResult<T> = Result<T, PaymentError>;

/// Payment error types
#[derive(Debug, Error)]
pub enum PaymentError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Card declined
    #[error("Card declined: {code} - {message}")]
    CardDeclined {
        /// Decline code
        code: String,
        /// Decline message
        message: String,
    },

    /// Insufficient funds
    #[error("Insufficient funds")]
    InsufficientFunds,

    /// Expired card
    #[error("Card expired")]
    ExpiredCard,

    /// Invalid card
    #[error("Invalid card: {0}")]
    InvalidCard(String),

    /// Payment already processed
    #[error("Payment already processed: {payment_id}")]
    AlreadyProcessed {
        /// Payment ID
        payment_id: String,
    },

    /// Payment not found
    #[error("Payment not found: {payment_id}")]
    PaymentNotFound {
        /// Payment ID
        payment_id: String,
    },

    /// Refund failed
    #[error("Refund failed: {0}")]
    RefundFailed(String),

    /// Webhook signature invalid
    #[error("Invalid webhook signature")]
    InvalidSignature,

    /// Rate limited
    #[error("Rate limited, retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds to wait before retry
        retry_after_secs: u64,
    },

    /// Service unavailable
    #[error("Payment service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid response from provider
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Timeout
    #[error("Request timeout")]
    Timeout,

    /// Currency mismatch
    #[error("Currency mismatch: expected {expected}, got {got}")]
    CurrencyMismatch {
        /// Expected currency
        expected: String,
        /// Actual currency
        got: String,
    },

    /// Amount too small
    #[error("Amount too small: minimum is {minimum}")]
    AmountTooSmall {
        /// Minimum amount
        minimum: String,
    },

    /// Amount too large
    #[error("Amount too large: maximum is {maximum}")]
    AmountTooLarge {
        /// Maximum amount
        maximum: String,
    },

    /// 3D Secure required
    #[error("3D Secure authentication required")]
    RequiresAuthentication {
        /// Client secret for 3DS
        client_secret: String,
    },

    /// Payment method not supported
    #[error("Payment method not supported: {0}")]
    PaymentMethodNotSupported(String),
}

impl PaymentError {
    /// Check if error is retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::Network(_)
                | Self::ServiceUnavailable(_)
                | Self::Timeout
                | Self::RateLimited { .. }
        )
    }

    /// Get suggested retry delay in seconds
    #[must_use]
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after_secs } => Some(*retry_after_secs),
            Self::Network(_) | Self::ServiceUnavailable(_) | Self::Timeout => Some(1),
            _ => None,
        }
    }

    /// Check if error is a card error (user-fixable)
    #[must_use]
    pub fn is_card_error(&self) -> bool {
        matches!(
            self,
            Self::CardDeclined { .. }
                | Self::InsufficientFunds
                | Self::ExpiredCard
                | Self::InvalidCard(_)
        )
    }

    /// Check if error requires user action
    #[must_use]
    pub fn requires_user_action(&self) -> bool {
        matches!(self, Self::RequiresAuthentication { .. })
    }

    /// Get user-friendly message
    #[must_use]
    pub fn user_message(&self) -> &str {
        match self {
            Self::CardDeclined { .. } => "Your card was declined. Please try a different card.",
            Self::InsufficientFunds => "Insufficient funds. Please try a different card.",
            Self::ExpiredCard => "Your card has expired. Please update your card details.",
            Self::InvalidCard(_) => "Invalid card details. Please check and try again.",
            Self::RequiresAuthentication { .. } => "Additional authentication required.",
            Self::PaymentMethodNotSupported(_) => "This payment method is not supported.",
            Self::AmountTooSmall { .. } => "The amount is below the minimum.",
            Self::AmountTooLarge { .. } => "The amount exceeds the maximum.",
            _ => "Payment could not be processed. Please try again.",
        }
    }

    /// HTTP status code for this error
    #[must_use]
    pub fn http_status(&self) -> u16 {
        match self {
            Self::Configuration(_) => 500,
            Self::AuthenticationFailed(_) | Self::InvalidSignature => 401,
            Self::CardDeclined { .. }
            | Self::InsufficientFunds
            | Self::ExpiredCard
            | Self::InvalidCard(_) => 400,
            Self::AlreadyProcessed { .. } => 409,
            Self::PaymentNotFound { .. } => 404,
            Self::RateLimited { .. } => 429,
            Self::ServiceUnavailable(_) | Self::Network(_) | Self::Timeout => 503,
            Self::RequiresAuthentication { .. } => 402,
            _ => 400,
        }
    }
}

impl From<reqwest::Error> for PaymentError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else if err.is_connect() {
            Self::Network(format!("Connection failed: {err}"))
        } else {
            Self::Network(err.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        assert!(PaymentError::Timeout.is_retryable());
        assert!(PaymentError::Network("test".to_string()).is_retryable());
        assert!(PaymentError::RateLimited {
            retry_after_secs: 60
        }
        .is_retryable());

        assert!(!PaymentError::CardDeclined {
            code: "declined".to_string(),
            message: "Generic decline".to_string(),
        }
        .is_retryable());
    }

    #[test]
    fn test_error_card_error() {
        assert!(PaymentError::CardDeclined {
            code: "declined".to_string(),
            message: "test".to_string(),
        }
        .is_card_error());
        assert!(PaymentError::InsufficientFunds.is_card_error());
        assert!(PaymentError::ExpiredCard.is_card_error());

        assert!(!PaymentError::Timeout.is_card_error());
    }

    #[test]
    fn test_error_http_status() {
        assert_eq!(
            PaymentError::AuthenticationFailed("test".to_string()).http_status(),
            401
        );
        assert_eq!(
            PaymentError::PaymentNotFound {
                payment_id: "123".to_string()
            }
            .http_status(),
            404
        );
        assert_eq!(
            PaymentError::RateLimited {
                retry_after_secs: 60
            }
            .http_status(),
            429
        );
    }
}
