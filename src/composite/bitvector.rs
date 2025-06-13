//! Serialization , deserialization and merkleization for BitVector.

use crate::{
    Merkleize, SSZError, SimpleDeserialize, SimpleSerialize, SszTypeInfo,
    merkleization::{merkleize, pack_bits},
};
use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::B256;
use core::{option::Option, result::Result};

#[derive(Debug, PartialEq)]
pub struct BitVector<const N: usize> {
    bits: Vec<bool>,
}

impl<const N: usize> Default for BitVector<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> BitVector<N> {
    pub fn new() -> Self {
        Self {
            bits: vec![false; N],
        }
    }

    pub fn set(&mut self, index: usize, value: bool) -> Result<(), SSZError> {
        if index >= N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: index,
            });
        }
        self.bits[index] = value;
        Ok(())
    }
}

impl<const N: usize> SszTypeInfo for BitVector<N> {
    /// Indicates that the bit vector is fixed-size.
    fn is_fixed_size() -> bool {
        true
    }

    /// Returns the fixed size of the bit vector in bytes.
    fn fixed_size() -> Option<usize> {
        Some(N.div_ceil(8))
    }
}

impl<const N: usize> SimpleSerialize for BitVector<N> {
    /// Serializes a  bit vector.
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let byte_length = N.div_ceil(8);
        let mut bytes = vec![0u8; byte_length];

        for (i, &bit) in self.bits.iter().enumerate() {
            if bit {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        buffer.extend_from_slice(&bytes);
        Ok(byte_length)
    }
}

impl<const N: usize> SimpleDeserialize for BitVector<N> {
    /// Deserializes a bit vector.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        let expected_bytes = N.div_ceil(8);
        if data.len() != expected_bytes {
            return Err(SSZError::InvalidLength {
                expected: expected_bytes,
                got: data.len(),
            });
        }

        let mut bv = Self::new();
        for i in 0..N {
            let byte = data[i / 8];
            let bit = (byte >> (i % 8)) & 1 == 1;
            bv.bits[i] = bit;
        }

        Ok(bv)
    }
}

/// Implements `hash_tree_root` for BitVector
impl<const N: usize> Merkleize for BitVector<N> {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let mut bytes = vec![0u8; N.div_ceil(8)];
        for (i, &bit) in self.bits.iter().enumerate() {
            if bit {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        let chunks = pack_bits(&bytes);
        let chunk_count = Self::chunk_count();
        let root = merkleize(&chunks, Some(chunk_count))?;
        Ok(root)
    }

    fn chunk_count() -> usize {
        N.div_ceil(256)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloy_primitives::hex;

    #[test]
    fn test_bitvector_serialize() {
        let mut buffer = vec![];
        let mut bv = BitVector::<8>::new();
        bv.set(3, true).unwrap();
        bv.set(4, true).unwrap();
        let _ = bv.serialize(&mut buffer);
        assert_eq!(buffer, vec![24u8]);
    }

    #[test]
    fn test_bitvector_deserialize() {
        let data = vec![24u8, 1u8];
        let bv = BitVector::<12>::deserialize(&data).unwrap();
        assert_eq!(
            bv.bits,
            vec![
                false, false, false, true, true, false, false, false, true, false, false, false
            ]
        );
    }

    #[test]
    fn test_invalid_length() {
        assert!(BitVector::<8>::deserialize(&[0, 0]).is_err());
        let mut bv = BitVector::<8>::new();
        assert!(bv.set(8, true).is_err());
    }

    #[test]
    fn roundtrip_test() {
        let mut buffer = vec![];
        let input = vec![24u8, 1u8];
        let bv = BitVector::<16>::deserialize(&input).unwrap();
        let expected = vec![
            false, false, false, true, true, false, false, false, true, false, false, false, false,
            false, false, false,
        ];
        assert_eq!(bv.bits, expected);
        bv.serialize(&mut buffer).unwrap();
        assert_eq!(buffer, input);
    }

    #[test]
    fn test_bitvector_merkleization() {
        let empty = BitVector::<8>::new();
        let root = empty.hash_tree_root().expect("can merkleize empty");
        assert_eq!(
            root,
            B256::from(hex!(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ))
        );

        let mut single = BitVector::<8>::new();
        single.set(3, true).unwrap();
        let root_single = single.hash_tree_root().expect("can merkleize single");
        assert_ne!(root_single, root);

        let mut multi = BitVector::<8>::new();
        multi.set(3, true).unwrap();
        multi.set(4, true).unwrap();
        let root_multi = multi.hash_tree_root().expect("can merkleize multi");
        assert_ne!(root_multi, root_single);

        assert_eq!(BitVector::<256>::chunk_count(), 1);
        assert_eq!(BitVector::<257>::chunk_count(), 2);
    }

    #[test]
    fn test_bitvector_known_root() {
        let mut bv = BitVector::<8>::new();
        bv.set(3, true).unwrap();
        bv.set(4, true).unwrap();

        let root = bv.hash_tree_root().expect("can merkleize");
        assert_eq!(
            root,
            B256::from(hex!(
                "1800000000000000000000000000000000000000000000000000000000000000"
            ))
        );
    }

    #[test]
    fn ssz_merkle_verification() {
        let mut bv = BitVector::<11>::new();
        for (i, &bit) in [
            false, false, true, true, false, false, false, false, true, false, false,
        ]
        .iter()
        .enumerate()
        {
            bv.set(i, bit).unwrap();
        }
        let root = bv.hash_tree_root().expect("can merkleize");
        assert_eq!(
            root,
            B256::from(hex!(
                "0x0c01000000000000000000000000000000000000000000000000000000000000"
            ))
        );
    }

    #[test]
    fn ssz_merkle_verification_1() {
        let mut bv = BitVector::<32>::new();
        for (i, &bit) in [
            true, false, true, false, true, false, true, false, false, true, false, true, false,
            true, false, true, true, true, false, false, true, true, false, false, false, false,
            true, true, false, false, true, true,
        ]
        .iter()
        .enumerate()
        {
            bv.set(i, bit).unwrap();
        }
        let root = bv.hash_tree_root().expect("can merkleize");
        assert_eq!(
            root,
            B256::from(hex!(
                "0x55aa33cc00000000000000000000000000000000000000000000000000000000"
            ))
        );
    }
}
