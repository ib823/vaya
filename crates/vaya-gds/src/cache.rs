//! GDS Response caching using VayaCache

use std::time::Duration;
use vaya_cache::Cache;

use crate::types::FlightOffer;

/// GDS response cache using VayaCache (sharded LRU with TTL)
pub struct GdsCache {
    /// Flight search results cache
    search_cache: Cache<String, Vec<FlightOffer>>,
    /// Pricing cache (offer_id -> priced offer)
    pricing_cache: Cache<String, FlightOffer>,
    /// Default TTL for search results
    search_ttl: Duration,
    /// Default TTL for pricing
    pricing_ttl: Duration,
}

impl GdsCache {
    /// Create new GDS cache with default settings
    ///
    /// Defaults:
    /// - 1000 search results, 16 shards
    /// - 500 pricing results, 8 shards
    /// - 5 minute search TTL
    /// - 1 minute pricing TTL
    #[must_use]
    pub fn new() -> Self {
        Self {
            search_cache: Cache::new(1000, 16),
            pricing_cache: Cache::new(500, 8),
            search_ttl: Duration::from_secs(300),
            pricing_ttl: Duration::from_secs(60),
        }
    }

    /// Create with custom capacity
    #[must_use]
    pub fn with_capacity(search_capacity: usize, pricing_capacity: usize) -> Self {
        Self {
            search_cache: Cache::new(search_capacity, 16),
            pricing_cache: Cache::new(pricing_capacity, 8),
            search_ttl: Duration::from_secs(300),
            pricing_ttl: Duration::from_secs(60),
        }
    }

    /// Set search cache TTL
    #[must_use]
    pub fn with_search_ttl(mut self, ttl: Duration) -> Self {
        self.search_ttl = ttl;
        self
    }

    /// Set pricing cache TTL
    #[must_use]
    pub fn with_pricing_ttl(mut self, ttl: Duration) -> Self {
        self.pricing_ttl = ttl;
        self
    }

    /// Get cached search results
    #[must_use]
    pub fn get_search(&self, cache_key: &str) -> Option<Vec<FlightOffer>> {
        self.search_cache.get(&cache_key.to_string())
    }

    /// Cache search results
    pub fn put_search(&self, cache_key: &str, offers: Vec<FlightOffer>) {
        self.search_cache
            .insert(cache_key.to_string(), offers, Some(self.search_ttl));
    }

    /// Get cached pricing
    #[must_use]
    pub fn get_pricing(&self, offer_id: &str) -> Option<FlightOffer> {
        self.pricing_cache.get(&offer_id.to_string())
    }

    /// Cache pricing result
    pub fn put_pricing(&self, offer_id: &str, offer: FlightOffer) {
        self.pricing_cache
            .insert(offer_id.to_string(), offer, Some(self.pricing_ttl));
    }

    /// Invalidate search cache for a key
    pub fn invalidate_search(&self, cache_key: &str) {
        self.search_cache.remove(&cache_key.to_string());
    }

    /// Invalidate pricing cache for an offer
    pub fn invalidate_pricing(&self, offer_id: &str) {
        self.pricing_cache.remove(&offer_id.to_string());
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.search_cache.clear();
        self.pricing_cache.clear();
    }

    /// Purge expired entries from all caches
    pub fn purge_expired(&self) -> usize {
        self.search_cache.purge_expired() + self.pricing_cache.purge_expired()
    }

    /// Get cache statistics
    #[must_use]
    pub fn stats(&self) -> GdsCacheStats {
        let search_stats = self.search_cache.stats();
        let pricing_stats = self.pricing_cache.stats();

        GdsCacheStats {
            search_hits: search_stats.hits,
            search_misses: search_stats.misses,
            search_size: search_stats.size,
            search_hit_rate: search_stats.hit_rate,
            pricing_hits: pricing_stats.hits,
            pricing_misses: pricing_stats.misses,
            pricing_size: pricing_stats.size,
            pricing_hit_rate: pricing_stats.hit_rate,
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        self.search_cache.reset_stats();
        self.pricing_cache.reset_stats();
    }
}

impl Default for GdsCache {
    fn default() -> Self {
        Self::new()
    }
}

/// GDS cache statistics
#[derive(Debug, Clone)]
pub struct GdsCacheStats {
    /// Search cache hits
    pub search_hits: u64,
    /// Search cache misses
    pub search_misses: u64,
    /// Search cache size
    pub search_size: usize,
    /// Search cache hit rate
    pub search_hit_rate: f64,
    /// Pricing cache hits
    pub pricing_hits: u64,
    /// Pricing cache misses
    pub pricing_misses: u64,
    /// Pricing cache size
    pub pricing_size: usize,
    /// Pricing cache hit rate
    pub pricing_hit_rate: f64,
}

impl GdsCacheStats {
    /// Overall hit rate
    #[must_use]
    pub fn overall_hit_rate(&self) -> f64 {
        let total_hits = self.search_hits + self.pricing_hits;
        let total_requests =
            self.search_hits + self.search_misses + self.pricing_hits + self.pricing_misses;

        if total_requests > 0 {
            total_hits as f64 / total_requests as f64
        } else {
            0.0
        }
    }

