//! Error types for the networking layer

use std::fmt;
use std::io;

/// Network operation result type
pub type NetResult<T> = Result<T, NetError>;

/// Network error types
#[derive(Debug)]
pub enum NetError {
    /// I/O error
    Io(io::Error),
    /// TLS error
    Tls(String),
    /// Invalid HTTP request
    InvalidRequest(String),
    /// Invalid HTTP response
    InvalidResponse(String),
    /// Request too large
    RequestTooLarge,
    /// Header too large
    HeaderTooLarge,
    /// Connection closed
    ConnectionClosed,
    /// Timeout
    Timeout,
    /// Invalid URL
    InvalidUrl(String),
    /// Route not found
    NotFound,
    /// Method not allowed
    MethodNotAllowed,
    /// WebSocket error
    WebSocket(String),
    /// Protocol error
    Protocol(String),
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::Io(e) => write!(f, "I/O error: {}", e),
            NetError::Tls(e) => write!(f, "TLS error: {}", e),
            NetError::InvalidRequest(e) => write!(f, "Invalid request: {}", e),
            NetError::InvalidResponse(e) => write!(f, "Invalid response: {}", e),
            NetError::RequestTooLarge => write!(f, "Request too large"),
            NetError::HeaderTooLarge => write!(f, "Header too large"),
            NetError::ConnectionClosed => write!(f, "Connection closed"),
            NetError::Timeout => write!(f, "Request timeout"),
            NetError::InvalidUrl(e) => write!(f, "Invalid URL: {}", e),
            NetError::NotFound => write!(f, "Route not found"),
            NetError::MethodNotAllowed => write!(f, "Method not allowed"),
            NetError::WebSocket(e) => write!(f, "WebSocket error: {}", e),
            NetError::Protocol(e) => write!(f, "Protocol error: {}", e),
        }
    }
}

impl std::error::Error for NetError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NetError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for NetError {
    fn from(err: io::Error) -> Self {
        NetError::Io(err)
    }
}
