//! vaya-store: Relational layer on top of vaya-db
//!
//! This crate provides table-like abstractions, schemas, indexing,
//! and query capabilities on top of the LSM-tree storage engine.

pub mod error;
pub mod index;
pub mod query;
pub mod schema;
pub mod table;

pub use error::{StoreError, StoreResult};
pub use index::{Index, IndexType};
pub use query::{Query, QueryBuilder};
pub use schema::{Column, ColumnType, Schema};
pub use table::Table;

/// Store version for compatibility checking
pub const STORE_VERSION: u32 = 1;

/// Key prefix for table metadata
pub const TABLE_META_PREFIX: &[u8] = b"_meta_table_";

/// Key prefix for index data
pub const INDEX_PREFIX: &[u8] = b"_idx_";

/// Key prefix for schema definitions
pub const SCHEMA_PREFIX: &[u8] = b"_schema_";
