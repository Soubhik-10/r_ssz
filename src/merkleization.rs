//!Helper functions for merkleization operation

use crate::{BYTES_PER_CHUNK, SSZError};
use alloy_primitives::B256;
use sha2::{Digest, Sha256};

/// Pack bytes into BYTES_PER_CHUNK-sized chunks
pub fn pack_bytes(bytes: &[u8]) -> Result<Vec<Vec<u8>>, SSZError> {
    let mut chunks = Vec::new();
    let mut chunk = vec![0u8; BYTES_PER_CHUNK];

    // Copy input bytes, leaving rest as zeros
    let len = std::cmp::min(bytes.len(), BYTES_PER_CHUNK);
    chunk[..len].copy_from_slice(&bytes[..len]);
    chunks.push(chunk);

    Ok(chunks)
}

/// Calculate next power of two
pub fn next_pow_of_two(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let n = n - 1;
    1 << (usize::BITS - n.leading_zeros())
}

/// Merkleize a sequence of chunks into a single root
pub fn merkleize(chunks: &[Vec<u8>], limit: Option<usize>) -> Result<B256, SSZError> {
    // Validate chunk sizes
    if chunks.iter().any(|c| c.len() != BYTES_PER_CHUNK) {
        return Err(SSZError::InvalidChunkSize);
    }

    // Check against limit
    if let Some(limit) = limit {
        if chunks.len() > limit {
            return Err(SSZError::ChunkCountExceedsLimit {
                limit,
                count: chunks.len(),
            });
        }
    }
    // Handle empty input
    if chunks.is_empty() {
        return Ok(B256::new([0u8; 32]));
    }

    // Calculate padded size
    let padded_len = match limit {
        Some(l) => next_pow_of_two(l),
        None => next_pow_of_two(chunks.len()),
    };

    // Single chunk case
    if padded_len == 1 {
        return Ok(B256::from_slice(&chunks[0]));
    }
    // Build merkle tree
    let mut layer = chunks.to_vec();
    while layer.len() > 1 {
        let mut next_layer = Vec::new();
        for pair in layer.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(&pair[0]);
            hasher.update(pair.get(1).unwrap_or(&pair[0]));
            next_layer.push(hasher.finalize().to_vec());
        }
        layer = next_layer;
    }

    Ok(B256::from_slice(&layer[0]))
}

/// Mix in length with a root
pub fn mix_in_length(root: B256, length: usize) -> B256 {
    let mut hasher = Sha256::new();
    hasher.update(root.as_slice());
    hasher.update((length as u64).to_le_bytes());
    B256::from_slice(&hasher.finalize())
}
/// Mix in selector with a root
pub fn mix_in_selector(root: B256, selector: u8) -> B256 {
    let mut hasher = Sha256::new();
    hasher.update(root.as_slice());
    hasher.update([selector]);
    B256::from_slice(&hasher.finalize())
}
