//! Schema definitions for tables

use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

use crate::{StoreError, StoreResult};

/// Column data types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub enum ColumnType {
    /// 64-bit signed integer
    Int64,
    /// 32-bit floating point
    Float32,
    /// 64-bit floating point
    Float64,
    /// UTF-8 string
    String,
    /// Binary data
    Bytes,
    /// Boolean
    Bool,
    /// Unix timestamp (milliseconds)
    Timestamp,
    /// UUID (128 bits stored as bytes)
    Uuid,
}

impl ColumnType {
    /// Get the type name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            ColumnType::Int64 => "int64",
            ColumnType::Float32 => "float32",
            ColumnType::Float64 => "float64",
            ColumnType::String => "string",
            ColumnType::Bytes => "bytes",
            ColumnType::Bool => "bool",
            ColumnType::Timestamp => "timestamp",
            ColumnType::Uuid => "uuid",
        }
    }

    /// Parse a type from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "int64" | "integer" | "bigint" => Some(ColumnType::Int64),
            "float32" | "float" => Some(ColumnType::Float32),
            "float64" | "double" => Some(ColumnType::Float64),
            "string" | "text" | "varchar" => Some(ColumnType::String),
            "bytes" | "blob" | "binary" => Some(ColumnType::Bytes),
            "bool" | "boolean" => Some(ColumnType::Bool),
            "timestamp" | "datetime" => Some(ColumnType::Timestamp),
            "uuid" => Some(ColumnType::Uuid),
            _ => None,
        }
    }
}

/// Column definition
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Column {
    /// Column name
    pub name: String,
    /// Column type
    pub column_type: ColumnType,
    /// Whether the column is nullable
    pub nullable: bool,
    /// Whether the column is the primary key
    pub primary_key: bool,
    /// Whether the column has a unique constraint
    pub unique: bool,
    /// Default value (serialized)
    pub default: Option<Vec<u8>>,
}

impl Column {
    /// Create a new column definition
    pub fn new(name: impl Into<String>, column_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            column_type,
            nullable: true,
            primary_key: false,
            unique: false,
            default: None,
        }
    }

    /// Mark column as not nullable
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Mark column as primary key
    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self.nullable = false;
        self.unique = true;
        self
    }

    /// Mark column as unique
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Set a default value
    pub fn default(mut self, value: Vec<u8>) -> Self {
        self.default = Some(value);
        self
    }
}

/// Table schema definition
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct Schema {
    /// Schema version for migrations
    pub version: u32,
    /// Table name
    pub table_name: String,
    /// Column definitions
    pub columns: Vec<Column>,
    /// Column name to index mapping
    #[with(rkyv::with::Skip)]
    column_indices: HashMap<String, usize>,
}

impl Schema {
    /// Create a new schema
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            version: 1,
            table_name: table_name.into(),
            columns: Vec::new(),
            column_indices: HashMap::new(),
        }
    }

    /// Add a column to the schema
    pub fn column(mut self, column: Column) -> Self {
        let idx = self.columns.len();
        self.column_indices.insert(column.name.clone(), idx);
        self.columns.push(column);
        self
    }

    /// Get a column by name
    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.column_indices
            .get(name)
            .and_then(|&idx| self.columns.get(idx))
    }

    /// Get the column index by name
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.column_indices.get(name).copied()
    }

    /// Get the primary key column
    pub fn primary_key_column(&self) -> Option<&Column> {
        self.columns.iter().find(|c| c.primary_key)
    }

    /// Get all unique columns
    pub fn unique_columns(&self) -> Vec<&Column> {
        self.columns.iter().filter(|c| c.unique && !c.primary_key).collect()
    }

    /// Validate a record against the schema
    pub fn validate(&self, record: &Record) -> StoreResult<()> {
        for column in &self.columns {
            match record.get(&column.name) {
                Some(value) => {
                    // Check type compatibility
                    if !value.is_compatible_with(column.column_type) {
                        return Err(StoreError::InvalidColumnType(format!(
                            "Column {} expects {:?}, got {:?}",
                            column.name, column.column_type, value
                        )));
                    }
                }
                None => {
                    if !column.nullable && column.default.is_none() {
                        return Err(StoreError::NullViolation(column.name.clone()));
                    }
                }
            }
        }
        Ok(())
    }

    /// Rebuild column indices after deserialization
    pub fn rebuild_indices(&mut self) {
        self.column_indices.clear();
        for (idx, column) in self.columns.iter().enumerate() {
            self.column_indices.insert(column.name.clone(), idx);
        }
    }

    /// Get the number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }
}

