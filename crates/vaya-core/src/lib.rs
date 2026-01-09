//! vaya-core: Core business logic for VAYA flight booking platform
//!
//! This crate provides the central business logic that orchestrates
//! all VAYA services including:
//!
//! - **Flight search**: Search flights through GDS providers
//! - **Booking**: Create, manage, and cancel bookings
//! - **User management**: Registration, authentication, profiles
//! - **Payments**: Payment processing and refunds
//! - **Notifications**: Email and SMS confirmations
//!
//! # Architecture
//!
//! vaya-core depends on all other VAYA crates and provides high-level
//! services that compose functionality from:
//!
//! - `vaya-gds`: GDS integration (Amadeus, etc.)
//! - `vaya-payment`: Payment processing (Stripe)
//! - `vaya-notification`: Email/SMS (SendGrid, Twilio)
//! - `vaya-oracle`: Price predictions
//! - `vaya-auth`: Authentication
//! - `vaya-db`: Database storage
//! - `vaya-cache`: Caching layer
//!
//! # Example
//!
//! ```ignore
//! use vaya_core::{SearchService, BookingService, SearchRequest};
//!
//! // Create search request
//! let search = SearchRequest::round_trip(
//!     IataCode::KUL,
//!     IataCode::SIN,
//!     "2025-06-15",
//!     "2025-06-20",
//! );
//!
//! // Search flights
//! let results = search_service.search(&search).await?;
//!
//! // Book a flight
//! let booking = booking_service.create_booking(request).await?;
//! ```

#![warn(missing_docs)]

pub mod booking;
pub mod error;
pub mod search;
pub mod types;
pub mod user;

pub use booking::{BookingConfig, BookingService, CancellationResult, PaymentResult};
pub use error::{CoreError, CoreResult};
pub use search::{SearchResponse, SearchService, SearchPriceInsight};
pub use types::*;
pub use user::{AuthConfig, AuthResponse, LoginRequest, ProfileUpdate, RegisterRequest, User, UserService, UserStatus};

/// Core configuration
#[derive(Debug, Clone)]
pub struct CoreConfig {
    /// Search configuration
    pub search_timeout_secs: u64,
    /// Search max results
    pub search_max_results: usize,
    /// Booking payment timeout minutes
    pub booking_payment_timeout_minutes: u32,
    /// Enable email notifications
    pub enable_email_notifications: bool,
    /// Enable SMS notifications
    pub enable_sms_notifications: bool,
    /// Enable price predictions
    pub enable_price_predictions: bool,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            search_timeout_secs: 30,
            search_max_results: 100,
            booking_payment_timeout_minutes: 30,
            enable_email_notifications: true,
            enable_sms_notifications: false,
            enable_price_predictions: true,
        }
    }
}

impl CoreConfig {
    /// Create new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set search timeout
    pub fn with_search_timeout(mut self, secs: u64) -> Self {
        self.search_timeout_secs = secs;
        self
    }

    /// Set payment timeout
    pub fn with_payment_timeout(mut self, minutes: u32) -> Self {
        self.booking_payment_timeout_minutes = minutes;
        self
    }

    /// Disable email notifications
    pub fn without_email(mut self) -> Self {
        self.enable_email_notifications = false;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_config_defaults() {
        let config = CoreConfig::new();
        assert_eq!(config.search_timeout_secs, 30);
        assert_eq!(config.search_max_results, 100);
        assert!(config.enable_email_notifications);
    }

    #[test]
    fn test_core_config_builder() {
        let config = CoreConfig::new()
            .with_search_timeout(60)
            .with_payment_timeout(45)
            .without_email();

        assert_eq!(config.search_timeout_secs, 60);
        assert_eq!(config.booking_payment_timeout_minutes, 45);
        assert!(!config.enable_email_notifications);
    }
}
