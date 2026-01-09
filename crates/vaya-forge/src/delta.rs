//! Delta encoding for efficient updates

use crate::{ForgeError, ForgeResult};

/// Delta encoding result
pub type DeltaResult<T> = Result<T, ForgeError>;

/// Delta patch representing changes between versions
#[derive(Debug, Clone)]
pub struct DeltaPatch {
    /// Source version hash
    pub source_hash: String,
    /// Target version hash
    pub target_hash: String,
    /// Patch operations
    pub operations: Vec<PatchOp>,
    /// Compressed patch data
    pub data: Vec<u8>,
}

/// Patch operation
#[derive(Debug, Clone)]
pub enum PatchOp {
    /// Copy bytes from source
    Copy { offset: usize, length: usize },
    /// Insert new bytes
    Insert { data: Vec<u8> },
}

impl DeltaPatch {
    /// Create new delta patch
    pub fn new(source_hash: String, target_hash: String) -> Self {
        Self {
            source_hash,
            target_hash,
            operations: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Add copy operation
    pub fn add_copy(&mut self, offset: usize, length: usize) {
        self.operations.push(PatchOp::Copy { offset, length });
    }

    /// Add insert operation
    pub fn add_insert(&mut self, data: Vec<u8>) {
        self.operations.push(PatchOp::Insert { data });
    }

    /// Get patch size
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Check if patch is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}

/// Delta encoder using rolling hash
#[derive(Debug)]
pub struct DeltaEncoder {
    /// Chunk size for matching
    chunk_size: usize,
    /// Minimum match length
    min_match: usize,
}

impl Default for DeltaEncoder {
    fn default() -> Self {
        Self {
            chunk_size: crate::defaults::CHUNK_SIZE,
            min_match: 64,
        }
    }
}

impl DeltaEncoder {
    /// Create new encoder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set chunk size
    pub fn with_chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = size.max(16);
        self
    }

    /// Compute delta between source and target
    pub fn encode(&self, source: &[u8], target: &[u8]) -> DeltaResult<DeltaPatch> {
        let source_hash = hash_data(source);
        let target_hash = hash_data(target);

        let mut patch = DeltaPatch::new(source_hash, target_hash);

        if source.is_empty() {
            // Full insert
            patch.add_insert(target.to_vec());
            patch.data = lz4_flex::compress_prepend_size(target);
            return Ok(patch);
        }

        // Simple delta encoding using chunk matching
        // In production, use rsync-style rolling hash
        let mut target_pos = 0;
        let mut pending_insert = Vec::new();

        while target_pos < target.len() {
            let remaining = target.len() - target_pos;
            let chunk_len = self.chunk_size.min(remaining);
            let target_chunk = &target[target_pos..target_pos + chunk_len];

            // Try to find chunk in source
            if let Some(source_pos) = find_chunk(source, target_chunk, self.min_match) {
                // Flush pending inserts
                if !pending_insert.is_empty() {
                    patch.add_insert(std::mem::take(&mut pending_insert));
                }

                // Extend match as far as possible
                let match_len = extend_match(source, target, source_pos, target_pos);
                patch.add_copy(source_pos, match_len);
                target_pos += match_len;
            } else {
                // Add byte to pending insert
                pending_insert.push(target[target_pos]);
                target_pos += 1;
            }
        }

        // Flush remaining inserts
        if !pending_insert.is_empty() {
            patch.add_insert(pending_insert);
        }

        // Serialize and compress patch data
        patch.data = serialize_patch(&patch)?;

        Ok(patch)
    }

    /// Apply delta patch to source
    pub fn apply(&self, source: &[u8], patch: &DeltaPatch) -> DeltaResult<Vec<u8>> {
        let source_hash = hash_data(source);
        if source_hash != patch.source_hash {
            return Err(ForgeError::DeltaError("Source hash mismatch".into()));
        }

        let mut result = Vec::new();

        for op in &patch.operations {
            match op {
                PatchOp::Copy { offset, length } => {
                    if *offset + *length > source.len() {
                        return Err(ForgeError::DeltaError("Invalid copy range".into()));
                    }
                    result.extend_from_slice(&source[*offset..*offset + *length]);
                }
                PatchOp::Insert { data } => {
                    result.extend_from_slice(data);
                }
            }
        }

        // Verify result
        let result_hash = hash_data(&result);
        if result_hash != patch.target_hash {
            return Err(ForgeError::DeltaError("Result hash mismatch".into()));
        }

        Ok(result)
    }
}

/// Hash data
fn hash_data(data: &[u8]) -> String {
    use vaya_crypto::hash::sha256;
    let hash = sha256(data);
    hash.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
}

/// Find chunk in source
fn find_chunk(source: &[u8], chunk: &[u8], min_match: usize) -> Option<usize> {
    if chunk.len() < min_match || source.len() < chunk.len() {
        return None;
    }

    let search_len = min_match.min(chunk.len());
    let needle = &chunk[..search_len];

    source.windows(search_len).position(|w| w == needle)
}

/// Extend match as far as possible
fn extend_match(source: &[u8], target: &[u8], source_pos: usize, target_pos: usize) -> usize {
    let max_len = (source.len() - source_pos).min(target.len() - target_pos);
    let source_slice = &source[source_pos..source_pos + max_len];
    let target_slice = &target[target_pos..target_pos + max_len];

    source_slice
        .iter()
        .zip(target_slice.iter())
        .take_while(|(a, b)| a == b)
        .count()
        .max(1)
}

/// Serialize patch
fn serialize_patch(patch: &DeltaPatch) -> DeltaResult<Vec<u8>> {
    // Simple serialization format
    let mut data = Vec::new();

    for op in &patch.operations {
        match op {
            PatchOp::Copy { offset, length } => {
                data.push(0x01); // Copy marker
                data.extend(&(*offset as u64).to_le_bytes());
                data.extend(&(*length as u64).to_le_bytes());
            }
            PatchOp::Insert { data: insert_data } => {
                data.push(0x02); // Insert marker
                data.extend(&(insert_data.len() as u64).to_le_bytes());
                data.extend(insert_data);
            }
        }
    }

    Ok(lz4_flex::compress_prepend_size(&data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_identical() {
        let encoder = DeltaEncoder::new();
        let data = b"Hello, World!";
        let patch = encoder.encode(data, data).unwrap();
        assert!(!patch.is_empty());
    }

    #[test]
    fn test_delta_empty_source() {
        let encoder = DeltaEncoder::new();
        let patch = encoder.encode(b"", b"new data").unwrap();
        assert!(!patch.is_empty());
    }

    #[test]
    fn test_delta_apply() {
        let encoder = DeltaEncoder::new();
        let source = b"Hello, World!";
        let target = b"Hello, World! Updated";

        let patch = encoder.encode(source, target).unwrap();
        let result = encoder.apply(source, &patch).unwrap();
        assert_eq!(result, target.to_vec());
    }

    #[test]
    fn test_patch_size() {
        let patch = DeltaPatch::new("a".into(), "b".into());
        assert!(patch.is_empty());
    }
}
