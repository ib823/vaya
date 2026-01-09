//! Hermetic build system

use std::collections::HashMap;
use std::path::PathBuf;

use crate::{Artifact, ArtifactId, ArtifactMetadata, ForgeError, ForgeResult};

/// Build configuration
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Target name
    pub target: String,
    /// Target platform
    pub platform: String,
    /// Optimization level
    pub opt_level: u8,
    /// Enable LTO
    pub lto: bool,
    /// Strip symbols
    pub strip: bool,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Build features
    pub features: Vec<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            target: "release".into(),
            platform: format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
            opt_level: 3,
            lto: true,
            strip: true,
            env: HashMap::new(),
            features: Vec::new(),
        }
    }
}

impl BuildConfig {
    /// Create new build config
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            target: target.into(),
            ..Default::default()
        }
    }

    /// Set optimization level
    pub fn with_opt_level(mut self, level: u8) -> Self {
        self.opt_level = level.min(3);
        self
    }

    /// Add feature
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Compute config hash for reproducibility
    pub fn config_hash(&self) -> String {
        use vaya_crypto::hash::sha256;
        let config_str = format!(
            "{}:{}:{}:{}:{}:{:?}",
            self.target, self.platform, self.opt_level, self.lto, self.strip, self.features
        );
        let hash = sha256(config_str.as_bytes());
        hash.as_ref().iter().take(16).map(|b| format!("{:02x}", b)).collect()
    }
}

/// Build context
#[derive(Debug)]
pub struct BuildContext {
    /// Source directory
    pub source_dir: PathBuf,
    /// Output directory
    pub output_dir: PathBuf,
    /// Build configuration
    pub config: BuildConfig,
    /// Cached artifacts
    pub cache: HashMap<String, ArtifactId>,
}

impl BuildContext {
    /// Create new build context
    pub fn new(source_dir: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            source_dir,
            output_dir,
            config: BuildConfig::default(),
            cache: HashMap::new(),
        }
    }

    /// Set build config
    pub fn with_config(mut self, config: BuildConfig) -> Self {
        self.config = config;
        self
    }
}

/// Build result
#[derive(Debug)]
pub struct BuildResult {
    /// Built artifact
    pub artifact: Artifact,
    /// Build duration in milliseconds
    pub duration_ms: u64,
    /// Cache hit
    pub cache_hit: bool,
    /// Warnings
    pub warnings: Vec<String>,
}

/// Hermetic builder
#[derive(Debug)]
pub struct HermeticBuilder {
    /// Build context
    context: BuildContext,
}

impl HermeticBuilder {
    /// Create new builder
    pub fn new(context: BuildContext) -> Self {
        Self { context }
    }

    /// Execute build
    pub fn build(&self) -> ForgeResult<BuildResult> {
        let start = std::time::Instant::now();

        // Simulate hermetic build
        // In production, this would:
        // 1. Create isolated build environment
        // 2. Copy source files
        // 3. Execute cargo build with locked deps
        // 4. Collect artifacts

        let metadata = ArtifactMetadata::new(
            &self.context.config.target,
            env!("CARGO_PKG_VERSION"),
        );

        // Placeholder artifact data
        let data = format!(
            "VAYA Build Artifact\nTarget: {}\nPlatform: {}\nConfig: {}",
            self.context.config.target,
            self.context.config.platform,
            self.context.config.config_hash()
        ).into_bytes();

        let compressed = lz4_flex::compress_prepend_size(&data);
        let metadata = metadata.with_size(data.len(), compressed.len());

        let artifact = Artifact::new(metadata, compressed)?;

        Ok(BuildResult {
            artifact,
            duration_ms: start.elapsed().as_millis() as u64,
            cache_hit: false,
            warnings: Vec::new(),
        })
    }

    /// Check if build is cached
    pub fn is_cached(&self) -> bool {
        let config_hash = self.context.config.config_hash();
        self.context.cache.contains_key(&config_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_config_default() {
        let config = BuildConfig::default();
        assert_eq!(config.opt_level, 3);
        assert!(config.lto);
    }

    #[test]
    fn test_config_hash_reproducible() {
        let config1 = BuildConfig::new("release");
        let config2 = BuildConfig::new("release");
        assert_eq!(config1.config_hash(), config2.config_hash());
    }

    #[test]
    fn test_build_context() {
        let ctx = BuildContext::new(
            PathBuf::from("/src"),
            PathBuf::from("/out"),
        );
        assert_eq!(ctx.source_dir, PathBuf::from("/src"));
    }

    #[test]
    fn test_hermetic_build() {
        let ctx = BuildContext::new(
            PathBuf::from("/src"),
            PathBuf::from("/out"),
        );
        let builder = HermeticBuilder::new(ctx);
        let result = builder.build().unwrap();
        assert!(!result.cache_hit);
    }
}
