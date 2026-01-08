//! LRU (Least Recently Used) cache implementation

use std::collections::HashMap;
use std::hash::Hash;

/// A node in the LRU doubly-linked list
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

/// A simple LRU cache using a HashMap and doubly-linked list
pub struct LruCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Storage for nodes
    nodes: Vec<Node<K, V>>,
    /// Map from key to node index
    map: HashMap<K, usize>,
    /// Index of the head (most recently used)
    head: Option<usize>,
    /// Index of the tail (least recently used)
    tail: Option<usize>,
    /// Free list of node indices
    free: Vec<usize>,
    /// Maximum capacity
    capacity: usize,
}

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    /// Create a new LRU cache with the given capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            map: HashMap::with_capacity(capacity),
            head: None,
            tail: None,
            free: Vec::new(),
            capacity,
        }
    }

    /// Get a value, updating its position to most recently used
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(&idx) = self.map.get(key) {
            let value = self.nodes[idx].value.clone();
            self.move_to_front(idx);
            Some(value)
        } else {
            None
        }
    }

    /// Peek at a value without updating its position
    pub fn peek(&self, key: &K) -> Option<&V> {
        self.map.get(key).map(|&idx| &self.nodes[idx].value)
    }

    /// Insert a key-value pair, returning true if an eviction occurred
    pub fn insert(&mut self, key: K, value: V) -> bool {
        // If key exists, update value and move to front
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            self.move_to_front(idx);
            return false;
        }

        let evicted = self.map.len() >= self.capacity;

        // If at capacity, evict LRU item
        if evicted {
            self.evict_lru();
        }

        // Get a node index (reuse from free list or allocate new)
        let idx = if let Some(idx) = self.free.pop() {
            self.nodes[idx] = Node {
                key: key.clone(),
                value,
                prev: None,
                next: self.head,
            };
            idx
        } else {
            let idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                prev: None,
                next: self.head,
            });
            idx
        };

        // Update head's prev pointer
        if let Some(head) = self.head {
            self.nodes[head].prev = Some(idx);
        }

        // Update head
        self.head = Some(idx);

        // Update tail if this is the first item
        if self.tail.is_none() {
            self.tail = Some(idx);
        }

        // Add to map
        self.map.insert(key, idx);

        evicted
    }

    /// Remove a key from the cache
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(idx) = self.map.remove(key) {
            let value = self.nodes[idx].value.clone();
            self.unlink(idx);
            self.free.push(idx);
            Some(value)
        } else {
            None
        }
    }

    /// Check if the cache contains a key
    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Get the number of items in the cache
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clear all items from the cache
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.map.clear();
        self.head = None;
        self.tail = None;
        self.free.clear();
    }

    /// Move a node to the front (most recently used)
    fn move_to_front(&mut self, idx: usize) {
        if self.head == Some(idx) {
            return; // Already at front
        }

        self.unlink(idx);

        // Link at front
        self.nodes[idx].prev = None;
        self.nodes[idx].next = self.head;

        if let Some(head) = self.head {
            self.nodes[head].prev = Some(idx);
        }

        self.head = Some(idx);

        if self.tail.is_none() {
            self.tail = Some(idx);
        }
    }

    /// Unlink a node from the list
    fn unlink(&mut self, idx: usize) {
        let (prev, next) = {
            let node = &self.nodes[idx];
            (node.prev, node.next)
        };

        // Update prev's next pointer
        if let Some(prev) = prev {
            self.nodes[prev].next = next;
        } else {
            self.head = next;
        }

        // Update next's prev pointer
        if let Some(next) = next {
            self.nodes[next].prev = prev;
        } else {
            self.tail = prev;
        }
    }

    /// Evict the least recently used item
    fn evict_lru(&mut self) {
        if let Some(tail) = self.tail {
            let key = self.nodes[tail].key.clone();
            self.map.remove(&key);
            self.unlink(tail);
            self.free.push(tail);
        }
    }

    /// Iterate over keys in LRU order (most recent first)
    pub fn keys(&self) -> impl Iterator<Item = &K> {
        LruIterator {
            cache: self,
            current: self.head,
        }
    }
}

/// Iterator over LRU cache keys
struct LruIterator<'a, K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    cache: &'a LruCache<K, V>,
    current: Option<usize>,
}

impl<'a, K, V> Iterator for LruIterator<'a, K, V>
where
    K: Hash + Eq + Clone,
    V: Clone,
{
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.current {
            let node = &self.cache.nodes[idx];
            self.current = node.next;
            Some(&node.key)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lru() {
        let mut cache = LruCache::new(3);

        cache.insert("a", 1);
        cache.insert("b", 2);
        cache.insert("c", 3);

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), Some(2));
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = LruCache::new(2);

        cache.insert("a", 1);
        cache.insert("b", 2);

        // Access "a" to make it most recently used
        cache.get(&"a");

        // Insert "c", should evict "b" (least recently used)
        cache.insert("c", 3);

        assert_eq!(cache.get(&"a"), Some(1));
        assert_eq!(cache.get(&"b"), None); // Evicted
        assert_eq!(cache.get(&"c"), Some(3));
    }

    #[test]
    fn test_update_existing() {
        let mut cache = LruCache::new(3);

        cache.insert("a", 1);
        cache.insert("a", 10);

        assert_eq!(cache.get(&"a"), Some(10));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut cache = LruCache::new(3);

        cache.insert("a", 1);
        cache.insert("b", 2);

        assert_eq!(cache.remove(&"a"), Some(1));
        assert_eq!(cache.get(&"a"), None);
        assert_eq!(cache.len(), 1);
    }
}
