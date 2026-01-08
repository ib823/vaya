//! Error types for the store layer

use std::fmt;
use vaya_db::DbError;

/// Store operation result type
pub type StoreResult<T> = Result<T, StoreError>;

/// Store error types
#[derive(Debug)]
pub enum StoreError {
    /// Underlying database error
    Database(DbError),
    /// Table not found
    TableNotFound(String),
    /// Table already exists
    TableExists(String),
    /// Schema mismatch
    SchemaMismatch(String),
    /// Invalid column type
    InvalidColumnType(String),
    /// Column not found
    ColumnNotFound(String),
    /// Index not found
    IndexNotFound(String),
    /// Index already exists
    IndexExists(String),
    /// Primary key violation
    PrimaryKeyViolation,
    /// Unique constraint violation
    UniqueViolation(String),
    /// Null constraint violation
    NullViolation(String),
    /// Invalid query
    InvalidQuery(String),
    /// Serialization error
    Serialization(String),
    /// Record not found
    NotFound,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Database(e) => write!(f, "Database error: {}", e),
            StoreError::TableNotFound(name) => write!(f, "Table not found: {}", name),
            StoreError::TableExists(name) => write!(f, "Table already exists: {}", name),
            StoreError::SchemaMismatch(msg) => write!(f, "Schema mismatch: {}", msg),
            StoreError::InvalidColumnType(msg) => write!(f, "Invalid column type: {}", msg),
            StoreError::ColumnNotFound(name) => write!(f, "Column not found: {}", name),
            StoreError::IndexNotFound(name) => write!(f, "Index not found: {}", name),
            StoreError::IndexExists(name) => write!(f, "Index already exists: {}", name),
            StoreError::PrimaryKeyViolation => write!(f, "Primary key violation"),
            StoreError::UniqueViolation(col) => write!(f, "Unique constraint violation on: {}", col),
            StoreError::NullViolation(col) => write!(f, "Null constraint violation on: {}", col),
            StoreError::InvalidQuery(msg) => write!(f, "Invalid query: {}", msg),
            StoreError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            StoreError::NotFound => write!(f, "Record not found"),
        }
    }
}

impl std::error::Error for StoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StoreError::Database(e) => Some(e),
            _ => None,
        }
    }
}

impl From<DbError> for StoreError {
    fn from(err: DbError) -> Self {
        StoreError::Database(err)
    }
}
