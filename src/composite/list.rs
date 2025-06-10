use alloy_primitives::B256;

use crate::{merkleization::{merkleize, mix_in_length, pack}, Merkleize, SSZError, SimpleSerialize, SszTypeInfo, BYTES_PER_CHUNK};
use core::convert::TryInto;

impl<T, const N: usize> SszTypeInfo for [T; N]
where
    T: SszTypeInfo,
{
    fn is_fixed_size() -> bool {
        T::is_fixed_size()
    }

    fn fixed_size() -> Option<usize> {
        if T::is_fixed_size() {
            Some(T::fixed_size().unwrap() * N)
        } else {
            None
        }
    }
}

impl<T, const N: usize> SimpleSerialize for [T; N]
where
    T: SimpleSerialize + Clone + SszTypeInfo,
{
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        if T::is_fixed_size() {
            let mut out = Vec::with_capacity(N * T::fixed_size().unwrap());
            for item in self.iter() {
                out.extend(item.serialize()?);
            }
            Ok(out)
        } else {
            // Variable-size: offsets + data
            let mut offsets = Vec::with_capacity(N);
            let mut data_parts = Vec::with_capacity(N);
            let mut offset = (crate::BYTES_PER_LENGTH_OFFSET * N) as u32;

            for item in self.iter() {
                offsets.push(offset);
                let bytes = item.serialize()?;
                offset += bytes.len() as u32;
                data_parts.push(bytes);
            }

            let mut out = Vec::with_capacity(offset as usize);
            for off in offsets {
                out.extend(off.to_le_bytes());
            }
            for part in data_parts {
                out.extend(part);
            }
            Ok(out)
        }
    }

    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if T::is_fixed_size() {
            let size = T::fixed_size().unwrap();
            let total = N * size;
            if data.len() != total {
                return Err(SSZError::InvalidLength {
                    expected: total,
                    got: data.len(),
                });
            }

            let mut elements = Vec::with_capacity(N);
            for i in 0..N {
                let start = i * size;
                let end = start + size;
                let elem = T::deserialize(&data[start..end])?;
                elements.push(elem);
            }
            // Convert Vec<T> to [T; N]
            elements
                .clone()
                .into_iter()
                .collect::<Vec<T>>()
                .try_into()
                .map_err(|_| SSZError::InvalidLength {
                    expected: N,
                    got: elements.len(),
                })
        } else {
            let offset_size = crate::BYTES_PER_LENGTH_OFFSET;
            let expected_header = offset_size * N;
            if data.len() < expected_header {
                return Err(SSZError::InvalidLength {
                    expected: expected_header,
                    got: data.len(),
                });
            }

            let mut offsets = Vec::with_capacity(N);
            for i in 0..N {
                let start = i * offset_size;
                let end = start + offset_size;
                let offset_bytes = &data[start..end];
                let offset = u32::from_le_bytes(offset_bytes.try_into().unwrap()) as usize;
                if offset > data.len() {
                    return Err(SSZError::OffsetOutOfBounds);
                }
                offsets.push(offset);
            }

            let mut elements = Vec::with_capacity(N);
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
                let elem = T::deserialize(&data[start..end])?;
                elements.push(elem);
            }
            elements
                .to_vec()
                .try_into()
                .map_err(|_| SSZError::InvalidLength {
                    expected: N,
                    got: elements.len(),
                })
        }
    }
}

impl<T, const N: usize> Merkleize for [T; N]
where
    T: SimpleSerialize + SszTypeInfo + Clone + Merkleize,
{
fn hash_tree_root(&self) -> Result<B256, SSZError> {
    if T::is_basic_type() {
        // For basic type arrays (always vectors since arrays are fixed-size):
       
        let serialized = self.serialize()?;
        
        
        let mut chunks = pack(&serialized);
        
        
        if chunks.is_empty() {
            chunks.push([0u8; BYTES_PER_CHUNK]);
        }
        
        
        let root = merkleize(&chunks, Some(T::chunk_count()))?;
        let hashed_root=mix_in_length(root, self.len());
        
        Ok(hashed_root)
    } else {
        // For composite type arrays (always vectors):
        let mut chunk_roots = Vec::with_capacity(self.len());
        
        for element in self {
            let hash = element.hash_tree_root()?;
            chunk_roots.push(hash.as_slice().try_into().unwrap());
        }
        
        
        let root = merkleize(&chunk_roots, Some(T::chunk_count()))?;
         let hashed_root=mix_in_length(root, self.len());

        Ok(hashed_root)
    }
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
    use alloy_primitives::{hex::{self, FromHex}, B256};

    use crate::{ Merkleize,SimpleSerialize};

    #[test]
    fn test_serialize_deserialize_fixed_array_u64() {
        let arr: [u64; 3] = [10, 20, 30];
        let serialized = arr.serialize().unwrap();
        let deserialized = <[u64; 3]>::deserialize(&serialized).unwrap();
        assert_eq!(arr, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_array_option_u64() {
        // Assume Option<u64> uses tagged serialization (0 for None, 1 + serialized u64 for Some)
        let arr: [Option<u64>; 3] = [Some(42), None, Some(99)];
        let serialized = arr.serialize().unwrap();
        let deserialized = <[Option<u64>; 3]>::deserialize(&serialized).unwrap();
        assert_eq!(arr, deserialized);
    }

    #[test]
    fn test_deserialize_invalid_length_fixed_array() {
        // Length not a multiple of 8 bytes (u64 size)
        let bad_data = vec![0u8; 10];
        let result = <[u64; 2]>::deserialize(&bad_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_some_arrays() {
        let a = [22u8; 3];
        let serialized_a = a.serialize().unwrap();
        let recovered_a = <[u8; 3]>::deserialize(&serialized_a).unwrap();
        assert_eq!(a, recovered_a);

        let a = [22u8; 333];
        let serialized_a = a.serialize().unwrap();
        let recovered_a = <[u8; 333]>::deserialize(&serialized_a).unwrap();
        assert_eq!(a, recovered_a);
    }

#[test]
fn test_ssz_merkle() {
    let a:[u16;8] = [1,2,3,4,5,6,7,8];
    let root = a.hash_tree_root().expect("can compute root");
    
    let expected = B256::from_hex(
        "0xfb5fb49a69a1d04c26047dd760f560fae276a812cfecefa1f2a483d468486b0e"
    ).expect("valid hex");
    assert_eq!(root, expected, "\nExpected: 0x{}\nActual:   0x{}", 
        hex::encode(expected),
        hex::encode(root)
    );
}
}