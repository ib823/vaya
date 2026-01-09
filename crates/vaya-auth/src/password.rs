//! Password hashing and verification using Argon2-like algorithm
//!
//! Uses PBKDF2 with SHA-256 since Argon2 would require additional dependencies.
//! In production, consider using Argon2id for better security.

use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use std::num::NonZeroU32;

use crate::{AuthError, AuthResult};

/// Password hasher using PBKDF2-SHA256
pub struct PasswordHasher {
    /// Number of iterations
    iterations: NonZeroU32,
    /// Salt length in bytes
    salt_len: usize,
    /// Output hash length in bytes
    hash_len: usize,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher {
    /// Create a new password hasher with secure defaults
    pub fn new() -> Self {
        Self {
            iterations: NonZeroU32::new(100_000).unwrap(),
            salt_len: 16,
            hash_len: 32,
        }
    }

    /// Create hasher with custom iterations (for testing)
    pub fn with_iterations(iterations: u32) -> Self {
        Self {
            iterations: NonZeroU32::new(iterations).unwrap_or(NonZeroU32::new(1).unwrap()),
            salt_len: 16,
            hash_len: 32,
        }
    }

    /// Hash a password, returning the hash string
    ///
    /// Format: `$pbkdf2-sha256$iterations$salt$hash`
    /// where salt and hash are base64-encoded
    pub fn hash(&self, password: &str) -> AuthResult<String> {
        // Validate password
        self.validate_password(password)?;

        // Generate random salt
        let rng = SystemRandom::new();
        let mut salt = vec![0u8; self.salt_len];
        rng.fill(&mut salt)
            .map_err(|_| AuthError::Internal("Failed to generate salt".into()))?;

        // Compute hash
        let mut hash = vec![0u8; self.hash_len];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            self.iterations,
            &salt,
            password.as_bytes(),
            &mut hash,
        );

        // Encode as string
        let salt_b64 = base64_encode(&salt);
        let hash_b64 = base64_encode(&hash);

        Ok(format!(
            "$pbkdf2-sha256${}${}${}",
            self.iterations, salt_b64, hash_b64
        ))
    }

    /// Verify a password against a hash string
    pub fn verify(&self, password: &str, hash_string: &str) -> AuthResult<bool> {
        // Parse hash string
        let parts: Vec<&str> = hash_string.split('$').collect();
        if parts.len() != 5 || parts[1] != "pbkdf2-sha256" {
            return Err(AuthError::InvalidToken("Invalid hash format".into()));
        }

        let iterations: u32 = parts[2]
            .parse()
            .map_err(|_| AuthError::InvalidToken("Invalid iterations".into()))?;
        let iterations = NonZeroU32::new(iterations)
            .ok_or_else(|| AuthError::InvalidToken("Invalid iterations".into()))?;

        let salt = base64_decode(parts[3])
            .map_err(|e| AuthError::InvalidToken(format!("Invalid salt: {}", e)))?;
        let expected_hash = base64_decode(parts[4])
            .map_err(|e| AuthError::InvalidToken(format!("Invalid hash: {}", e)))?;

        // Verify using constant-time comparison
        let result = pbkdf2::verify(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            &salt,
            password.as_bytes(),
            &expected_hash,
        );

        Ok(result.is_ok())
    }

    /// Validate password meets minimum requirements
    pub fn validate_password(&self, password: &str) -> AuthResult<()> {
        if password.len() < 8 {
            return Err(AuthError::InvalidPasswordFormat(
                "Password must be at least 8 characters".into(),
            ));
        }

        if password.len() > 128 {
            return Err(AuthError::InvalidPasswordFormat(
                "Password must be at most 128 characters".into(),
            ));
        }

        // Check for basic complexity
        let has_lower = password.chars().any(|c| c.is_ascii_lowercase());
        let has_upper = password.chars().any(|c| c.is_ascii_uppercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        if !has_lower || !has_upper || !has_digit {
            return Err(AuthError::WeakPassword);
        }

        Ok(())
    }

    /// Check if a hash needs to be upgraded (e.g., more iterations)
    pub fn needs_upgrade(&self, hash_string: &str) -> bool {
        let parts: Vec<&str> = hash_string.split('$').collect();
        if parts.len() != 5 || parts[1] != "pbkdf2-sha256" {
            return true;
        }

        let iterations: u32 = parts[2].parse().unwrap_or(0);
        iterations < self.iterations.get()
    }
}

/// Base64 encoding without padding
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        let n = (b0 << 16) | (b1 << 8) | b2;

