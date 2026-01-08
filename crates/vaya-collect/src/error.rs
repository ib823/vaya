//! Error types for vaya-collect

use std::fmt;
use std::io;

/// Result type for collect operations
pub type CollectResult<T> = Result<T, CollectError>;

/// Errors that can occur during data collection
#[derive(Debug)]
pub enum CollectError {
    /// I/O error
    Io(io::Error),
    /// DNS resolution failed
    DnsResolution(String),
    /// Connection failed
    ConnectionFailed(String),
    /// TLS handshake failed
    TlsError(String),
    /// Request timeout
    Timeout,
    /// Invalid URL
    InvalidUrl(String),
    /// Invalid response
    InvalidResponse(String),
    /// HTTP error status
    HttpError(u16, String),
    /// Too many redirects
    TooManyRedirects,
    /// Rate limited
    RateLimited(u64),
    /// Request cancelled
    Cancelled,
    /// Connection pool exhausted
    PoolExhausted,
    /// Parse error
    ParseError(String),
}

impl fmt::Display for CollectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollectError::Io(e) => write!(f, "I/O error: {}", e),
            CollectError::DnsResolution(host) => write!(f, "DNS resolution failed for: {}", host),
            CollectError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            CollectError::TlsError(msg) => write!(f, "TLS error: {}", msg),
            CollectError::Timeout => write!(f, "Request timeout"),
            CollectError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            CollectError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            CollectError::HttpError(code, msg) => write!(f, "HTTP error {}: {}", code, msg),
            CollectError::TooManyRedirects => write!(f, "Too many redirects"),
            CollectError::RateLimited(retry_after) => {
                write!(f, "Rate limited, retry after {} seconds", retry_after)
            }
            CollectError::Cancelled => write!(f, "Request cancelled"),
            CollectError::PoolExhausted => write!(f, "Connection pool exhausted"),
            CollectError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for CollectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CollectError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for CollectError {
    fn from(err: io::Error) -> Self {
        CollectError::Io(err)
    }
}