/// A value that can be stored in a column
#[derive(Debug, Clone, PartialEq, Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub enum Value {
    /// Null value
    Null,
    /// 64-bit integer
    Int64(i64),
    /// 32-bit float
    Float32(f32),
    /// 64-bit float
    Float64(f64),
    /// String value
    String(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// Boolean
    Bool(bool),
}

impl Value {
    /// Check if this value is compatible with a column type
    pub fn is_compatible_with(&self, column_type: ColumnType) -> bool {
        match (self, column_type) {
            (Value::Null, _) => true, // Null is handled separately
            (Value::Int64(_), ColumnType::Int64) => true,
            (Value::Int64(_), ColumnType::Timestamp) => true,
            (Value::Float32(_), ColumnType::Float32) => true,
            (Value::Float64(_), ColumnType::Float64) => true,
            (Value::String(_), ColumnType::String) => true,
            (Value::Bytes(_), ColumnType::Bytes) => true,
            (Value::Bytes(_), ColumnType::Uuid) => true,
            (Value::Bool(_), ColumnType::Bool) => true,
            _ => false,
        }
    }

    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Value::Null => vec![0],
            Value::Int64(v) => {
                let mut bytes = vec![1];
                bytes.extend_from_slice(&v.to_le_bytes());
                bytes
            }
            Value::Float32(v) => {
                let mut bytes = vec![2];
                bytes.extend_from_slice(&v.to_le_bytes());
                bytes
            }
            Value::Float64(v) => {
                let mut bytes = vec![3];
                bytes.extend_from_slice(&v.to_le_bytes());
                bytes
            }
            Value::String(v) => {
                let mut bytes = vec![4];
                bytes.extend_from_slice(&(v.len() as u32).to_le_bytes());
                bytes.extend_from_slice(v.as_bytes());
                bytes
            }
            Value::Bytes(v) => {
                let mut bytes = vec![5];
                bytes.extend_from_slice(&(v.len() as u32).to_le_bytes());
                bytes.extend_from_slice(v);
                bytes
            }
            Value::Bool(v) => vec![6, if *v { 1 } else { 0 }],
        }
    }

    /// Parse from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        match bytes[0] {
            0 => Some(Value::Null),
            1 if bytes.len() >= 9 => {
                let v = i64::from_le_bytes(bytes[1..9].try_into().ok()?);
                Some(Value::Int64(v))
            }
            2 if bytes.len() >= 5 => {
                let v = f32::from_le_bytes(bytes[1..5].try_into().ok()?);
                Some(Value::Float32(v))
            }
            3 if bytes.len() >= 9 => {
                let v = f64::from_le_bytes(bytes[1..9].try_into().ok()?);
                Some(Value::Float64(v))
            }
            4 if bytes.len() >= 5 => {
                let len = u32::from_le_bytes(bytes[1..5].try_into().ok()?) as usize;
                if bytes.len() >= 5 + len {
                    let s = String::from_utf8(bytes[5..5 + len].to_vec()).ok()?;
                    Some(Value::String(s))
                } else {
                    None
                }
            }
            5 if bytes.len() >= 5 => {
                let len = u32::from_le_bytes(bytes[1..5].try_into().ok()?) as usize;
                if bytes.len() >= 5 + len {
                    Some(Value::Bytes(bytes[5..5 + len].to_vec()))
                } else {
                    None
                }
            }
            6 if bytes.len() >= 2 => Some(Value::Bool(bytes[1] != 0)),
            _ => None,
        }
    }

    /// Get as i64 if possible
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int64(v) => Some(*v),
            _ => None,
        }
    }

    /// Get as string if possible
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(v) => Some(v),
            _ => None,
        }
    }

    /// Get as bytes if possible
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(v) => Some(v),
            _ => None,
        }
    }

    /// Check if this is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// A record (row) in a table
#[derive(Debug, Clone, Default)]
pub struct Record {
    /// Column name to value mapping
    fields: HashMap<String, Value>,
}

