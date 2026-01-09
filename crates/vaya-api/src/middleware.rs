//! API Middleware for authentication, rate limiting, and logging

use std::collections::HashMap;
use std::sync::Mutex;
use time::OffsetDateTime;

use crate::{ApiError, ApiResult, Request, Response};

/// Middleware function type
pub type Middleware = fn(&mut Request) -> ApiResult<()>;

/// Middleware chain
#[derive(Default)]
pub struct MiddlewareChain {
    middlewares: Vec<(&'static str, Middleware)>,
}

impl std::fmt::Debug for MiddlewareChain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareChain")
            .field("middleware_count", &self.middlewares.len())
            .finish()
    }
}

impl MiddlewareChain {
    /// Create a new middleware chain
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add middleware to chain
    pub fn add(&mut self, name: &'static str, middleware: Middleware) {
        self.middlewares.push((name, middleware));
    }

    /// Execute all middleware
    pub fn execute(&self, request: &mut Request) -> ApiResult<()> {
        for (name, middleware) in &self.middlewares {
            tracing::trace!(middleware = %name, "Executing middleware");
            middleware(request)?;
        }
        Ok(())
    }
}

/// Authentication middleware state
pub struct AuthMiddleware {
    /// Required for this middleware
    jwt_secret: Vec<u8>,
    /// Skip auth for these paths
    skip_paths: Vec<String>,
}

impl AuthMiddleware {
    /// Create new auth middleware
    pub fn new(jwt_secret: &[u8]) -> Self {
        Self {
            jwt_secret: jwt_secret.to_vec(),
            skip_paths: vec![
                "/health".into(),
                "/api/v1/auth/login".into(),
                "/api/v1/auth/register".into(),
            ],
        }
    }

    /// Add path to skip list
    pub fn skip_path(&mut self, path: &str) {
        self.skip_paths.push(path.to_string());
    }

    /// Check if path should skip auth
    fn should_skip(&self, path: &str) -> bool {
        self.skip_paths.iter().any(|p| path.starts_with(p))
    }

    /// Validate request (would be called as middleware)
    pub fn validate(&self, request: &mut Request) -> ApiResult<()> {
        // Skip auth for certain paths
        if self.should_skip(&request.path) {
            return Ok(());
        }

        // Get token from header
        let token = request.auth_token().ok_or(ApiError::Unauthorized(
            "Missing authorization header".into(),
        ))?;

        // Validate token (simplified - would use vaya_auth::JwtManager)
        let claims = self.validate_token(token)?;

        // Set user info on request
        request.user_id = Some(claims.user_id);
        request.user_roles = claims.roles;

        Ok(())
    }

    /// Validate JWT token (simplified)
    fn validate_token(&self, token: &str) -> ApiResult<TokenClaims> {
        // In production, this would use vaya_auth::JwtManager
        // For now, simple validation
        if token.is_empty() {
            return Err(ApiError::Unauthorized("Empty token".into()));
        }

        // Check token format (three parts separated by dots)
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(ApiError::Unauthorized("Invalid token format".into()));
        }

        // Would verify signature and decode payload here
        // For now, return mock claims
        Ok(TokenClaims {
            user_id: "mock-user".into(),
            roles: vec!["user".into()],
            exp: OffsetDateTime::now_utc().unix_timestamp() + 3600,
        })
    }
}

/// Token claims
#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: String,
    pub roles: Vec<String>,
    pub exp: i64,
}

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    /// Requests per window
    requests_per_window: u32,
    /// Window size in seconds
    window_seconds: i64,
    /// Client buckets (would use proper storage in production)
    buckets: Mutex<HashMap<String, RateBucket>>,
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("requests_per_window", &self.requests_per_window)
            .field("window_seconds", &self.window_seconds)
            .finish()
    }
}

#[derive(Debug, Clone)]
struct RateBucket {
    tokens: u32,
    last_refill: i64,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(requests_per_window: u32, window_seconds: i64) -> Self {
        Self {
            requests_per_window,
            window_seconds,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    /// Check rate limit for client
    pub fn check(&self, client_id: &str) -> ApiResult<RateLimitInfo> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        let mut buckets = self.buckets.lock().unwrap();

        let bucket = buckets.entry(client_id.to_string()).or_insert(RateBucket {
            tokens: self.requests_per_window,
            last_refill: now,
        });

        // Refill tokens if window has passed
        let elapsed = now - bucket.last_refill;
        if elapsed >= self.window_seconds {
            bucket.tokens = self.requests_per_window;
            bucket.last_refill = now;
        }

        // Check if tokens available
        if bucket.tokens == 0 {
            let retry_after = self.window_seconds - elapsed;
            return Err(ApiError::RateLimited {
                retry_after: retry_after as u32,
            });
        }

        // Consume token
        bucket.tokens -= 1;

        Ok(RateLimitInfo {
            remaining: bucket.tokens,
            limit: self.requests_per_window,
            reset_at: bucket.last_refill + self.window_seconds,
        })
    }

    /// Apply rate limit headers to response
    pub fn apply_headers(&self, response: &mut Response, info: &RateLimitInfo) {
        response
            .headers
            .insert("x-ratelimit-remaining".into(), info.remaining.to_string());
        response
            .headers
            .insert("x-ratelimit-limit".into(), info.limit.to_string());
        response
            .headers
            .insert("x-ratelimit-reset".into(), info.reset_at.to_string());
    }
}

/// Rate limit info
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    pub remaining: u32,
    pub limit: u32,
    pub reset_at: i64,
}

/// Request logging middleware
#[derive(Debug)]
pub struct RequestLogger {
    /// Log slow requests (ms)
    slow_threshold_ms: u64,
}

