//! vaya-api: REST API layer with routing, middleware, and handlers
//!
//! This crate provides the HTTP API infrastructure for VAYA:
//!
//! - **Router**: Path matching and handler dispatch
//! - **Middleware**: Authentication, rate limiting, CORS, logging
//! - **Request/Response**: Type-safe HTTP types
//! - **Error handling**: Consistent error responses
//!
//! # Architecture
//!
//! The API follows RESTful conventions with versioned endpoints:
//!
//! - `/api/v1/search` - Flight search
//! - `/api/v1/bookings` - Booking management
//! - `/api/v1/pools` - Group buying pools
//! - `/api/v1/alerts` - Price alerts
//! - `/api/v1/users` - User management
//!
//! # Example
//!
//! ```ignore
//! use vaya_api::{Router, Request, Response, ApiResult};
//!
//! fn list_users(req: &Request) -> ApiResult<Response> {
//!     Ok(Response::ok())
//! }
//!
//! let mut router = Router::with_prefix("/api/v1");
//! router.get("/users", list_users, "list_users");
//! ```

mod error;
pub mod handlers;
mod middleware;
mod router;
mod types;

pub use error::{ApiError, ApiResult, FieldError};
pub use middleware::{
    AuthMiddleware, CorsConfig, Middleware, MiddlewareChain, RateLimitInfo, RateLimiter,
    RequestLogger, TokenClaims,
};
pub use router::{Handler, Method, Route, Router};
pub use types::{
    parse_query_string, ErrorBody, JsonSerialize, PaginatedBody, Request, Response, SuccessBody,
};

/// API version
pub const API_VERSION: &str = "v1";

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// API prefix (e.g., "/api/v1")
    pub prefix: String,
    /// Rate limit requests per window
    pub rate_limit_requests: u32,
    /// Rate limit window (seconds)
    pub rate_limit_window: i64,
    /// JWT secret
    pub jwt_secret: Vec<u8>,
    /// Enable CORS
    pub enable_cors: bool,
    /// Allowed CORS origins
    pub cors_origins: Vec<String>,
    /// Request timeout (seconds)
    pub request_timeout: u64,
    /// Max request body size (bytes)
    pub max_body_size: usize,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            prefix: "/api/v1".into(),
            rate_limit_requests: 100,
            rate_limit_window: 60,
            jwt_secret: Vec::new(),
            enable_cors: true,
            cors_origins: vec!["*".into()],
            request_timeout: 30,
            max_body_size: 1024 * 1024, // 1MB
        }
    }
}

impl ApiConfig {
    /// Create new configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set API prefix
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Set rate limit
    pub fn with_rate_limit(mut self, requests: u32, window_secs: i64) -> Self {
        self.rate_limit_requests = requests;
        self.rate_limit_window = window_secs;
        self
    }

    /// Set JWT secret
    pub fn with_jwt_secret(mut self, secret: Vec<u8>) -> Self {
        self.jwt_secret = secret;
        self
    }

    /// Set CORS origins
    pub fn with_cors_origins(mut self, origins: Vec<String>) -> Self {
        self.cors_origins = origins;
        self
    }
}

/// API server builder
#[derive(Debug)]
pub struct ApiServer {
    /// Configuration
    config: ApiConfig,
    /// Main router
    router: Router,
    /// Middleware chain
    middleware: MiddlewareChain,
    /// Rate limiter
    rate_limiter: Option<RateLimiter>,
    /// CORS config
    cors: Option<CorsConfig>,
    /// Request logger
    logger: RequestLogger,
}

impl ApiServer {
    /// Create a new API server
    pub fn new(config: ApiConfig) -> Self {
        let router = Router::with_prefix(&config.prefix);
        let rate_limiter = Some(RateLimiter::new(
            config.rate_limit_requests,
            config.rate_limit_window,
        ));
        let cors = if config.enable_cors {
            Some(CorsConfig::new().with_origins(config.cors_origins.clone()))
        } else {
            None
        };

        Self {
            config,
            router,
            middleware: MiddlewareChain::new(),
            rate_limiter,
            cors,
            logger: RequestLogger::new(),
        }
    }

    /// Add a GET route
    pub fn get(&mut self, path: &str, handler: Handler, name: &str) {
        self.router.get(path, handler, name);
    }

    /// Add a POST route
    pub fn post(&mut self, path: &str, handler: Handler, name: &str) {
        self.router.post(path, handler, name);
    }

    /// Add a PUT route
    pub fn put(&mut self, path: &str, handler: Handler, name: &str) {
        self.router.put(path, handler, name);
    }

