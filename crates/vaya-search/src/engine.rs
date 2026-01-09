//! Search engine for processing flight searches

use std::sync::Mutex;

use vaya_cache::LruCache;

use crate::request::{SearchRequest, SortBy, SortOrder};
use crate::types::FlightOffer;
use crate::SearchResult;

/// Search response
#[derive(Debug, Clone)]
pub struct SearchResponse {
    /// Request ID
    pub request_id: String,
    /// Matching offers
    pub offers: Vec<FlightOffer>,
    /// Total offers found (before limiting)
    pub total_count: usize,
    /// Search duration in milliseconds
    pub duration_ms: u64,
    /// Was this a cached response
    pub from_cache: bool,
    /// Warnings/notices
    pub warnings: Vec<String>,
}

impl SearchResponse {
    /// Check if any results found
    pub fn has_results(&self) -> bool {
        !self.offers.is_empty()
    }

    /// Get the cheapest offer
    pub fn cheapest(&self) -> Option<&FlightOffer> {
        self.offers.iter().min_by_key(|o| o.price.total().as_i64())
    }

    /// Get the fastest offer
    pub fn fastest(&self) -> Option<&FlightOffer> {
        self.offers
            .iter()
            .min_by_key(|o| o.total_duration_minutes())
    }

    /// Sort offers by criteria
    pub fn sorted(&self, by: SortBy, order: SortOrder) -> Vec<&FlightOffer> {
        let mut offers: Vec<&FlightOffer> = self.offers.iter().collect();

        match by {
            SortBy::Price => {
                offers.sort_by_key(|o| o.price.total().as_i64());
            }
            SortBy::Duration => {
                offers.sort_by_key(|o| o.total_duration_minutes());
            }
            SortBy::Departure => {
                offers.sort_by(|a, b| {
                    let a_time = a.outbound.departure_time();
                    let b_time = b.outbound.departure_time();
                    a_time.cmp(&b_time)
                });
            }
            SortBy::Arrival => {
                offers.sort_by(|a, b| {
                    let a_time = a.outbound.arrival_time();
                    let b_time = b.outbound.arrival_time();
                    a_time.cmp(&b_time)
                });
            }
            SortBy::Stops => {
                offers.sort_by_key(|o| o.outbound.stops());
            }
        }

        if order == SortOrder::Descending {
            offers.reverse();
        }

        offers
    }
}

/// Search engine configuration
#[derive(Debug, Clone)]
pub struct SearchEngineConfig {
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Maximum cached searches
    pub max_cached_searches: usize,
    /// Search timeout in milliseconds
    pub timeout_ms: u64,
    /// Maximum results per search
    pub max_results: usize,
}

impl Default for SearchEngineConfig {
    fn default() -> Self {
        Self {
            cache_ttl_secs: 300, // 5 minutes
            max_cached_searches: 1000,
            timeout_ms: 30_000,
            max_results: 100,
        }
    }
}

/// Cached search result
#[derive(Clone)]
struct CachedSearch {
    response: SearchResponse,
    cached_at: i64,
}

/// Flight search engine
pub struct SearchEngine {
    config: SearchEngineConfig,
    cache: Mutex<LruCache<String, CachedSearch>>,
    providers: Vec<Box<dyn SearchProvider>>,
    request_counter: Mutex<u64>,
}

/// Search provider trait
pub trait SearchProvider: Send + Sync {
    /// Provider name
    fn name(&self) -> &str;

    /// Search for flights
    fn search(&self, request: &SearchRequest) -> SearchResult<Vec<FlightOffer>>;

    /// Check if provider is available
    fn is_available(&self) -> bool {
        true
    }

