//! vaya-net: Zero-dependency HTTP server with TLS and WebSocket support
//!
//! This crate provides a custom HTTP/1.1 server built directly on tokio and rustls,
//! without relying on external HTTP frameworks like hyper or axum.

pub mod error;
pub mod http;
pub mod request;
pub mod response;
pub mod router;
pub mod server;
pub mod websocket;

pub use error::{NetError, NetResult};
pub use http::{Method, StatusCode, Version};
pub use request::Request;
pub use response::Response;
pub use router::Router;
pub use server::{Server, ServerConfig};
pub use websocket::WebSocket;

/// HTTP protocol version
pub const HTTP_VERSION: &str = "HTTP/1.1";

/// Maximum header size (8KB)
pub const MAX_HEADER_SIZE: usize = 8 * 1024;

/// Maximum body size (16MB)
pub const MAX_BODY_SIZE: usize = 16 * 1024 * 1024;

/// Default read timeout in seconds
pub const DEFAULT_READ_TIMEOUT: u64 = 30;

/// Default write timeout in seconds
pub const DEFAULT_WRITE_TIMEOUT: u64 = 30;
