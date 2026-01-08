//! Cryptographic hash functions using ring

use ring::digest::{self, Algorithm, SHA256, SHA384, SHA512};
use crate::random::hex_encode;

/// SHA-256 hash output (32 bytes)
pub struct Sha256Hash([u8; 32]);

impl Sha256Hash {
    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get as byte slice
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        hex_encode(&self.0)
    }
}

impl AsRef<[u8]> for Sha256Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// SHA-384 hash output (48 bytes)
pub struct Sha384Hash([u8; 48]);

impl Sha384Hash {
    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 48]) -> Self {
        Self(bytes)
    }

    /// Get as byte slice
    pub fn as_bytes(&self) -> &[u8; 48] {
        &self.0
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        hex_encode(&self.0)
    }
}

impl AsRef<[u8]> for Sha384Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// SHA-512 hash output (64 bytes)
pub struct Sha512Hash([u8; 64]);

impl Sha512Hash {
    /// Create from raw bytes
    pub fn from_bytes(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Get as byte slice
    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }

    /// Get as hex string
    pub fn to_hex(&self) -> String {
        hex_encode(&self.0)
    }
}

impl AsRef<[u8]> for Sha512Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Compute SHA-256 hash
pub fn sha256(data: &[u8]) -> Sha256Hash {
    let digest = digest::digest(&SHA256, data);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(digest.as_ref());
    Sha256Hash(hash)
}

/// Compute SHA-384 hash
pub fn sha384(data: &[u8]) -> Sha384Hash {
    let digest = digest::digest(&SHA384, data);
    let mut hash = [0u8; 48];
    hash.copy_from_slice(digest.as_ref());
    Sha384Hash(hash)
}

/// Compute SHA-512 hash
pub fn sha512(data: &[u8]) -> Sha512Hash {
    let digest = digest::digest(&SHA512, data);
    let mut hash = [0u8; 64];
    hash.copy_from_slice(digest.as_ref());
    Sha512Hash(hash)
}

/// Incremental SHA-256 hasher
pub struct Sha256Hasher {
    context: digest::Context,
}

impl Sha256Hasher {
    /// Create a new hasher
    pub fn new() -> Self {
        Self {
            context: digest::Context::new(&SHA256),
        }
    }

    /// Update with more data
    pub fn update(&mut self, data: &[u8]) {
        self.context.update(data);
    }

    /// Finalize and get the hash
    pub fn finish(self) -> Sha256Hash {
        let digest = self.context.finish();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(digest.as_ref());
        Sha256Hash(hash)
    }
}

impl Default for Sha256Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Incremental SHA-512 hasher
pub struct Sha512Hasher {
    context: digest::Context,
}

impl Sha512Hasher {
    /// Create a new hasher
    pub fn new() -> Self {
        Self {
            context: digest::Context::new(&SHA512),
        }
    }

    /// Update with more data
    pub fn update(&mut self, data: &[u8]) {
        self.context.update(data);
    }

    /// Finalize and get the hash
    pub fn finish(self) -> Sha512Hash {
        let digest = self.context.finish();
        let mut hash = [0u8; 64];
        hash.copy_from_slice(digest.as_ref());
        Sha512Hash(hash)
    }
}

impl Default for Sha512Hasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Constant-time byte comparison (to prevent timing attacks)
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        // Known test vector
        let hash = sha256(b"hello");
        assert_eq!(
            hash.to_hex(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_sha512() {
        let hash = sha512(b"hello");
        assert_eq!(hash.as_bytes().len(), 64);
    }

    #[test]
    fn test_incremental_hash() {
        let mut hasher = Sha256Hasher::new();
        hasher.update(b"hel");
        hasher.update(b"lo");
        let hash = hasher.finish();

        assert_eq!(hash.to_hex(), sha256(b"hello").to_hex());
    }

    #[test]
    fn test_constant_time_eq() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hello", b"hell"));
    }
}
