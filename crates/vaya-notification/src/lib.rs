//! vaya-notification: Email and SMS Notifications for VAYA
//!
//! This crate provides notification delivery via email and SMS.
//! Uses VAYA's sovereign infrastructure:
//!
//! - Uses `vaya-common` types
//! - Uses `vaya-cache` for rate limiting
//! - NO external database dependencies
//!
//! # Supported Providers
//!
//! - **Email**: `SendGrid`, Mailgun (via HTTP API)
//! - **SMS**: Twilio (via HTTP API)
//!
//! # Example
//!
//! ```ignore
//! use vaya_notification::{EmailClient, EmailRequest};
//!
//! let client = EmailClient::new(config)?;
//!
//! let email = EmailRequest::new(
//!     "user@example.com",
//!     "Booking Confirmation",
//! )
//! .with_template("booking_confirmed", context);
//!
//! client.send(&email).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

pub mod email;
pub mod error;
pub mod sms;
pub mod templates;
pub mod types;

pub use email::EmailClient;
pub use error::{NotificationError, NotificationResult};
pub use sms::SmsClient;
pub use templates::TemplateEngine;
pub use types::*;

/// Notification configuration
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    /// `SendGrid` API key
    pub sendgrid_api_key: String,
    /// Sender email address
    pub from_email: String,
    /// Sender name
    pub from_name: String,
    /// Twilio Account SID
    pub twilio_account_sid: String,
    /// Twilio Auth Token
    pub twilio_auth_token: String,
    /// Twilio phone number
    pub twilio_phone_number: String,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Enable sandbox mode (no actual sends)
    pub sandbox_mode: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            sendgrid_api_key: String::new(),
            from_email: String::new(),
            from_name: "VAYA Flights".to_string(),
            twilio_account_sid: String::new(),
            twilio_auth_token: String::new(),
            twilio_phone_number: String::new(),
            request_timeout_secs: 30,
            max_retries: 3,
            sandbox_mode: false,
        }
    }
}

impl NotificationConfig {
    /// Create config with `SendGrid` key
    pub fn with_sendgrid(api_key: impl Into<String>, from_email: impl Into<String>) -> Self {
        Self {
            sendgrid_api_key: api_key.into(),
            from_email: from_email.into(),
            ..Default::default()
        }
    }

    /// Add Twilio configuration
    #[must_use]
    pub fn with_twilio(
        mut self,
        account_sid: impl Into<String>,
        auth_token: impl Into<String>,
        phone_number: impl Into<String>,
    ) -> Self {
        self.twilio_account_sid = account_sid.into();
        self.twilio_auth_token = auth_token.into();
        self.twilio_phone_number = phone_number.into();
        self
    }

    /// Set sender name
    #[must_use]
    pub fn with_sender_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = name.into();
        self
    }

    /// Enable sandbox mode
    #[must_use]
    pub fn sandbox(mut self) -> Self {
        self.sandbox_mode = true;
        self
    }

    /// Set timeout
    #[must_use]
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.request_timeout_secs = secs;
        self
    }

    /// Validate email configuration
    pub fn validate_email(&self) -> NotificationResult<()> {
        if self.sendgrid_api_key.is_empty() {
            return Err(NotificationError::Configuration(
                "SendGrid API key is required".to_string(),
            ));
        }
        if self.from_email.is_empty() {
            return Err(NotificationError::Configuration(
                "From email is required".to_string(),
            ));
        }
        Ok(())
    }

    /// Validate SMS configuration
    pub fn validate_sms(&self) -> NotificationResult<()> {
        if self.twilio_account_sid.is_empty() {
            return Err(NotificationError::Configuration(
                "Twilio Account SID is required".to_string(),
            ));
        }
        if self.twilio_auth_token.is_empty() {
            return Err(NotificationError::Configuration(
                "Twilio Auth Token is required".to_string(),
            ));
        }
        if self.twilio_phone_number.is_empty() {
            return Err(NotificationError::Configuration(
                "Twilio phone number is required".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = NotificationConfig::default();
        assert!(config.sendgrid_api_key.is_empty());
        assert_eq!(config.from_name, "VAYA Flights");
        assert!(!config.sandbox_mode);
    }

    #[test]
    fn test_config_with_sendgrid() {
        let config = NotificationConfig::with_sendgrid("SG.key", "noreply@vaya.my")
            .with_sender_name("VAYA Bookings")
            .sandbox();

        assert_eq!(config.sendgrid_api_key, "SG.key");
        assert_eq!(config.from_email, "noreply@vaya.my");
        assert_eq!(config.from_name, "VAYA Bookings");
        assert!(config.sandbox_mode);
    }

    #[test]
    fn test_config_with_twilio() {
        let config = NotificationConfig::default().with_twilio("AC123", "auth123", "+60123456789");

        assert_eq!(config.twilio_account_sid, "AC123");
        assert_eq!(config.twilio_auth_token, "auth123");
        assert_eq!(config.twilio_phone_number, "+60123456789");
    }

    #[test]
    fn test_config_validation() {
        let config = NotificationConfig::default();
        assert!(config.validate_email().is_err());
        assert!(config.validate_sms().is_err());

        let config = NotificationConfig::with_sendgrid("SG.key", "test@vaya.my");
        assert!(config.validate_email().is_ok());
    }
}
