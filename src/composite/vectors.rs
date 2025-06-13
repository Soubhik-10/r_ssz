//! Serialization,deserialzation and merkleization for vectors.

use crate::SimpleDeserialize;
use crate::merkleization::{SSZType, chunk_count, pack};
use crate::{
    Merkleize,
    SSZError::{self},
    SszTypeInfo,
    merkleization::merkleize,
    ssz::SimpleSerialize,
};
use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::B256;

impl<T> SszTypeInfo for Vec<T>
where
    T: SszTypeInfo,
{
    ///Returns false since vectors are not primitive types.
    fn is_fixed_size() -> bool {
        false
    }

    ///Returns `None` since it is not of fixed size.
    fn fixed_size() -> Option<usize> {
        None
    }
}

impl<T> SimpleSerialize for Vec<T>
where
    T: SimpleSerialize + SszTypeInfo,
{
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let start_len = buffer.len();

        if T::is_fixed_size() {
            // Fixed-size elements - direct concatenation
            let fixed_size = T::fixed_size().ok_or(SSZError::InvalidByte)?;
            buffer.reserve(self.len() * fixed_size);

            for item in self {
                item.serialize(buffer)?;
            }
        } else {
            // Variable-size elements - offset-based serialization
            let offsets_len = self.len() * crate::BYTES_PER_LENGTH_OFFSET;
            buffer.reserve(offsets_len);

            // First pass - serialize all items to temporary buffers and calculate offsets
            let mut data_parts = Vec::with_capacity(self.len());
            let mut total_data_len = 0;

            for item in self {
                let mut part = Vec::new();
                item.serialize(&mut part)?;
                total_data_len += part.len();
                data_parts.push(part);
            }

            // Write offsets (pointing to the start of each variable-length element)
            let mut current_offset = offsets_len;
            for part in &data_parts {
                buffer.extend(&(current_offset as u32).to_le_bytes());
                current_offset += part.len();
            }

            // Write actual data
            buffer.reserve(total_data_len);
            for part in data_parts {
                buffer.extend(part);
            }
        }

        Ok(buffer.len() - start_len)
    }
}

impl<T> SimpleDeserialize for Vec<T>
where
    T: SimpleDeserialize + SszTypeInfo,
{
    /// Deserializes the vector.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if T::is_fixed_size() {
            let elem_size = T::fixed_size().ok_or(SSZError::InvalidLength {
                expected: 0,
                got: data.len(),
            })?;

            if data.len() % elem_size != 0 {
                return Err(SSZError::InvalidLength {
                    expected: elem_size,
                    got: data.len(),
                });
            }

            let count = data.len() / elem_size;
            let mut result = Vec::with_capacity(count);

            for i in 0..count {
                let start = i * elem_size;
                let end = start + elem_size;
                let elem_data = &data[start..end];
                let elem = T::deserialize(elem_data)?;
                result.push(elem);
            }

            Ok(result)
        } else {
            const OFFSET_SIZE: usize = crate::BYTES_PER_LENGTH_OFFSET;

            if data.len() < OFFSET_SIZE {
                return Err(SSZError::InvalidLength {
                    expected: OFFSET_SIZE,
                    got: data.len(),
                });
            }

            let mut offsets = Vec::new();
            let mut i = 0;
            while i + OFFSET_SIZE <= data.len() {
                let offset_bytes = &data[i..i + OFFSET_SIZE];
                let offset = u32::from_le_bytes(offset_bytes.try_into().unwrap()) as usize;
                if offset > data.len() {
                    return Err(SSZError::OffsetOutOfBounds);
                }
                offsets.push(offset);
                i += OFFSET_SIZE;

                if i >= offsets[0] {
                    break;
                }
            }

            let count = offsets.len();
            let mut result = Vec::with_capacity(count);

            for j in 0..count {
                let start = offsets[j];
                let end = if j + 1 < count {
                    offsets[j + 1]
                } else {
                    data.len()
                };

                if start > end || end > data.len() {
                    return Err(SSZError::InvalidOffsetRange { start, end });
                }

                let elem_data = &data[start..end];
                let elem = T::deserialize(elem_data)?;
                result.push(elem);
            }

            Ok(result)
        }
    }
}

impl<T> Merkleize for Vec<T>
where
    T: SszTypeInfo + SimpleSerialize + Merkleize,
{
    /// Calculates the `hash_tree_root` for vector.
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        if T::is_basic_type() {
            // For basic types: Serialize, pack into chunks, then merkleize.
            let mut serialized = vec![];
            self.serialize(&mut serialized)?;
            let packed = pack(&serialized);
            let chunk_count = chunk_count(SSZType::VectorBasic {
                elem_size: T::fixed_size().unwrap(),
                count: self.len(),
            });
            merkleize(&packed, Some(chunk_count))
        } else {
            // For composite types: Compute hash_tree_root for each element, collect as Vec<[u8; 32]>
            let roots: Result<Vec<[u8; 32]>, SSZError> = self
                .iter()
                .map(|element| element.hash_tree_root().map(|b256| b256.0))
                .collect();
            let roots_bytes = roots?;
            merkleize(&roots_bytes, Some(Self::chunk_count()))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::ssz::SimpleSerialize;
    use alloc::vec;
    use alloy_primitives::{B256, hex};

    #[test]
    fn test_vec_fixed_size_serialization() {
        let v: Vec<u16> = vec![1, 2, 3, 4];
        let mut buffer = vec![];
        v.serialize(&mut buffer).expect("serialize fixed size vec");
        let deserialized =
            Vec::<u16>::deserialize(&mut buffer).expect("deserialize fixed size vec");
        assert_eq!(v, deserialized);
    }

    #[test]
    fn test_vec_variable_size_serialization() {
        let v: Vec<Vec<u8>> = vec![vec![1, 2], vec![3, 4, 5], vec![6]];
        let mut buffer = vec![];
        v.serialize(&mut buffer)
            .expect("serialize variable size vec");
        let deserialized =
            Vec::<Vec<u8>>::deserialize(&mut buffer).expect("deserialize variable size vec");
        assert_eq!(v, deserialized);
    }

    #[test]
    fn test_vec_empty() {
        let v: Vec<u8> = Vec::new();
        let mut buffer = vec![];
        v.serialize(&mut buffer).expect("serialize empty vec");
        let deserialized = Vec::<u8>::deserialize(&mut buffer).expect("deserialize empty vec");
        assert_eq!(v, deserialized);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_vec_hash_tree_root() {
        let v: Vec<u8> = vec![1, 2, 3, 4];
        let root = v.hash_tree_root().expect("hash tree root for basic vec");
        let expected_root = B256::from(hex!(
            "0102030400000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(root, expected_root, "Hash tree root mismatch for basic vec");
    }
}