impl Record {
    /// Create a new empty record
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }

    /// Set a field value
    pub fn set(&mut self, name: impl Into<String>, value: Value) {
        self.fields.insert(name.into(), value);
    }

    /// Get a field value
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    /// Check if a field exists
    pub fn has(&self, name: &str) -> bool {
        self.fields.contains_key(name)
    }

    /// Get all field names
    pub fn field_names(&self) -> impl Iterator<Item = &str> {
        self.fields.keys().map(|s| s.as_str())
    }

    /// Get the number of fields
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Serialize the record to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Number of fields
        result.extend_from_slice(&(self.fields.len() as u32).to_le_bytes());

        for (name, value) in &self.fields {
            // Field name
            let name_bytes = name.as_bytes();
            result.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
            result.extend_from_slice(name_bytes);

            // Field value
            let value_bytes = value.to_bytes();
            result.extend_from_slice(&(value_bytes.len() as u32).to_le_bytes());
            result.extend_from_slice(&value_bytes);
        }

        result
    }

    /// Deserialize a record from bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 {
            return None;
        }

        let num_fields = u32::from_le_bytes(bytes[0..4].try_into().ok()?) as usize;
        let mut fields = HashMap::with_capacity(num_fields);
        let mut offset = 4;

        for _ in 0..num_fields {
            // Read field name
            if offset + 2 > bytes.len() {
                return None;
            }
            let name_len = u16::from_le_bytes(bytes[offset..offset + 2].try_into().ok()?) as usize;
            offset += 2;

            if offset + name_len > bytes.len() {
                return None;
            }
            let name = String::from_utf8(bytes[offset..offset + name_len].to_vec()).ok()?;
            offset += name_len;

            // Read field value
            if offset + 4 > bytes.len() {
                return None;
            }
            let value_len =
                u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
            offset += 4;

            if offset + value_len > bytes.len() {
                return None;
            }
            let value = Value::from_bytes(&bytes[offset..offset + value_len])?;
            offset += value_len;

            fields.insert(name, value);
        }

        Some(Self { fields })
    }
}

/// Builder for creating records
pub struct RecordBuilder {
    record: Record,
}

impl RecordBuilder {
    /// Create a new record builder
    pub fn new() -> Self {
        Self {
            record: Record::new(),
        }
    }

    /// Set an i64 field
    pub fn int64(mut self, name: impl Into<String>, value: i64) -> Self {
        self.record.set(name, Value::Int64(value));
        self
    }

    /// Set a string field
    pub fn string(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.record.set(name, Value::String(value.into()));
        self
    }

    /// Set a bytes field
    pub fn bytes(mut self, name: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        self.record.set(name, Value::Bytes(value.into()));
        self
    }

    /// Set a bool field
    pub fn bool(mut self, name: impl Into<String>, value: bool) -> Self {
        self.record.set(name, Value::Bool(value));
        self
    }

    /// Set a f64 field
    pub fn float64(mut self, name: impl Into<String>, value: f64) -> Self {
        self.record.set(name, Value::Float64(value));
        self
    }

    /// Set a timestamp field (milliseconds)
    pub fn timestamp(mut self, name: impl Into<String>, value: i64) -> Self {
        self.record.set(name, Value::Int64(value));
        self
    }

    /// Set a null field
    pub fn null(mut self, name: impl Into<String>) -> Self {
        self.record.set(name, Value::Null);
        self
    }

    /// Build the record
    pub fn build(self) -> Record {
        self.record
    }
}

impl Default for RecordBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_definition() {
        let col = Column::new("id", ColumnType::Int64).primary_key();
        assert!(col.primary_key);
        assert!(!col.nullable);
        assert!(col.unique);
    }

    #[test]
    fn test_schema() {
        let schema = Schema::new("users")
            .column(Column::new("id", ColumnType::Int64).primary_key())
            .column(Column::new("name", ColumnType::String).not_null())
            .column(Column::new("email", ColumnType::String).unique());

        assert_eq!(schema.column_count(), 3);
        assert!(schema.get_column("id").is_some());
        assert!(schema.primary_key_column().is_some());
        assert_eq!(schema.unique_columns().len(), 1);
    }

    #[test]
    fn test_value_serialization() {
        let values = vec![
            Value::Null,
            Value::Int64(42),
            Value::Float64(3.14),
            Value::String("hello".to_string()),
            Value::Bytes(vec![1, 2, 3]),
            Value::Bool(true),
        ];

        for value in values {
            let bytes = value.to_bytes();
            let recovered = Value::from_bytes(&bytes).unwrap();
            assert_eq!(value, recovered);
        }
    }

    #[test]
    fn test_record_builder() {
        let record = RecordBuilder::new()
            .int64("id", 1)
            .string("name", "Alice")
            .bool("active", true)
            .build();

        assert_eq!(record.get("id"), Some(&Value::Int64(1)));
        assert_eq!(record.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(record.get("active"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_record_serialization() {
        let record = RecordBuilder::new()
            .int64("id", 42)
            .string("name", "Test")
            .build();

        let bytes = record.to_bytes();
        let recovered = Record::from_bytes(&bytes).unwrap();

        assert_eq!(recovered.get("id"), Some(&Value::Int64(42)));
        assert_eq!(recovered.get("name"), Some(&Value::String("Test".to_string())));
    }
}
