// ! Serialization and deserialization for bitlist

use alloy_primitives::B256;

use crate::{
    Merkleize, SSZError, SimpleSerialize, SszTypeInfo,
    merkleization::{merkleize, mix_in_length, pack_bytes},
};

#[derive(Debug, PartialEq)]
pub struct BitList<const N: usize> {
    bits: Vec<bool>,
}

impl<const N: usize> Default for BitList<N> {
    fn default() -> Self {
        Self { bits: vec![] }
    }
}

impl<const N: usize> TryFrom<&[bool]> for BitList<N> {
    type Error = SSZError;

    fn try_from(slice: &[bool]) -> Result<Self, Self::Error> {
        BitList::from_vec(slice.to_vec())
    }
}

impl<const N: usize> BitList<N> {
    pub fn new() -> Self {
        Self { bits: vec![] }
    }

    pub fn from_vec(bits: Vec<bool>) -> Result<Self, SSZError> {
        if bits.len() > N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: bits.len(),
            });
        }
        Ok(Self { bits })
    }

    pub fn push(&mut self, bit: bool) -> Result<(), SSZError> {
        if self.bits.len() >= N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: self.bits.len() + 1,
            });
        }
        self.bits.push(bit);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }
}

impl<const N: usize> SszTypeInfo for BitList<N> {
    fn is_fixed_size() -> bool {
        false
    }

    fn fixed_size() -> Option<usize> {
        None
    }
}

impl<const N: usize> SimpleSerialize for BitList<N> {
    /// Serializes a bit list.
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        let bit_len = self.bits.len();
        if bit_len > N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: bit_len,
            });
        }

        let byte_len = bit_len.div_ceil(8) + 1;
        let mut bytes = vec![0u8; byte_len];

        for (i, &bit) in self.bits.iter().enumerate() {
            if bit {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        let dbyte = bit_len / 8;
        let dbit = bit_len % 8;
        bytes[dbyte] |= 1 << dbit;

        Ok(bytes)
    }

    /// Deserializes a bit list.    
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.is_empty() {
            return Err(SSZError::InvalidLength {
                expected: 1,
                got: 0,
            });
        }

        let last = data[data.len() - 1];
        if last == 0 {
            return Err(SSZError::OffsetOutOfBounds);
        }

        let mut bits = Vec::new();
        let total_bits = data.len() * 8;
        let mut _found_delimiter = false;
        let mut logical_bits = 0;

        #[allow(unused_labels)]
        'outer: for (i, byte) in data.iter().enumerate() {
            for j in 0..8 {
                let global_bit_index = i * 8 + j;
                if global_bit_index >= total_bits {
                    break;
                }
                if (byte >> j) & 1 != 0 {
                    logical_bits = global_bit_index;
                }
            }
        }

        for i in 0..logical_bits {
            let byte = data[i / 8];
            let bit = (byte >> (i % 8)) & 1;
            bits.push(bit == 1);
        }

        if bits.len() > N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: bits.len(),
            });
        }

        Ok(Self { bits })
    }
}

impl<const N: usize> Merkleize for BitList<N> {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        // Pack the bits into bytes (excluding delimiter)
        let mut bytes = vec![0u8; self.bits.len().div_ceil(8)];
        for (i, &bit) in self.bits.iter().enumerate() {
            if bit {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        // Pack bytes into BYTES_PER_CHUNK-byte chunks
        let chunks = pack_bytes(&bytes)?;

        // Calculate chunk count limit
        let chunk_count = Self::chunk_count();

        // Merkleize with chunk count limit
        let root = merkleize(&chunks, Some(chunk_count))?;

        // Mix in length
        let final_root = mix_in_length(root, self.bits.len());

        Ok(final_root)
    }

    fn chunk_count() -> usize {
        N.div_ceil(256) // Chunk count for BitList[N]
    }
}

#[cfg(test)]
mod tests {

    use alloy_primitives::hex;

    use super::*;

    #[test]
    fn test_bitlist_edge_cases() {
        // Test empty bitlist
        let empty: BitList<32> = BitList::default();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());

        // Serialization of empty bitlist should work
        let encoding = empty.serialize().expect("can encode empty");
        assert_eq!(encoding, vec![1]); // Just delimiter bit

        // Deserialization of empty bitlist should work
        let decoded = BitList::<32>::deserialize(&[1]).expect("can decode empty");
        assert_eq!(decoded, empty);

        // Test exceeding max length during push
        let mut list = BitList::<3>::default();
        list.push(true).unwrap();
        list.push(false).unwrap();
        list.push(true).unwrap();
        assert_eq!(list.len(), 3);

        // Pushing beyond limit should fail
        let result = list.push(true);
        assert!(matches!(
            result,
            Err(SSZError::InvalidLength {
                expected: 3,
                got: 4
            })
        ));

        // Test exceeding max length during creation
        let too_many_bits = vec![true; 4];
        let result = BitList::<3>::from_vec(too_many_bits);
        assert!(matches!(
            result,
            Err(SSZError::InvalidLength {
                expected: 3,
                got: 4
            })
        ));

