//! Sorted String Table (SSTable) - on-disk storage format
//!
//! # File Format
//!
//! ```text
//! +----------------+
//! | Magic (4B)     |
//! +----------------+
//! | Version (4B)   |
//! +----------------+
//! | Data Blocks    |
//! | ...            |
//! +----------------+
//! | Index Block    |
//! +----------------+
//! | Bloom Filter   |
//! +----------------+
//! | Footer (48B)   |
//! +----------------+
//! ```
//!
//! Data blocks are compressed with LZ4.

use crate::error::{DbError, DbResult};
use crate::memtable::{InternalKey, MemTable, ValueType};
use crate::{MAGIC_BYTES, DB_VERSION};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// SSTable file metadata
#[derive(Debug, Clone)]
pub struct SsTableMeta {
    /// Unique table ID
    pub id: u64,
    /// Level in the LSM tree (0 = newest)
    pub level: u32,
    /// Smallest key in the table
    pub smallest_key: Vec<u8>,
    /// Largest key in the table
    pub largest_key: Vec<u8>,
    /// Number of entries
    pub entry_count: u64,
    /// File size in bytes
    pub file_size: u64,
    /// Minimum sequence number
    pub min_sequence: u64,
    /// Maximum sequence number
    pub max_sequence: u64,
}

/// Footer of an SSTable (fixed 48 bytes)
#[derive(Debug, Clone)]
struct Footer {
    /// Offset of the index block
    index_offset: u64,
    /// Size of the index block
    index_size: u64,
    /// Offset of the bloom filter
    bloom_offset: u64,
    /// Size of the bloom filter
    bloom_size: u64,
    /// Number of data blocks
    block_count: u64,
    /// Magic bytes for verification
    magic: [u8; 4],
}

impl Footer {
    const SIZE: usize = 48;

    fn encode(&self) -> [u8; Self::SIZE] {
        let mut buf = [0u8; Self::SIZE];
        buf[0..8].copy_from_slice(&self.index_offset.to_le_bytes());
        buf[8..16].copy_from_slice(&self.index_size.to_le_bytes());
        buf[16..24].copy_from_slice(&self.bloom_offset.to_le_bytes());
        buf[24..32].copy_from_slice(&self.bloom_size.to_le_bytes());
        buf[32..40].copy_from_slice(&self.block_count.to_le_bytes());
        buf[40..44].copy_from_slice(&self.magic);
        // 4 bytes padding
        buf
    }

    fn decode(data: &[u8; Self::SIZE]) -> DbResult<Self> {
        let magic: [u8; 4] = data[40..44].try_into().unwrap();
        if magic != MAGIC_BYTES {
            return Err(DbError::Corruption("Invalid SSTable magic bytes".into()));
        }

        Ok(Self {
            index_offset: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            index_size: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            bloom_offset: u64::from_le_bytes(data[16..24].try_into().unwrap()),
            bloom_size: u64::from_le_bytes(data[24..32].try_into().unwrap()),
            block_count: u64::from_le_bytes(data[32..40].try_into().unwrap()),
            magic,
        })
    }
}

/// Index entry pointing to a data block
#[derive(Debug, Clone)]
struct IndexEntry {
    /// First key in the block
    key: Vec<u8>,
    /// Offset of the block
    offset: u64,
    /// Size of the block (compressed)
    size: u64,
}

/// Simple bloom filter
pub struct BloomFilter {
    /// Bit array
    bits: Vec<u8>,
    /// Number of hash functions
    k: u32,
}

impl BloomFilter {
    /// Create a new bloom filter with the given expected number of items and false positive rate
    pub fn new(expected_items: usize, fp_rate: f64) -> Self {
        // Calculate optimal size: m = -n * ln(p) / (ln(2)^2)
        let ln2 = std::f64::consts::LN_2;
        let m = (-(expected_items as f64) * fp_rate.ln() / (ln2 * ln2)).ceil() as usize;
        let m = m.max(8); // Minimum 8 bits

        // Calculate optimal number of hash functions: k = (m/n) * ln(2)
        let k = ((m as f64 / expected_items as f64) * ln2).ceil() as u32;
        let k = k.clamp(1, 16); // Reasonable bounds

        Self {
            bits: vec![0; (m + 7) / 8],
            k,
        }
    }

