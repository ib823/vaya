//! VayaDB Engine - Main database interface
//!
//! This module provides the main `VayaDb` struct that coordinates all
//! database operations across the memtable, WAL, and SSTables.

use crate::config::DbConfig;
use crate::error::{DbError, DbResult};
use crate::memtable::MemTable;
use crate::sstable::{flush_memtable, SsTableMeta, SsTableReader};
use crate::wal::{Wal, WalRecord};
use parking_lot::{Mutex, RwLock};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

/// The main VayaDB database engine
pub struct VayaDb {
    /// Database configuration
    config: DbConfig,
    /// Active memtable (current writes go here)
    memtable: Arc<RwLock<MemTable>>,
    /// Immutable memtables being flushed
    immutable_memtables: Arc<Mutex<Vec<Arc<MemTable>>>>,
    /// Write-ahead log
    wal: Arc<Mutex<Option<Wal>>>,
    /// SSTable metadata organized by level
    levels: Arc<RwLock<Vec<Vec<SsTableMeta>>>>,
    /// Open SSTable readers (cached)
    readers: Arc<RwLock<BTreeMap<u64, SsTableReader>>>,
    /// Next SSTable ID
    next_sst_id: AtomicU64,
    /// Next sequence number
    sequence: AtomicU64,
    /// Whether the database is closed
    closed: AtomicBool,
}

impl VayaDb {
    /// Open or create a database at the given path
    pub fn open(config: DbConfig) -> DbResult<Self> {
        config
            .validate()
            .map_err(|e| DbError::InvalidConfig(e))?;

        // Create directories
        fs::create_dir_all(&config.path)?;
        fs::create_dir_all(config.sstables_path())?;

        // Open or create WAL
        let wal = if config.wal_enabled {
            Some(Wal::open(config.wal_path(), config.wal_sync)?)
        } else {
            None
        };

        // Load existing SSTables
        let (levels, next_sst_id, readers) = Self::load_sstables(&config)?;

        // Recover from WAL if needed
        let memtable = Arc::new(RwLock::new(MemTable::new()));
        let sequence = AtomicU64::new(1);

        if let Some(ref wal) = wal {
            let records = wal.read_all()?;
            let mt = memtable.write();
            for record in records {
                sequence.fetch_max(record.sequence + 1, Ordering::SeqCst);
                match record.record_type {
                    crate::wal::RecordType::Put => {
                        mt.put(&record.key, &record.value, record.sequence);
                    }
                    crate::wal::RecordType::Delete => {
                        mt.delete(&record.key, record.sequence);
                    }
                }
            }
        }

        Ok(Self {
            config,
            memtable,
            immutable_memtables: Arc::new(Mutex::new(Vec::new())),
            wal: Arc::new(Mutex::new(wal)),
            levels: Arc::new(RwLock::new(levels)),
            readers: Arc::new(RwLock::new(readers)),
            next_sst_id: AtomicU64::new(next_sst_id),
            sequence,
            closed: AtomicBool::new(false),
        })
    }

    /// Put a key-value pair
    pub fn put(&self, key: &[u8], value: &[u8]) -> DbResult<()> {
        self.check_closed()?;

        if value.len() > self.config.max_value_size {
            return Err(DbError::ValueTooLarge {
                size: value.len(),
                max: self.config.max_value_size,
            });
        }

        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);

