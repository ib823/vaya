//! High-level data collector with caching and retry

use std::sync::Mutex;
use std::time::Duration;

use vaya_cache::LruCache;

use crate::client::{Client, ClientConfig2};
use crate::response::Response;
use crate::retry::{CircuitBreaker, RateLimiter, RetryStrategy};
use crate::url::Url;
use crate::{CollectError, CollectResult};

/// Collector configuration
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Client configuration
    pub client_config: ClientConfig2,
    /// Retry strategy
    pub retry_strategy: RetryStrategy,
    /// Cache TTL in seconds (0 = no caching)
    pub cache_ttl_secs: u64,
    /// Maximum cache entries
    pub cache_max_entries: usize,
    /// Requests per second per host
    pub rate_limit: u32,
    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,
    /// Circuit breaker reset timeout
    pub circuit_breaker_timeout: Duration,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            client_config: ClientConfig2::default(),
            retry_strategy: RetryStrategy::default(),
            cache_ttl_secs: 300, // 5 minutes
            cache_max_entries: 1000,
            rate_limit: 10,
            circuit_breaker_threshold: 5,
            circuit_breaker_timeout: Duration::from_secs(30),
        }
    }
}

/// Cached response
#[derive(Clone)]
struct CachedResponse {
    status: u16,
    body: Vec<u8>,
    content_type: Option<String>,
}

/// Data collector with caching, retry, and rate limiting
pub struct Collector {
    client: Client,
    cache: Option<Mutex<LruCache<String, CachedResponse>>>,
    cache_capacity: usize,
    rate_limiter: RateLimiter,
    circuit_breaker: CircuitBreaker,
    retry_strategy: RetryStrategy,
    #[allow(dead_code)]
    cache_ttl: Duration,
}

impl Collector {
    /// Create a new collector with default config
    pub fn new() -> CollectResult<Self> {
        Self::with_config(CollectorConfig::default())
    }

    /// Create collector with custom config
    pub fn with_config(config: CollectorConfig) -> CollectResult<Self> {
        let client = Client::with_config(config.client_config)?;

        let cache = if config.cache_ttl_secs > 0 {
            Some(Mutex::new(LruCache::new(config.cache_max_entries)))
        } else {
            None
        };

        Ok(Self {
            client,
            cache,
            cache_capacity: config.cache_max_entries,
            rate_limiter: RateLimiter::new(config.rate_limit),
            circuit_breaker: CircuitBreaker::new(
                config.circuit_breaker_threshold,
                config.circuit_breaker_timeout,
            ),
            retry_strategy: config.retry_strategy,
            cache_ttl: Duration::from_secs(config.cache_ttl_secs),
        })
    }

    /// Fetch URL with caching, retry, and rate limiting
    pub fn fetch(&self, url: &str) -> CollectResult<Response> {
        let parsed = Url::parse(url)?;

        // Check circuit breaker
        self.circuit_breaker.check(&parsed.host)?;

        // Check cache
        if let Some(cache) = &self.cache {
            let url_key = url.to_string();
            if let Some(cached) = cache.lock().unwrap().get(&url_key) {
                return Ok(Response {
                    status: cached.status,
                    reason: "OK".to_string(),
                    headers: crate::response::ResponseHeaders::new(),
                    body: cached.body,
                });
            }
        }

        // Check rate limit
        self.rate_limiter.check(&parsed.host)?;

        // Execute with retry
        let result = self.fetch_with_retry(url);

        // Update circuit breaker
        match &result {
            Ok(response) if response.is_success() => {
                self.circuit_breaker.record_success(&parsed.host);
            }
            Err(_) => {
                self.circuit_breaker.record_failure(&parsed.host);
            }
            _ => {}
        }

        // Cache successful responses
        if let Ok(response) = &result {
            if response.is_success() {
                if let Some(cache) = &self.cache {
                    cache.lock().unwrap().insert(
                        url.to_string(),
                        CachedResponse {
                            status: response.status,
                            body: response.body.clone(),
                            content_type: response.content_type().map(|s| s.to_string()),
                        },
                    );
                }
            }
        }

        result
    }

