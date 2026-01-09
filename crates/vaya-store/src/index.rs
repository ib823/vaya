//! Index management for efficient lookups

use rkyv::{Archive, Deserialize, Serialize};

use crate::schema::Value;

/// Index types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub enum IndexType {
    /// B-tree style index for range queries (implemented via sorted keys)
    BTree,
    /// Hash-based index for exact matches
    Hash,
    /// Unique index (like BTree but enforces uniqueness)
    Unique,
}

/// Index definition
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Index {
    /// Index name
    pub name: String,
    /// Table name
    pub table_name: String,
    /// Column being indexed
    pub column_name: String,
    /// Index type
    pub index_type: IndexType,
}

impl Index {
    /// Create a new index
    pub fn new(
        name: impl Into<String>,
        table_name: impl Into<String>,
        column_name: impl Into<String>,
        index_type: IndexType,
    ) -> Self {
        Self {
            name: name.into(),
            table_name: table_name.into(),
            column_name: column_name.into(),
            index_type,
        }
    }

    /// Create a BTree index
    pub fn btree(
        name: impl Into<String>,
        table_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> Self {
        Self::new(name, table_name, column_name, IndexType::BTree)
    }

    /// Create a unique index
    pub fn unique(
        name: impl Into<String>,
        table_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> Self {
        Self::new(name, table_name, column_name, IndexType::Unique)
    }

    /// Create a hash index
    pub fn hash(
        name: impl Into<String>,
        table_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> Self {
        Self::new(name, table_name, column_name, IndexType::Hash)
    }

    /// Generate the index key prefix for this index
    pub fn key_prefix(&self) -> Vec<u8> {
        let mut key = crate::INDEX_PREFIX.to_vec();
        key.extend_from_slice(self.table_name.as_bytes());
        key.push(b'/');
        key.extend_from_slice(self.name.as_bytes());
        key.push(b'/');
        key
    }

    /// Generate the full index key for a value
    pub fn key_for_value(&self, value: &Value, primary_key: &[u8]) -> Vec<u8> {
        let mut key = self.key_prefix();
        key.extend_from_slice(&self.encode_value(value));
        key.push(b'/');
        key.extend_from_slice(primary_key);
        key
    }

    /// Encode a value for use in an index key (sortable encoding)
    fn encode_value(&self, value: &Value) -> Vec<u8> {
        match value {
            Value::Null => vec![0],
            Value::Int64(v) => {
                // Use XOR with sign bit for sortable encoding
                let sortable = (*v as u64) ^ (1u64 << 63);
                let mut bytes = vec![1];
                bytes.extend_from_slice(&sortable.to_be_bytes());
                bytes
            }
            Value::Float64(v) => {
                // IEEE 754 sortable encoding
                let bits = v.to_bits();
                let sortable = if (bits & (1u64 << 63)) != 0 {
                    !bits
                } else {
                    bits ^ (1u64 << 63)
                };
                let mut bytes = vec![2];
                bytes.extend_from_slice(&sortable.to_be_bytes());
                bytes
            }
            Value::Float32(v) => {
                let bits = v.to_bits();
                let sortable = if (bits & (1u32 << 31)) != 0 {
                    !bits
                } else {
                    bits ^ (1u32 << 31)
                };
                let mut bytes = vec![3];
                bytes.extend_from_slice(&sortable.to_be_bytes());
                bytes
            }
            Value::String(s) => {
                let mut bytes = vec![4];
                bytes.extend_from_slice(s.as_bytes());
                bytes
            }
            Value::Bytes(b) => {
                let mut bytes = vec![5];
                bytes.extend_from_slice(b);
                bytes
            }
            Value::Bool(b) => vec![6, if *b { 1 } else { 0 }],
        }
    }

    /// Check if this is a unique index
    pub fn is_unique(&self) -> bool {
        self.index_type == IndexType::Unique
    }
}

/// Index entry for storage
#[derive(Debug, Clone)]
pub struct IndexEntry {
    /// The indexed value
    pub value: Value,
    /// The primary key of the record
    pub primary_key: Vec<u8>,
}

impl IndexEntry {
    /// Create a new index entry
    pub fn new(value: Value, primary_key: Vec<u8>) -> Self {
        Self { value, primary_key }
    }
}

/// Composite index (multiple columns)
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct CompositeIndex {
    /// Index name
    pub name: String,
    /// Table name
    pub table_name: String,
    /// Columns being indexed (in order)
    pub column_names: Vec<String>,
    /// Whether the index is unique
    pub unique: bool,
}

impl CompositeIndex {
    /// Create a new composite index
    pub fn new(
        name: impl Into<String>,
        table_name: impl Into<String>,
        column_names: Vec<String>,
        unique: bool,
    ) -> Self {
        Self {
            name: name.into(),
            table_name: table_name.into(),
            column_names,
            unique,
        }
    }

    /// Generate the index key prefix
    pub fn key_prefix(&self) -> Vec<u8> {
        let mut key = crate::INDEX_PREFIX.to_vec();
        key.extend_from_slice(self.table_name.as_bytes());
        key.push(b'/');
        key.extend_from_slice(self.name.as_bytes());
        key.push(b'/');
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_creation() {
        let idx = Index::btree("idx_name", "users", "name");
        assert_eq!(idx.name, "idx_name");
        assert_eq!(idx.index_type, IndexType::BTree);
        assert!(!idx.is_unique());

        let unique_idx = Index::unique("idx_email", "users", "email");
        assert!(unique_idx.is_unique());
    }

    #[test]
    fn test_index_key_encoding() {
        let idx = Index::btree("idx_id", "users", "id");

        // Test sortable encoding for integers
        let key1 = idx.encode_value(&Value::Int64(-10));
        let key2 = idx.encode_value(&Value::Int64(0));
        let key3 = idx.encode_value(&Value::Int64(10));

        // Keys should be ordered correctly
        assert!(key1 < key2);
        assert!(key2 < key3);
    }

    #[test]
    fn test_index_key_prefix() {
        let idx = Index::btree("idx_name", "users", "name");
        let prefix = idx.key_prefix();

        assert!(prefix.starts_with(crate::INDEX_PREFIX));
        assert!(String::from_utf8_lossy(&prefix).contains("users"));
        assert!(String::from_utf8_lossy(&prefix).contains("idx_name"));
    }
}
