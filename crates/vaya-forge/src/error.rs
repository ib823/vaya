//! Forge error types

use std::fmt;

/// Result type for forge operations
pub type ForgeResult<T> = Result<T, ForgeError>;

/// Forge errors
#[derive(Debug, Clone)]
pub enum ForgeError {
    /// Build failed
    BuildFailed(String),
    /// Artifact not found
    ArtifactNotFound(String),
    /// Invalid artifact
    InvalidArtifact(String),
    /// Checksum mismatch
    ChecksumMismatch { expected: String, actual: String },
    /// Compression error
    CompressionError(String),
    /// Delta encoding error
    DeltaError(String),
    /// Registry error
    RegistryError(String),
    /// IO error
    IoError(String),
    /// Size limit exceeded
    SizeLimitExceeded { size: usize, max: usize },
}

impl fmt::Display for ForgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForgeError::BuildFailed(msg) => write!(f, "Build failed: {}", msg),
            ForgeError::ArtifactNotFound(id) => write!(f, "Artifact not found: {}", id),
            ForgeError::InvalidArtifact(msg) => write!(f, "Invalid artifact: {}", msg),
            ForgeError::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "Checksum mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            ForgeError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            ForgeError::DeltaError(msg) => write!(f, "Delta error: {}", msg),
            ForgeError::RegistryError(msg) => write!(f, "Registry error: {}", msg),
            ForgeError::IoError(msg) => write!(f, "IO error: {}", msg),
            ForgeError::SizeLimitExceeded { size, max } => {
                write!(f, "Size limit exceeded: {} > {}", size, max)
            }
        }
    }
}

impl std::error::Error for ForgeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ForgeError::BuildFailed("test".into());
        assert!(err.to_string().contains("Build failed"));
    }

    #[test]
    fn test_checksum_mismatch() {
        let err = ForgeError::ChecksumMismatch {
            expected: "abc".into(),
            actual: "def".into(),
        };
        assert!(err.to_string().contains("abc"));
        assert!(err.to_string().contains("def"));
    }
}