    /// Add a key to the filter
    pub fn add(&mut self, key: &[u8]) {
        for i in 0..self.k {
            let hash = self.hash(key, i);
            let bit_pos = hash as usize % (self.bits.len() * 8);
            self.bits[bit_pos / 8] |= 1 << (bit_pos % 8);
        }
    }

    /// Check if a key might be in the set (may return false positives)
    pub fn may_contain(&self, key: &[u8]) -> bool {
        for i in 0..self.k {
            let hash = self.hash(key, i);
            let bit_pos = hash as usize % (self.bits.len() * 8);
            if self.bits[bit_pos / 8] & (1 << (bit_pos % 8)) == 0 {
                return false;
            }
        }
        true
    }

    /// Encode the bloom filter to bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + self.bits.len());
        buf.extend_from_slice(&self.k.to_le_bytes());
        buf.extend_from_slice(&self.bits);
        buf
    }

    /// Decode a bloom filter from bytes
    pub fn decode(data: &[u8]) -> DbResult<Self> {
        if data.len() < 4 {
            return Err(DbError::Corruption("Bloom filter too short".into()));
        }
        let k = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let bits = data[4..].to_vec();
        Ok(Self { bits, k })
    }

    /// Simple hash function using FNV-1a
    fn hash(&self, key: &[u8], seed: u32) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325 ^ (seed as u64);
        for byte in key {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash
    }
}

/// Builder for creating SSTables
pub struct SsTableBuilder {
    /// Temporary file writer
    writer: BufWriter<File>,
    /// Path to the output file
    path: PathBuf,
    /// Block size target
    block_size: usize,
    /// Enable compression
    compression: bool,
    /// Current block buffer
    current_block: Vec<u8>,
    /// Index entries
    index: Vec<IndexEntry>,
    /// Bloom filter
    bloom: BloomFilter,
    /// First key in current block
    block_first_key: Option<Vec<u8>>,
    /// Current offset in file
    offset: u64,
    /// Smallest key seen
    smallest_key: Option<Vec<u8>>,
    /// Largest key seen
    largest_key: Option<Vec<u8>>,
    /// Entry count
    entry_count: u64,
    /// Min sequence
    min_sequence: u64,
    /// Max sequence
    max_sequence: u64,
}

