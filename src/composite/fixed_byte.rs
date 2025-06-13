use crate::{Merkleize, SSZError, SimpleDeserialize, SimpleSerialize, SszTypeInfo};
use alloc::vec::Vec;
use alloy_primitives::{B256, FixedBytes};

/// SSZTypeInfo implementation for FixedBytes
impl<const N: usize> SszTypeInfo for FixedBytes<N> {
    fn is_fixed_size() -> bool {
        true
    }

    fn fixed_size() -> Option<usize> {
        Some(N)
    }

    fn is_basic_type() -> bool {
        true
    }
}

impl<const N: usize> SimpleSerialize for FixedBytes<N> {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        buffer.extend_from_slice(&self.0);
        Ok(N)
    }
}

impl<const N: usize> SimpleDeserialize for FixedBytes<N> {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.len() != N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: data.len(),
            });
        }
        let bytes: [u8; N] = data.try_into().map_err(|_| SSZError::InvalidByte)?;
        Ok(FixedBytes(bytes))
    }
}

impl<const N: usize> Merkleize for FixedBytes<N> {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        self.0.hash_tree_root()
    }

    fn chunk_count() -> usize {
        <[u8; N] as Merkleize>::chunk_count()
    }
}