impl RequestLogger {
    /// Create new request logger
    pub fn new() -> Self {
        Self {
            slow_threshold_ms: 1000,
        }
    }

    /// Set slow request threshold
    pub fn with_slow_threshold(mut self, ms: u64) -> Self {
        self.slow_threshold_ms = ms;
        self
    }

    /// Log request start
    pub fn log_start(&self, request: &Request) {
        tracing::info!(
            request_id = %request.request_id,
            method = %request.method,
            path = %request.path,
            client_ip = ?request.client_ip,
            "Request started"
        );
    }

    /// Log request completion
    pub fn log_complete(&self, request: &Request, response: &Response, duration_ms: u64) {
        if duration_ms > self.slow_threshold_ms {
            tracing::warn!(
                request_id = %request.request_id,
                method = %request.method,
                path = %request.path,
                status = response.status,
                duration_ms = duration_ms,
                "Slow request"
            );
        } else {
            tracing::info!(
                request_id = %request.request_id,
                status = response.status,
                duration_ms = duration_ms,
                "Request completed"
            );
        }
    }
}

impl Default for RequestLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// CORS middleware settings
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    pub allowed_headers: Vec<String>,
    /// Expose headers
    pub expose_headers: Vec<String>,
    /// Allow credentials
    pub allow_credentials: bool,
    /// Max age for preflight cache
    pub max_age_seconds: u32,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".into()],
            allowed_methods: vec!["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"]
                .into_iter()
                .map(String::from)
                .collect(),
            allowed_headers: vec!["Content-Type", "Authorization", "X-Request-ID"]
                .into_iter()
                .map(String::from)
                .collect(),
            expose_headers: vec!["X-RateLimit-Remaining", "X-RateLimit-Limit"]
                .into_iter()
                .map(String::from)
                .collect(),
            allow_credentials: false,
            max_age_seconds: 86400,
        }
    }
}

impl CorsConfig {
    /// Create new CORS config
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow specific origins
    pub fn with_origins(mut self, origins: Vec<String>) -> Self {
        self.allowed_origins = origins;
        self
    }

    /// Allow credentials
    pub fn with_credentials(mut self) -> Self {
        self.allow_credentials = true;
        self
    }

    /// Apply CORS headers to response
    pub fn apply(&self, request: &Request, response: &mut Response) {
        // Get origin from request
        let origin = request.header("origin").map(|s| s.as_str()).unwrap_or("*");

        // Check if origin is allowed
        let allowed_origin = if self.allowed_origins.contains(&"*".to_string()) {
            origin.to_string()
        } else if self.allowed_origins.iter().any(|o| o == origin) {
            origin.to_string()
        } else {
            return; // Origin not allowed
        };

        response
            .headers
            .insert("access-control-allow-origin".into(), allowed_origin);

        response.headers.insert(
            "access-control-allow-methods".into(),
            self.allowed_methods.join(", "),
        );

        response.headers.insert(
            "access-control-allow-headers".into(),
            self.allowed_headers.join(", "),
        );

        if !self.expose_headers.is_empty() {
            response.headers.insert(
                "access-control-expose-headers".into(),
                self.expose_headers.join(", "),
            );
        }

        if self.allow_credentials {
            response
                .headers
                .insert("access-control-allow-credentials".into(), "true".into());
        }

        response.headers.insert(
            "access-control-max-age".into(),
            self.max_age_seconds.to_string(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_skip_paths() {
        let auth = AuthMiddleware::new(b"secret");
        assert!(auth.should_skip("/health"));
        assert!(auth.should_skip("/api/v1/auth/login"));
        assert!(!auth.should_skip("/api/v1/users"));
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, 60);

        // First request should pass
        let info = limiter.check("client-1").unwrap();
        assert_eq!(info.remaining, 4);
        assert_eq!(info.limit, 5);

        // Consume all tokens
        for _ in 0..4 {
            limiter.check("client-1").unwrap();
        }

        // Should be rate limited
        let result = limiter.check("client-1");
        assert!(matches!(result, Err(ApiError::RateLimited { .. })));

        // Different client should work
        let info = limiter.check("client-2").unwrap();
        assert_eq!(info.remaining, 4);
    }

    #[test]
    fn test_rate_limit_headers() {
        let limiter = RateLimiter::new(100, 60);
        let mut response = Response::ok();

        let info = limiter.check("client").unwrap();
        limiter.apply_headers(&mut response, &info);

        assert!(response.headers.contains_key("x-ratelimit-remaining"));
        assert!(response.headers.contains_key("x-ratelimit-limit"));
    }

    #[test]
    fn test_cors_config() {
        let cors = CorsConfig::new()
            .with_origins(vec!["https://example.com".into()])
            .with_credentials();

        assert!(cors.allow_credentials);
        assert!(cors
            .allowed_origins
            .contains(&"https://example.com".to_string()));
    }

    #[test]
    fn test_cors_apply() {
        let cors = CorsConfig::new();
        let mut req = Request::new("GET", "/api");
        req.headers
            .insert("origin".into(), "https://test.com".into());

        let mut response = Response::ok();
        cors.apply(&req, &mut response);

        assert!(response.headers.contains_key("access-control-allow-origin"));
        assert!(response
            .headers
            .contains_key("access-control-allow-methods"));
    }

    #[test]
    fn test_middleware_chain() {
        fn test_middleware(req: &mut Request) -> ApiResult<()> {
            req.headers.insert("x-test".into(), "value".into());
            Ok(())
        }

        let mut chain = MiddlewareChain::new();
        chain.add("test", test_middleware);

        let mut request = Request::new("GET", "/api");
        chain.execute(&mut request).unwrap();

        assert_eq!(request.header("x-test"), Some(&"value".to_string()));
    }
}
