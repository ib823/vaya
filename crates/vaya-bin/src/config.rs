//! Application configuration
//!
//! Configuration is loaded from environment variables with sensible defaults.
//! No external config file parsing to maintain zero-dependency philosophy.

use std::env;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Database configuration
    pub database: DatabaseConfig,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// API configuration
    pub api: ApiConfig,
    /// Collector configuration
    pub collector: CollectorConfig,
    /// Logging configuration
    pub logging: LogConfig,
}

impl Config {
    /// Load configuration from environment
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            server: ServerConfig::from_env()?,
            database: DatabaseConfig::from_env()?,
            cache: CacheConfig::from_env()?,
            auth: AuthConfig::from_env()?,
            api: ApiConfig::from_env()?,
            collector: CollectorConfig::from_env()?,
            logging: LogConfig::from_env()?,
        })
    }

    /// Get environment name
    pub fn environment(&self) -> &str {
        &self.server.environment
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.server.environment == "production"
    }
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Bind address
    pub bind_addr: SocketAddr,
    /// Environment (development, staging, production)
    pub environment: String,
    /// Worker threads
    pub workers: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Graceful shutdown timeout in seconds
    pub shutdown_timeout: u64,
}

impl ServerConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let host: IpAddr = env::var("VAYA_HOST")
            .unwrap_or_else(|_| "0.0.0.0".into())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("VAYA_HOST".into()))?;

        let port: u16 = env::var("VAYA_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("VAYA_PORT".into()))?;

        let workers = env::var("VAYA_WORKERS")
            .unwrap_or_else(|_| num_cpus().to_string())
            .parse()
            .map_err(|_| ConfigError::InvalidValue("VAYA_WORKERS".into()))?;

        Ok(Self {
            bind_addr: SocketAddr::new(host, port),
            environment: env::var("VAYA_ENV").unwrap_or_else(|_| "development".into()),
            workers,
            request_timeout: env::var("VAYA_REQUEST_TIMEOUT")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
            shutdown_timeout: env::var("VAYA_SHUTDOWN_TIMEOUT")
                .unwrap_or_else(|_| "30".into())
                .parse()
                .unwrap_or(30),
        })
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080),
            environment: "development".into(),
            workers: num_cpus(),
            request_timeout: 30,
            shutdown_timeout: 30,
        }
    }
}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database directory
    pub data_dir: PathBuf,
    /// WAL directory (separate for performance)
    pub wal_dir: PathBuf,
    /// Max memtable size in bytes
    pub memtable_size: usize,
    /// Bloom filter false positive rate
    pub bloom_fp_rate: f64,
    /// Enable compression
    pub compression: bool,
    /// Compaction threads
    pub compaction_threads: usize,
}

impl DatabaseConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let data_dir =
            PathBuf::from(env::var("VAYA_DATA_DIR").unwrap_or_else(|_| "./data/db".into()));

        let wal_dir =
            PathBuf::from(env::var("VAYA_WAL_DIR").unwrap_or_else(|_| "./data/wal".into()));

        Ok(Self {
            data_dir,
            wal_dir,
            memtable_size: env::var("VAYA_MEMTABLE_SIZE")
                .unwrap_or_else(|_| "67108864".into()) // 64MB
                .parse()
                .unwrap_or(64 * 1024 * 1024),
            bloom_fp_rate: env::var("VAYA_BLOOM_FP_RATE")
                .unwrap_or_else(|_| "0.01".into())
                .parse()
                .unwrap_or(0.01),
            compression: env::var("VAYA_DB_COMPRESSION")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            compaction_threads: env::var("VAYA_COMPACTION_THREADS")
                .unwrap_or_else(|_| "2".into())
                .parse()
                .unwrap_or(2),
        })
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data/db"),
            wal_dir: PathBuf::from("./data/wal"),
            memtable_size: 64 * 1024 * 1024,
            bloom_fp_rate: 0.01,
            compression: true,
            compaction_threads: 2,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size in bytes
    pub max_size: usize,
    /// Default TTL in seconds
    pub default_ttl: u64,
    /// Search result cache TTL
    pub search_ttl: u64,
    /// Session cache TTL
    pub session_ttl: u64,
}

