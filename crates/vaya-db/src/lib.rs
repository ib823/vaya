//! VayaDB - Custom LSM-tree Storage Engine
//!
//! A high-performance, zero-copy storage engine optimized for VAYA's workloads:
//! - Time-series data (price observations)
//! - Key-value storage with range queries
//! - Write-ahead logging for durability
//!
//! # Architecture
//!
//! ```text
//! Writes: MemTable (in-memory) -> WAL (durability) -> SSTable (disk)
//! Reads:  MemTable -> L0 SSTables -> L1 SSTables -> ...
//! ```
//!
//! # NO external database dependencies
//! - NO PostgreSQL
//! - NO SQLite
//! - NO Redis
//!
//! Pure Rust, zero-copy with rkyv, LZ4 compression.

#![warn(missing_docs)]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod config;
pub mod engine;
pub mod error;
pub mod memtable;
pub mod sstable;
pub mod wal;

pub use config::DbConfig;
pub use engine::VayaDb;
pub use error::{DbError, DbResult};

/// Database version for compatibility checks
pub const DB_VERSION: u32 = 1;

/// Magic bytes for VayaDB files
pub const MAGIC_BYTES: [u8; 4] = *b"VYDB";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(DB_VERSION, 1);
        assert_eq!(&MAGIC_BYTES, b"VYDB");
    }
}
