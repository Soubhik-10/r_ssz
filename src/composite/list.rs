//! Serializes,deserializes and merkleization of list.

use crate::{
    BYTES_PER_CHUNK, BYTES_PER_LENGTH_OFFSET, Merkleize, SSZError, SimpleDeserialize,
    SimpleSerialize, SszTypeInfo,
    merkleization::{merkleize, mix_in_length, pack},
};
use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::B256;
use core::convert::TryInto;

impl<T, const N: usize> SszTypeInfo for [T; N]
where
    T: SszTypeInfo,
{
    /// Returns true if size is fixed else false.
    fn is_fixed_size() -> bool {
        T::is_fixed_size()
    }

    /// Provides size info.
    fn fixed_size() -> Option<usize> {
        if T::is_fixed_size() {
            Some(T::fixed_size().unwrap() * N)
        } else {
            None
        }
    }
}

/// Implements serialization for list.
impl<T, const N: usize> SimpleSerialize for [T; N]
where
    T: SimpleSerialize + Clone + SszTypeInfo,
{
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let start_len = buffer.len();

        if T::is_fixed_size() {
            for item in self.iter() {
                item.serialize(buffer)?;
            }
        } else {
            let offset_bytes_len = N * BYTES_PER_LENGTH_OFFSET;
            let mut parts = Vec::with_capacity(N);

            for item in self.iter() {
                let mut part = Vec::new();
                item.serialize(&mut part)?;
                parts.push(part);
            }
            let mut offset = offset_bytes_len;
            for part in &parts {
                buffer.extend(&(offset as u32).to_le_bytes());
                offset += part.len();
            }
            for part in parts {
                buffer.extend(part);
            }
        }

        Ok(buffer.len() - start_len)
    }
}

/// Implements deserialization for list.
impl<T, const N: usize> SimpleDeserialize for [T; N]
where
    T: SimpleDeserialize + Clone + SszTypeInfo,
{
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if T::is_fixed_size() {
            let size = T::fixed_size().ok_or(SSZError::InvalidByte)?;
            let total = size * N;

            if data.len() != total {
                return Err(SSZError::InvalidLength {
                    expected: total,
                    got: data.len(),
                });
            }

            let mut out_fixed: Vec<T> = Vec::with_capacity(N);
            for i in 0..N {
                let start = i * size;
                let end = start + size;
                let item = T::deserialize(&data[start..end])?;
                out_fixed.push(item);
            }

            out_fixed
                .clone()
                .try_into()
                .map_err(|_| SSZError::InvalidLength {
                    expected: N,
                    got: out_fixed.len(),
                })
        } else {
            let offset_bytes_len = BYTES_PER_LENGTH_OFFSET * N;
            if data.len() < offset_bytes_len {
                return Err(SSZError::InvalidLength {
                    expected: offset_bytes_len,
                    got: data.len(),
                });
            }

            let mut offsets = Vec::with_capacity(N);
            for i in 0..N {
                let start = i * BYTES_PER_LENGTH_OFFSET;
                let end = start + BYTES_PER_LENGTH_OFFSET;
                let offset = u32::from_le_bytes(data[start..end].try_into().unwrap()) as usize;
                if offset > data.len() {
                    return Err(SSZError::OffsetOutOfBounds);
                }
                offsets.push(offset);
            }

            let mut out_var: Vec<T> = Vec::with_capacity(N);
            for i in 0..N {
                let start = offsets[i];
                let end = if i + 1 < N {
                    offsets[i + 1]
                } else {
                    data.len()
                };
                if start > end || end > data.len() {
                    return Err(SSZError::InvalidOffsetRange { start, end });
                }
                let item = T::deserialize(&data[start..end])?;
                out_var.push(item);
            }

            out_var
                .clone()
                .try_into()
                .map_err(|_| SSZError::InvalidLength {
                    expected: N,
                    got: out_var.len(),
                })
        }
    }
}

/// Implements `hash_tree_root` for List.
impl<T, const N: usize> Merkleize for [T; N]
where
    T: SimpleSerialize + SszTypeInfo + Clone + Merkleize,
{
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let chunks = if T::is_basic_type() {
            // For basic type arrays (always vectors since arrays are fixed-size):
            let mut serialized = vec![];
            self.serialize(&mut serialized)?;
            let mut chunks = pack(&serialized);
            if chunks.is_empty() {
                chunks.push([0u8; BYTES_PER_CHUNK]);
            }
            chunks
        } else {
            // For composite type arrays (always vectors):
            let mut chunks = Vec::with_capacity(self.len());
            for element in self {
                let hash = element.hash_tree_root()?;
                chunks.push(hash.as_slice().try_into().unwrap());
            }
            chunks
        };
        let root = merkleize(&chunks, Some(T::chunk_count()))?;
        let final_root = mix_in_length(root, self.len());
        Ok(final_root)
    }

    fn chunk_count() -> usize {
        if T::is_basic_type() {
            // (N * size_of(T) + 31) / 32 for basic types
            let elem_size = T::fixed_size().expect("Basic types should have fixed size");
            (N * elem_size).div_ceil(32)
        } else {
            N // Number of elements for composite types
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::ssz::SimpleDeserialize;
    use crate::{Merkleize, SimpleSerialize};
    use alloc::vec;
    use alloy_primitives::{
        B256,
        hex::{self, FromHex},
    };

    #[test]
    fn test_serialize_deserialize_fixed_array_u64() {
        let arr: [u64; 3] = [10, 20, 30];
        let mut buffer = vec![];
        arr.serialize(&mut buffer).unwrap();
        let deserialized = <[u64; 3]>::deserialize(&buffer).unwrap();
        assert_eq!(arr, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_array_option_u64() {
        let arr: [Option<u64>; 3] = [Some(42), None, Some(99)];
        let mut buffer = vec![];
        let _ = arr.serialize(&mut buffer).unwrap();
        let deserialized = <[Option<u64>; 3]>::deserialize(&buffer).unwrap();
        assert_eq!(arr, deserialized);
    }

    #[test]
    fn test_deserialize_invalid_length_fixed_array() {
        let bad_data = vec![0u8; 10];
        let result = <[u64; 2]>::deserialize(&bad_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_some_arrays() {
        let a = [22u8; 3];
        let mut buffer = vec![];
        a.serialize(&mut buffer).unwrap();
        let recovered_a = <[u8; 3]>::deserialize(&buffer).unwrap();
        assert_eq!(a, recovered_a);

        let a = [22u8; 333];
        let mut buffer = vec![];
        a.serialize(&mut buffer).unwrap();
        let recovered_a = <[u8; 333]>::deserialize(&buffer).unwrap();
        assert_eq!(a, recovered_a);
    }

    #[test]
    fn test_ssz_merkle() {
        let a: [u16; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
        let root = a.hash_tree_root().expect("can compute root");

        let expected =
            B256::from_hex("0xfb5fb49a69a1d04c26047dd760f560fae276a812cfecefa1f2a483d468486b0e")
                .expect("valid hex");
        assert_eq!(
            root,
            expected,
            "\nExpected: 0x{}\nActual:   0x{}",
            hex::encode(expected),
            hex::encode(root)
        );
    }
}
