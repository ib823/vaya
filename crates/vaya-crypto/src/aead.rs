//! Authenticated Encryption with Associated Data (AEAD) using AES-256-GCM
//!
//! Provides symmetric encryption with authentication using ring's AES-GCM implementation.

use crate::random::{base64_decode, base64_encode, random_bytes};
use ring::aead::{
    Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM,
};
use ring::error::Unspecified;
use vaya_common::{ErrorCode, Result, VayaError};

/// AES-256-GCM key length
const KEY_LENGTH: usize = 32;

/// Nonce length for AES-GCM
const NONCE_LENGTH: usize = 12;

/// Authentication tag length
const TAG_LENGTH: usize = 16;

/// Counter-based nonce sequence
struct CounterNonce {
    counter: u64,
    prefix: [u8; 4],
}

impl CounterNonce {
    fn new() -> Result<Self> {
        let prefix_bytes = random_bytes(4)?;
        let mut prefix = [0u8; 4];
        prefix.copy_from_slice(&prefix_bytes);
        Ok(Self { counter: 0, prefix })
    }

    fn from_bytes(nonce_bytes: [u8; NONCE_LENGTH]) -> Self {
        let mut prefix = [0u8; 4];
        prefix.copy_from_slice(&nonce_bytes[..4]);
        let counter = u64::from_be_bytes([
            nonce_bytes[4],
            nonce_bytes[5],
            nonce_bytes[6],
            nonce_bytes[7],
            nonce_bytes[8],
            nonce_bytes[9],
            nonce_bytes[10],
            nonce_bytes[11],
        ]);
        Self { counter, prefix }
    }

    fn current_bytes(&self) -> [u8; NONCE_LENGTH] {
        let mut bytes = [0u8; NONCE_LENGTH];
        bytes[..4].copy_from_slice(&self.prefix);
        bytes[4..].copy_from_slice(&self.counter.to_be_bytes());
        bytes
    }
}

impl NonceSequence for CounterNonce {
    fn advance(&mut self) -> std::result::Result<Nonce, Unspecified> {
        let bytes = self.current_bytes();
        self.counter = self.counter.checked_add(1).ok_or(Unspecified)?;
        Nonce::try_assume_unique_for_key(&bytes)
    }
}

/// AES-256-GCM encryption key
pub struct AeadKey {
    key_bytes: [u8; KEY_LENGTH],
}

impl AeadKey {
    /// Create a new key from bytes (must be exactly 32 bytes)
    pub fn new(key_bytes: &[u8]) -> Result<Self> {
        if key_bytes.len() != KEY_LENGTH {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                format!("AEAD key must be exactly {} bytes", KEY_LENGTH),
            ));
        }
        let mut key = [0u8; KEY_LENGTH];
        key.copy_from_slice(key_bytes);
        Ok(Self { key_bytes: key })
    }

    /// Generate a random key
    pub fn generate() -> Result<Self> {
        let key_bytes = random_bytes(KEY_LENGTH)?;
        Self::new(&key_bytes)
    }

    /// Encrypt plaintext with optional associated data
    pub fn encrypt(&self, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        let nonce_seq = CounterNonce::new()?;
        let nonce_bytes = nonce_seq.current_bytes();

        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.key_bytes)
            .map_err(|_| VayaError::new(ErrorCode::CryptoError, "Failed to create key"))?;

        let mut sealing_key = SealingKey::new(unbound_key, nonce_seq);

        let mut in_out = plaintext.to_vec();
        in_out.reserve(TAG_LENGTH);

        sealing_key
            .seal_in_place_append_tag(Aad::from(aad), &mut in_out)
            .map_err(|_| VayaError::new(ErrorCode::CryptoError, "Encryption failed"))?;

        // Prepend nonce to ciphertext
        let mut result = Vec::with_capacity(NONCE_LENGTH + in_out.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&in_out);

        Ok(result)
    }

    /// Decrypt ciphertext with optional associated data
    pub fn decrypt(&self, ciphertext: &[u8], aad: &[u8]) -> Result<Vec<u8>> {
        if ciphertext.len() < NONCE_LENGTH + TAG_LENGTH {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "Ciphertext too short",
            ));
        }

        // Extract nonce
        let mut nonce_bytes = [0u8; NONCE_LENGTH];
        nonce_bytes.copy_from_slice(&ciphertext[..NONCE_LENGTH]);
        let nonce_seq = CounterNonce::from_bytes(nonce_bytes);

        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.key_bytes)
            .map_err(|_| VayaError::new(ErrorCode::CryptoError, "Failed to create key"))?;

        let mut opening_key = OpeningKey::new(unbound_key, nonce_seq);

        let mut in_out = ciphertext[NONCE_LENGTH..].to_vec();

        let plaintext = opening_key
            .open_in_place(Aad::from(aad), &mut in_out)
            .map_err(|_| {
                VayaError::new(
                    ErrorCode::CryptoError,
                    "Decryption failed - invalid key or corrupted data",
                )
            })?;

        Ok(plaintext.to_vec())
    }

    /// Encrypt and encode to base64
    pub fn encrypt_to_base64(&self, plaintext: &[u8], aad: &[u8]) -> Result<String> {
        let ciphertext = self.encrypt(plaintext, aad)?;
        Ok(base64_encode(&ciphertext))
    }

    /// Decrypt from base64
    pub fn decrypt_from_base64(&self, ciphertext_b64: &str, aad: &[u8]) -> Result<Vec<u8>> {
        let ciphertext = base64_decode(ciphertext_b64)?;
        self.decrypt(&ciphertext, aad)
    }

    /// Get the raw key bytes (use with caution)
    pub fn as_bytes(&self) -> &[u8; KEY_LENGTH] {
        &self.key_bytes
    }
}

