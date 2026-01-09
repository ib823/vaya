//! vaya-payment: Payment Processing for VAYA
//!
//! This crate provides payment processing via Stripe integration.
//! It uses VAYA's sovereign infrastructure:
//!
//! - Uses `vaya-common` types (Price, `CurrencyCode`, etc.)
//! - Uses `vaya-cache` for idempotency key caching
//! - NO external database dependencies
//!
//! # Supported Payment Methods
//!
//! - **Card Payments**: Via Stripe
//! - **FPX**: Malaysian bank transfers
//! - **`GrabPay`**: Malaysian e-wallet
//!
//! # Example
//!
//! ```ignore
//! use vaya_payment::{StripeClient, PaymentRequest};
//!
//! let client = StripeClient::new(config)?;
//!
//! let request = PaymentRequest {
//!     amount: Price::myr(50000), // RM 500.00
//!     booking_ref: "VAY123456".to_string(),
//!     customer_email: "user@example.com".to_string(),
//!     ..Default::default()
//! };
//!
//! let payment = client.create_payment(&request).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

pub mod error;
pub mod stripe;
pub mod types;
mod webhook;

pub use error::{PaymentError, PaymentResult};
pub use stripe::{PaymentProvider, StripeClient};
pub use types::*;
pub use webhook::WebhookHandler;

/// Payment configuration
#[derive(Debug, Clone)]
pub struct PaymentConfig {
    /// Stripe secret key
    pub stripe_secret_key: String,
    /// Stripe publishable key (for frontend)
    pub stripe_publishable_key: String,
    /// Stripe webhook signing secret
    pub stripe_webhook_secret: String,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Default currency
    pub default_currency: vaya_common::CurrencyCode,
}

impl Default for PaymentConfig {
    fn default() -> Self {
        Self {
            stripe_secret_key: String::new(),
            stripe_publishable_key: String::new(),
            stripe_webhook_secret: String::new(),
            request_timeout_secs: 30,
            max_retries: 3,
            default_currency: vaya_common::CurrencyCode::MYR,
        }
    }
}

impl PaymentConfig {
    /// Create new config with API keys
    pub fn new(secret_key: impl Into<String>, publishable_key: impl Into<String>) -> Self {
        Self {
            stripe_secret_key: secret_key.into(),
            stripe_publishable_key: publishable_key.into(),
            ..Default::default()
        }
    }

    /// Set webhook secret
    #[must_use]
    pub fn with_webhook_secret(mut self, secret: impl Into<String>) -> Self {
        self.stripe_webhook_secret = secret.into();
        self
    }

    /// Set timeout
    #[must_use]
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.request_timeout_secs = secs;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> PaymentResult<()> {
        if self.stripe_secret_key.is_empty() {
            return Err(PaymentError::Configuration(
                "Stripe secret key is required".to_string(),
            ));
        }
        if !self.stripe_secret_key.starts_with("sk_") {
            return Err(PaymentError::Configuration(
                "Invalid Stripe secret key format".to_string(),
            ));
        }
        Ok(())
    }

    /// Check if this is a test/sandbox configuration
    #[must_use]
    pub fn is_test_mode(&self) -> bool {
        self.stripe_secret_key.starts_with("sk_test_")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = PaymentConfig::default();
        assert!(config.stripe_secret_key.is_empty());
        assert_eq!(config.request_timeout_secs, 30);
    }

    #[test]
    fn test_config_new() {
        let config = PaymentConfig::new("sk_test_123", "pk_test_456")
            .with_webhook_secret("whsec_123")
            .with_timeout(60);

        assert_eq!(config.stripe_secret_key, "sk_test_123");
        assert_eq!(config.stripe_publishable_key, "pk_test_456");
        assert_eq!(config.stripe_webhook_secret, "whsec_123");
        assert_eq!(config.request_timeout_secs, 60);
    }

    #[test]
    fn test_config_validation() {
        let config = PaymentConfig::default();
        assert!(config.validate().is_err());

        let config = PaymentConfig::new("invalid_key", "pk_test_456");
        assert!(config.validate().is_err());

        let config = PaymentConfig::new("sk_test_123", "pk_test_456");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_is_test_mode() {
        let test_config = PaymentConfig::new("sk_test_123", "pk_test_456");
        assert!(test_config.is_test_mode());

        let live_config = PaymentConfig::new("sk_live_123", "pk_live_456");
        assert!(!live_config.is_test_mode());
    }
}
