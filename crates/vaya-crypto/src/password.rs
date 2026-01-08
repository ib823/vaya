//! Password hashing using PBKDF2-HMAC-SHA256 from ring
//!
//! This module provides secure password hashing following OWASP recommendations.
//! Uses PBKDF2 with HMAC-SHA256, 100,000 iterations (configurable), and 16-byte salt.

use ring::pbkdf2;
use std::num::NonZeroU32;
use vaya_common::{Result, VayaError, ErrorCode};
use crate::random::{random_bytes, base64_encode, base64_decode};

/// Default number of PBKDF2 iterations (OWASP 2024 recommendation)
pub const DEFAULT_ITERATIONS: u32 = 100_000;

/// Salt length in bytes
const SALT_LENGTH: usize = 16;

/// Hash output length in bytes
const HASH_LENGTH: usize = 32;

/// A hashed password with salt and iteration count
#[derive(Debug, Clone)]
pub struct PasswordHash {
    /// The salt used for this hash
    salt: [u8; SALT_LENGTH],
    /// The derived key
    hash: [u8; HASH_LENGTH],
    /// Number of iterations used
    iterations: u32,
}

impl PasswordHash {
    /// Hash a password with default iterations
    pub fn create(password: &str) -> Result<Self> {
        Self::create_with_iterations(password, DEFAULT_ITERATIONS)
    }

    /// Hash a password with custom iteration count
    pub fn create_with_iterations(password: &str, iterations: u32) -> Result<Self> {
        if password.is_empty() {
            return Err(VayaError::new(
                ErrorCode::ValidationFailed,
                "Password cannot be empty",
            ));
        }

        if iterations < 10_000 {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "Iteration count must be at least 10,000",
            ));
        }

        // Generate random salt
        let salt_vec = random_bytes(SALT_LENGTH)?;
        let mut salt = [0u8; SALT_LENGTH];
        salt.copy_from_slice(&salt_vec);

        // Derive key
        let mut hash = [0u8; HASH_LENGTH];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(iterations).ok_or_else(|| {
                VayaError::new(ErrorCode::CryptoError, "Invalid iteration count")
            })?,
            &salt,
            password.as_bytes(),
            &mut hash,
        );

        Ok(Self {
            salt,
            hash,
            iterations,
        })
    }

    /// Verify a password against this hash
    pub fn verify(&self, password: &str) -> bool {
        let iterations = match NonZeroU32::new(self.iterations) {
            Some(i) => i,
            None => return false,
        };

        pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            &self.salt,
            password.as_bytes(),
            &self.hash,
        )
        .is_ok()
    }

    /// Encode to string format: $pbkdf2-sha256$iterations$salt$hash
    pub fn encode(&self) -> String {
        format!(
            "$pbkdf2-sha256${}${}${}",
            self.iterations,
            base64_encode(&self.salt),
            base64_encode(&self.hash),
        )
    }

    /// Decode from string format
    pub fn decode(encoded: &str) -> Result<Self> {
        let parts: Vec<&str> = encoded.split('$').collect();

        if parts.len() != 5 || parts[0] != "" || parts[1] != "pbkdf2-sha256" {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "Invalid password hash format",
            ));
        }

        let iterations: u32 = parts[2].parse().map_err(|_| {
            VayaError::new(ErrorCode::CryptoError, "Invalid iteration count")
        })?;

        let salt_vec = base64_decode(parts[3])?;
        if salt_vec.len() != SALT_LENGTH {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "Invalid salt length",
            ));
        }
        let mut salt = [0u8; SALT_LENGTH];
        salt.copy_from_slice(&salt_vec);

        let hash_vec = base64_decode(parts[4])?;
        if hash_vec.len() != HASH_LENGTH {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "Invalid hash length",
            ));
        }
        let mut hash = [0u8; HASH_LENGTH];
        hash.copy_from_slice(&hash_vec);

        Ok(Self {
            salt,
            hash,
            iterations,
        })
    }

    /// Check if this hash needs rehashing (e.g., iteration count too low)
    pub fn needs_rehash(&self, min_iterations: u32) -> bool {
        self.iterations < min_iterations
    }

    /// Get the iteration count
    pub fn iterations(&self) -> u32 {
        self.iterations
    }
}

/// Hash a password (convenience function)
pub fn hash_password(password: &str) -> Result<String> {
    let hash = PasswordHash::create(password)?;
    Ok(hash.encode())
}

/// Verify a password against a hash string (convenience function)
pub fn verify_password(password: &str, hash_string: &str) -> Result<bool> {
    let hash = PasswordHash::decode(hash_string)?;
    Ok(hash.verify(password))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_create_verify() {
        let hash = PasswordHash::create("mySecurePassword123!").unwrap();

        assert!(hash.verify("mySecurePassword123!"));
        assert!(!hash.verify("wrongPassword"));
        assert!(!hash.verify(""));
    }

    #[test]
    fn test_password_hash_encode_decode() {
        let hash = PasswordHash::create("testPassword").unwrap();
        let encoded = hash.encode();

        assert!(encoded.starts_with("$pbkdf2-sha256$"));

        let decoded = PasswordHash::decode(&encoded).unwrap();
        assert!(decoded.verify("testPassword"));
        assert!(!decoded.verify("wrongPassword"));
    }

    #[test]
    fn test_password_convenience_functions() {
        let hash = hash_password("password123").unwrap();

        assert!(verify_password("password123", &hash).unwrap());
        assert!(!verify_password("password456", &hash).unwrap());
    }

    #[test]
    fn test_password_empty_rejected() {
        let result = PasswordHash::create("");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_needs_rehash() {
        let hash = PasswordHash::create_with_iterations("test", 50_000).unwrap();

        assert!(hash.needs_rehash(100_000));
        assert!(!hash.needs_rehash(50_000));
        assert!(!hash.needs_rehash(10_000));
    }

    #[test]
    fn test_password_different_salts() {
        let hash1 = PasswordHash::create("samePassword").unwrap();
        let hash2 = PasswordHash::create("samePassword").unwrap();

        // Same password should produce different hashes due to random salt
        assert_ne!(hash1.encode(), hash2.encode());

        // But both should verify correctly
        assert!(hash1.verify("samePassword"));
        assert!(hash2.verify("samePassword"));
    }

    #[test]
    fn test_low_iterations_rejected() {
        let result = PasswordHash::create_with_iterations("test", 1000);
        assert!(result.is_err());
    }
}
