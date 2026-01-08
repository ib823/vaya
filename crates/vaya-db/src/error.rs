//! Database error types

use std::fmt;
use std::io;
use vaya_common::VayaError;

/// Result type for database operations
pub type DbResult<T> = Result<T, DbError>;

/// Database error types
#[derive(Debug)]
pub enum DbError {
    /// I/O error during file operations
    Io(io::Error),
    /// Data corruption detected
    Corruption(String),
    /// Key not found
    NotFound,
    /// Invalid configuration
    InvalidConfig(String),
    /// WAL is corrupted or invalid
    WalCorruption(String),
    /// Serialization/deserialization error
    Serialization(String),
    /// Database is closed
    Closed,
    /// Compaction error
    Compaction(String),
    /// Invalid key format
    InvalidKey(String),
    /// Value too large
    ValueTooLarge { size: usize, max: usize },
    /// Database version mismatch
    VersionMismatch { expected: u32, found: u32 },
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::Io(e) => write!(f, "I/O error: {}", e),
            DbError::Corruption(msg) => write!(f, "Data corruption: {}", msg),
            DbError::NotFound => write!(f, "Key not found"),
            DbError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            DbError::WalCorruption(msg) => write!(f, "WAL corruption: {}", msg),
            DbError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            DbError::Closed => write!(f, "Database is closed"),
            DbError::Compaction(msg) => write!(f, "Compaction error: {}", msg),
            DbError::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            DbError::ValueTooLarge { size, max } => {
                write!(f, "Value too large: {} bytes (max: {} bytes)", size, max)
            }
            DbError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}

impl std::error::Error for DbError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DbError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for DbError {
    fn from(err: io::Error) -> Self {
        DbError::Io(err)
    }
}

impl From<DbError> for VayaError {
    fn from(err: DbError) -> Self {
        VayaError::new(vaya_common::ErrorCode::DatabaseError, err.to_string())
    }
}
