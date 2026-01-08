//! Cache shard with TTL support

use crate::lru::LruCache;
use std::hash::Hash;
use std::time::Instant;

/// Cached entry with optional expiration
#[derive(Clone)]
struct Entry<V>
where
    V: Clone,
{
    value: V,
    expires_at: Option<Instant>,
}

impl<V: Clone> Entry<V> {
    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| Instant::now() > exp)
    }
}

/// A single cache shard with LRU eviction and TTL support
pub struct CacheShard<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// LRU cache for ordering
    lru: LruCache<K, Entry<V>>,
}

impl<K, V> CacheShard<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new cache shard with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            lru: LruCache::new(capacity),
        }
    }

    /// Insert a key-value pair with optional expiration, returns true if eviction occurred
    pub fn insert(&mut self, key: K, value: V, expires_at: Option<Instant>) -> bool {
        let entry = Entry { value, expires_at };
        self.lru.insert(key, entry)
    }

    /// Get a value, checking for expiration
    pub fn get(&mut self, key: &K) -> Option<V> {
        // Get from LRU and check expiration
        if let Some(entry) = self.lru.get(key) {
            if entry.is_expired() {
                // Remove expired entry
                self.lru.remove(key);
                None
            } else {
                Some(entry.value)
            }
        } else {
            None
        }
    }

    /// Check if a key exists and is not expired
    pub fn contains(&self, key: &K) -> bool {
        if let Some(entry) = self.lru.peek(key) {
            !entry.is_expired()
        } else {
            false
        }
    }

    /// Remove a key from the shard
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.lru.remove(key).map(|entry| entry.value)
    }

    /// Get the number of items in the shard
    pub fn len(&self) -> usize {
        self.lru.len()
    }

    /// Check if the shard is empty
    pub fn is_empty(&self) -> bool {
        self.lru.is_empty()
    }

    /// Clear all items from the shard
    pub fn clear(&mut self) {
        self.lru.clear();
    }

    /// Purge expired entries, returning the number purged
    pub fn purge_expired(&mut self) -> usize {
        // Collect keys to remove (we can't mutate while iterating)
        let expired_keys: Vec<K> = self
            .lru
            .keys()
            .filter_map(|k| {
                if let Some(entry) = self.lru.peek(k) {
                    if entry.is_expired() {
                        return Some(k.clone());
                    }
                }
                None
            })
            .collect();

        let count = expired_keys.len();
        for key in expired_keys {
            self.lru.remove(&key);
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_shard_basic() {
        let mut shard: CacheShard<String, i32> = CacheShard::new(10);

        shard.insert("key1".to_string(), 42, None);
        assert_eq!(shard.get(&"key1".to_string()), Some(42));
    }

    #[test]
    fn test_shard_ttl() {
        let mut shard: CacheShard<String, i32> = CacheShard::new(10);

        let expires_at = Instant::now() + Duration::from_millis(50);
        shard.insert("key1".to_string(), 42, Some(expires_at));

        // Should exist immediately
        assert!(shard.contains(&"key1".to_string()));

        // Wait for expiration
        thread::sleep(Duration::from_millis(100));

        // Should be expired
        assert_eq!(shard.get(&"key1".to_string()), None);
    }

    #[test]
    fn test_purge_expired() {
        let mut shard: CacheShard<String, i32> = CacheShard::new(10);

        let expires_at = Instant::now() + Duration::from_millis(50);
        shard.insert("key1".to_string(), 1, Some(expires_at));
        shard.insert("key2".to_string(), 2, None); // No expiration

        thread::sleep(Duration::from_millis(100));

        let purged = shard.purge_expired();
        assert_eq!(purged, 1);
        assert_eq!(shard.len(), 1);
        assert!(shard.contains(&"key2".to_string()));
    }
}