    /// Fetch with retry logic
    fn fetch_with_retry(&self, url: &str) -> CollectResult<Response> {
        let mut last_error = None;

        for attempt in 0..=self.retry_strategy.max_retries {
            if attempt > 0 {
                let delay = self.retry_strategy.delay_for_attempt(attempt - 1);
                std::thread::sleep(delay);
            }

            match self.client.get(url) {
                Ok(response) => {
                    // Check for rate limit response
                    if response.status == 429 {
                        let retry_after = response
                            .headers
                            .get("retry-after")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or(60);
                        last_error = Some(CollectError::RateLimited(retry_after));
                        continue;
                    }

                    return Ok(response);
                }
                Err(e) => {
                    if !self.retry_strategy.should_retry(&e, attempt) {
                        return Err(e);
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or(CollectError::Timeout))
    }

    /// Fetch JSON and return as string
    pub fn fetch_json(&self, url: &str) -> CollectResult<String> {
        let response = self.fetch(url)?;
        response.text()
    }

    /// Post data to URL
    pub fn post(&self, url: &str, body: &[u8], content_type: &str) -> CollectResult<Response> {
        let parsed = Url::parse(url)?;

        self.circuit_breaker.check(&parsed.host)?;
        self.rate_limiter.check(&parsed.host)?;

        let result = self.client.post(url, body, content_type);

        match &result {
            Ok(response) if response.is_success() => {
                self.circuit_breaker.record_success(&parsed.host);
            }
            Err(_) => {
                self.circuit_breaker.record_failure(&parsed.host);
            }
            _ => {}
        }

        result
    }

    /// Post JSON data
    pub fn post_json(&self, url: &str, json: &str) -> CollectResult<Response> {
        self.post(url, json.as_bytes(), "application/json")
    }

    /// Invalidate cache entry
    pub fn invalidate(&self, url: &str) {
        if let Some(cache) = &self.cache {
            let url_key = url.to_string();
            cache.lock().unwrap().remove(&url_key);
        }
    }

    /// Clear all cache
    pub fn clear_cache(&self) {
        if let Some(cache) = &self.cache {
            cache.lock().unwrap().clear();
        }
    }

    /// Get cache statistics (current size, capacity)
    pub fn cache_stats(&self) -> (usize, usize) {
        if let Some(cache) = &self.cache {
            (cache.lock().unwrap().len(), self.cache_capacity)
        } else {
            (0, 0)
        }
    }
}

impl Default for Collector {
    fn default() -> Self {
        Self::new().expect("Failed to create default collector")
    }
}

/// Collector builder
pub struct CollectorBuilder {
    config: CollectorConfig,
}

impl CollectorBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            config: CollectorConfig::default(),
        }
    }

    /// Set timeout
    pub fn timeout(mut self, ms: u64) -> Self {
        self.config.client_config.timeout_ms = ms;
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.config.client_config.user_agent = agent.into();
        self
    }

    /// Set cache TTL
    pub fn cache_ttl(mut self, secs: u64) -> Self {
        self.config.cache_ttl_secs = secs;
        self
    }

    /// Disable caching
    pub fn no_cache(mut self) -> Self {
        self.config.cache_ttl_secs = 0;
        self
    }

    /// Set rate limit
    pub fn rate_limit(mut self, requests_per_second: u32) -> Self {
        self.config.rate_limit = requests_per_second;
        self
    }

    /// Set retry strategy
    pub fn retry(mut self, strategy: RetryStrategy) -> Self {
        self.config.retry_strategy = strategy;
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, count: u32) -> Self {
        self.config.retry_strategy.max_retries = count;
        self
    }

    /// Build the collector
    pub fn build(self) -> CollectResult<Collector> {
        Collector::with_config(self.config)
    }
}

impl Default for CollectorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collector_creation() {
        let collector = Collector::new().unwrap();
        let (len, cap) = collector.cache_stats();
        assert_eq!(len, 0);
        assert!(cap > 0);
    }

    #[test]
    fn test_collector_builder() {
        let collector = CollectorBuilder::new()
            .timeout(5000)
            .cache_ttl(60)
            .rate_limit(5)
            .max_retries(2)
            .build()
            .unwrap();

        assert_eq!(collector.cache_stats().1, 1000);
    }

    #[test]
    fn test_cache_invalidation() {
        let collector = Collector::new().unwrap();
        collector.invalidate("https://example.com");
        collector.clear_cache();
    }
}
