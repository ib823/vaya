//! In-memory table (MemTable) using a lock-free skip list
//!
//! The MemTable is the write buffer for the LSM-tree. All writes go here first,
//! then are flushed to SSTables when the MemTable reaches a size threshold.

use crossbeam_skiplist::SkipMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Internal key format: user_key + sequence_number + value_type
/// This allows multiple versions of the same key to exist
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternalKey {
    /// The user-visible key
    pub user_key: Vec<u8>,
    /// Sequence number (higher = newer)
    pub sequence: u64,
    /// Type of value (Put or Delete)
    pub value_type: ValueType,
}

/// Type of value stored
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ValueType {
    /// A deletion marker (tombstone)
    Delete = 0,
    /// A value
    Put = 1,
}

impl InternalKey {
    /// Create a new internal key for a Put operation
    pub fn put(user_key: Vec<u8>, sequence: u64) -> Self {
        Self {
            user_key,
            sequence,
            value_type: ValueType::Put,
        }
    }

    /// Create a new internal key for a Delete operation
    pub fn delete(user_key: Vec<u8>, sequence: u64) -> Self {
        Self {
            user_key,
            sequence,
            value_type: ValueType::Delete,
        }
    }

    /// Encode the internal key to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.user_key.len() + 9);
        buf.extend_from_slice(&self.user_key);
        // Store sequence in big-endian so keys sort correctly
        // (higher sequence = newer, should come first for same user_key)
        // We invert the sequence so that newer keys sort before older
        let inverted_seq = u64::MAX - self.sequence;
        buf.extend_from_slice(&inverted_seq.to_be_bytes());
        buf.push(self.value_type as u8);
        buf
    }

    /// Decode an internal key from bytes
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 9 {
            return None;
        }
        let user_key_len = data.len() - 9;
        let user_key = data[..user_key_len].to_vec();
        let inverted_seq =
            u64::from_be_bytes(data[user_key_len..user_key_len + 8].try_into().ok()?);
        let sequence = u64::MAX - inverted_seq;
        let value_type = match data[user_key_len + 8] {
            0 => ValueType::Delete,
            1 => ValueType::Put,
            _ => return None,
        };
        Some(Self {
            user_key,
            sequence,
            value_type,
        })
    }
}

/// In-memory sorted table using a skip list
pub struct MemTable {
    /// The skip list storing key-value pairs
    map: SkipMap<Vec<u8>, Vec<u8>>,
    /// Approximate size of the memtable in bytes
    size: AtomicUsize,
}

impl MemTable {
    /// Create a new empty MemTable
    pub fn new() -> Self {
        Self {
            map: SkipMap::new(),
            size: AtomicUsize::new(0),
        }
    }

    /// Insert a key-value pair
    pub fn put(&self, key: &[u8], value: &[u8], sequence: u64) {
        let internal_key = InternalKey::put(key.to_vec(), sequence);
        let encoded_key = internal_key.encode();
        let size_delta = encoded_key.len() + value.len();

        self.map.insert(encoded_key, value.to_vec());
        self.size.fetch_add(size_delta, Ordering::Relaxed);
    }

    /// Mark a key as deleted
    pub fn delete(&self, key: &[u8], sequence: u64) {
        let internal_key = InternalKey::delete(key.to_vec(), sequence);
        let encoded_key = internal_key.encode();
        let size_delta = encoded_key.len();

        self.map.insert(encoded_key, Vec::new());
        self.size.fetch_add(size_delta, Ordering::Relaxed);
    }

    /// Get a value by key (returns the most recent version)
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        // Since sequence is inverted (u64::MAX - seq), the highest real sequence has the lowest
        // inverted value. So entries are sorted: newest first (smallest inverted) to oldest.
        // We create a start key with u64::MAX sequence (inverted=0, smallest possible)
        // and iterate forward to find the first entry matching our key.
        let start_key = InternalKey::put(key.to_vec(), u64::MAX).encode();
        let end_key = InternalKey::put(key.to_vec(), 0).encode();

        // Iterate forward from start_key - first matching entry is the newest
        for entry in self.map.range(start_key..=end_key) {
            let internal_key = InternalKey::decode(entry.key())?;
            if internal_key.user_key == key {
                match internal_key.value_type {
                    ValueType::Put => return Some(entry.value().clone()),
                    ValueType::Delete => return None, // Key was deleted
                }
            }
        }
        None
    }

    /// Check if the memtable contains a key
    pub fn contains(&self, key: &[u8]) -> bool {
        self.get(key).is_some()
    }

    /// Get the approximate size of the memtable in bytes
    pub fn size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    /// Check if the memtable is empty
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Get the number of entries in the memtable
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Iterate over all entries in sorted order
    pub fn iter(&self) -> impl Iterator<Item = (InternalKey, Vec<u8>)> + '_ {
        self.map.iter().filter_map(|entry| {
            let key = InternalKey::decode(entry.key())?;
            Some((key, entry.value().clone()))
        })
    }

    /// Iterate over entries in a key range
    pub fn range(
        &self,
        start: &[u8],
        end: &[u8],
    ) -> impl Iterator<Item = (InternalKey, Vec<u8>)> + '_ {
        let start_key = InternalKey::put(start.to_vec(), u64::MAX).encode();
        let end_key = InternalKey::put(end.to_vec(), 0).encode();

        self.map.range(start_key..=end_key).filter_map(|entry| {
            let key = InternalKey::decode(entry.key())?;
            Some((key, entry.value().clone()))
        })
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_key_encoding() {
        let key = InternalKey::put(b"hello".to_vec(), 42);
        let encoded = key.encode();
        let decoded = InternalKey::decode(&encoded).unwrap();

        assert_eq!(decoded.user_key, b"hello");
        assert_eq!(decoded.sequence, 42);
        assert_eq!(decoded.value_type, ValueType::Put);
    }

    #[test]
    fn test_memtable_put_get() {
        let mt = MemTable::new();
        mt.put(b"key1", b"value1", 1);
        mt.put(b"key2", b"value2", 2);

        assert_eq!(mt.get(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(mt.get(b"key2"), Some(b"value2".to_vec()));
        assert_eq!(mt.get(b"key3"), None);
    }

    #[test]
    fn test_memtable_delete() {
        let mt = MemTable::new();
        mt.put(b"key1", b"value1", 1);
        mt.delete(b"key1", 2);

        assert_eq!(mt.get(b"key1"), None);
    }

    #[test]
    fn test_memtable_versioning() {
        let mt = MemTable::new();
        mt.put(b"key", b"v1", 1);
        mt.put(b"key", b"v2", 2);
        mt.put(b"key", b"v3", 3);

        // Should get the latest version
        assert_eq!(mt.get(b"key"), Some(b"v3".to_vec()));
    }

    #[test]
    fn test_memtable_size() {
        let mt = MemTable::new();
        assert_eq!(mt.size(), 0);

        mt.put(b"key", b"value", 1);
        assert!(mt.size() > 0);
    }
}