        result.push(ALPHABET[(n >> 18) & 0x3F] as char);
        result.push(ALPHABET[(n >> 12) & 0x3F] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[(n >> 6) & 0x3F] as char);
        }

        if chunk.len() > 2 {
            result.push(ALPHABET[n & 0x3F] as char);
        }
    }

    result
}

/// Base64 decoding
fn base64_decode(s: &str) -> Result<Vec<u8>, String> {
    const DECODE: [i8; 256] = {
        let mut table = [-1i8; 256];
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 {
            table[alphabet[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let bytes = s.as_bytes();
    let mut result = Vec::with_capacity(bytes.len().div_ceil(4) * 3);

    let mut i = 0;
    while i < bytes.len() {
        let b0 = DECODE[bytes[i] as usize];
        let b1 = bytes.get(i + 1).map(|&b| DECODE[b as usize]).unwrap_or(-1);
        let b2 = bytes.get(i + 2).map(|&b| DECODE[b as usize]).unwrap_or(-1);
        let b3 = bytes.get(i + 3).map(|&b| DECODE[b as usize]).unwrap_or(-1);

        if b0 < 0 || b1 < 0 {
            return Err("Invalid base64 character".into());
        }

        let n = ((b0 as u32) << 18)
            | ((b1 as u32) << 12)
            | (if b2 >= 0 { (b2 as u32) << 6 } else { 0 })
            | (if b3 >= 0 { b3 as u32 } else { 0 });

        result.push((n >> 16) as u8);
        if b2 >= 0 {
            result.push((n >> 8) as u8);
        }
        if b3 >= 0 {
            result.push(n as u8);
        }

        i += 4;
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let hasher = PasswordHasher::with_iterations(1000); // Faster for tests
        let password = "SecurePassword123";

        let hash = hasher.hash(password).unwrap();
        assert!(hash.starts_with("$pbkdf2-sha256$"));

        assert!(hasher.verify(password, &hash).unwrap());
        assert!(!hasher.verify("WrongPassword123", &hash).unwrap());
    }

    #[test]
    fn test_different_hashes() {
        let hasher = PasswordHasher::with_iterations(1000);
        let password = "SecurePassword123";

        let hash1 = hasher.hash(password).unwrap();
        let hash2 = hasher.hash(password).unwrap();

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);

        // But both should verify
        assert!(hasher.verify(password, &hash1).unwrap());
        assert!(hasher.verify(password, &hash2).unwrap());
    }

    #[test]
    fn test_password_validation() {
        let hasher = PasswordHasher::new();

        // Too short
        assert!(hasher.validate_password("Short1").is_err());

        // No uppercase
        assert!(hasher.validate_password("alllowercase123").is_err());

        // No lowercase
        assert!(hasher.validate_password("ALLUPPERCASE123").is_err());

        // No digits
        assert!(hasher.validate_password("NoDigitsHere").is_err());

        // Valid
        assert!(hasher.validate_password("ValidPassword123").is_ok());
    }

    #[test]
    fn test_needs_upgrade() {
        let hasher = PasswordHasher::with_iterations(100_000);

        // Old hash with fewer iterations
        let old_hash = "$pbkdf2-sha256$10000$abc$def";
        assert!(hasher.needs_upgrade(old_hash));

        // Current hash
        let current_hash = "$pbkdf2-sha256$100000$abc$def";
        assert!(!hasher.needs_upgrade(current_hash));
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"hello world";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }
}