    /// Add a PATCH route
    pub fn patch(&mut self, path: &str, handler: Handler, name: &str) {
        self.router.patch(path, handler, name);
    }

    /// Add a DELETE route
    pub fn delete(&mut self, path: &str, handler: Handler, name: &str) {
        self.router.delete(path, handler, name);
    }

    /// Add middleware
    pub fn add_middleware(&mut self, name: &'static str, middleware: Middleware) {
        self.middleware.add(name, middleware);
    }

    /// Merge another router
    pub fn mount(&mut self, path: &str, router: Router) {
        self.router.merge(router, Some(path));
    }

    /// Handle a request
    pub fn handle(&self, mut request: Request) -> Response {
        let start = std::time::Instant::now();

        // Log request start
        self.logger.log_start(&request);

        // Check rate limit
        if let Some(ref limiter) = self.rate_limiter {
            let client_id = request.client_ip.clone().unwrap_or_else(|| "unknown".into());
            match limiter.check(&client_id) {
                Ok(info) => {
                    // Rate limit OK, continue
                    let _ = info; // Would apply headers later
                }
                Err(e) => {
                    return e.to_response();
                }
            }
        }

        // Execute middleware chain
        if let Err(e) = self.middleware.execute(&mut request) {
            return e.to_response();
        }

        // Route request
        let mut response = match self.router.route(&request) {
            Ok(r) => r,
            Err(e) => e.to_response(),
        };

        // Apply CORS headers
        if let Some(ref cors) = self.cors {
            cors.apply(&request, &mut response);
        }

        // Log completion
        let duration = start.elapsed().as_millis() as u64;
        self.logger.log_complete(&request, &response, duration);

        response
    }

    /// Get router reference
    pub fn router(&self) -> &Router {
        &self.router
    }

    /// Get config reference
    pub fn config(&self) -> &ApiConfig {
        &self.config
    }
}

/// Health check response
#[derive(Debug, Clone)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
}

impl JsonSerialize for HealthResponse {
    fn to_json(&self) -> String {
        format!(
            r#"{{"status":"{}","version":"{}","uptime_seconds":{}}}"#,
            self.status, self.version, self.uptime_seconds
        )
    }
}

/// Standard health check handler
pub fn health_handler(_req: &Request) -> ApiResult<Response> {
    let health = HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        uptime_seconds: 0, // Would be calculated from server start
    };

    let mut response = Response::ok();
    response.set_json_body(&health);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_config_defaults() {
        let config = ApiConfig::default();
        assert_eq!(config.prefix, "/api/v1");
        assert_eq!(config.rate_limit_requests, 100);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_api_config_builder() {
        let config = ApiConfig::new()
            .with_prefix("/api/v2")
            .with_rate_limit(50, 30)
            .with_jwt_secret(b"secret".to_vec());

        assert_eq!(config.prefix, "/api/v2");
        assert_eq!(config.rate_limit_requests, 50);
        assert_eq!(config.rate_limit_window, 30);
    }

    #[test]
    fn test_api_server_creation() {
        let config = ApiConfig::new();
        let server = ApiServer::new(config);

        assert!(server.rate_limiter.is_some());
        assert!(server.cors.is_some());
    }

    #[test]
    fn test_health_handler() {
        let request = Request::new("GET", "/health");
        let response = health_handler(&request).unwrap();

        assert_eq!(response.status, 200);
        let body = response.body_string().unwrap();
        assert!(body.contains("status"));
        assert!(body.contains("ok"));
    }

    #[test]
    fn test_server_routing() {
        fn test_handler(_req: &Request) -> ApiResult<Response> {
            Ok(Response::ok().with_body(b"test".to_vec()))
        }

        let config = ApiConfig::new().with_prefix("/api");
        let mut server = ApiServer::new(config);
        server.get("/test", test_handler, "test");

        let request = Request::new("GET", "/api/test");
        let response = server.handle(request);

        assert_eq!(response.status, 200);
    }

    #[test]
    fn test_server_not_found() {
        let config = ApiConfig::new();
        let server = ApiServer::new(config);

        let request = Request::new("GET", "/nonexistent");
        let response = server.handle(request);

        assert_eq!(response.status, 404);
    }

    #[test]
    fn test_health_response_json() {
        let health = HealthResponse {
            status: "ok".into(),
            version: "1.0.0".into(),
            uptime_seconds: 3600,
        };

        let json = health.to_json();
        assert!(json.contains(r#""status":"ok""#));
        assert!(json.contains(r#""uptime_seconds":3600"#));
    }
}