impl Clone for AeadKey {
    fn clone(&self) -> Self {
        Self {
            key_bytes: self.key_bytes,
        }
    }
}

/// Encrypt data with a random key and return both key and ciphertext
pub fn encrypt_with_new_key(plaintext: &[u8]) -> Result<(AeadKey, Vec<u8>)> {
    let key = AeadKey::generate()?;
    let ciphertext = key.encrypt(plaintext, &[])?;
    Ok((key, ciphertext))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aead_encrypt_decrypt() {
        let key = AeadKey::generate().unwrap();
        let plaintext = b"Hello, World!";
        let aad = b"additional data";

        let ciphertext = key.encrypt(plaintext, aad).unwrap();
        let decrypted = key.decrypt(&ciphertext, aad).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_aead_wrong_key() {
        let key1 = AeadKey::generate().unwrap();
        let key2 = AeadKey::generate().unwrap();

        let plaintext = b"secret message";
        let ciphertext = key1.encrypt(plaintext, &[]).unwrap();

        // Decrypting with wrong key should fail
        let result = key2.decrypt(&ciphertext, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_aead_wrong_aad() {
        let key = AeadKey::generate().unwrap();
        let plaintext = b"secret message";
        let aad1 = b"correct aad";
        let aad2 = b"wrong aad";

        let ciphertext = key.encrypt(plaintext, aad1).unwrap();

        // Decrypting with wrong AAD should fail
        let result = key.decrypt(&ciphertext, aad2);
        assert!(result.is_err());
    }

    #[test]
    fn test_aead_tampered_ciphertext() {
        let key = AeadKey::generate().unwrap();
        let plaintext = b"secret message";

        let mut ciphertext = key.encrypt(plaintext, &[]).unwrap();

        // Tamper with ciphertext
        if let Some(byte) = ciphertext.get_mut(NONCE_LENGTH + 5) {
            *byte ^= 0xFF;
        }

        // Should fail authentication
        let result = key.decrypt(&ciphertext, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_aead_base64() {
        let key = AeadKey::generate().unwrap();
        let plaintext = b"Hello, World!";

        let encrypted = key.encrypt_to_base64(plaintext, &[]).unwrap();
        let decrypted = key.decrypt_from_base64(&encrypted, &[]).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_aead_empty_plaintext() {
        let key = AeadKey::generate().unwrap();
        let plaintext = b"";

        let ciphertext = key.encrypt(plaintext, &[]).unwrap();
        let decrypted = key.decrypt(&ciphertext, &[]).unwrap();

        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_aead_large_plaintext() {
        let key = AeadKey::generate().unwrap();
        let plaintext = vec![0xAB; 1024 * 1024]; // 1MB

        let ciphertext = key.encrypt(&plaintext, &[]).unwrap();
        let decrypted = key.decrypt(&ciphertext, &[]).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_aead_key_length() {
        // Too short
        let result = AeadKey::new(&[0u8; 16]);
        assert!(result.is_err());

        // Too long
        let result = AeadKey::new(&[0u8; 64]);
        assert!(result.is_err());

        // Correct length
        let result = AeadKey::new(&[0u8; 32]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_with_new_key() {
        let plaintext = b"test data";
        let (key, ciphertext) = encrypt_with_new_key(plaintext).unwrap();

        let decrypted = key.decrypt(&ciphertext, &[]).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }
}