impl CacheConfig {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            max_size: env::var("VAYA_CACHE_SIZE")
                .unwrap_or_else(|_| "268435456".into()) // 256MB
                .parse()
                .unwrap_or(256 * 1024 * 1024),
            default_ttl: env::var("VAYA_CACHE_TTL")
                .unwrap_or_else(|_| "3600".into())
                .parse()
                .unwrap_or(3600),
            search_ttl: env::var("VAYA_SEARCH_CACHE_TTL")
                .unwrap_or_else(|_| "300".into()) // 5 minutes
                .parse()
                .unwrap_or(300),
            session_ttl: env::var("VAYA_SESSION_TTL")
                .unwrap_or_else(|_| "86400".into()) // 24 hours
                .parse()
                .unwrap_or(86400),
        })
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size: 256 * 1024 * 1024,
            default_ttl: 3600,
            search_ttl: 300,
            session_ttl: 86400,
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// JWT secret (must be set in production)
    pub jwt_secret: Vec<u8>,
    /// JWT access token TTL in seconds
    pub access_token_ttl: u64,
    /// JWT refresh token TTL in seconds
    pub refresh_token_ttl: u64,
    /// Password minimum length
    pub password_min_length: usize,
    /// Argon2 memory cost in KB
    pub argon2_memory: u32,
    /// Argon2 iterations
    pub argon2_iterations: u32,
    /// Max login attempts before lockout
    pub max_login_attempts: u32,
    /// Lockout duration in seconds
    pub lockout_duration: u64,
}

impl AuthConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let jwt_secret = env::var("VAYA_JWT_SECRET")
            .map(|s| s.into_bytes())
            .unwrap_or_else(|_| {
                // Only allow missing in development
                if env::var("VAYA_ENV").unwrap_or_default() == "production" {
                    Vec::new() // Will fail validation
                } else {
                    b"development-secret-do-not-use-in-production".to_vec()
                }
            });

        if jwt_secret.is_empty() {
            return Err(ConfigError::MissingRequired("VAYA_JWT_SECRET".into()));
        }

        if jwt_secret.len() < 32 {
            return Err(ConfigError::InvalidValue(
                "VAYA_JWT_SECRET must be at least 32 bytes".into(),
            ));
        }

        Ok(Self {
            jwt_secret,
            access_token_ttl: env::var("VAYA_ACCESS_TOKEN_TTL")
                .unwrap_or_else(|_| "900".into()) // 15 minutes
                .parse()
                .unwrap_or(900),
            refresh_token_ttl: env::var("VAYA_REFRESH_TOKEN_TTL")
                .unwrap_or_else(|_| "604800".into()) // 7 days
                .parse()
                .unwrap_or(604800),
            password_min_length: env::var("VAYA_PASSWORD_MIN_LENGTH")
                .unwrap_or_else(|_| "12".into())
                .parse()
                .unwrap_or(12),
            argon2_memory: env::var("VAYA_ARGON2_MEMORY")
                .unwrap_or_else(|_| "65536".into()) // 64MB
                .parse()
                .unwrap_or(65536),
            argon2_iterations: env::var("VAYA_ARGON2_ITERATIONS")
                .unwrap_or_else(|_| "3".into())
                .parse()
                .unwrap_or(3),
            max_login_attempts: env::var("VAYA_MAX_LOGIN_ATTEMPTS")
                .unwrap_or_else(|_| "5".into())
                .parse()
                .unwrap_or(5),
            lockout_duration: env::var("VAYA_LOCKOUT_DURATION")
                .unwrap_or_else(|_| "900".into()) // 15 minutes
                .parse()
                .unwrap_or(900),
        })
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: b"development-secret-do-not-use-in-production".to_vec(),
            access_token_ttl: 900,
            refresh_token_ttl: 604800,
            password_min_length: 12,
            argon2_memory: 65536,
            argon2_iterations: 3,
            max_login_attempts: 5,
            lockout_duration: 900,
        }
    }
}

/// API configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    /// API prefix
    pub prefix: String,
    /// Rate limit requests per window
    pub rate_limit_requests: u32,
    /// Rate limit window in seconds
    pub rate_limit_window: u64,
    /// Enable CORS
    pub cors_enabled: bool,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
    /// Max request body size in bytes
    pub max_body_size: usize,
}

