//! JWT token generation and validation
//!
//! Implements HS256 (HMAC-SHA256) JWT tokens.

use ring::hmac;
use time::{Duration, OffsetDateTime};

use crate::{AuthError, AuthResult};

/// JWT token claims
#[derive(Debug, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: Option<String>,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Not before (Unix timestamp)
    pub nbf: Option<i64>,
    /// JWT ID
    pub jti: Option<String>,
    /// Custom claims
    pub custom: Vec<(String, String)>,
}

impl Claims {
    /// Create new claims with subject and expiration
    pub fn new(
        subject: impl Into<String>,
        issuer: impl Into<String>,
        expires_in: Duration,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            sub: subject.into(),
            iss: issuer.into(),
            aud: None,
            exp: (now + expires_in).unix_timestamp(),
            iat: now.unix_timestamp(),
            nbf: None,
            jti: None,
            custom: Vec::new(),
        }
    }

    /// Set audience
    pub fn audience(mut self, aud: impl Into<String>) -> Self {
        self.aud = Some(aud.into());
        self
    }

    /// Set JWT ID
    pub fn jti(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    /// Add a custom claim
    pub fn claim(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.push((key.into(), value.into()));
        self
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc().unix_timestamp() > self.exp
    }

    /// Get expiration as OffsetDateTime
    pub fn expires_at(&self) -> Option<OffsetDateTime> {
        OffsetDateTime::from_unix_timestamp(self.exp).ok()
    }

    /// Encode claims as JSON
    fn to_json(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("\"sub\":\"{}\"", escape_json(&self.sub)));
        parts.push(format!("\"iss\":\"{}\"", escape_json(&self.iss)));

        if let Some(ref aud) = self.aud {
            parts.push(format!("\"aud\":\"{}\"", escape_json(aud)));
        }

        parts.push(format!("\"exp\":{}", self.exp));
        parts.push(format!("\"iat\":{}", self.iat));

        if let Some(nbf) = self.nbf {
            parts.push(format!("\"nbf\":{}", nbf));
        }

        if let Some(ref jti) = self.jti {
            parts.push(format!("\"jti\":\"{}\"", escape_json(jti)));
        }

        for (key, value) in &self.custom {
            parts.push(format!(
                "\"{}\":\"{}\"",
                escape_json(key),
                escape_json(value)
            ));
        }

        format!("{{{}}}", parts.join(","))
    }

    /// Parse claims from JSON
    fn from_json(json: &str) -> AuthResult<Self> {
        let json = json.trim();
        if !json.starts_with('{') || !json.ends_with('}') {
            return Err(AuthError::InvalidToken("Invalid claims JSON".into()));
        }

        let inner = &json[1..json.len() - 1];

        let mut claims = Claims {
            sub: String::new(),
            iss: String::new(),
            aud: None,
            exp: 0,
            iat: 0,
            nbf: None,
            jti: None,
            custom: Vec::new(),
        };

        // Simple JSON parser for our format
        for part in split_json_fields(inner) {
            let (key, value) = parse_json_field(part)?;
            match key.as_str() {
                "sub" => claims.sub = value,
                "iss" => claims.iss = value,
                "aud" => claims.aud = Some(value),
                "exp" => {
                    claims.exp = value
                        .parse()
                        .map_err(|_| AuthError::InvalidToken("Invalid exp".into()))?
                }
                "iat" => {
                    claims.iat = value
                        .parse()
                        .map_err(|_| AuthError::InvalidToken("Invalid iat".into()))?
                }
                "nbf" => {
                    claims.nbf = Some(
                        value
                            .parse()
                            .map_err(|_| AuthError::InvalidToken("Invalid nbf".into()))?,
                    )
                }
                "jti" => claims.jti = Some(value),
                _ => claims.custom.push((key, value)),
            }
        }

        Ok(claims)
    }
}

/// JWT token generator and validator
pub struct JwtTokenizer {
    /// HMAC key
    key: hmac::Key,
    /// Token issuer
    issuer: String,
    /// Default expiration duration
    default_expiration: Duration,
}

impl JwtTokenizer {
    /// Create a new tokenizer with a secret key
    pub fn new(secret: &[u8], issuer: impl Into<String>) -> Self {
        Self {
            key: hmac::Key::new(hmac::HMAC_SHA256, secret),
            issuer: issuer.into(),
            default_expiration: Duration::hours(24),
        }
    }

    /// Set default expiration
    pub fn with_expiration(mut self, duration: Duration) -> Self {
        self.default_expiration = duration;
        self
    }

    /// Generate a token for a subject (user ID)
    pub fn generate(&self, subject: impl Into<String>) -> AuthResult<String> {
        let claims = Claims::new(subject, &self.issuer, self.default_expiration);
        self.generate_with_claims(claims)
    }

    /// Generate a token with custom claims
    pub fn generate_with_claims(&self, claims: Claims) -> AuthResult<String> {
        // Header
        let header = r#"{"alg":"HS256","typ":"JWT"}"#;
        let header_b64 = base64url_encode(header.as_bytes());

        // Payload
        let payload = claims.to_json();
        let payload_b64 = base64url_encode(payload.as_bytes());

        // Signature
        let message = format!("{}.{}", header_b64, payload_b64);
        let signature = hmac::sign(&self.key, message.as_bytes());
        let signature_b64 = base64url_encode(signature.as_ref());

        Ok(format!("{}.{}.{}", header_b64, payload_b64, signature_b64))
    }

