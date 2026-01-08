//! HTTP client and data collection for external APIs
//!
//! This crate provides a high-level HTTP client with:
//! - TLS support via rustls
//! - Automatic retry with exponential backoff
//! - Rate limiting per host
//! - Circuit breaker for failing services
//! - Response caching with TTL
//! - URL parsing and encoding
//!
//! # Example
//!
//! ```no_run
//! use vaya_collect::{Collector, CollectorBuilder};
//!
//! let collector = CollectorBuilder::new()
//!     .timeout(5000)
//!     .cache_ttl(300)
//!     .rate_limit(10)
//!     .build()
//!     .unwrap();
//!
//! // Fetch with automatic retry and caching
//! let response = collector.fetch("https://api.example.com/data").unwrap();
//! println!("Status: {}", response.status);
//! ```

pub mod client;
pub mod collector;
pub mod error;
pub mod request;
pub mod response;
pub mod retry;
pub mod url;

pub use client::{Client, ClientConfig2 as ClientConfig};
pub use collector::{Collector, CollectorBuilder, CollectorConfig};
pub use error::{CollectError, CollectResult};
pub use request::{Headers, Method, Request, RequestBuilder};
pub use response::Response;
pub use retry::{CircuitBreaker, CircuitStatus, RateLimiter, RetryStrategy};
pub use url::{Scheme, Url};
