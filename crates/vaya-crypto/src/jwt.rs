//! JWT (JSON Web Token) implementation using HMAC-SHA256
//!
//! Minimal, secure JWT implementation for authentication tokens.
//! Uses HMAC-SHA256 (HS256) for signing.

use ring::hmac;
use vaya_common::{Result, VayaError, ErrorCode, Timestamp};
use crate::random::{base64_encode, base64_decode};

/// JWT signing key
pub struct JwtKey {
    key: hmac::Key,
}

impl JwtKey {
    /// Create a new JWT key from bytes (must be at least 32 bytes)
    pub fn new(secret: &[u8]) -> Result<Self> {
        if secret.len() < 32 {
            return Err(VayaError::new(
                ErrorCode::CryptoError,
                "JWT secret must be at least 32 bytes",
            ));
        }
        Ok(Self {
            key: hmac::Key::new(hmac::HMAC_SHA256, secret),
        })
    }

    /// Generate a random JWT key
    pub fn generate() -> Result<Self> {
        let secret = crate::random_bytes(32)?;
        Self::new(&secret)
    }

    /// Sign a JWT and return the complete token
    pub fn sign(&self, claims: &JwtClaims) -> Result<String> {
        // Header: {"alg":"HS256","typ":"JWT"}
        let header = base64_encode(b"{\"alg\":\"HS256\",\"typ\":\"JWT\"}");

        // Payload
        let payload_json = claims.to_json()?;
        let payload = base64_encode(payload_json.as_bytes());

        // Signature
        let message = format!("{}.{}", header, payload);
        let sig = hmac::sign(&self.key, message.as_bytes());
        let signature = base64_encode(sig.as_ref());

        Ok(format!("{}.{}.{}", header, payload, signature))
    }

    /// Verify and decode a JWT
    pub fn verify(&self, token: &str) -> Result<JwtClaims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(VayaError::new(ErrorCode::InvalidToken, "Invalid JWT format"));
        }

        let header = parts[0];
        let payload = parts[1];
        let signature = parts[2];

        // Verify signature
        let message = format!("{}.{}", header, payload);
        let sig_bytes = base64_decode(signature)?;

        hmac::verify(&self.key, message.as_bytes(), &sig_bytes)
            .map_err(|_| VayaError::new(ErrorCode::InvalidToken, "Invalid JWT signature"))?;

        // Decode payload
        let payload_bytes = base64_decode(payload)?;
        let payload_str = String::from_utf8(payload_bytes)
            .map_err(|_| VayaError::new(ErrorCode::InvalidToken, "Invalid JWT payload encoding"))?;

        let claims = JwtClaims::from_json(&payload_str)?;

        // Check expiration
        if let Some(exp) = claims.exp {
            let now = Timestamp::now().as_unix();
            if now > exp {
                return Err(VayaError::new(ErrorCode::TokenExpired, "JWT has expired"));
            }
        }

        // Check not-before
        if let Some(nbf) = claims.nbf {
            let now = Timestamp::now().as_unix();
            if now < nbf {
                return Err(VayaError::new(ErrorCode::InvalidToken, "JWT not yet valid"));
            }
        }

        Ok(claims)
    }
}

/// JWT claims (standard + custom)
#[derive(Debug, Clone)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Issuer
    pub iss: Option<String>,
    /// Audience
    pub aud: Option<String>,
    /// Expiration time (Unix timestamp)
    pub exp: Option<i64>,
    /// Issued at (Unix timestamp)
    pub iat: Option<i64>,
    /// Not before (Unix timestamp)
    pub nbf: Option<i64>,
    /// JWT ID (unique identifier)
    pub jti: Option<String>,
    /// User role (custom claim)
    pub role: Option<String>,
    /// Session ID (custom claim)
    pub sid: Option<String>,
}

impl JwtClaims {
    /// Create new claims with subject
    pub fn new(subject: impl Into<String>) -> Self {
        let now = Timestamp::now().as_unix();
        Self {
            sub: subject.into(),
            iss: None,
            aud: None,
            exp: None,
            iat: Some(now),
            nbf: None,
            jti: None,
            role: None,
            sid: None,
        }
    }

