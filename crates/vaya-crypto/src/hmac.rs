//! HMAC (Hash-based Message Authentication Code) using ring

use crate::random::hex_encode;
use ring::hmac;
use vaya_common::{ErrorCode, Result, VayaError};

/// HMAC-SHA256 key
pub struct HmacKey {
    key: hmac::Key,
}

impl HmacKey {
    /// Create a new HMAC key from bytes (must be at least 32 bytes for SHA256)
    pub fn new(key_bytes: &[u8]) -> Result<Self> {
        if key_bytes.len() < 32 {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "HMAC key must be at least 32 bytes",
            ));
        }
        Ok(Self {
            key: hmac::Key::new(hmac::HMAC_SHA256, key_bytes),
        })
    }

    /// Generate a new random HMAC key
    pub fn generate() -> Result<Self> {
        let key_bytes = crate::random_bytes(32)?;
        Self::new(&key_bytes)
    }

    /// Sign data and return the tag
    pub fn sign(&self, data: &[u8]) -> HmacTag {
        let tag = hmac::sign(&self.key, data);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(tag.as_ref());
        HmacTag(bytes)
    }

    /// Verify a tag against data
    pub fn verify(&self, data: &[u8], tag: &HmacTag) -> bool {
        hmac::verify(&self.key, data, tag.as_ref()).is_ok()
    }
}

/// HMAC-SHA256 authentication tag (32 bytes)
pub struct HmacTag([u8; 32]);

impl HmacTag {
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

    /// Parse from hex string
    pub fn from_hex(hex: &str) -> Result<Self> {
        let bytes = crate::random::hex_decode(hex)?;
        if bytes.len() != 32 {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "HMAC tag must be 32 bytes",
            ));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        Ok(Self(arr))
    }
}

impl AsRef<[u8]> for HmacTag {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// HMAC-SHA512 key
pub struct HmacKey512 {
    key: hmac::Key,
}

impl HmacKey512 {
    /// Create a new HMAC-SHA512 key from bytes (must be at least 64 bytes)
    pub fn new(key_bytes: &[u8]) -> Result<Self> {
        if key_bytes.len() < 64 {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "HMAC-SHA512 key must be at least 64 bytes",
            ));
        }
        Ok(Self {
            key: hmac::Key::new(hmac::HMAC_SHA512, key_bytes),
        })
    }

    /// Generate a new random HMAC-SHA512 key
    pub fn generate() -> Result<Self> {
        let key_bytes = crate::random_bytes(64)?;
        Self::new(&key_bytes)
    }

    /// Sign data and return the tag
    pub fn sign(&self, data: &[u8]) -> HmacTag512 {
        let tag = hmac::sign(&self.key, data);
        let mut bytes = [0u8; 64];
        bytes.copy_from_slice(tag.as_ref());
        HmacTag512(bytes)
    }

    /// Verify a tag against data
    pub fn verify(&self, data: &[u8], tag: &HmacTag512) -> bool {
        hmac::verify(&self.key, data, tag.as_ref()).is_ok()
    }
}

/// HMAC-SHA512 authentication tag (64 bytes)
pub struct HmacTag512([u8; 64]);

impl HmacTag512 {
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

impl AsRef<[u8]> for HmacTag512 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hmac_sign_verify() {
        let key = HmacKey::generate().unwrap();
        let data = b"Hello, World!";

        let tag = key.sign(data);
        assert!(key.verify(data, &tag));

        // Should fail with different data
        assert!(!key.verify(b"Hello, Universe!", &tag));
    }

    #[test]
    fn test_hmac_deterministic() {
        let key_bytes = crate::random_bytes(32).unwrap();
        let key1 = HmacKey::new(&key_bytes).unwrap();
        let key2 = HmacKey::new(&key_bytes).unwrap();

        let data = b"test data";
        assert_eq!(key1.sign(data).to_hex(), key2.sign(data).to_hex());
    }

    #[test]
    fn test_hmac_key_too_short() {
        let result = HmacKey::new(&[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn test_hmac_tag_hex() {
        let key = HmacKey::generate().unwrap();
        let tag = key.sign(b"test");
        let hex = tag.to_hex();
        let parsed = HmacTag::from_hex(&hex).unwrap();
        assert_eq!(tag.as_bytes(), parsed.as_bytes());
    }
}
