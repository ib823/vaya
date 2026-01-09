//! Notification error types

use thiserror::Error;

/// Notification result type
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Notification error types
#[derive(Debug, Error)]
pub enum NotificationError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Invalid recipient
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),

    /// Template not found
    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    /// Template rendering failed
    #[error("Template error: {0}")]
    TemplateError(String),

    /// Rate limited
    #[error("Rate limited, retry after {retry_after_secs} seconds")]
    RateLimited {
        /// Seconds to wait before retry
        retry_after_secs: u64,
    },

    /// Service unavailable
    #[error("Notification service unavailable: {0}")]
    ServiceUnavailable(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Delivery failed
    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),

    /// Bounced email
    #[error("Email bounced: {email}")]
    Bounced {
        /// Email that bounced
        email: String,
    },

    /// Spam complaint
    #[error("Spam complaint from: {email}")]
    SpamComplaint {
        /// Email that complained
        email: String,
    },

    /// Invalid phone number
    #[error("Invalid phone number: {0}")]
    InvalidPhoneNumber(String),

    /// SMS delivery failed
    #[error("SMS delivery failed: {0}")]
    SmsDeliveryFailed(String),

    /// Timeout
    #[error("Request timeout")]
    Timeout,

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

impl NotificationError {
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

    /// Get retry delay in seconds
    #[must_use]
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after_secs } => Some(*retry_after_secs),
            Self::Network(_) | Self::ServiceUnavailable(_) | Self::Timeout => Some(1),
            _ => None,
        }
    }

    /// Check if this is a permanent failure (don't retry)
    #[must_use]
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            Self::InvalidRecipient(_)
            | Self::InvalidPhoneNumber(_)
            | Self::Bounced { .. }
            | Self::SpamComplaint { .. }
        )
    }

    /// HTTP status code for this error
    #[must_use]
    pub fn http_status(&self) -> u16 {
        match self {
            Self::Configuration(_) => 500,
            Self::InvalidRecipient(_) | Self::InvalidPhoneNumber(_) => 400,
            Self::TemplateNotFound(_) => 404,
            Self::TemplateError(_) => 500,
            Self::RateLimited { .. } => 429,
            Self::ServiceUnavailable(_) | Self::Network(_) | Self::Timeout => 503,
            Self::DeliveryFailed(_) | Self::SmsDeliveryFailed(_) => 502,
            Self::Bounced { .. } | Self::SpamComplaint { .. } => 400,
            Self::InvalidResponse(_) => 502,
        }
    }
}

impl From<reqwest::Error> for NotificationError {
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

impl From<handlebars::RenderError> for NotificationError {
    fn from(err: handlebars::RenderError) -> Self {
        Self::TemplateError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_retryable() {
        assert!(NotificationError::Timeout.is_retryable());
        assert!(NotificationError::Network("test".to_string()).is_retryable());
        assert!(NotificationError::RateLimited { retry_after_secs: 60 }.is_retryable());

        assert!(!NotificationError::InvalidRecipient("test".to_string()).is_retryable());
    }

    #[test]
    fn test_error_permanent() {
        assert!(NotificationError::InvalidRecipient("test".to_string()).is_permanent());
        assert!(NotificationError::Bounced { email: "test@test.com".to_string() }.is_permanent());

        assert!(!NotificationError::Timeout.is_permanent());
    }

    #[test]
    fn test_error_http_status() {
        assert_eq!(NotificationError::InvalidRecipient("test".to_string()).http_status(), 400);
        assert_eq!(NotificationError::TemplateNotFound("test".to_string()).http_status(), 404);
        assert_eq!(NotificationError::RateLimited { retry_after_secs: 60 }.http_status(), 429);
    }
}
