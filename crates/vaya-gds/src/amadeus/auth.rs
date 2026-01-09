//! Amadeus OAuth2 token management

use parking_lot::RwLock;
use serde::Deserialize;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

use crate::error::{GdsError, GdsResult};
use crate::GdsConfig;

/// OAuth2 token response from Amadeus
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    /// Access token
    pub access_token: String,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Expires in seconds
    pub expires_in: u64,
}

/// Cached token with expiry tracking
#[derive(Debug)]
struct CachedToken {
    /// The access token
    token: String,
    /// When the token was acquired
    acquired_at: Instant,
    /// How long the token is valid for
    valid_for: Duration,
}

impl CachedToken {
    /// Check if token is still valid (with 60 second buffer)
    fn is_valid(&self) -> bool {
        let elapsed = self.acquired_at.elapsed();
        let buffer = Duration::from_secs(60); // Refresh 60 seconds before expiry
        elapsed + buffer < self.valid_for
    }
}

/// Token manager for Amadeus OAuth2 authentication
pub struct TokenManager {
    /// HTTP client
    http_client: reqwest::Client,
    /// API key
    api_key: String,
    /// API secret
    api_secret: String,
    /// Base URL
    base_url: String,
    /// Cached token
    cached_token: RwLock<Option<CachedToken>>,
}

impl TokenManager {
    /// Create new token manager
    pub fn new(config: &GdsConfig, http_client: reqwest::Client) -> Self {
        Self {
            http_client,
            api_key: config.amadeus_api_key.clone(),
            api_secret: config.amadeus_api_secret.clone(),
            base_url: config.amadeus_base_url.clone(),
            cached_token: RwLock::new(None),
        }
    }

    /// Get valid access token (refreshes if expired)
    pub async fn get_token(&self) -> GdsResult<String> {
        // Check if we have a valid cached token
        {
            let cache = self.cached_token.read();
            if let Some(ref cached) = *cache {
                if cached.is_valid() {
                    debug!("Using cached Amadeus access token");
                    return Ok(cached.token.clone());
                }
            }
        }

        // Need to fetch a new token
        debug!("Fetching new Amadeus access token");
        self.fetch_token().await
    }

    /// Force refresh the token
    pub async fn refresh_token(&self) -> GdsResult<String> {
        // Clear cached token
        {
            let mut cache = self.cached_token.write();
            *cache = None;
        }

        self.fetch_token().await
    }

    /// Fetch new token from Amadeus
    async fn fetch_token(&self) -> GdsResult<String> {
        let url = format!("{}/v1/security/oauth2/token", self.base_url);

        let response = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.api_key),
                ("client_secret", &self.api_secret),
            ])
            .send()
            .await
            .map_err(GdsError::from)?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.as_u16() == 401 {
                return Err(GdsError::AuthenticationFailed(format!(
                    "Invalid API credentials: {body}"
                )));
            }

            if status.as_u16() == 429 {
                return Err(GdsError::RateLimited {
                    retry_after_secs: 60,
                });
            }

            return Err(GdsError::AuthenticationFailed(format!(
                "Token request failed: {status} - {body}"
            )));
        }

        let token_response: TokenResponse = response.json().await.map_err(|e| {
            GdsError::InvalidResponse(format!("Failed to parse token response: {e}"))
        })?;

        let token = token_response.access_token.clone();

        // Cache the token
        {
            let mut cache = self.cached_token.write();
            *cache = Some(CachedToken {
                token: token_response.access_token,
                acquired_at: Instant::now(),
                valid_for: Duration::from_secs(token_response.expires_in),
            });
        }

        debug!(
            "Acquired new Amadeus token, valid for {} seconds",
            token_response.expires_in
        );

        Ok(token)
    }

    /// Invalidate cached token (call when API returns 401)
    pub fn invalidate(&self) {
        let mut cache = self.cached_token.write();
        *cache = None;
        warn!("Amadeus token invalidated");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cached_token_validity() {
        let token = CachedToken {
            token: "test".to_string(),
            acquired_at: Instant::now(),
            valid_for: Duration::from_secs(1800), // 30 minutes
        };

        // Fresh token should be valid
        assert!(token.is_valid());
    }

    #[test]
    fn test_cached_token_expired() {
        let token = CachedToken {
            token: "test".to_string(),
            acquired_at: Instant::now() - Duration::from_secs(3600), // 1 hour ago
            valid_for: Duration::from_secs(1800),                    // 30 minutes
        };

        // Expired token should not be valid
        assert!(!token.is_valid());
    }
}