    /// Validate and decode a token
    pub fn validate(&self, token: &str) -> AuthResult<Claims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AuthError::InvalidToken("Invalid token format".into()));
        }

        // Verify signature
        let message = format!("{}.{}", parts[0], parts[1]);
        let signature = base64url_decode(parts[2])
            .map_err(|_| AuthError::InvalidToken("Invalid signature encoding".into()))?;

        hmac::verify(&self.key, message.as_bytes(), &signature)
            .map_err(|_| AuthError::SignatureInvalid)?;

        // Decode payload
        let payload_bytes = base64url_decode(parts[1])
            .map_err(|_| AuthError::InvalidToken("Invalid payload encoding".into()))?;
        let payload = String::from_utf8(payload_bytes)
            .map_err(|_| AuthError::InvalidToken("Invalid payload UTF-8".into()))?;

        let claims = Claims::from_json(&payload)?;

        // Check expiration
        if claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        // Check issuer
        if claims.iss != self.issuer {
            return Err(AuthError::InvalidToken("Invalid issuer".into()));
        }

        Ok(claims)
    }

    /// Refresh a token (generate new token with same subject)
    pub fn refresh(&self, token: &str) -> AuthResult<String> {
        let claims = self.validate(token)?;
        self.generate(&claims.sub)
    }
}

/// Base64url encoding (no padding)
fn base64url_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

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

/// Base64url decoding
fn base64url_decode(s: &str) -> Result<Vec<u8>, String> {
    const DECODE: [i8; 256] = {
        let mut table = [-1i8; 256];
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
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
            return Err("Invalid base64url character".into());
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

/// Escape JSON string
fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            _ => result.push(c),
        }
    }
    result
}

/// Split JSON into fields
fn split_json_fields(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let mut escape = false;

    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            ',' if !in_string => {
                result.push(s[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < s.len() {
        result.push(s[start..].trim());
    }

    result
}

/// Parse a JSON field
fn parse_json_field(s: &str) -> AuthResult<(String, String)> {
    let colon = s
        .find(':')
        .ok_or_else(|| AuthError::InvalidToken("Invalid JSON field".into()))?;

    let key = s[..colon].trim().trim_matches('"').to_string();
    let value = s[colon + 1..].trim();

    // Parse value
    let value = if value.starts_with('"') && value.ends_with('"') {
        unescape_json(&value[1..value.len() - 1])
    } else {
        value.to_string()
    };

    Ok((key, value))
}

/// Unescape JSON string
fn unescape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(c) => {
                    result.push('\\');
                    result.push(c);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate() {
        let tokenizer = JwtTokenizer::new(b"super-secret-key-for-testing", "vaya");

        let token = tokenizer.generate("user-123").unwrap();
        assert!(token.contains('.'));

        let claims = tokenizer.validate(&token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.iss, "vaya");
        assert!(!claims.is_expired());
    }

    #[test]
    fn test_custom_claims() {
        let tokenizer = JwtTokenizer::new(b"secret", "vaya");

        let claims = Claims::new("user-123", "vaya", Duration::hours(1))
            .claim("role", "admin")
            .claim("tier", "premium");

        let token = tokenizer.generate_with_claims(claims).unwrap();
        let decoded = tokenizer.validate(&token).unwrap();

        assert_eq!(decoded.custom.len(), 2);
        assert!(decoded
            .custom
            .iter()
            .any(|(k, v)| k == "role" && v == "admin"));
    }

    #[test]
    fn test_expired_token() {
        let tokenizer = JwtTokenizer::new(b"secret", "vaya");

        let claims = Claims::new("user-123", "vaya", Duration::seconds(-1));
        let token = tokenizer.generate_with_claims(claims).unwrap();

        assert!(matches!(
            tokenizer.validate(&token),
            Err(AuthError::TokenExpired)
        ));
    }

    #[test]
    fn test_invalid_signature() {
        let tokenizer1 = JwtTokenizer::new(b"secret1", "vaya");
        let tokenizer2 = JwtTokenizer::new(b"secret2", "vaya");

        let token = tokenizer1.generate("user-123").unwrap();

        assert!(matches!(
            tokenizer2.validate(&token),
            Err(AuthError::SignatureInvalid)
        ));
    }

    #[test]
    fn test_refresh_token() {
        let tokenizer = JwtTokenizer::new(b"secret", "vaya");

        let token1 = tokenizer.generate("user-123").unwrap();
        let token2 = tokenizer.refresh(&token1).unwrap();

        // Both tokens should be valid and have same subject
        let claims1 = tokenizer.validate(&token1).unwrap();
        let claims2 = tokenizer.validate(&token2).unwrap();
        assert_eq!(claims1.sub, "user-123");
        assert_eq!(claims2.sub, "user-123");
    }

    #[test]
    fn test_base64url_roundtrip() {
        let data = b"hello world!";
        let encoded = base64url_encode(data);
        let decoded = base64url_decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }
}
