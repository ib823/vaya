//! VAYA Common - Core types, enums, and error handling
//!
//! This crate provides the foundational types used throughout VAYA.
//! All types are designed for zero-copy serialization with rkyv.
//!
//! # Architecture
//!
//! VAYA uses a 100% sovereign architecture with ZERO external database dependencies:
//! - VayaDB: Custom LSM-tree + B+Tree hybrid storage engine
//! - VayaCache: Custom sharded LRU cache
//! - rkyv: Zero-copy serialization
//!
//! # Modules
//!
//! - `types`: Core primitive types (IataCode, Price, Timestamp, Uuid, etc.)
//! - `enums`: Domain enums (UserStatus, BookingStatus, PoolStatus, etc.)
//! - `error`: Error types and error codes

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod enums;
pub mod error;
pub mod types;

// Re-export commonly used types at crate root
pub use enums::*;
pub use error::{ErrorCode, FieldError, Result, ValidationError, VayaError};
pub use types::*;

/// Version of the VAYA protocol
pub const VAYA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build timestamp (set at compile time)
pub const BUILD_TIMESTAMP: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    //! Convenience re-exports of commonly used types
    pub use crate::enums::*;
    pub use crate::error::{ErrorCode, Result, VayaError};
    pub use crate::types::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VAYA_VERSION.is_empty());
    }

    #[test]
    fn test_prelude() {
        use prelude::*;

        let route = Route::from_codes("KUL", "NRT");
        assert!(route.is_valid());

        let price = Price::myr(15000);
        assert_eq!(price.currency, CurrencyCode::MYR);
    }
}
