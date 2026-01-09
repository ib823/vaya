//! Error types for vaya-oracle

use std::fmt;

/// Result type for oracle operations
pub type OracleResult<T> = Result<T, OracleError>;

/// Errors that can occur during oracle operations
#[derive(Debug, Clone)]
pub enum OracleError {
    // === Data Errors ===
    /// Insufficient historical data for prediction
    InsufficientData { required: usize, available: usize },
    /// Data too old for accurate prediction
    StaleData { age_hours: u64, max_hours: u64 },
    /// No price data available for route
    NoPriceData { origin: String, destination: String },
    /// Invalid data format
    InvalidData(String),

    // === Prediction Errors ===
    /// Model not trained
    ModelNotTrained,
    /// Prediction confidence too low
    LowConfidence { confidence: f64, threshold: f64 },
    /// Date out of prediction range
    DateOutOfRange { days_ahead: u32, max_days: u32 },
    /// Route not supported
    UnsupportedRoute(String),

    // === Alert Errors ===
    /// Alert not found
    AlertNotFound(String),
    /// Alert already exists
    AlertExists(String),
    /// Alert limit reached for user
    AlertLimitReached { current: u32, max: u32 },
    /// Invalid alert threshold
    InvalidThreshold(String),
    /// Alert already triggered
    AlertAlreadyTriggered,

    // === Configuration Errors ===
    /// Invalid configuration
    InvalidConfig(String),
    /// Missing required parameter
    MissingParameter(String),

    // === System Errors ===
    /// Model error
    ModelError(String),
    /// Internal error
    Internal(String),
    /// Serialization error
    SerializationError(String),
}

impl fmt::Display for OracleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Data
            OracleError::InsufficientData { required, available } => {
                write!(f, "Insufficient data: need {} samples, have {}", required, available)
            }
            OracleError::StaleData { age_hours, max_hours } => {
                write!(f, "Data too old: {} hours old, max {} hours", age_hours, max_hours)
            }
            OracleError::NoPriceData { origin, destination } => {
                write!(f, "No price data for route {}-{}", origin, destination)
            }
            OracleError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),

            // Prediction
            OracleError::ModelNotTrained => write!(f, "Model not trained"),
            OracleError::LowConfidence { confidence, threshold } => {
                write!(f, "Confidence too low: {:.2}% < {:.2}%", confidence * 100.0, threshold * 100.0)
            }
            OracleError::DateOutOfRange { days_ahead, max_days } => {
                write!(f, "Date out of range: {} days ahead, max {} days", days_ahead, max_days)
            }
            OracleError::UnsupportedRoute(route) => write!(f, "Unsupported route: {}", route),

            // Alert
            OracleError::AlertNotFound(id) => write!(f, "Alert not found: {}", id),
            OracleError::AlertExists(id) => write!(f, "Alert already exists: {}", id),
            OracleError::AlertLimitReached { current, max } => {
                write!(f, "Alert limit reached: {} of {} max", current, max)
            }
            OracleError::InvalidThreshold(msg) => write!(f, "Invalid threshold: {}", msg),
            OracleError::AlertAlreadyTriggered => write!(f, "Alert already triggered"),

            // Config
            OracleError::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            OracleError::MissingParameter(param) => write!(f, "Missing parameter: {}", param),

            // System
            OracleError::ModelError(msg) => write!(f, "Model error: {}", msg),
            OracleError::Internal(msg) => write!(f, "Internal error: {}", msg),
            OracleError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for OracleError {}

impl OracleError {
    /// Check if error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            OracleError::StaleData { .. }
                | OracleError::LowConfidence { .. }
                | OracleError::Internal(_)
        )
    }

    /// Check if error is a data quality issue
    pub fn is_data_issue(&self) -> bool {
        matches!(
            self,
            OracleError::InsufficientData { .. }
                | OracleError::StaleData { .. }
                | OracleError::NoPriceData { .. }
                | OracleError::InvalidData(_)
        )
    }
}
