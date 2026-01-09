//! Artifact registry for versioned binaries

use std::collections::HashMap;
use std::path::PathBuf;

use crate::{Artifact, ArtifactId, ArtifactMetadata, ForgeError, ForgeResult};

/// Registry configuration
#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Storage path
    pub path: PathBuf,
    /// Maximum storage size
    pub max_size: usize,
    /// Enable compression
    pub compression: bool,
    /// Enable deduplication
    pub deduplication: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("./artifacts"),
            max_size: 10 * 1024 * 1024 * 1024, // 10GB
            compression: true,
            deduplication: true,
        }
    }
}

/// Artifact registry
#[derive(Debug)]
pub struct ArtifactRegistry {
    /// Configuration
    config: RegistryConfig,
    /// Artifact index (id -> metadata)
    index: HashMap<ArtifactId, ArtifactMetadata>,
    /// Name -> version -> id mapping
    versions: HashMap<String, HashMap<String, ArtifactId>>,
    /// Current storage size
    current_size: usize,
}

impl ArtifactRegistry {
    /// Create new registry
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            config,
            index: HashMap::new(),
            versions: HashMap::new(),
            current_size: 0,
        }
    }

    /// Open existing registry or create new
    pub fn open(path: impl Into<PathBuf>) -> ForgeResult<Self> {
        let config = RegistryConfig {
            path: path.into(),
            ..Default::default()
        };

        // In production, load index from disk
        Ok(Self::new(config))
    }

    /// Store artifact
    pub fn store(&mut self, artifact: Artifact) -> ForgeResult<ArtifactId> {
        let id = artifact.id().clone();

        // Check for duplicates
        if self.config.deduplication && self.index.contains_key(&id) {
            return Ok(id);
        }

        // Check size limit
        let new_size = self.current_size + artifact.data.len();
        if new_size > self.config.max_size {
            return Err(ForgeError::SizeLimitExceeded {
                size: new_size,
                max: self.config.max_size,
            });
        }

        // Store metadata
        let metadata = artifact.metadata.clone();
        let name = metadata.name.clone();
        let version = metadata.version.clone();

        self.index.insert(id.clone(), metadata);
        self.versions
            .entry(name)
            .or_default()
            .insert(version, id.clone());
        self.current_size = new_size;

        // In production, write to disk
        Ok(id)
    }

    /// Get artifact by ID
    pub fn get(&self, id: &ArtifactId) -> ForgeResult<Option<&ArtifactMetadata>> {
        Ok(self.index.get(id))
    }

    /// Get artifact by name and version
    pub fn get_version(&self, name: &str, version: &str) -> Option<&ArtifactId> {
        self.versions.get(name)?.get(version)
    }

    /// List all versions of an artifact
    pub fn list_versions(&self, name: &str) -> Vec<&str> {
        self.versions
            .get(name)
            .map(|v| v.keys().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    }

    /// Get latest version
    pub fn latest(&self, name: &str) -> Option<&ArtifactId> {
        let versions = self.versions.get(name)?;

        // Simple version comparison (would use semver in production)
        versions
            .iter()
            .max_by(|a, b| a.0.cmp(b.0))
            .map(|(_, id)| id)
    }

    /// Delete artifact
    pub fn delete(&mut self, id: &ArtifactId) -> ForgeResult<bool> {
        if let Some(metadata) = self.index.remove(id) {
            // Remove from versions
            if let Some(versions) = self.versions.get_mut(&metadata.name) {
                versions.retain(|_, v| v != id);
            }

            self.current_size = self.current_size.saturating_sub(metadata.compressed_size);
            return Ok(true);
        }
        Ok(false)
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            artifact_count: self.index.len(),
            total_size: self.current_size,
            max_size: self.config.max_size,
            unique_names: self.versions.len(),
        }
    }

    /// Garbage collect unreferenced artifacts
    pub fn gc(&mut self) -> ForgeResult<usize> {
        // In production, remove unreferenced artifacts
        Ok(0)
    }
}

/// Registry statistics
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Number of artifacts
    pub artifact_count: usize,
    /// Total size in bytes
    pub total_size: usize,
    /// Maximum size
    pub max_size: usize,
    /// Unique artifact names
    pub unique_names: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_create() {
        let config = RegistryConfig::default();
        let registry = ArtifactRegistry::new(config);
        assert_eq!(registry.stats().artifact_count, 0);
    }

    #[test]
    fn test_registry_store() {
        let config = RegistryConfig::default();
        let mut registry = ArtifactRegistry::new(config);

        let metadata = ArtifactMetadata::new("test", "1.0.0");
        let artifact = Artifact::new(metadata, b"test data".to_vec()).unwrap();

        let id = registry.store(artifact).unwrap();
        assert!(registry.get(&id).unwrap().is_some());
    }

    #[test]
    fn test_registry_dedup() {
        let config = RegistryConfig::default();
        let mut registry = ArtifactRegistry::new(config);

        let metadata1 = ArtifactMetadata::new("test", "1.0.0");
        let artifact1 = Artifact::new(metadata1, b"test data".to_vec()).unwrap();
        let id1 = registry.store(artifact1).unwrap();

        let metadata2 = ArtifactMetadata::new("test", "1.0.1");
        let artifact2 = Artifact::new(metadata2, b"test data".to_vec()).unwrap();
        let id2 = registry.store(artifact2).unwrap();

        // Same content = same ID
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_registry_versions() {
        let config = RegistryConfig::default();
        let mut registry = ArtifactRegistry::new(config);

        let meta1 = ArtifactMetadata::new("app", "1.0.0");
        let art1 = Artifact::new(meta1, b"v1".to_vec()).unwrap();
        registry.store(art1).unwrap();

        let meta2 = ArtifactMetadata::new("app", "2.0.0");
        let art2 = Artifact::new(meta2, b"v2".to_vec()).unwrap();
        registry.store(art2).unwrap();

        let versions = registry.list_versions("app");
        assert_eq!(versions.len(), 2);
    }
}
