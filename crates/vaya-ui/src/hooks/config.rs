//! Application Configuration
//!
//! Environment-based configuration for API endpoints and feature flags.

/// Get the API base URL from environment or use default
pub fn api_base() -> &'static str {
    // In WASM, we can't use env vars at runtime, so we use compile-time config
    // Set VAYA_API_URL env var during build to customize
    option_env!("VAYA_API_URL").unwrap_or("/api/v1")
}

/// Get the WebSocket URL for real-time updates
pub fn ws_url() -> &'static str {
    option_env!("VAYA_WS_URL").unwrap_or("/ws")
}

/// Check if running in development mode
pub fn is_dev() -> bool {
    option_env!("VAYA_ENV").map_or(true, |e| e == "development")
}

/// App version from Cargo.toml
pub fn app_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Feature flags
pub mod features {
    /// Enable demand pools feature
    pub fn pools_enabled() -> bool {
        option_env!("VAYA_FEATURE_POOLS").map_or(true, |v| v == "true")
    }

    /// Enable price alerts feature
    pub fn alerts_enabled() -> bool {
        option_env!("VAYA_FEATURE_ALERTS").map_or(true, |v| v == "true")
    }

    /// Enable dark/light theme toggle (default: dark only)
    pub fn theme_toggle_enabled() -> bool {
        option_env!("VAYA_FEATURE_THEME_TOGGLE").map_or(false, |v| v == "true")
    }
}

/// API request timeouts (in milliseconds)
pub mod timeouts {
    /// Default request timeout
    pub const DEFAULT: u32 = 30_000;
    /// Search request timeout (may be slower)
    pub const SEARCH: u32 = 60_000;
    /// Oracle prediction timeout
    pub const ORACLE: u32 = 45_000;
}
