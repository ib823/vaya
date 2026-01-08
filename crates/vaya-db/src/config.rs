//! Database configuration

use std::path::PathBuf;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// Path to the database directory
    pub path: PathBuf,
    /// Maximum size of the memtable before flushing (bytes)
    pub memtable_size: usize,
    /// Maximum number of level-0 SSTables before compaction
    pub l0_compaction_threshold: usize,
    /// Size ratio between levels (level n+1 is this many times larger than level n)
    pub level_size_multiplier: usize,
    /// Maximum number of levels
    pub max_levels: usize,
    /// Block size for SSTables (bytes)
    pub block_size: usize,
    /// Enable compression for SSTables
    pub compression: bool,
    /// Enable WAL for durability
    pub wal_enabled: bool,
    /// Sync WAL on every write (slower but safer)
    pub wal_sync: bool,
    /// Maximum value size (bytes)
    pub max_value_size: usize,
    /// Bloom filter false positive rate
    pub bloom_fp_rate: f64,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            path: PathBuf::from("vaya_db"),
            memtable_size: 64 * 1024 * 1024, // 64 MB
            l0_compaction_threshold: 4,
            level_size_multiplier: 10,
            max_levels: 7,
            block_size: 4096,      // 4 KB (match OS page size)
            compression: true,
            wal_enabled: true,
            wal_sync: false,       // fsync on commit, not every write
            max_value_size: 10 * 1024 * 1024, // 10 MB
            bloom_fp_rate: 0.01,   // 1% false positive rate
        }
    }
}

impl DbConfig {
    /// Create a new configuration with the given path
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }

    /// Set the memtable size
    pub fn memtable_size(mut self, size: usize) -> Self {
        self.memtable_size = size;
        self
    }

    /// Enable or disable compression
    pub fn compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }

    /// Enable or disable WAL
    pub fn wal_enabled(mut self, enabled: bool) -> Self {
        self.wal_enabled = enabled;
        self
    }

    /// Enable or disable WAL sync on every write
    pub fn wal_sync(mut self, sync: bool) -> Self {
        self.wal_sync = sync;
        self
    }

    /// Get the WAL file path
    pub fn wal_path(&self) -> PathBuf {
        self.path.join("wal")
    }

    /// Get the SSTables directory path
    pub fn sstables_path(&self) -> PathBuf {
        self.path.join("sst")
    }

    /// Get the manifest file path
    pub fn manifest_path(&self) -> PathBuf {
        self.path.join("MANIFEST")
    }

    /// Get the lock file path
    pub fn lock_path(&self) -> PathBuf {
        self.path.join("LOCK")
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.memtable_size < 1024 {
            return Err("memtable_size must be at least 1KB".into());
        }
        if self.block_size < 512 {
            return Err("block_size must be at least 512 bytes".into());
        }
        if self.max_levels == 0 {
            return Err("max_levels must be at least 1".into());
        }
        if self.bloom_fp_rate <= 0.0 || self.bloom_fp_rate >= 1.0 {
            return Err("bloom_fp_rate must be between 0 and 1".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DbConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = DbConfig::new("/tmp/test_db")
            .memtable_size(32 * 1024 * 1024)
            .compression(false)
            .wal_enabled(true);

        assert_eq!(config.memtable_size, 32 * 1024 * 1024);
        assert!(!config.compression);
        assert!(config.wal_enabled);
    }
}