        // Write to WAL first
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(&WalRecord::put(key.to_vec(), value.to_vec(), seq))?;
        }

        // Then write to memtable
        {
            let memtable = self.memtable.read();
            memtable.put(key, value, seq);
        }

        // Check if memtable needs flushing
        self.maybe_flush()?;

        Ok(())
    }

    /// Get a value by key
    pub fn get(&self, key: &[u8]) -> DbResult<Option<Vec<u8>>> {
        self.check_closed()?;

        // Check memtable first (newest data)
        {
            let memtable = self.memtable.read();
            if let Some(value) = memtable.get(key) {
                return Ok(Some(value));
            }
        }

        // Check immutable memtables
        {
            let immutables = self.immutable_memtables.lock();
            for mt in immutables.iter().rev() {
                if let Some(value) = mt.get(key) {
                    return Ok(Some(value));
                }
            }
        }

        // Check SSTables (level 0 first, then level 1, etc.)
        let levels = self.levels.read();
        let mut readers = self.readers.write();

        for level_tables in levels.iter() {
            // Level 0: check all tables (they may overlap)
            // Other levels: binary search for the right table
            for meta in level_tables.iter().rev() {
                // Skip if key is out of range
                if key < meta.smallest_key.as_slice() || key > meta.largest_key.as_slice() {
                    continue;
                }

                // Get or open the reader
                let reader = if let Some(r) = readers.get_mut(&meta.id) {
                    r
                } else {
                    let path = self.sstable_path(meta.id);
                    let r = SsTableReader::open(&path)?;
                    readers.insert(meta.id, r);
                    readers.get_mut(&meta.id).unwrap()
                };

                if let Some(value) = reader.get(key)? {
                    return Ok(Some(value));
                }
            }
        }

        Ok(None)
    }

    /// Delete a key
    pub fn delete(&self, key: &[u8]) -> DbResult<()> {
        self.check_closed()?;

        let seq = self.sequence.fetch_add(1, Ordering::SeqCst);

        // Write to WAL first
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.append(&WalRecord::delete(key.to_vec(), seq))?;
        }

        // Then write tombstone to memtable
        {
            let memtable = self.memtable.read();
            memtable.delete(key, seq);
        }

        // Check if memtable needs flushing
        self.maybe_flush()?;

        Ok(())
    }

    /// Force a memtable flush
    pub fn flush(&self) -> DbResult<()> {
        self.check_closed()?;

        let memtable = {
            let mut mt = self.memtable.write();
            let old = std::mem::replace(&mut *mt, MemTable::new());
            Arc::new(old)
        };

        if memtable.is_empty() {
            return Ok(());
        }

        // Add to immutable list
        {
            let mut immutables = self.immutable_memtables.lock();
            immutables.push(memtable.clone());
        }

        // Flush to SSTable
        let id = self.next_sst_id.fetch_add(1, Ordering::SeqCst);
        let path = self.sstable_path(id);
        let meta = flush_memtable(
            &memtable,
            &path,
            id,
            0, // Level 0
            self.config.block_size,
            self.config.compression,
        )?;

        // Add to level 0
        {
            let mut levels = self.levels.write();
            if levels.is_empty() {
                levels.push(Vec::new());
            }
            levels[0].push(meta);
        }

        // Remove from immutable list
        {
            let mut immutables = self.immutable_memtables.lock();
            immutables.retain(|mt| !Arc::ptr_eq(mt, &memtable));
        }

        // Truncate WAL
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.truncate()?;
        }

        // Maybe trigger compaction
        self.maybe_compact()?;

        Ok(())
    }

    /// Sync WAL to disk
    pub fn sync(&self) -> DbResult<()> {
        if let Some(ref mut wal) = *self.wal.lock() {
            wal.sync()?;
        }
        Ok(())
    }

    /// Close the database
    pub fn close(&self) -> DbResult<()> {
        if self.closed.swap(true, Ordering::SeqCst) {
            return Ok(()); // Already closed
        }

        // Flush any remaining data
        self.flush()?;

        // Sync WAL
        self.sync()?;

        Ok(())
    }

    /// Check if a flush is needed and perform it
    fn maybe_flush(&self) -> DbResult<()> {
        let should_flush = {
            let memtable = self.memtable.read();
            memtable.size() >= self.config.memtable_size
        };

        if should_flush {
            self.flush()?;
        }

        Ok(())
    }

    /// Check if compaction is needed and trigger it
    fn maybe_compact(&self) -> DbResult<()> {
        let needs_compaction = {
            let levels = self.levels.read();
            !levels.is_empty() && levels[0].len() > self.config.l0_compaction_threshold
        };

        if needs_compaction {
            // TODO: Implement compaction
            // For now, just log
            tracing::info!("Compaction needed but not yet implemented");
        }

        Ok(())
    }

    /// Check if the database is closed
    fn check_closed(&self) -> DbResult<()> {
        if self.closed.load(Ordering::SeqCst) {
            Err(DbError::Closed)
        } else {
            Ok(())
        }
    }

    /// Get the path for an SSTable by ID
    fn sstable_path(&self, id: u64) -> PathBuf {
        self.config.sstables_path().join(format!("{:016x}.sst", id))
    }

    /// Load existing SSTables from disk
    fn load_sstables(
        config: &DbConfig,
    ) -> DbResult<(Vec<Vec<SsTableMeta>>, u64, BTreeMap<u64, SsTableReader>)> {
        let mut levels = Vec::new();
        let mut max_id = 0u64;
        let readers = BTreeMap::new();

        let sst_dir = config.sstables_path();
        if !sst_dir.exists() {
            return Ok((levels, 1, readers));
        }

        // Read manifest if it exists
        let manifest_path = config.manifest_path();
        if manifest_path.exists() {
            // TODO: Implement manifest parsing
            // For now, scan the directory
        }

        // Scan for SSTable files
        for entry in fs::read_dir(&sst_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map_or(false, |e| e == "sst") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(id) = u64::from_str_radix(stem, 16) {
                        max_id = max_id.max(id);

                        // Read SSTable metadata
                        let reader = SsTableReader::open(&path)?;
                        // For now, put everything in level 0
                        if levels.is_empty() {
                            levels.push(Vec::new());
                        }
                        // We'd need to extract metadata from the reader
                        // For now, skip this complexity
                    }
                }
            }
        }

        Ok((levels, max_id + 1, readers))
    }

    /// Get database statistics
    pub fn stats(&self) -> DbStats {
        let memtable = self.memtable.read();
        let levels = self.levels.read();

        let mut level_stats = Vec::new();
        for (level, tables) in levels.iter().enumerate() {
            let total_size: u64 = tables.iter().map(|t| t.file_size).sum();
            let total_entries: u64 = tables.iter().map(|t| t.entry_count).sum();
            level_stats.push(LevelStats {
                level: level as u32,
                table_count: tables.len(),
                total_size,
                total_entries,
            });
        }

        DbStats {
            memtable_size: memtable.size(),
            memtable_entries: memtable.len(),
            immutable_count: self.immutable_memtables.lock().len(),
            levels: level_stats,
            sequence: self.sequence.load(Ordering::SeqCst),
        }
    }
}

