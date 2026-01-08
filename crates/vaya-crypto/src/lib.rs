//! VAYA Crypto - Cryptographic primitives using ring
//!
//! This crate provides all cryptographic functionality for VAYA:
//! - Password hashing (PBKDF2-HMAC-SHA256)
//! - JWT tokens (HMAC-SHA256)
//! - Random number generation
//! - HMAC
//! - AES-GCM encryption
//! - SHA-256/384/512 hashing
//!
//! # Architecture
//!
//! All crypto uses `ring` as the underlying primitive library.
//! NO OpenSSL dependency - pure Rust.

#![warn(missing_docs)]

pub mod hash;
pub mod hmac;
pub mod jwt;
pub mod password;
pub mod random;
pub mod aead;

pub use hash::*;
pub use hmac::*;
pub use jwt::*;
pub use password::*;
pub use random::*;
pub use aead::*;

use vaya_common::{Result, VayaError, ErrorCode};

/// Crypto-specific error helper
pub fn crypto_error(msg: impl Into<String>) -> VayaError {
    VayaError::new(ErrorCode::CryptoError, msg)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_loads() {
        // Basic sanity test
        assert!(true);
    }
}
