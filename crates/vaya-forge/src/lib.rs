//! vaya-forge: VAYA Build System
//!
//! Hermetic build system for VAYA with:
//! - Reproducible builds with content-addressed artifacts
//! - Delta updates for efficient distribution
//! - Artifact registry for versioned binaries
//! - Build verification and integrity checking

mod artifact;
mod build;
mod delta;
mod error;
mod registry;

pub use artifact::{Artifact, ArtifactId, ArtifactMetadata};
pub use build::{BuildConfig, BuildContext, BuildResult, HermeticBuilder};
pub use delta::{DeltaEncoder, DeltaPatch, DeltaResult};
pub use error::{ForgeError, ForgeResult};
pub use registry::{ArtifactRegistry, RegistryConfig};

/// Forge version
pub const FORGE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build configuration defaults
pub mod defaults {
    /// Default compression level
    pub const COMPRESSION_LEVEL: u32 = 9;
    /// Default chunk size for delta encoding
    pub const CHUNK_SIZE: usize = 4096;
    /// Maximum artifact size (1GB)
    pub const MAX_ARTIFACT_SIZE: usize = 1024 * 1024 * 1024;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forge_version() {
        assert!(!FORGE_VERSION.is_empty());
    }

    #[test]
    fn test_defaults() {
        assert!(defaults::CHUNK_SIZE > 0);
        assert!(defaults::MAX_ARTIFACT_SIZE > 0);
    }
}