    /// Set issuer
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.iss = Some(issuer.into());
        self
    }

    /// Set audience
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.aud = Some(audience.into());
        self
    }

    /// Set expiration (duration from now in seconds)
    pub fn with_expiration(mut self, seconds: i64) -> Self {
        let now = Timestamp::now().as_unix();
        self.exp = Some(now + seconds);
        self
    }

    /// Set expiration at specific timestamp
    pub fn expires_at(mut self, timestamp: i64) -> Self {
        self.exp = Some(timestamp);
        self
    }

    /// Set not-before time
    pub fn not_before(mut self, timestamp: i64) -> Self {
        self.nbf = Some(timestamp);
        self
    }

    /// Set JWT ID
    pub fn with_jti(mut self, jti: impl Into<String>) -> Self {
        self.jti = Some(jti.into());
        self
    }

    /// Set role claim
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Set session ID claim
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.sid = Some(session_id.into());
        self
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.exp {
            let now = Timestamp::now().as_unix();
            now > exp
        } else {
            false
        }
    }

    /// Serialize to JSON (minimal implementation, no serde dependency)
    fn to_json(&self) -> Result<String> {
        let mut parts = Vec::new();

        parts.push(format!("\"sub\":\"{}\"", escape_json(&self.sub)));

        if let Some(ref iss) = self.iss {
            parts.push(format!("\"iss\":\"{}\"", escape_json(iss)));
        }
        if let Some(ref aud) = self.aud {
            parts.push(format!("\"aud\":\"{}\"", escape_json(aud)));
        }
        if let Some(exp) = self.exp {
            parts.push(format!("\"exp\":{}", exp));
        }
        if let Some(iat) = self.iat {
            parts.push(format!("\"iat\":{}", iat));
        }
        if let Some(nbf) = self.nbf {
            parts.push(format!("\"nbf\":{}", nbf));
        }
        if let Some(ref jti) = self.jti {
            parts.push(format!("\"jti\":\"{}\"", escape_json(jti)));
        }
        if let Some(ref role) = self.role {
            parts.push(format!("\"role\":\"{}\"", escape_json(role)));
        }
        if let Some(ref sid) = self.sid {
            parts.push(format!("\"sid\":\"{}\"", escape_json(sid)));
        }

        Ok(format!("{{{}}}", parts.join(",")))
    }

    /// Parse from JSON (minimal implementation)
    fn from_json(json: &str) -> Result<Self> {
        let json = json.trim();
        if !json.starts_with('{') || !json.ends_with('}') {
            return Err(VayaError::new(ErrorCode::InvalidToken, "Invalid JSON format"));
        }

        let inner = &json[1..json.len() - 1];
        let mut claims = JwtClaims {
            sub: String::new(),
            iss: None,
            aud: None,
            exp: None,
            iat: None,
            nbf: None,
            jti: None,
            role: None,
            sid: None,
        };

        // Simple JSON parser for our limited format
        for part in split_json_fields(inner) {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let (key, value) = parse_json_field(part)?;
            match key {
                "sub" => claims.sub = parse_json_string(value)?,
                "iss" => claims.iss = Some(parse_json_string(value)?),
                "aud" => claims.aud = Some(parse_json_string(value)?),
                "exp" => claims.exp = Some(parse_json_number(value)?),
                "iat" => claims.iat = Some(parse_json_number(value)?),
                "nbf" => claims.nbf = Some(parse_json_number(value)?),
                "jti" => claims.jti = Some(parse_json_string(value)?),
                "role" => claims.role = Some(parse_json_string(value)?),
                "sid" => claims.sid = Some(parse_json_string(value)?),
                _ => {} // Ignore unknown fields
            }
        }

        if claims.sub.is_empty() {
            return Err(VayaError::new(ErrorCode::InvalidToken, "Missing 'sub' claim"));
        }

        Ok(claims)
    }
}

// Simple JSON helpers (avoiding serde dependency)

fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

fn split_json_fields(s: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '{' | '[' if !in_string => depth += 1,
            '}' | ']' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    if start < s.len() {
        result.push(&s[start..]);
    }

    result
}

fn parse_json_field(s: &str) -> Result<(&str, &str)> {
    let s = s.trim();
    let colon_pos = s.find(':').ok_or_else(|| {
        VayaError::new(ErrorCode::InvalidToken, "Invalid JSON field format")
    })?;

    let key = s[..colon_pos].trim();
    let value = s[colon_pos + 1..].trim();

    // Remove quotes from key
    let key = key.trim_matches('"');

    Ok((key, value))
}

fn parse_json_string(s: &str) -> Result<String> {
    let s = s.trim();
    if !s.starts_with('"') || !s.ends_with('"') {
        return Err(VayaError::new(ErrorCode::InvalidToken, "Invalid JSON string"));
    }

    let inner = &s[1..s.len() - 1];
    let mut result = String::with_capacity(inner.len());
    let mut chars = inner.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('u') => {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Ok(code) = u32::from_str_radix(&hex, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                }
                Some(other) => result.push(other),
                None => break,
            }
        } else {
            result.push(c);
        }
    }

    Ok(result)
}

fn parse_json_number(s: &str) -> Result<i64> {
    s.trim()
        .parse()
        .map_err(|_| VayaError::new(ErrorCode::InvalidToken, "Invalid JSON number"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_sign_verify() {
        let key = JwtKey::generate().unwrap();
        let claims = JwtClaims::new("user123")
            .with_issuer("vaya")
            .with_expiration(3600);

        let token = key.sign(&claims).unwrap();
        let verified = key.verify(&token).unwrap();

        assert_eq!(verified.sub, "user123");
        assert_eq!(verified.iss, Some("vaya".to_string()));
    }

    #[test]
    fn test_jwt_expired() {
        let key = JwtKey::generate().unwrap();
        let claims = JwtClaims::new("user123")
            .expires_at(0); // Expired in 1970

        let token = key.sign(&claims).unwrap();
        let result = key.verify(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_invalid_signature() {
        let key1 = JwtKey::generate().unwrap();
        let key2 = JwtKey::generate().unwrap();

        let claims = JwtClaims::new("user123");
        let token = key1.sign(&claims).unwrap();

        // Verify with different key should fail
        let result = key2.verify(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_claims_builder() {
        let claims = JwtClaims::new("user456")
            .with_issuer("vaya-auth")
            .with_audience("vaya-api")
            .with_role("admin")
            .with_session("session123")
            .with_expiration(7200);

        assert_eq!(claims.sub, "user456");
        assert_eq!(claims.iss, Some("vaya-auth".to_string()));
        assert_eq!(claims.aud, Some("vaya-api".to_string()));
        assert_eq!(claims.role, Some("admin".to_string()));
        assert_eq!(claims.sid, Some("session123".to_string()));
        assert!(claims.exp.is_some());
    }

    #[test]
    fn test_jwt_key_too_short() {
        let result = JwtKey::new(&[0u8; 16]);
        assert!(result.is_err());
    }

    #[test]
    fn test_json_escaping() {
        let key = JwtKey::generate().unwrap();
        let claims = JwtClaims::new("user with \"quotes\" and \\backslash");

        let token = key.sign(&claims).unwrap();
        let verified = key.verify(&token).unwrap();

        assert_eq!(verified.sub, "user with \"quotes\" and \\backslash");
    }
}
