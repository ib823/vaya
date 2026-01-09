//! Secure random number generation using ring's SystemRandom

use ring::rand::{SecureRandom, SystemRandom};
use vaya_common::{ErrorCode, Result, VayaError};

/// Thread-safe random number generator
pub struct VayaRandom {
    rng: SystemRandom,
}

impl VayaRandom {
    /// Create a new random generator
    pub fn new() -> Self {
        Self {
            rng: SystemRandom::new(),
        }
    }

    /// Fill a buffer with random bytes
    pub fn fill(&self, dest: &mut [u8]) -> Result<()> {
        self.rng
            .fill(dest)
            .map_err(|_| VayaError::new(ErrorCode::CryptoError, "RNG failure"))
    }

    /// Generate random bytes of specified length
    pub fn bytes(&self, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.fill(&mut buf)?;
        Ok(buf)
    }

    /// Generate a random u64
    pub fn u64(&self) -> Result<u64> {
        let mut buf = [0u8; 8];
        self.fill(&mut buf)?;
        Ok(u64::from_le_bytes(buf))
    }

    /// Generate a random u32
    pub fn u32(&self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.fill(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    /// Generate a random value in range [0, max)
    pub fn range(&self, max: u64) -> Result<u64> {
        if max == 0 {
            return Ok(0);
        }
        // Use rejection sampling to avoid bias
        let threshold = u64::MAX - (u64::MAX % max);
        loop {
            let val = self.u64()?;
            if val < threshold {
                return Ok(val % max);
            }
        }
    }

    /// Generate a random alphanumeric string
    pub fn alphanumeric(&self, len: usize) -> Result<String> {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut result = String::with_capacity(len);
        for _ in 0..len {
            let idx = self.range(CHARSET.len() as u64)? as usize;
            result.push(CHARSET[idx] as char);
        }
        Ok(result)
    }

    /// Generate a random hex string
    pub fn hex(&self, bytes: usize) -> Result<String> {
        let buf = self.bytes(bytes)?;
        Ok(hex_encode(&buf))
    }

    /// Generate a random base64 string
    pub fn base64(&self, bytes: usize) -> Result<String> {
        let buf = self.bytes(bytes)?;
        Ok(base64_encode(&buf))
    }
}

impl Default for VayaRandom {
    fn default() -> Self {
        Self::new()
    }
}

/// Global random generator (thread-safe)
static RANDOM: std::sync::OnceLock<VayaRandom> = std::sync::OnceLock::new();

/// Get the global random generator
pub fn random() -> &'static VayaRandom {
    RANDOM.get_or_init(VayaRandom::new)
}

/// Generate random bytes (convenience function)
pub fn random_bytes(len: usize) -> Result<Vec<u8>> {
    random().bytes(len)
}

/// Generate a random hex string (convenience function)
pub fn random_hex(bytes: usize) -> Result<String> {
    random().hex(bytes)
}

/// Generate a random alphanumeric string (convenience function)
pub fn random_alphanumeric(len: usize) -> Result<String> {
    random().alphanumeric(len)
}

// Encoding utilities

/// Encode bytes to hex string
pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push(HEX_CHARS[(byte >> 4) as usize] as char);
        result.push(HEX_CHARS[(byte & 0x0f) as usize] as char);
    }
    result
}

/// Decode hex string to bytes
pub fn hex_decode(hex: &str) -> Result<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return Err(VayaError::new(ErrorCode::CryptoError, "Invalid hex length"));
    }

    let mut result = Vec::with_capacity(hex.len() / 2);
    for chunk in hex.as_bytes().chunks(2) {
        let high = hex_char_to_nibble(chunk[0])?;
        let low = hex_char_to_nibble(chunk[1])?;
        result.push((high << 4) | low);
    }
    Ok(result)
}

fn hex_char_to_nibble(c: u8) -> Result<u8> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(VayaError::new(
            ErrorCode::CryptoError,
            "Invalid hex character",
        )),
    }
}

/// Encode bytes to base64 (URL-safe, no padding)
pub fn base64_encode(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

    let mut result = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let chunks = bytes.chunks(3);

    for chunk in chunks {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        }
    }

    result
}

/// Decode base64 (URL-safe, with or without padding) to bytes
pub fn base64_decode(input: &str) -> Result<Vec<u8>> {
    // Remove any padding
    let input = input.trim_end_matches('=');

    let mut result = Vec::with_capacity(input.len() * 3 / 4);
    let mut buffer = 0u32;
    let mut bits = 0u8;

    for c in input.chars() {
        let val = match c {
            'A'..='Z' => (c as u32) - ('A' as u32),
            'a'..='z' => (c as u32) - ('a' as u32) + 26,
            '0'..='9' => (c as u32) - ('0' as u32) + 52,
            '-' | '+' => 62,
            '_' | '/' => 63,
            _ => {
                return Err(VayaError::new(
                    ErrorCode::CryptoError,
                    "Invalid base64 character",
                ))
            }
        };

        buffer = (buffer << 6) | val;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_bytes() {
        let bytes = random_bytes(32).unwrap();
        assert_eq!(bytes.len(), 32);

        let bytes2 = random_bytes(32).unwrap();
        assert_ne!(bytes, bytes2); // Should be different (with overwhelming probability)
    }

    #[test]
    fn test_random_alphanumeric() {
        let s = random_alphanumeric(16).unwrap();
        assert_eq!(s.len(), 16);
        assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_hex_encode_decode() {
        let original = b"Hello, World!";
        let encoded = hex_encode(original);
        let decoded = hex_decode(&encoded).unwrap();
        assert_eq!(original.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_base64_encode_decode() {
        let original = b"Hello, World!";
        let encoded = base64_encode(original);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(original.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_base64_url_safe() {
        // Test that we use URL-safe characters
        let encoded = base64_encode(&[0xff, 0xff, 0xff]);
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
    }
}