impl ApiConfig {
    fn from_env() -> Result<Self, ConfigError> {
        let cors_origins = env::var("VAYA_CORS_ORIGINS")
            .unwrap_or_else(|_| "*".into())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Self {
            prefix: env::var("VAYA_API_PREFIX").unwrap_or_else(|_| "/api/v1".into()),
            rate_limit_requests: env::var("VAYA_RATE_LIMIT_REQUESTS")
                .unwrap_or_else(|_| "100".into())
                .parse()
                .unwrap_or(100),
            rate_limit_window: env::var("VAYA_RATE_LIMIT_WINDOW")
                .unwrap_or_else(|_| "60".into())
                .parse()
                .unwrap_or(60),
            cors_enabled: env::var("VAYA_CORS_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            cors_origins,
            max_body_size: env::var("VAYA_MAX_BODY_SIZE")
                .unwrap_or_else(|_| "1048576".into()) // 1MB
                .parse()
                .unwrap_or(1024 * 1024),
        })
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            prefix: "/api/v1".into(),
            rate_limit_requests: 100,
            rate_limit_window: 60,
            cors_enabled: true,
            cors_origins: vec!["*".into()],
            max_body_size: 1024 * 1024,
        }
    }
}

/// Collector configuration
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// Enable data collection
    pub enabled: bool,
    /// Collection interval in seconds
    pub interval: u64,
    /// Batch size for processing
    pub batch_size: usize,
    /// Retry attempts for failed collections
    pub retry_attempts: u32,
}

impl CollectorConfig {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            enabled: env::var("VAYA_COLLECTOR_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            interval: env::var("VAYA_COLLECTOR_INTERVAL")
                .unwrap_or_else(|_| "300".into()) // 5 minutes
                .parse()
                .unwrap_or(300),
            batch_size: env::var("VAYA_COLLECTOR_BATCH_SIZE")
                .unwrap_or_else(|_| "100".into())
                .parse()
                .unwrap_or(100),
            retry_attempts: env::var("VAYA_COLLECTOR_RETRIES")
                .unwrap_or_else(|_| "3".into())
                .parse()
                .unwrap_or(3),
        })
    }
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 300,
            batch_size: 100,
            retry_attempts: 3,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, pretty)
    pub format: String,
    /// Include timestamps
    pub timestamps: bool,
    /// Include file/line info
    pub file_info: bool,
}

impl LogConfig {
    fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            level: env::var("VAYA_LOG_LEVEL").unwrap_or_else(|_| "info".into()),
            format: env::var("VAYA_LOG_FORMAT").unwrap_or_else(|_| "json".into()),
            timestamps: env::var("VAYA_LOG_TIMESTAMPS")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            file_info: env::var("VAYA_LOG_FILE_INFO")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        })
    }
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".into(),
            format: "json".into(),
            timestamps: true,
            file_info: false,
        }
    }
}

/// Configuration error
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Missing required configuration
    MissingRequired(String),
    /// Invalid value
    InvalidValue(String),
    /// IO error
    IoError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingRequired(key) => {
                write!(f, "Missing required configuration: {}", key)
            }
            ConfigError::InvalidValue(msg) => write!(f, "Invalid configuration value: {}", msg),
            ConfigError::IoError(msg) => write!(f, "Configuration IO error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Get number of CPUs (simplified, no external dep)
fn num_cpus() -> usize {
    // Try to read from /proc/cpuinfo on Linux
    #[cfg(target_os = "linux")]
    {
        if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
            let count = content
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count();
            if count > 0 {
                return count;
            }
        }
    }
    // Default fallback
    4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.bind_addr.port(), 8080);
        assert_eq!(config.environment, "development");
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.memtable_size, 64 * 1024 * 1024);
        assert!(config.compression);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_size, 256 * 1024 * 1024);
        assert_eq!(config.default_ttl, 3600);
    }

    #[test]
    fn test_auth_config_default() {
        let config = AuthConfig::default();
        assert_eq!(config.access_token_ttl, 900);
        assert_eq!(config.password_min_length, 12);
    }

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.prefix, "/api/v1");
        assert_eq!(config.rate_limit_requests, 100);
        assert!(config.cors_enabled);
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "json");
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::MissingRequired("TEST_KEY".into());
        assert!(err.to_string().contains("TEST_KEY"));
    }
}
