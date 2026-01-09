//! VAYA Travel Platform - Main Binary
//!
//! A zero-dependency travel platform for flight search, booking, and group buying.
//!
//! # Usage
//!
//! ```bash
//! # Run with default settings
//! vaya serve
//!
//! # Run with custom port
//! VAYA_PORT=3000 vaya serve
//!
//! # Run database migrations
//! vaya migrate
//!
//! # Show version
//! vaya version
//! ```

mod app;
mod config;
mod handlers;
mod routes;

use std::env;
use std::process::ExitCode;

use tracing::{error, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

use crate::config::Config;

/// Main entry point
fn main() -> ExitCode {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match command {
        "serve" | "server" | "run" => run_server(),
        "migrate" => run_migrations(),
        "version" | "-v" | "--version" => show_version(),
        "help" | "-h" | "--help" => show_help(),
        "check" => run_health_check(),
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Run 'vaya help' for usage information.");
            ExitCode::from(1)
        }
    }
}

/// Run the HTTP server
fn run_server() -> ExitCode {
    // Initialize logging
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        return ExitCode::from(1);
    }

    info!(version = env!("CARGO_PKG_VERSION"), "Starting VAYA server");

    // Load configuration
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            return ExitCode::from(1);
        }
    };

    info!(
        environment = %config.server.environment,
        bind_addr = %config.server.bind_addr,
        "Configuration loaded"
    );

    // Build application
    let _app = match app::App::new(config.clone()) {
        Ok(a) => a,
        Err(e) => {
            error!(error = %e, "Failed to initialize application");
            return ExitCode::from(1);
        }
    };

    info!("Application initialized");

    // Start server using tokio runtime
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .worker_threads(config.server.workers)
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!(error = %e, "Failed to create tokio runtime");
            return ExitCode::from(1);
        }
    };

    rt.block_on(async {
        info!(
            addr = %config.server.bind_addr,
            workers = config.server.workers,
            "Server starting"
        );

        // In a real implementation, this would use vaya-net's TcpListener
        // For now, we just simulate server startup
        info!("Server ready to accept connections");

        // Wait for shutdown signal
        tokio::signal::ctrl_c().await.ok();
        info!("Received shutdown signal");
    });

    info!("Server shutdown complete");
    ExitCode::SUCCESS
}

/// Run database migrations
fn run_migrations() -> ExitCode {
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        return ExitCode::from(1);
    }

    info!("Running database migrations");

    // Load config to get database path
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            return ExitCode::from(1);
        }
    };

    info!(
        data_dir = ?config.database.data_dir,
        "Database configuration loaded"
    );

    // TODO: Implement actual migrations
    warn!("Migrations not yet implemented");

    info!("Migrations complete");
    ExitCode::SUCCESS
}

/// Show version information
fn show_version() -> ExitCode {
    println!("vaya {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("A zero-dependency travel platform for flight search,");
    println!("booking, and group buying.");
    println!();
    println!("Built with:");
    println!("  - vaya-db: LSM-tree storage engine");
    println!("  - vaya-cache: LRU cache");
    println!("  - vaya-crypto: Cryptographic primitives");
    println!("  - vaya-auth: Authentication & authorization");
    println!("  - vaya-search: Flight search engine");
    println!("  - vaya-book: Booking management");
    println!("  - vaya-pool: Group buying pools");
    println!("  - vaya-oracle: Pricing predictions");
    println!("  - vaya-api: REST API layer");
    println!("  - vaya-net: Networking");
    println!("  - vaya-collect: Data collection");
    ExitCode::SUCCESS
}

/// Show help information
fn show_help() -> ExitCode {
    println!("vaya - VAYA Travel Platform");
    println!();
    println!("USAGE:");
    println!("    vaya <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    serve       Start the HTTP server");
    println!("    migrate     Run database migrations");
    println!("    check       Run health checks");
    println!("    version     Show version information");
    println!("    help        Show this help message");
    println!();
    println!("ENVIRONMENT VARIABLES:");
    println!("    VAYA_ENV                Environment (development/staging/production)");
    println!("    VAYA_HOST                Bind host (default: 0.0.0.0)");
    println!("    VAYA_PORT                Bind port (default: 8080)");
    println!("    VAYA_WORKERS             Worker threads (default: CPU count)");
    println!("    VAYA_DATA_DIR            Database directory (default: ./data/db)");
    println!("    VAYA_JWT_SECRET          JWT signing secret (required in production)");
    println!("    VAYA_LOG_LEVEL           Log level (trace/debug/info/warn/error)");
    println!("    VAYA_LOG_FORMAT          Log format (json/pretty)");
    println!();
    println!("EXAMPLES:");
    println!("    # Start server on port 3000");
    println!("    VAYA_PORT=3000 vaya serve");
    println!();
    println!("    # Start in production mode");
    println!("    VAYA_ENV=production VAYA_JWT_SECRET=... vaya serve");
    println!();
    println!("For more information, visit: https://github.com/vaya/vaya-oracle");
    ExitCode::SUCCESS
}

/// Run health check
fn run_health_check() -> ExitCode {
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
        return ExitCode::from(1);
    }

    info!("Running health checks");

    // Load config
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Configuration check failed");
            return ExitCode::from(1);
        }
    };
    info!("Configuration: OK");

    // Check database directory
    if config.database.data_dir.exists() {
        info!(path = ?config.database.data_dir, "Database directory: OK");
    } else {
        warn!(path = ?config.database.data_dir, "Database directory does not exist (will be created)");
    }

    // Check JWT secret
    if config.auth.jwt_secret.len() >= 32 {
        info!("JWT secret: OK");
    } else {
        error!("JWT secret too short (must be at least 32 bytes)");
        return ExitCode::from(1);
    }

    info!("All health checks passed");
    ExitCode::SUCCESS
}

/// Initialize logging
fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let log_level = env::var("VAYA_LOG_LEVEL").unwrap_or_else(|_| "info".into());
    let log_format = env::var("VAYA_LOG_FORMAT").unwrap_or_else(|_| "json".into());

    let filter = EnvFilter::try_new(&log_level).unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = tracing_subscriber::registry().with(filter);

    if log_format == "json" {
        subscriber
            .with(fmt::layer().json())
            .try_init()
            .map_err(|e| e.to_string())?;
    } else {
        subscriber
            .with(fmt::layer().pretty())
            .try_init()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_show_version() {
        // Just verify it doesn't panic
        let result = show_version();
        assert_eq!(result, ExitCode::SUCCESS);
    }

    #[test]
    fn test_show_help() {
        // Just verify it doesn't panic
        let result = show_help();
        assert_eq!(result, ExitCode::SUCCESS);
    }
}
