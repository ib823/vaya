//! Application state and lifecycle management

use std::sync::Arc;
use std::time::Instant;

use vaya_api::{ApiConfig, ApiServer, RateLimiter};
use vaya_auth::{JwtTokenizer, PasswordHasher, SessionStore};
use vaya_cache::LruCache;
use vaya_db::{DbConfig, VayaDb};

use crate::config::Config;
use crate::routes;

/// Application state shared across requests
pub struct AppState {
    /// Configuration
    pub config: Arc<Config>,
    /// Database
    pub db: Arc<VayaDb>,
    /// Cache
    pub cache: Arc<LruCache<String, Vec<u8>>>,
    /// JWT tokenizer
    pub jwt: Arc<JwtTokenizer>,
    /// Password hasher
    pub hasher: Arc<PasswordHasher>,
    /// Session store
    pub sessions: Arc<SessionStore>,
    /// Rate limiter
    pub rate_limiter: Arc<RateLimiter>,
    /// Start time
    pub started_at: Instant,
}

impl AppState {
    /// Create new application state
    pub fn new(config: Config) -> Result<Self, AppError> {
        let config = Arc::new(config);

        // Initialize database
        let db_config = DbConfig::new(&config.database.data_dir)
            .memtable_size(config.database.memtable_size)
            .compression(config.database.compression);

        let db = VayaDb::open(db_config).map_err(|e| AppError::DatabaseInit(e.to_string()))?;
        let db = Arc::new(db);

        // Initialize cache
        let cache = LruCache::new(config.cache.max_size);
        let cache = Arc::new(cache);

        // Initialize auth components
        let jwt = JwtTokenizer::new(&config.auth.jwt_secret, "vaya");
        let jwt = Arc::new(jwt);

        let hasher = PasswordHasher::new();
        let hasher = Arc::new(hasher);

        let sessions = SessionStore::new();
        let sessions = Arc::new(sessions);

        // Initialize rate limiter
        let rate_limiter = RateLimiter::new(
            config.api.rate_limit_requests,
            config.api.rate_limit_window as i64,
        );
        let rate_limiter = Arc::new(rate_limiter);

        Ok(Self {
            config,
            db,
            cache,
            jwt,
            hasher,
            sessions,
            rate_limiter,
            started_at: Instant::now(),
        })
    }

    /// Get uptime in seconds
    pub fn uptime_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Check if healthy
    pub fn is_healthy(&self) -> bool {
        // Could add more checks here
        true
    }
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("environment", &self.config.server.environment)
            .field("uptime_seconds", &self.uptime_seconds())
            .finish()
    }
}

/// Application builder
pub struct AppBuilder {
    config: Config,
}

impl AppBuilder {
    /// Create new builder with config
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Build the application
    pub fn build(self) -> Result<App, AppError> {
        let state = AppState::new(self.config)?;
        let state = Arc::new(state);

        // Build API server
        let api_config = ApiConfig::new()
            .with_prefix(&state.config.api.prefix)
            .with_rate_limit(
                state.config.api.rate_limit_requests,
                state.config.api.rate_limit_window as i64,
            )
            .with_jwt_secret(state.config.auth.jwt_secret.clone())
            .with_cors_origins(state.config.api.cors_origins.clone());

        let mut server = ApiServer::new(api_config);

        // Register routes
        routes::register_routes(&mut server, Arc::clone(&state));

        Ok(App { state, server })
    }
}

/// The main application
pub struct App {
    /// Shared state
    pub state: Arc<AppState>,
    /// API server
    pub server: ApiServer,
}

impl App {
    /// Create new application from config
    pub fn new(config: Config) -> Result<Self, AppError> {
        AppBuilder::new(config).build()
    }

    /// Get state reference
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get server reference
    pub fn server(&self) -> &ApiServer {
        &self.server
    }

    /// Handle a request
    pub fn handle(&self, request: vaya_api::Request) -> vaya_api::Response {
        self.server.handle(request)
    }
}

impl std::fmt::Debug for App {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("App")
            .field("state", &self.state)
            .finish()
    }
}

/// Application error
#[derive(Debug, Clone)]
pub enum AppError {
    /// Configuration error
    Config(String),
    /// Database initialization error
    DatabaseInit(String),
    /// Cache initialization error
    CacheInit(String),
    /// Auth initialization error
    AuthInit(String),
    /// Server error
    Server(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Config(msg) => write!(f, "Configuration error: {}", msg),
            AppError::DatabaseInit(msg) => write!(f, "Database initialization error: {}", msg),
            AppError::CacheInit(msg) => write!(f, "Cache initialization error: {}", msg),
            AppError::AuthInit(msg) => write!(f, "Auth initialization error: {}", msg),
            AppError::Server(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<crate::config::ConfigError> for AppError {
    fn from(e: crate::config::ConfigError) -> Self {
        AppError::Config(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn test_config() -> Config {
        Config {
            server: crate::config::ServerConfig::default(),
            database: crate::config::DatabaseConfig::default(),
            cache: crate::config::CacheConfig::default(),
            auth: crate::config::AuthConfig::default(),
            api: crate::config::ApiConfig::default(),
            collector: crate::config::CollectorConfig::default(),
            logging: crate::config::LogConfig::default(),
        }
    }

    #[test]
    fn test_app_error_display() {
        let err = AppError::Config("test error".into());
        assert!(err.to_string().contains("Configuration error"));
    }

    #[test]
    fn test_app_builder() {
        let config = test_config();
        let _builder = AppBuilder::new(config);
        // Just verify builder can be created
    }
}
