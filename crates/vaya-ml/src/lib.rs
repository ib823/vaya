//! VayaML - Custom Machine Learning Library
//!
//! A pure Rust machine learning library optimized for VAYA's price prediction
//! and demand forecasting workloads.
//!
//! # NO external ML dependencies
//! - NO scikit-learn
//! - NO PyTorch
//! - NO TensorFlow
//! - NO ONNX
//!
//! Everything is built from scratch in pure Rust.
//!
//! # Components
//!
//! - `matrix` - Matrix operations with SIMD optimization
//! - `activation` - Activation functions (ReLU, sigmoid, tanh, etc.)
//! - `layer` - Neural network layers
//! - `network` - Neural network architectures
//! - `xgboost` - Gradient boosting implementation
//! - `scaler` - Feature scaling utilities

#![warn(missing_docs)]

pub mod activation;
pub mod layer;
pub mod matrix;
pub mod network;
pub mod scaler;
pub mod tree;

pub use activation::Activation;
pub use layer::Layer;
pub use matrix::Matrix;
pub use network::NeuralNetwork;
pub use scaler::StandardScaler;
pub use tree::DecisionTree;

/// Machine learning error types
#[derive(Debug)]
pub enum MlError {
    /// Matrix dimension mismatch
    DimensionMismatch {
        /// Expected dimensions
        expected: (usize, usize),
        /// Actual dimensions
        actual: (usize, usize),
    },
    /// Invalid parameter
    InvalidParameter(String),
    /// Training failed
    TrainingFailed(String),
    /// Model not trained
    NotTrained,
    /// Serialization error
    Serialization(String),
}

impl std::fmt::Display for MlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MlError::DimensionMismatch { expected, actual } => {
                write!(
                    f,
                    "Dimension mismatch: expected {:?}, got {:?}",
                    expected, actual
                )
            }
            MlError::InvalidParameter(msg) => write!(f, "Invalid parameter: {}", msg),
            MlError::TrainingFailed(msg) => write!(f, "Training failed: {}", msg),
            MlError::NotTrained => write!(f, "Model has not been trained"),
            MlError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for MlError {}

/// Result type for ML operations
pub type MlResult<T> = Result<T, MlError>;