        // Test deserializing invalid data
        let result = BitList::<32>::deserialize(&[]);
        assert!(matches!(
            result,
            Err(SSZError::InvalidLength {
                expected: 1,
                got: 0
            })
        ));

        // Test deserializing data without delimiter bit
        let result = BitList::<32>::deserialize(&[0]);
        assert!(matches!(result, Err(SSZError::OffsetOutOfBounds)));
    }

    #[test]
    fn test_bitlist_serialize() {
        let value: BitList<10> = BitList::default();
        let encoding = (value).serialize().expect("can encode");
        let expected = [1u8];
        assert_eq!(encoding, expected);

        let mut value: BitList<32> = BitList::default();
        let _ = value.push(false);
        let _ = value.push(true);
        let encoding = (value).serialize().expect("can encode");
        let expected = [6u8, 0u8];
        assert_eq!(encoding, expected);

        let mut value: BitList<32> = BitList::default();
        let _ = value.push(false);
        let _ = value.push(false);
        let _ = value.push(false);
        let _ = value.push(true);
        let _ = value.push(true);
        let _ = value.push(false);
        let _ = value.push(false);
        let _ = value.push(false);

        let encoding = (value).serialize().expect("can encode");
        let expected = [24u8, 1u8];
        assert_eq!(encoding, expected);
    }

    #[test]
    fn decode_bitlist() {
        let bytes = vec![1u8];
        let result = BitList::<32>::deserialize(&bytes).expect("test data is correct");
        let expected = BitList::<32>::default();
        assert_eq!(result, expected);

        let bytes = vec![24u8, 1u8];
        let result = BitList::<32>::deserialize(&bytes).expect("test data is correct");
        let expected =
            BitList::try_from([false, false, false, true, true, false, false, false].as_ref())
                .unwrap();
        assert_eq!(result, expected);

        let bytes = vec![24u8, 2u8];
        let result = BitList::<32>::deserialize(&bytes).expect("test data is correct");
        let expected = BitList::try_from(
            [false, false, false, true, true, false, false, false, false].as_ref(),
        )
        .unwrap();
        assert_eq!(result, expected);
        let bytes = vec![24u8, 3u8];
        let result = BitList::<32>::deserialize(&bytes).expect("test data is correct");
        let expected = BitList::try_from(
            [false, false, false, true, true, false, false, false, true].as_ref(),
        )
        .unwrap();
        assert_eq!(result, expected);

        let bytes = vec![24u8, 0u8];
        let result = BitList::<32>::deserialize(&bytes).expect_err("test data is incorrect");
        let expected = SSZError::OffsetOutOfBounds;
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn roundtrip_bitlist() {
        let input = BitList::<32>::try_from(
            [
                false, false, false, true, true, false, false, false, false, false, false, false,
                false, false, false, true, true, false, false, false, false, false, false, false,
                false, false, false, true, true, false, false, false,
            ]
            .as_ref(),
        )
        .unwrap();
        let buffer = input.serialize().expect("can serialize");
        let recovered = BitList::<32>::deserialize(&buffer).expect("can decode");
        assert_eq!(input, recovered);
    }

    #[test]
    fn test_bitlist_merkleization() {
        // Test empty list
        let empty: BitList<32> = BitList::default();
        let root = empty.hash_tree_root().expect("can merkleize empty list");
        assert_ne!(root, B256::default());

        // Test single bit
        let mut single = BitList::<32>::default();
        single.push(true).unwrap();
        let root_single = single.hash_tree_root().expect("can merkleize single bit");
        assert_ne!(root_single, root); // Should be different from empty

        // Test multiple bits
        let mut multi = BitList::<32>::default();
        multi.push(true).unwrap();
        multi.push(false).unwrap();
        multi.push(true).unwrap();
        let root_multi = multi.hash_tree_root().expect("can merkleize multiple bits");
        assert_ne!(root_multi, root_single); // Should be different from single

        // Test max length
        let max_bits = vec![true; 32];
        let max_list = BitList::<32>::from_vec(max_bits).unwrap();
        max_list.hash_tree_root().expect("can merkleize max length");

        // Test length limit
        let too_long = vec![true; 33];
        let result = BitList::<32>::from_vec(too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_bitlist_chunk_count() {
        assert_eq!(BitList::<256>::chunk_count(), 1);
        assert_eq!(BitList::<257>::chunk_count(), 2);
        assert_eq!(BitList::<512>::chunk_count(), 2);
        assert_eq!(BitList::<513>::chunk_count(), 3);
    }
    #[test]
    fn test_bitlist_9_merkleization_example() {
        let bits = vec![true, true, false, true, false, true, true, false, true];

        let mut bitlist = BitList::<9>::default();
        for bit in bits {
            bitlist.push(bit).expect("should push within limit");
        }

        let root = bitlist
            .hash_tree_root()
            .expect("should compute merkle root");

        assert_ne!(root, [0u8; 32], "Merkle root should not be all zero");

        println!("Merkle root for BitList<9>: 0x{}", hex::encode(root));
    }
}