    /// Total cache size
    #[must_use]
    pub fn total_size(&self) -> usize {
        self.search_size + self.pricing_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use vaya_common::{AirlineCode, CurrencyCode, IataCode, MinorUnits, Price, Timestamp};

    fn create_test_offer(id: &str) -> FlightOffer {
        let segment = FlightSegment {
            departure: FlightPoint::new(IataCode::KUL, Timestamp::now()),
            arrival: FlightPoint::new(IataCode::NRT, Timestamp::now().add_hours(7)),
            airline: AirlineCode::MH,
            flight_number: "88".to_string(),
            duration_minutes: 420,
            aircraft: None,
            cabin_class: CabinClass::Economy,
            booking_class: None,
            stops: 0,
        };

        FlightOffer {
            id: id.to_string(),
            outbound: Itinerary {
                segments: vec![segment],
                total_duration_minutes: 420,
            },
            return_itinerary: None,
            price: PriceBreakdown::simple(
                Price::new(MinorUnits::new(50000), CurrencyCode::MYR),
                Price::new(MinorUnits::new(5000), CurrencyCode::MYR),
            ),
            validating_airline: AirlineCode::MH,
            available_seats: Some(9),
            created_at: Timestamp::now(),
            expires_at: Some(Timestamp::now().add_mins(30)),
            instant_ticketing: true,
            fare_rules: None,
        }
    }

    #[test]
    fn test_cache_creation() {
        let cache = GdsCache::new();
        assert_eq!(cache.stats().search_size, 0);
        assert_eq!(cache.stats().pricing_size, 0);
    }

    #[test]
    fn test_search_cache() {
        let cache = GdsCache::new();
        let offers = vec![create_test_offer("OFFER1"), create_test_offer("OFFER2")];

        // Cache miss
        assert!(cache.get_search("test-key").is_none());

        // Cache put
        cache.put_search("test-key", offers.clone());

        // Cache hit
        let cached = cache.get_search("test-key");
        assert!(cached.is_some());
        assert_eq!(cached.as_ref().map(|v| v.len()), Some(2));
    }

    #[test]
    fn test_pricing_cache() {
        let cache = GdsCache::new();
        let offer = create_test_offer("OFFER1");

        // Cache miss
        assert!(cache.get_pricing("OFFER1").is_none());

        // Cache put
        cache.put_pricing("OFFER1", offer.clone());

        // Cache hit
        let cached = cache.get_pricing("OFFER1");
        assert!(cached.is_some());
        assert_eq!(cached.as_ref().map(|o| o.id.as_str()), Some("OFFER1"));
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = GdsCache::new();
        let offers = vec![create_test_offer("OFFER1")];

        cache.put_search("test-key", offers);
        assert!(cache.get_search("test-key").is_some());

        cache.invalidate_search("test-key");
        assert!(cache.get_search("test-key").is_none());
    }

    #[test]
    fn test_cache_clear() {
        let cache = GdsCache::new();

        cache.put_search("key1", vec![create_test_offer("O1")]);
        cache.put_pricing("O1", create_test_offer("O1"));

        assert!(cache.stats().total_size() > 0);

        cache.clear();
        assert_eq!(cache.stats().total_size(), 0);
    }

    #[test]
    fn test_cache_stats() {
        let cache = GdsCache::new();

        // Generate some hits and misses
        cache.get_search("miss1");
        cache.get_search("miss2");

        cache.put_search("hit", vec![create_test_offer("O1")]);
        cache.get_search("hit");
        cache.get_search("hit");

        let stats = cache.stats();
        assert_eq!(stats.search_misses, 2);
        assert_eq!(stats.search_hits, 2);
        assert!(stats.search_hit_rate > 0.0);
    }
}
