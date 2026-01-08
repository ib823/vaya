//! Write-Ahead Log (WAL) for durability
//!
//! The WAL ensures durability by writing all operations to disk before
//! they are acknowledged. On recovery, the WAL is replayed to restore
//! the database state.
//!
//! # Format
//!
//! Each record in the WAL has the format:
//! ```text
//! +----------+----------+----------+----------+----------+
//! | CRC (4B) | Len (4B) | Type (1B)| Key (var)| Val (var)|
//! +----------+----------+----------+----------+----------+
//! ```

use crate::error::{DbError, DbResult};
use crate::memtable::ValueType;
use crate::MAGIC_BYTES;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// WAL record type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordType {
    /// A put operation
    Put = 1,
    /// A delete operation
    Delete = 2,
}

impl TryFrom<u8> for RecordType {
    type Error = DbError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(RecordType::Put),
            2 => Ok(RecordType::Delete),
            _ => Err(DbError::WalCorruption(format!(
                "Invalid record type: {}",
                value
            ))),
        }
    }
}

impl From<ValueType> for RecordType {
    fn from(vt: ValueType) -> Self {
        match vt {
            ValueType::Put => RecordType::Put,
            ValueType::Delete => RecordType::Delete,
        }
    }
}

/// A single WAL record
#[derive(Debug, Clone)]
pub struct WalRecord {
    /// The type of operation
    pub record_type: RecordType,
    /// The key
    pub key: Vec<u8>,
    /// The value (empty for deletes)
    pub value: Vec<u8>,
    /// Sequence number
    pub sequence: u64,
}

impl WalRecord {
    /// Create a new Put record
    pub fn put(key: Vec<u8>, value: Vec<u8>, sequence: u64) -> Self {
        Self {
            record_type: RecordType::Put,
            key,
            value,
            sequence,
        }
    }

    /// Create a new Delete record
    pub fn delete(key: Vec<u8>, sequence: u64) -> Self {
        Self {
            record_type: RecordType::Delete,
            key,
            value: Vec::new(),
            sequence,
        }
    }

    /// Encode the record to bytes
    pub fn encode(&self) -> Vec<u8> {
        let key_len = self.key.len() as u32;
        let val_len = self.value.len() as u32;
        let total_len = 1 + 8 + 4 + key_len as usize + 4 + val_len as usize;

        let mut buf = Vec::with_capacity(4 + 4 + total_len);

        // Placeholder for CRC (will be calculated later)
        buf.extend_from_slice(&[0u8; 4]);

        // Length (excluding CRC and length fields)
        buf.extend_from_slice(&(total_len as u32).to_le_bytes());

        // Record type
        buf.push(self.record_type as u8);

        // Sequence number
        buf.extend_from_slice(&self.sequence.to_le_bytes());

        // Key length and key
        buf.extend_from_slice(&key_len.to_le_bytes());
        buf.extend_from_slice(&self.key);

        // Value length and value
        buf.extend_from_slice(&val_len.to_le_bytes());
        buf.extend_from_slice(&self.value);

        // Calculate and set CRC
        let crc = crc32_hash(&buf[8..]);
        buf[0..4].copy_from_slice(&crc.to_le_bytes());

        buf
    }

    /// Decode a record from bytes
    pub fn decode(data: &[u8]) -> DbResult<Self> {
        if data.len() < 8 {
            return Err(DbError::WalCorruption("Record too short".into()));
        }

        let stored_crc = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let len = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;

        if data.len() < 8 + len {
            return Err(DbError::WalCorruption("Incomplete record".into()));
        }

        // Verify CRC
        let computed_crc = crc32_hash(&data[8..8 + len]);
        if stored_crc != computed_crc {
            return Err(DbError::WalCorruption("CRC mismatch".into()));
        }

        let mut offset = 8;

        // Record type
        let record_type = RecordType::try_from(data[offset])?;
        offset += 1;

        // Sequence number
        let sequence = u64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        offset += 8;

        // Key
        let key_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let key = data[offset..offset + key_len].to_vec();
        offset += key_len;

        // Value
        let val_len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        let value = data[offset..offset + val_len].to_vec();

        Ok(Self {
            record_type,
            key,
            value,
            sequence,
        })
    }

    /// Get the encoded size of this record
    pub fn encoded_size(&self) -> usize {
        4 + 4 + 1 + 8 + 4 + self.key.len() + 4 + self.value.len()
    }
}

/// Write-Ahead Log
pub struct Wal {
    /// The underlying file
    writer: BufWriter<File>,
    /// Path to the WAL file
    path: std::path::PathBuf,
    /// Whether to sync after each write
    sync_on_write: bool,
    /// Current file size
    size: u64,
}

