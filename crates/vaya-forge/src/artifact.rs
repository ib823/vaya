//! Artifact types and management

use std::collections::HashMap;
use time::OffsetDateTime;

use crate::{ForgeError, ForgeResult};

/// Unique artifact identifier (content-addressed hash)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArtifactId(pub String);

impl ArtifactId {
    /// Create from hex string
    pub fn from_hex(hex: &str) -> ForgeResult<Self> {
        if hex.len() != 64 {
            return Err(ForgeError::InvalidArtifact(
                "Invalid artifact ID length".into(),
            ));
        }
        Ok(Self(hex.to_string()))
    }

    /// Get as hex string
    pub fn as_hex(&self) -> &str {
        &self.0
    }

    /// Compute from content
    pub fn from_content(content: &[u8]) -> Self {
        use vaya_crypto::hash::sha256;
        let hash = sha256(content);
        Self(hex_encode(hash.as_ref()))
    }
}

/// Artifact metadata
#[derive(Debug, Clone)]
pub struct ArtifactMetadata {
    /// Artifact ID
    pub id: ArtifactId,
    /// Human-readable name
    pub name: String,
    /// Version string
    pub version: String,
    /// Target platform
    pub platform: String,
    /// Uncompressed size
    pub size: usize,
    /// Compressed size
    pub compressed_size: usize,
    /// Creation timestamp
    pub created_at: i64,
    /// Build configuration hash
    pub build_config_hash: String,
    /// Dependencies
    pub dependencies: Vec<ArtifactId>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl ArtifactMetadata {
    /// Create new metadata
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            id: ArtifactId("pending".into()),
            name: name.into(),
            version: version.into(),
            platform: current_platform(),
            size: 0,
            compressed_size: 0,
            created_at: OffsetDateTime::now_utc().unix_timestamp(),
            build_config_hash: String::new(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set artifact ID
    pub fn with_id(mut self, id: ArtifactId) -> Self {
        self.id = id;
        self
    }

    /// Set size
    pub fn with_size(mut self, size: usize, compressed: usize) -> Self {
        self.size = size;
        self.compressed_size = compressed;
        self
    }
}

/// Complete artifact with data
#[derive(Debug, Clone)]
pub struct Artifact {
    /// Metadata
    pub metadata: ArtifactMetadata,
    /// Compressed data
    pub data: Vec<u8>,
}

impl Artifact {
    /// Create new artifact from data
    pub fn new(metadata: ArtifactMetadata, data: Vec<u8>) -> ForgeResult<Self> {
        // Validate size limits
        if data.len() > crate::defaults::MAX_ARTIFACT_SIZE {
            return Err(ForgeError::SizeLimitExceeded {
                size: data.len(),
                max: crate::defaults::MAX_ARTIFACT_SIZE,
            });
        }

        // Compute content-addressed ID
        let id = ArtifactId::from_content(&data);
        let metadata = metadata.with_id(id);

        Ok(Self { metadata, data })
    }

    /// Get artifact ID
    pub fn id(&self) -> &ArtifactId {
        &self.metadata.id
    }

    /// Get data
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Verify integrity
    pub fn verify(&self) -> ForgeResult<()> {
        let computed = ArtifactId::from_content(&self.data);
        if computed != self.metadata.id {
            return Err(ForgeError::ChecksumMismatch {
                expected: self.metadata.id.as_hex().into(),
                actual: computed.as_hex().into(),
            });
        }
        Ok(())
    }
}

/// Get current platform string
fn current_platform() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Hex encode bytes
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_id_from_content() {
        let id = ArtifactId::from_content(b"test data");
        assert_eq!(id.as_hex().len(), 64);
    }

    #[test]
    fn test_artifact_metadata() {
        let meta = ArtifactMetadata::new("vaya", "1.0.0");
        assert_eq!(meta.name, "vaya");
        assert_eq!(meta.version, "1.0.0");
    }

    #[test]
    fn test_artifact_verify() {
        let meta = ArtifactMetadata::new("test", "1.0");
        let artifact = Artifact::new(meta, b"test data".to_vec()).unwrap();
        assert!(artifact.verify().is_ok());
    }
}
