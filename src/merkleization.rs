//! SSZ Merkleization helper functions.

use crate::SSZError;
use alloc::vec::Vec;
use alloy_primitives::B256;
use sha2::{Digest, Sha256};

pub const BYTES_PER_CHUNK: usize = 32;

/// Returns the next power of two >= i. 0 → 1
pub fn next_pow_of_two(i: usize) -> usize {
    if i == 0 {
        1
    } else {
        1 << (usize::BITS - (i - 1).leading_zeros())
    }
}

/// Returns the number of 32-byte chunks required for merkleization.
pub fn chunk_count(ty: SSZType) -> usize {
    match ty {
        SSZType::Basic { size: _size } => 1,
        SSZType::BitList { limit } => limit.div_ceil(256),
        SSZType::BitVector { len } => len.div_ceil(256),
        SSZType::ListBasic { elem_size, limit } => (limit * elem_size).div_ceil(32),
        SSZType::VectorBasic { elem_size, count } => (count * elem_size).div_ceil(32),
        SSZType::ListComposite { limit } => limit,
        SSZType::VectorComposite { count } => count,
        SSZType::Container { field_count } => field_count,
    }
}

/// Packs serialized basic values into 32-byte chunks with right-padding.
pub fn pack(bytes: &[u8]) -> Vec<[u8; BYTES_PER_CHUNK]> {
    let mut out = Vec::new();
    for chunk in bytes.chunks(BYTES_PER_CHUNK) {
        let mut chunk_buf = [0u8; BYTES_PER_CHUNK];
        chunk_buf[..chunk.len()].copy_from_slice(chunk);
        out.push(chunk_buf);
    }
    out
}

/// Packs bitfield bits into 32-byte chunks, excluding length bit for BitList.
pub fn pack_bits(bitfield_bytes: &[u8]) -> Vec<[u8; BYTES_PER_CHUNK]> {
    pack(bitfield_bytes)
}

/// Merkleize a list of 32-byte chunks.
/// Optionally apply a chunk count limit (e.g., for lists or bitlists).
pub fn merkleize(chunks: &[[u8; BYTES_PER_CHUNK]], limit: Option<usize>) -> Result<B256, SSZError> {
    if let Some(limit) = limit {
        if chunks.len() > limit {
            return Err(SSZError::ChunkCountExceedsLimit {
                limit,
                count: chunks.len(),
            });
        }
    }

    let padded_len = match limit {
        Some(l) => next_pow_of_two(l),
        None => next_pow_of_two(chunks.len()),
    };

    let mut layer: Vec<[u8; BYTES_PER_CHUNK]> = Vec::with_capacity(padded_len);
    layer.extend_from_slice(chunks);

    // Pad with zero chunks
    while layer.len() < padded_len {
        layer.push([0u8; BYTES_PER_CHUNK]);
    }

    if layer.len() == 1 {
        return Ok(B256::from(layer[0]));
    }

    // Merkleize
    while layer.len() > 1 {
        let mut next_layer = Vec::with_capacity(layer.len() / 2);
        for pair in layer.chunks(2) {
            let left = &pair[0];
            let right = if pair.len() == 2 { &pair[1] } else { &pair[0] };

            let mut hasher = Sha256::new();
            hasher.update(left);
            hasher.update(right);
            let hashed = hasher.finalize();
            next_layer.push(hashed.into());
        }
        layer = next_layer;
    }

    Ok(B256::from(layer[0]))
}

/// Mix in length into a Merkle root (used for lists and bitlists).
pub fn mix_in_length(root: B256, len: usize) -> B256 {
    let mut hasher = Sha256::new();
    hasher.update(root.as_slice());

    let mut len_bytes = [0u8; 32];
    len_bytes[..8].copy_from_slice(&(len as u64).to_le_bytes());
    hasher.update(len_bytes);

    B256::from_slice(&hasher.finalize())
}

/// Mix in selector (used for unions)
pub fn mix_in_selector(root: B256, selector: usize) -> B256 {
    let mut hasher = Sha256::new();
    hasher.update(root.as_slice());

    let mut sel_bytes = [0u8; 32];
    sel_bytes[..8].copy_from_slice(&(selector as u64).to_le_bytes());
    hasher.update(sel_bytes);

    B256::from_slice(&hasher.finalize())
}

/// Helper enum to represent type metadata for chunk_count
pub enum SSZType {
    Basic { size: usize },
    BitList { limit: usize },
    BitVector { len: usize },
    ListBasic { elem_size: usize, limit: usize },
    VectorBasic { elem_size: usize, count: usize },
    ListComposite { limit: usize },
    VectorComposite { count: usize },
    Container { field_count: usize },
}
