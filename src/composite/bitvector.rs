// ! Serialization and deserialization for BitVector

use alloy_primitives::B256;

use crate::{
    Merkleize, SSZError, SimpleSerialize, SszTypeInfo,
    merkleization::{merkleize, pack_bits},
};

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
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        let byte_length = N.div_ceil(8);
        let mut bytes = vec![0u8; byte_length];

        for (i, &bit) in self.bits.iter().enumerate() {
            if bit {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        Ok(bytes)
    }

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

/// implements `hash_tree_root` for BitVector
impl<const N: usize> Merkleize for BitVector<N> {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        // Pack the bits into bytes
        let mut bytes = vec![0u8; N.div_ceil(8)];
        for (i, &bit) in self.bits.iter().enumerate() {
            if bit {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        // Pack bytes into BYTES_PER_CHUNK-byte chunks
        let chunks = pack_bits(&bytes);

        // Calculate chunk count limit for bitvector: (N + 255) // 256
        let chunk_count = Self::chunk_count();

        // Merkleize with chunk count limit
        let root = merkleize(&chunks, Some(chunk_count))?;

        Ok(root)
    }

    fn chunk_count() -> usize {
        N.div_ceil(256)
    }
}

#[cfg(test)]
mod tests {

    use alloy_primitives::hex;

    use super::*;

    #[test]
    fn test_bitvector_serialize() {
        let mut bv = BitVector::<8>::new();
        bv.set(3, true).unwrap();
        bv.set(4, true).unwrap();
        assert_eq!(bv.serialize(), Ok(vec![24u8]));
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
        let input = vec![24u8, 1u8];
        let bv = BitVector::<16>::deserialize(&input).unwrap();
        let expected = vec![
            false, false, false, true, true, false, false, false, true, false, false, false, false,
            false, false, false,
        ];
        assert_eq!(bv.bits, expected);
        let serialized = bv.serialize().unwrap();
        assert_eq!(serialized, input);
    }

    #[test]
    fn test_bitvector_merkleization() {
        // Test empty bitvector
        let empty = BitVector::<8>::new();
        let root = empty.hash_tree_root().expect("can merkleize empty");
        assert_eq!(
            root,
            B256::from(hex!(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ))
        );

        // Test single bit set
        let mut single = BitVector::<8>::new();
        single.set(3, true).unwrap();
        let root_single = single.hash_tree_root().expect("can merkleize single");
        assert_ne!(root_single, root);

        // Test multiple bits
        let mut multi = BitVector::<8>::new();
        multi.set(3, true).unwrap();
        multi.set(4, true).unwrap();
        let root_multi = multi.hash_tree_root().expect("can merkleize multi");
        assert_ne!(root_multi, root_single);

        // Test chunk count calculation
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