impl SsTableBuilder {
    /// Create a new SSTable builder
    pub fn new(
        path: impl AsRef<Path>,
        block_size: usize,
        compression: bool,
        expected_entries: usize,
    ) -> DbResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);

        // Write header
        writer.write_all(&MAGIC_BYTES)?;
        writer.write_all(&DB_VERSION.to_le_bytes())?;

        Ok(Self {
            writer,
            path,
            block_size,
            compression,
            current_block: Vec::with_capacity(block_size),
            index: Vec::new(),
            bloom: BloomFilter::new(expected_entries.max(1), 0.01),
            block_first_key: None,
            offset: 8, // After header
            smallest_key: None,
            largest_key: None,
            entry_count: 0,
            min_sequence: u64::MAX,
            max_sequence: 0,
        })
    }

    /// Add a key-value pair
    pub fn add(&mut self, key: &InternalKey, value: &[u8]) -> DbResult<()> {
        let encoded_key = key.encode();

        // Add to bloom filter
        self.bloom.add(&key.user_key);

        // Track smallest/largest
        if self.smallest_key.is_none() {
            self.smallest_key = Some(key.user_key.clone());
        }
        self.largest_key = Some(key.user_key.clone());

        // Track sequence numbers
        self.min_sequence = self.min_sequence.min(key.sequence);
        self.max_sequence = self.max_sequence.max(key.sequence);

        // Encode entry: key_len (4) + key + value_len (4) + value
        let entry_len = 4 + encoded_key.len() + 4 + value.len();

        // Check if we need to flush the current block
        if !self.current_block.is_empty() && self.current_block.len() + entry_len > self.block_size
        {
            self.flush_block()?;
        }

        // Track first key of block
        if self.block_first_key.is_none() {
            self.block_first_key = Some(encoded_key.clone());
        }

        // Add entry to current block
        self.current_block
            .extend_from_slice(&(encoded_key.len() as u32).to_le_bytes());
        self.current_block.extend_from_slice(&encoded_key);
        self.current_block
            .extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.current_block.extend_from_slice(value);

        self.entry_count += 1;

        Ok(())
    }

    /// Flush the current block to disk
    fn flush_block(&mut self) -> DbResult<()> {
        if self.current_block.is_empty() {
            return Ok(());
        }

        let first_key = self.block_first_key.take().unwrap();

        // Compress if enabled
        let block_data = if self.compression {
            compress_prepend_size(&self.current_block)
        } else {
            self.current_block.clone()
        };

        // Write block
        self.writer.write_all(&block_data)?;

        // Add index entry
        self.index.push(IndexEntry {
            key: first_key,
            offset: self.offset,
            size: block_data.len() as u64,
        });

        self.offset += block_data.len() as u64;
        self.current_block.clear();

        Ok(())
    }

    /// Finish building the SSTable and return metadata
    pub fn finish(mut self, id: u64, level: u32) -> DbResult<SsTableMeta> {
        // Flush any remaining data
        self.flush_block()?;

        // Write index block
        let index_offset = self.offset;
        let index_data = self.encode_index();
        let compressed_index = compress_prepend_size(&index_data);
        self.writer.write_all(&compressed_index)?;
        let index_size = compressed_index.len() as u64;
        self.offset += index_size;

        // Write bloom filter
        let bloom_offset = self.offset;
        let bloom_data = self.bloom.encode();
        self.writer.write_all(&bloom_data)?;
        let bloom_size = bloom_data.len() as u64;
        self.offset += bloom_size;

        // Write footer
        let footer = Footer {
            index_offset,
            index_size,
            bloom_offset,
            bloom_size,
            block_count: self.index.len() as u64,
            magic: MAGIC_BYTES,
        };
        self.writer.write_all(&footer.encode())?;

        self.writer.flush()?;
        self.writer.get_ref().sync_all()?;

        let file_size = self.offset + Footer::SIZE as u64;

        Ok(SsTableMeta {
            id,
            level,
            smallest_key: self.smallest_key.unwrap_or_default(),
            largest_key: self.largest_key.unwrap_or_default(),
            entry_count: self.entry_count,
            file_size,
            min_sequence: self.min_sequence,
            max_sequence: self.max_sequence,
        })
    }

    fn encode_index(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&(self.index.len() as u32).to_le_bytes());

        for entry in &self.index {
            buf.extend_from_slice(&(entry.key.len() as u32).to_le_bytes());
            buf.extend_from_slice(&entry.key);
            buf.extend_from_slice(&entry.offset.to_le_bytes());
            buf.extend_from_slice(&entry.size.to_le_bytes());
        }

        buf
    }
}

/// Reader for SSTables
pub struct SsTableReader {
    /// The underlying file
    reader: BufReader<File>,
    /// Path to the file
    path: PathBuf,
    /// Cached index
    index: Vec<IndexEntry>,
    /// Bloom filter
    bloom: BloomFilter,
    /// Footer info
    footer: Footer,
}