    /// Priority (higher = searched first)
    fn priority(&self) -> u8 {
        50
    }
}

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self {
        Self::with_config(SearchEngineConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: SearchEngineConfig) -> Self {
        Self {
            cache: Mutex::new(LruCache::new(config.max_cached_searches)),
            config,
            providers: Vec::new(),
            request_counter: Mutex::new(0),
        }
    }

    /// Add a search provider
    pub fn add_provider(&mut self, provider: Box<dyn SearchProvider>) {
        self.providers.push(provider);
        // Sort by priority (descending)
        self.providers
            .sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Execute a search
    pub fn search(&self, request: &SearchRequest) -> SearchResult<SearchResponse> {
        // Validate request
        request.validate()?;

        // Check cache
        let cache_key = request.cache_key();
        if let Some(cached) = self.get_cached(&cache_key) {
            return Ok(cached);
        }

        // Generate request ID
        let request_id = self.generate_request_id();

        let start = std::time::Instant::now();

        // Search all providers
        let mut all_offers = Vec::new();
        let mut warnings = Vec::new();

        for provider in &self.providers {
            if !provider.is_available() {
                warnings.push(format!("Provider {} unavailable", provider.name()));
                continue;
            }

            match provider.search(request) {
                Ok(offers) => {
                    all_offers.extend(offers);
                }
                Err(e) => {
                    warnings.push(format!("Provider {} error: {}", provider.name(), e));
                }
            }
        }

        // Apply filters
        let mut filtered: Vec<FlightOffer> = all_offers
            .into_iter()
            .filter(|o| self.passes_filters(o, request))
            .collect();

        // Sort by price (default)
        filtered.sort_by_key(|o| o.price.total().as_i64());

        // Limit results
        let total_count = filtered.len();
        let max = request.max_results.unwrap_or(self.config.max_results);
        filtered.truncate(max);

        let duration_ms = start.elapsed().as_millis() as u64;

        let response = SearchResponse {
            request_id,
            offers: filtered,
            total_count,
            duration_ms,
            from_cache: false,
            warnings,
        };

        // Cache the response
        self.cache_response(&cache_key, &response);

        Ok(response)
    }

    /// Check if offer passes request filters
    fn passes_filters(&self, offer: &FlightOffer, request: &SearchRequest) -> bool {
        let filters = &request.filters;

        // Check stops
        if !filters.passes_stops(offer.outbound.stops()) {
            return false;
        }

        // Check price
        if !filters.passes_price(offer.price.total().as_i64()) {
            return false;
        }

        // Check duration
        if !filters.passes_duration(offer.outbound.total_duration_minutes) {
            return false;
        }

        // Check refundable
        if filters.refundable_only && !offer.refundable {
            return false;
        }

        // Check airlines
        for segment in &offer.outbound.segments {
            if !filters.passes_airline(&segment.airline) {
                return false;
            }
        }

        // Check expired
        if offer.is_expired() {
            return false;
        }

        true
    }

    /// Get cached response
    fn get_cached(&self, key: &str) -> Option<SearchResponse> {
        let mut cache = self.cache.lock().unwrap();
        let key_string = key.to_string();

        if let Some(cached) = cache.get(&key_string) {
            let now = time::OffsetDateTime::now_utc().unix_timestamp();
            let age = now - cached.cached_at;

            if age < self.config.cache_ttl_secs as i64 {
                let mut response = cached.response.clone();
                response.from_cache = true;
                return Some(response);
            }
        }

        None
    }

    /// Cache a response
    fn cache_response(&self, key: &str, response: &SearchResponse) {
        let mut cache = self.cache.lock().unwrap();
        let cached = CachedSearch {
            response: response.clone(),
            cached_at: time::OffsetDateTime::now_utc().unix_timestamp(),
        };
        cache.insert(key.to_string(), cached);
    }

    /// Generate unique request ID
    fn generate_request_id(&self) -> String {
        let mut counter = self.request_counter.lock().unwrap();
        *counter += 1;
        let timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
        format!("SR-{}-{}", timestamp, counter)
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }

    /// Get cache stats
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().unwrap();
        (cache.len(), self.config.max_cached_searches)
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock provider for testing
#[cfg(test)]
pub struct MockProvider {
    name: String,
    offers: Vec<FlightOffer>,
}

#[cfg(test)]
impl MockProvider {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            offers: Vec::new(),
        }
    }

    pub fn with_offers(mut self, offers: Vec<FlightOffer>) -> Self {
        self.offers = offers;
        self
    }
}

#[cfg(test)]
impl SearchProvider for MockProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn search(&self, _request: &SearchRequest) -> SearchResult<Vec<FlightOffer>> {
        Ok(self.offers.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_engine_creation() {
        let engine = SearchEngine::new();
        assert_eq!(engine.cache_stats().1, 1000);
    }

    #[test]
    fn test_search_engine_with_config() {
        let config = SearchEngineConfig {
            cache_ttl_secs: 60,
            max_cached_searches: 100,
            ..Default::default()
        };
        let engine = SearchEngine::with_config(config);
        assert_eq!(engine.cache_stats().1, 100);
    }

    #[test]
    fn test_request_id_generation() {
        let engine = SearchEngine::new();
        let id1 = engine.generate_request_id();
        let id2 = engine.generate_request_id();
        assert!(id1.starts_with("SR-"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_search_response_methods() {
        let response = SearchResponse {
            request_id: "test".into(),
            offers: vec![],
            total_count: 0,
            duration_ms: 100,
            from_cache: false,
            warnings: vec![],
        };
        assert!(!response.has_results());
        assert!(response.cheapest().is_none());
    }
}
