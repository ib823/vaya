//! Error types for vaya-search

use std::fmt;

/// Result type for search operations
pub type SearchResult<T> = Result<T, SearchError>;

/// Errors that can occur during flight search
#[derive(Debug, Clone)]
pub enum SearchError {
    /// Invalid search parameters
    InvalidParams(String),
    /// Invalid date range
    InvalidDateRange,
    /// Invalid route
    InvalidRoute(String),
    /// No results found
    NoResults,
    /// Provider error
    ProviderError(String),
    /// Rate limited
    RateLimited,
    /// Search timeout
    Timeout,
    /// Cache error
    CacheError(String),
    /// Internal error
    Internal(String),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::InvalidParams(msg) => write!(f, "Invalid search parameters: {}", msg),
            SearchError::InvalidDateRange => write!(f, "Invalid date range"),
            SearchError::InvalidRoute(msg) => write!(f, "Invalid route: {}", msg),
            SearchError::NoResults => write!(f, "No results found"),
            SearchError::ProviderError(msg) => write!(f, "Provider error: {}", msg),
            SearchError::RateLimited => write!(f, "Search rate limited"),
            SearchError::Timeout => write!(f, "Search timeout"),
            SearchError::CacheError(msg) => write!(f, "Cache error: {}", msg),
            SearchError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for SearchError {}