impl Wal {
    /// Create a new WAL file or open an existing one
    pub fn open(path: impl AsRef<Path>, sync_on_write: bool) -> DbResult<Self> {
        let path = path.as_ref().to_path_buf();

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&path)?;

        let size = file.metadata()?.len();

        // If this is a new file, write the header
        let mut writer = BufWriter::new(file);
        if size == 0 {
            // Write magic bytes and version
            writer.write_all(&MAGIC_BYTES)?;
            writer.write_all(&1u32.to_le_bytes())?; // Version 1
            writer.flush()?;
        }

        Ok(Self {
            writer,
            path,
            sync_on_write,
            size: if size == 0 { 8 } else { size },
        })
    }

    /// Append a record to the WAL
    pub fn append(&mut self, record: &WalRecord) -> DbResult<()> {
        let data = record.encode();
        self.writer.write_all(&data)?;
        self.size += data.len() as u64;

        if self.sync_on_write {
            self.sync()?;
        }

        Ok(())
    }

    /// Append multiple records atomically
    pub fn append_batch(&mut self, records: &[WalRecord]) -> DbResult<()> {
        for record in records {
            let data = record.encode();
            self.writer.write_all(&data)?;
            self.size += data.len() as u64;
        }

        if self.sync_on_write {
            self.sync()?;
        }

        Ok(())
    }

    /// Force sync to disk
    pub fn sync(&mut self) -> DbResult<()> {
        self.writer.flush()?;
        self.writer.get_ref().sync_all()?;
        Ok(())
    }

    /// Get the current size of the WAL
    pub fn size(&self) -> u64 {
        self.size
    }

    /// Read all records from the WAL
    pub fn read_all(&self) -> DbResult<Vec<WalRecord>> {
        let file = File::open(&self.path)?;
        let mut reader = BufReader::new(file);

        // Skip header
        reader.seek(SeekFrom::Start(8))?;

        let mut records = Vec::new();
        let mut buf = [0u8; 8];

        loop {
            // Try to read CRC + length
            match reader.read_exact(&mut buf) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }

            let len = u32::from_le_bytes(buf[4..8].try_into().unwrap()) as usize;

            // Read the rest of the record
            let mut record_data = vec![0u8; 8 + len];
            record_data[0..8].copy_from_slice(&buf);
            reader.read_exact(&mut record_data[8..])?;

            let record = WalRecord::decode(&record_data)?;
            records.push(record);
        }

        Ok(records)
    }

    /// Truncate the WAL (used after successful flush to SSTable)
    pub fn truncate(&mut self) -> DbResult<()> {
        // Close the current file
        self.writer.flush()?;

        // Reopen and truncate
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.path)?;

        self.writer = BufWriter::new(file);

        // Write header
        self.writer.write_all(&MAGIC_BYTES)?;
        self.writer.write_all(&1u32.to_le_bytes())?;
        self.writer.flush()?;

        self.size = 8;

        Ok(())
    }
}

/// Simple CRC32 implementation (IEEE polynomial)
fn crc32_hash(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB88320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_record_encoding() {
        let record = WalRecord::put(b"key".to_vec(), b"value".to_vec(), 42);
        let encoded = record.encode();
        let decoded = WalRecord::decode(&encoded).unwrap();

        assert_eq!(decoded.record_type, RecordType::Put);
        assert_eq!(decoded.key, b"key");
        assert_eq!(decoded.value, b"value");
        assert_eq!(decoded.sequence, 42);
    }

    #[test]
    fn test_wal_write_read() {
        let tmp = TempDir::new().unwrap();
        let wal_path = tmp.path().join("test.wal");

        // Write records
        {
            let mut wal = Wal::open(&wal_path, false).unwrap();
            wal.append(&WalRecord::put(b"k1".to_vec(), b"v1".to_vec(), 1))
                .unwrap();
            wal.append(&WalRecord::put(b"k2".to_vec(), b"v2".to_vec(), 2))
                .unwrap();
            wal.append(&WalRecord::delete(b"k1".to_vec(), 3)).unwrap();
            wal.sync().unwrap();
        }

        // Read records
        {
            let wal = Wal::open(&wal_path, false).unwrap();
            let records = wal.read_all().unwrap();

            assert_eq!(records.len(), 3);
            assert_eq!(records[0].key, b"k1");
            assert_eq!(records[1].key, b"k2");
            assert_eq!(records[2].record_type, RecordType::Delete);
        }
    }

    #[test]
    fn test_crc32() {
        assert_eq!(crc32_hash(b"hello"), 0x3610a686);
        assert_eq!(crc32_hash(b""), 0x00000000);
    }
}
