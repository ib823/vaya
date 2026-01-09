//! VayaCache - Custom Sharded LRU Cache with TTL
//!
//! A high-performance, thread-safe cache with:
//! - Sharded design for concurrent access
//! - LRU eviction policy
//! - TTL (time-to-live) support
//! - Zero external dependencies (no Redis!)
//!
//! # Example
//!
//! ```rust
//! use vaya_cache::Cache;
//! use std::time::Duration;
//!
//! let cache: Cache<String, String> = Cache::new(1000, 16);
//!
//! cache.insert("key1".to_string(), "value1".to_string(), Some(Duration::from_secs(60)));
//! assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
//! ```

#![warn(missing_docs)]

mod lru;
mod shard;

use parking_lot::RwLock;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub use lru::LruCache;
pub use shard::CacheShard;

/// A thread-safe, sharded LRU cache with TTL support
pub struct Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// The cache shards
    shards: Vec<RwLock<CacheShard<K, V>>>,
    /// Number of shards (power of 2 for fast modulo)
    shard_count: usize,
    /// Cache hits counter
    hits: AtomicU64,
    /// Cache misses counter
    misses: AtomicU64,
    /// Total items evicted
    evictions: AtomicU64,
}

impl<K, V> Cache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new cache with the given capacity and number of shards
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of items across all shards
    /// * `num_shards` - Number of shards (will be rounded up to power of 2)
    pub fn new(capacity: usize, num_shards: usize) -> Self {
        // Round up to power of 2 for fast modulo
        let shard_count = num_shards.next_power_of_two();
        let per_shard_capacity = (capacity / shard_count).max(1);

        let shards = (0..shard_count)
            .map(|_| RwLock::new(CacheShard::new(per_shard_capacity)))
            .collect();

        Self {
            shards,
            shard_count,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
        }
    }

    /// Insert a key-value pair with an optional TTL
    pub fn insert(&self, key: K, value: V, ttl: Option<Duration>) {
        let shard_idx = self.shard_index(&key);
        let expires_at = ttl.map(|d| Instant::now() + d);

        let mut shard = self.shards[shard_idx].write();
        if shard.insert(key, value, expires_at) {
            self.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get a value by key, returning None if not found or expired
    pub fn get(&self, key: &K) -> Option<V> {
        let shard_idx = self.shard_index(key);

        let mut shard = self.shards[shard_idx].write();
        match shard.get(key) {
            Some(value) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(value)
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Check if a key exists and is not expired
    pub fn contains(&self, key: &K) -> bool {
        let shard_idx = self.shard_index(key);
        let shard = self.shards[shard_idx].read();
        shard.contains(key)
    }

    /// Remove a key from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        let shard_idx = self.shard_index(key);
        let mut shard = self.shards[shard_idx].write();
        shard.remove(key)
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        for shard in &self.shards {
            shard.write().clear();
        }
    }

    /// Get the total number of items in the cache
    pub fn len(&self) -> usize {
        self.shards.iter().map(|s| s.read().len()).sum()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.shards.iter().all(|s| s.read().is_empty())
    }

    /// Purge expired entries from all shards
    pub fn purge_expired(&self) -> usize {
        let mut purged = 0;
        for shard in &self.shards {
            purged += shard.write().purge_expired();
        }
        purged
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let evictions = self.evictions.load(Ordering::Relaxed);
        let size = self.len();

        CacheStats {
            hits,
            misses,
            evictions,
            size,
            hit_rate: if hits + misses > 0 {
                hits as f64 / (hits + misses) as f64
            } else {
                0.0
            },
        }
    }

    /// Reset statistics counters
    pub fn reset_stats(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
    }

    /// Get the shard index for a key
    fn shard_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish() as usize & (self.shard_count - 1)
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Number of items evicted
    pub evictions: u64,
    /// Current number of items
    pub size: usize,
    /// Hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

/// Type alias for string-keyed cache
pub type StringCache<V> = Cache<String, V>;

/// Create a new string-keyed cache
pub fn string_cache<V: Clone>(capacity: usize) -> StringCache<V> {
    Cache::new(capacity, 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_operations() {
        let cache: Cache<String, String> = Cache::new(100, 4);

        cache.insert("key1".to_string(), "value1".to_string(), None);
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), None);
    }

    #[test]
    fn test_ttl() {
        let cache: Cache<String, String> = Cache::new(100, 4);

        cache.insert(
            "key1".to_string(),
            "value1".to_string(),
            Some(Duration::from_millis(50)),
        );

        // Should exist immediately
        assert!(cache.contains(&"key1".to_string()));

        // Wait for expiration
        thread::sleep(Duration::from_millis(100));

        // Should be expired
        assert_eq!(cache.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_eviction() {
        let cache: Cache<i32, i32> = Cache::new(4, 1); // 4 items, 1 shard

        // Insert more than capacity
        for i in 0..10 {
            cache.insert(i, i * 10, None);
        }

        // Should have evicted some items
        let stats = cache.stats();
        assert!(stats.evictions > 0);
        assert!(stats.size <= 4);
    }

    #[test]
    fn test_concurrent_access() {
        let cache: Cache<i32, i32> = Cache::new(1000, 16);
        let cache = std::sync::Arc::new(cache);

        let mut handles = vec![];

        // Spawn multiple writers
        for t in 0..4 {
            let cache = cache.clone();
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    cache.insert(t * 100 + i, i, None);
                }
            }));
        }

        // Spawn multiple readers
        for _ in 0..4 {
            let cache = cache.clone();
            handles.push(thread::spawn(move || {
                for i in 0..400 {
                    let _ = cache.get(&i);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert!(cache.len() > 0);
    }

    #[test]
    fn test_stats() {
        let cache: Cache<i32, i32> = Cache::new(100, 4);

        cache.insert(1, 10, None);
        cache.get(&1);
        cache.get(&2); // miss

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
}
