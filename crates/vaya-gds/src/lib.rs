//! vaya-gds: GDS (Global Distribution System) Integration
//!
//! This crate provides flight search, pricing, and booking via GDS providers.
//! It integrates with VAYA's sovereign infrastructure:
//!
//! - Uses `vaya-common` types (IataCode, Price, CurrencyCode, etc.)
//! - Uses `vaya-cache` for response caching
//! - NO external database dependencies
//!
//! # Supported GDS Providers
//!
//! - **Amadeus**: Primary GDS for APAC region
//! - **Travelport**: Secondary/fallback (future)
//!
//! # Example
//!
//! ```ignore
//! use vaya_gds::{AmadeusClient, GdsProvider, FlightSearchRequest};
//! use vaya_common::IataCode;
//!
//! let client = AmadeusClient::new(config).await?;
//!
//! let request = FlightSearchRequest {
//!     origin: IataCode::KUL,
//!     destination: IataCode::NRT,
//!     departure_date: Date::today().add_days(30),
//!     adults: 1,
//!     ..Default::default()
//! };
//!
//! let offers = client.search_flights(&request).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]

pub mod error;
pub mod types;
pub mod traits;
pub mod amadeus;
pub mod cache;

pub use error::{GdsError, GdsResult};
pub use types::*;
pub use traits::GdsProvider;
pub use amadeus::AmadeusClient;
pub use cache::GdsCache;

/// GDS configuration
#[derive(Debug, Clone)]
pub struct GdsConfig {
    /// Amadeus API key
    pub amadeus_api_key: String,
    /// Amadeus API secret
    pub amadeus_api_secret: String,
    /// Amadeus base URL (production or test)
    pub amadeus_base_url: String,
    /// Cache TTL for flight searches (seconds)
    pub search_cache_ttl_secs: u64,
    /// Cache TTL for pricing (seconds)
    pub pricing_cache_ttl_secs: u64,
    /// Request timeout (seconds)
    pub request_timeout_secs: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
}

impl Default for GdsConfig {
    fn default() -> Self {
        Self {
            amadeus_api_key: String::new(),
            amadeus_api_secret: String::new(),
            amadeus_base_url: "https://test.api.amadeus.com".to_string(),
            search_cache_ttl_secs: 300,      // 5 minutes
            pricing_cache_ttl_secs: 60,       // 1 minute
            request_timeout_secs: 30,
            max_retries: 3,
        }
    }
}

impl GdsConfig {
    /// Create new config with API credentials
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            amadeus_api_key: api_key.into(),
            amadeus_api_secret: api_secret.into(),
            ..Default::default()
        }
    }

    /// Use production Amadeus API
    #[must_use]
    pub fn with_production(mut self) -> Self {
        self.amadeus_base_url = "https://api.amadeus.com".to_string();
        self
    }

    /// Set search cache TTL
    #[must_use]
    pub fn with_search_cache_ttl(mut self, secs: u64) -> Self {
        self.search_cache_ttl_secs = secs;
        self
    }

    /// Set request timeout
    #[must_use]
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.request_timeout_secs = secs;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> GdsResult<()> {
        if self.amadeus_api_key.is_empty() {
            return Err(GdsError::Configuration("Amadeus API key is required".to_string()));
        }
        if self.amadeus_api_secret.is_empty() {
            return Err(GdsError::Configuration("Amadeus API secret is required".to_string()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = GdsConfig::default();
        assert!(config.amadeus_api_key.is_empty());
        assert_eq!(config.search_cache_ttl_secs, 300);
    }

    #[test]
    fn test_config_new() {
        let config = GdsConfig::new("key123", "secret456")
            .with_production()
            .with_timeout(60);

        assert_eq!(config.amadeus_api_key, "key123");
        assert_eq!(config.amadeus_api_secret, "secret456");
        assert_eq!(config.amadeus_base_url, "https://api.amadeus.com");
        assert_eq!(config.request_timeout_secs, 60);
    }

    #[test]
    fn test_config_validation() {
        let config = GdsConfig::default();
        assert!(config.validate().is_err());

        let config = GdsConfig::new("key", "secret");
        assert!(config.validate().is_ok());
    }
}