impl Drop for VayaDb {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DbStats {
    /// Size of the active memtable in bytes
    pub memtable_size: usize,
    /// Number of entries in the active memtable
    pub memtable_entries: usize,
    /// Number of immutable memtables pending flush
    pub immutable_count: usize,
    /// Statistics per level
    pub levels: Vec<LevelStats>,
    /// Current sequence number
    pub sequence: u64,
}

/// Statistics for a single level
#[derive(Debug, Clone)]
pub struct LevelStats {
    /// Level number
    pub level: u32,
    /// Number of SSTables at this level
    pub table_count: usize,
    /// Total size of all SSTables at this level
    pub total_size: u64,
    /// Total number of entries at this level
    pub total_entries: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config(path: &std::path::Path) -> DbConfig {
        DbConfig::new(path)
            .memtable_size(1024) // Small for testing
            .wal_enabled(true)
    }

    #[test]
    fn test_basic_operations() {
        let tmp = TempDir::new().unwrap();
        let db = VayaDb::open(test_config(tmp.path())).unwrap();

        // Put and get
        db.put(b"key1", b"value1").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), Some(b"value1".to_vec()));

        // Overwrite
        db.put(b"key1", b"value2").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), Some(b"value2".to_vec()));

        // Delete
        db.delete(b"key1").unwrap();
        assert_eq!(db.get(b"key1").unwrap(), None);

        // Non-existent key
        assert_eq!(db.get(b"nonexistent").unwrap(), None);
    }

    #[test]
    fn test_persistence() {
        let tmp = TempDir::new().unwrap();
        let config = test_config(tmp.path());

        // Write data
        {
            let db = VayaDb::open(config.clone()).unwrap();
            db.put(b"persistent_key", b"persistent_value").unwrap();
            db.close().unwrap();
        }

        // Reopen and verify
        {
            let db = VayaDb::open(config).unwrap();
            assert_eq!(
                db.get(b"persistent_key").unwrap(),
                Some(b"persistent_value".to_vec())
            );
        }
    }

    #[test]
    fn test_flush() {
        let tmp = TempDir::new().unwrap();
        let db = VayaDb::open(test_config(tmp.path())).unwrap();

        // Write enough data to trigger flush
        for i in 0..100 {
            let key = format!("key_{:05}", i);
            let value = format!("value_{:05}", i);
            db.put(key.as_bytes(), value.as_bytes()).unwrap();
        }

        db.flush().unwrap();

        // Verify data is still accessible
        for i in 0..100 {
            let key = format!("key_{:05}", i);
            let expected = format!("value_{:05}", i);
            assert_eq!(db.get(key.as_bytes()).unwrap(), Some(expected.into_bytes()));
        }
    }

    #[test]
    fn test_stats() {
        let tmp = TempDir::new().unwrap();
        let db = VayaDb::open(test_config(tmp.path())).unwrap();

        db.put(b"key1", b"value1").unwrap();
        db.put(b"key2", b"value2").unwrap();

        let stats = db.stats();
        assert_eq!(stats.memtable_entries, 2);
        assert!(stats.memtable_size > 0);
    }
}