impl SsTableReader {
    /// Open an existing SSTable
    pub fn open(path: impl AsRef<Path>) -> DbResult<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path)?;
        let file_len = file.metadata()?.len();

        let mut reader = BufReader::new(file);

        // Read and verify header
        let mut header = [0u8; 8];
        reader.read_exact(&mut header)?;
        if &header[0..4] != &MAGIC_BYTES {
            return Err(DbError::Corruption("Invalid SSTable magic bytes".into()));
        }
        let version = u32::from_le_bytes(header[4..8].try_into().unwrap());
        if version != DB_VERSION {
            return Err(DbError::VersionMismatch {
                expected: DB_VERSION,
                found: version,
            });
        }

        // Read footer
        reader.seek(SeekFrom::End(-(Footer::SIZE as i64)))?;
        let mut footer_buf = [0u8; Footer::SIZE];
        reader.read_exact(&mut footer_buf)?;
        let footer = Footer::decode(&footer_buf)?;

        // Read index
        reader.seek(SeekFrom::Start(footer.index_offset))?;
        let mut index_compressed = vec![0u8; footer.index_size as usize];
        reader.read_exact(&mut index_compressed)?;
        let index_data = decompress_size_prepended(&index_compressed)
            .map_err(|e| DbError::Corruption(format!("Index decompression failed: {}", e)))?;
        let index = Self::decode_index(&index_data)?;

        // Read bloom filter
        reader.seek(SeekFrom::Start(footer.bloom_offset))?;
        let mut bloom_data = vec![0u8; footer.bloom_size as usize];
        reader.read_exact(&mut bloom_data)?;
        let bloom = BloomFilter::decode(&bloom_data)?;

        Ok(Self {
            reader,
            path,
            index,
            bloom,
            footer,
        })
    }

    /// Get a value by key
    pub fn get(&mut self, user_key: &[u8]) -> DbResult<Option<Vec<u8>>> {
        // Check bloom filter first
        if !self.bloom.may_contain(user_key) {
            return Ok(None);
        }

        // Find the block that might contain the key
        let search_key = InternalKey::put(user_key.to_vec(), u64::MAX).encode();

        // Binary search for the right block
        let block_idx = self
            .index
            .partition_point(|entry| entry.key.as_slice() <= search_key.as_slice());

        // Could be in the previous block or not exist
        let start_idx = if block_idx > 0 { block_idx - 1 } else { 0 };

        // Search in candidate blocks
        for idx in start_idx..self.index.len().min(start_idx + 2) {
            let entry = &self.index[idx];

            // Read and decompress block
            self.reader.seek(SeekFrom::Start(entry.offset))?;
            let mut compressed = vec![0u8; entry.size as usize];
            self.reader.read_exact(&mut compressed)?;

            let block_data = decompress_size_prepended(&compressed)
                .map_err(|e| DbError::Corruption(format!("Block decompression failed: {}", e)))?;

            // Search within block
            if let Some(value) = self.search_block(&block_data, user_key)? {
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    /// Search for a key within a block
    fn search_block(&self, block_data: &[u8], user_key: &[u8]) -> DbResult<Option<Vec<u8>>> {
        let mut offset = 0;

        while offset < block_data.len() {
            // Read key length
            let key_len =
                u32::from_le_bytes(block_data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;

            // Read key
            let encoded_key = &block_data[offset..offset + key_len];
            offset += key_len;

            // Read value length
            let val_len =
                u32::from_le_bytes(block_data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;

            // Read value
            let value = &block_data[offset..offset + val_len];
            offset += val_len;

            // Decode and check key
            if let Some(internal_key) = InternalKey::decode(encoded_key) {
                if internal_key.user_key == user_key {
                    match internal_key.value_type {
                        ValueType::Put => return Ok(Some(value.to_vec())),
                        ValueType::Delete => return Ok(None), // Deleted
                    }
                }
            }
        }

        Ok(None)
    }

    fn decode_index(data: &[u8]) -> DbResult<Vec<IndexEntry>> {
        let count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
        let mut entries = Vec::with_capacity(count);
        let mut offset = 4;

        for _ in 0..count {
            let key_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;

            let key = data[offset..offset + key_len].to_vec();
            offset += key_len;

            let block_offset = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
            offset += 8;

            let size = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
            offset += 8;

            entries.push(IndexEntry {
                key,
                offset: block_offset,
                size,
            });
        }

        Ok(entries)
    }

    /// Get the path to this SSTable
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Extract metadata from this SSTable
    /// This reconstructs metadata from the index for reopened tables
    pub fn metadata(&self, id: u64, level: u32) -> DbResult<SsTableMeta> {
        // Extract user key from internal key (remove sequence+type suffix)
        let extract_user_key = |internal_key: &[u8]| -> Vec<u8> {
            if internal_key.len() >= 9 {
                internal_key[..internal_key.len() - 9].to_vec()
            } else {
                internal_key.to_vec()
            }
        };

        // Scan all entries to find true smallest/largest user keys
        let mut smallest_key: Option<Vec<u8>> = None;
        let mut largest_key: Option<Vec<u8>> = None;
        let mut entry_count = 0u64;
        let mut min_seq = u64::MAX;
        let mut max_seq = 0u64;

        for entry in &self.index {
            // Read the block and scan entries
            let block_data = self.read_block(entry)?;
            let entries = Self::decode_block_entries(&block_data)?;
            entry_count += entries.len() as u64;

            for (internal_key, _) in &entries {
                let user_key = extract_user_key(internal_key);

                // Update smallest/largest
                match &smallest_key {
                    None => smallest_key = Some(user_key.clone()),
                    Some(sk) if user_key < *sk => smallest_key = Some(user_key.clone()),
                    _ => {}
                }
                match &largest_key {
                    None => largest_key = Some(user_key.clone()),
                    Some(lk) if user_key > *lk => largest_key = Some(user_key.clone()),
                    _ => {}
                }

                // Extract sequence numbers
                if internal_key.len() >= 9 {
                    let key_len = internal_key.len() - 9;
                    let seq_bytes = &internal_key[key_len..key_len + 8];
                    let inverted_seq = u64::from_be_bytes(seq_bytes.try_into().unwrap());
                    let seq = u64::MAX - inverted_seq;
                    min_seq = min_seq.min(seq);
                    max_seq = max_seq.max(seq);
                }
            }
        }

        let file_size = std::fs::metadata(&self.path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(SsTableMeta {
            id,
            level,
            smallest_key: smallest_key.unwrap_or_default(),
            largest_key: largest_key.unwrap_or_default(),
            entry_count,
            file_size,
            min_sequence: if min_seq == u64::MAX { 0 } else { min_seq },
            max_sequence: max_seq,
        })
    }

    /// Read a block from the file
    fn read_block(&self, entry: &IndexEntry) -> DbResult<Vec<u8>> {
        let mut reader = BufReader::new(File::open(&self.path)?);
        reader.seek(SeekFrom::Start(entry.offset))?;
        let mut compressed = vec![0u8; entry.size as usize];
        reader.read_exact(&mut compressed)?;
        decompress_size_prepended(&compressed)
            .map_err(|e| DbError::Corruption(format!("Block decompression failed: {}", e)))
    }

    /// Decode block entries
    /// Block format: [key_len(4) | key | value_len(4) | value]*
    fn decode_block_entries(data: &[u8]) -> DbResult<Vec<(Vec<u8>, Vec<u8>)>> {
        let mut entries = Vec::new();
        let mut offset = 0;

        while offset + 4 <= data.len() {
            // Read key length
            let key_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;

            if offset + key_len + 4 > data.len() {
                break;
            }

            // Read key
            let key = data[offset..offset + key_len].to_vec();
            offset += key_len;

            // Read value length
            let value_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
            offset += 4;

            if offset + value_len > data.len() {
                break;
            }

            // Read value
            let value = data[offset..offset + value_len].to_vec();
            offset += value_len;

            entries.push((key, value));
        }

        Ok(entries)
    }
}

/// Flush a memtable to an SSTable
pub fn flush_memtable(
    memtable: &MemTable,
    path: impl AsRef<Path>,
    id: u64,
    level: u32,
    block_size: usize,
    compression: bool,
) -> DbResult<SsTableMeta> {
    let entry_count = memtable.len();
    let mut builder = SsTableBuilder::new(path, block_size, compression, entry_count)?;

    for (key, value) in memtable.iter() {
        builder.add(&key, &value)?;
    }

    builder.finish(id, level)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bloom_filter() {
        let mut bloom = BloomFilter::new(100, 0.01);
        bloom.add(b"hello");
        bloom.add(b"world");

        assert!(bloom.may_contain(b"hello"));
        assert!(bloom.may_contain(b"world"));
        // May have false positives, but shouldn't be common
    }

    #[test]
    fn test_sstable_write_read() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("test.sst");

        // Build SSTable
        let meta = {
            let mut builder = SsTableBuilder::new(&path, 4096, true, 3).unwrap();
            builder
                .add(&InternalKey::put(b"aaa".to_vec(), 1), b"val_a")
                .unwrap();
            builder
                .add(&InternalKey::put(b"bbb".to_vec(), 2), b"val_b")
                .unwrap();
            builder
                .add(&InternalKey::put(b"ccc".to_vec(), 3), b"val_c")
                .unwrap();
            builder.finish(1, 0).unwrap()
        };

        assert_eq!(meta.entry_count, 3);
        assert_eq!(meta.smallest_key, b"aaa");
        assert_eq!(meta.largest_key, b"ccc");

        // Read SSTable
        let mut reader = SsTableReader::open(&path).unwrap();
        assert_eq!(reader.get(b"aaa").unwrap(), Some(b"val_a".to_vec()));
        assert_eq!(reader.get(b"bbb").unwrap(), Some(b"val_b".to_vec()));
        assert_eq!(reader.get(b"ccc").unwrap(), Some(b"val_c".to_vec()));
        assert_eq!(reader.get(b"ddd").unwrap(), None);
    }
}
