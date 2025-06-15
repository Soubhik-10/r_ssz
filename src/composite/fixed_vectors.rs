//! Serializes , deserializes and merkleization of fixed vector.

use crate::{
    Merkleize, SSZError, SimpleDeserialize, SimpleSerialize, SszTypeInfo,
    merkleization::{SSZType, chunk_count, merkleize, pack},
};
use alloc::{vec, vec::Vec};
use alloy_primitives::B256;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixedVector<T, const N: usize>([T; N]);

impl<T, const N: usize> FixedVector<T, N> {
    pub fn new(data: [T; N]) -> Self {
        Self(data)
    }
}

impl<T, const N: usize> Deref for FixedVector<T, N> {
    type Target = [T; N];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const N: usize> DerefMut for FixedVector<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Provides `SszTypeInfo` for fixed vector.
impl<T, const N: usize> SszTypeInfo for FixedVector<T, N>
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

/// Implements serialization of fixed vector.
impl<T, const N: usize> SimpleSerialize for FixedVector<T, N>
where
    T: SimpleSerialize + SszTypeInfo,
{
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let start_len = buffer.len();

        if T::is_fixed_size() {
            for item in &self.0 {
                item.serialize(buffer)?;
            }
        } else {
            let mut data_parts = Vec::with_capacity(N);
            #[allow(unused_variables)]
            let mut total_data_len = 0;

            for item in &self.0 {
                let mut part = Vec::new();
                item.serialize(&mut part)?;
                total_data_len += part.len();
                data_parts.push(part);
            }

            let offsets_len = N * crate::BYTES_PER_LENGTH_OFFSET;
            let mut current_offset = offsets_len;
            for part in &data_parts {
                buffer.extend(&(current_offset as u32).to_le_bytes());
                current_offset += part.len();
            }

            for part in data_parts {
                buffer.extend(part);
            }
        }

        Ok(buffer.len() - start_len)
    }
}

/// Implements deserialization of fixed vector.
impl<T, const N: usize> SimpleDeserialize for FixedVector<T, N>
where
    T: SimpleDeserialize + SszTypeInfo,
{
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if T::is_fixed_size() {
            let elem_size = T::fixed_size().ok_or(SSZError::InvalidByte)?;
            if data.len() != elem_size * N {
                return Err(SSZError::InvalidLength {
                    expected: elem_size * N,
                    got: data.len(),
                });
            }

            let mut array: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
            for (i, slot) in array.iter_mut().enumerate() {
                let start = i * elem_size;
                let end = start + elem_size;
                *slot = MaybeUninit::new(T::deserialize(&data[start..end])?);
            }

            let initialized = unsafe { core::mem::transmute_copy::<_, [T; N]>(&array) };
            Ok(FixedVector(initialized))
        } else {
            let offset_size = crate::BYTES_PER_LENGTH_OFFSET;
            let expected_offsets = N * offset_size;
            if data.len() < expected_offsets {
                return Err(SSZError::InvalidLength {
                    expected: expected_offsets,
                    got: data.len(),
                });
            }

            let mut offsets = [0usize; N];
            for (i, item) in offsets.iter_mut().enumerate() {
                let start = i * offset_size;
                let end = start + offset_size;
                *item = u32::from_le_bytes(data[start..end].try_into().unwrap()) as usize;
            }

            let mut array: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
            for i in 0..N {
                let start = offsets[i];
                let end = if i + 1 < N {
                    offsets[i + 1]
                } else {
                    data.len()
                };
                array[i] = MaybeUninit::new(T::deserialize(&data[start..end])?);
            }

            let initialized = unsafe { core::mem::transmute_copy::<_, [T; N]>(&array) };
            Ok(FixedVector(initialized))
        }
    }
}

/// Implements Merkleization of fixed vector.
impl<T, const N: usize> Merkleize for FixedVector<T, N>
where
    T: Merkleize + SimpleSerialize + SszTypeInfo,
{
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        if T::is_basic_type() {
            let mut serialized = vec![];
            self.serialize(&mut serialized)?;
            let packed = pack(&serialized);
            let count = chunk_count(SSZType::VectorBasic {
                elem_size: T::fixed_size().unwrap(),
                count: N,
            });
            merkleize(&packed, Some(count))
        } else {
            let roots: Result<Vec<[u8; 32]>, SSZError> = self
                .0
                .iter()
                .map(|elem| elem.hash_tree_root().map(|b| b.0))
                .collect();
            merkleize(
                &roots?,
                Some(chunk_count(SSZType::VectorComposite { count: N })),
            )
        }
    }
}

#[cfg(test)]
mod fixed_vector_tests {
    use super::*;
    use crate::ssz::{SimpleDeserialize, SimpleSerialize};
    use alloc::vec::Vec;
    use alloy_primitives::{B256, hex};

    #[test]
    fn test_fixed_vector_fixed_type_serialization() {
        let fv = FixedVector::<u16, 4>::new([1, 2, 3, 4]);
        let mut buffer = vec![];
        fv.serialize(&mut buffer).expect("serialize fixed vector");
        let deserialized =
            FixedVector::<u16, 4>::deserialize(&buffer).expect("deserialize fixed vector");
        assert_eq!(fv, deserialized);
    }

    #[test]
    fn test_fixed_vector_variable_type_serialization() {
        let fv = FixedVector::<Vec<u8>, 3>::new([vec![1, 2], vec![3, 4, 5], vec![6]]);
        let mut buffer = vec![];
        fv.serialize(&mut buffer)
            .expect("serialize fixed vector of vec<u8>");
        let deserialized =
            FixedVector::<Vec<u8>, 3>::deserialize(&buffer).expect("deserialize vec<u8>");
        assert_eq!(fv, deserialized);
    }

    #[test]
    fn test_fixed_vector_empty_inner_vectors() {
        let fv = FixedVector::<Vec<u8>, 2>::new([vec![], vec![]]);
        let mut buffer = vec![];
        fv.serialize(&mut buffer)
            .expect("serialize empty inner vecs");
        let deserialized =
            FixedVector::<Vec<u8>, 2>::deserialize(&buffer).expect("deserialize empty inner vecs");
        assert_eq!(fv, deserialized);
    }

    #[test]
    fn test_fixed_vector_hash_tree_root_basic() {
        let fv = FixedVector::<u8, 4>::new([1, 2, 3, 4]);
        let root = fv
            .hash_tree_root()
            .expect("hash tree root for basic fixed vector");
        let expected_root = B256::from(hex!(
            "0102030400000000000000000000000000000000000000000000000000000000"
        ));
        assert_eq!(root, expected_root);
    }

    #[test]
    fn test_fixed_vector_deserialization_failure_wrong_size() {
        let invalid_data = vec![1, 2];
        let result = FixedVector::<u16, 2>::deserialize(&invalid_data);
        assert!(result.is_err());
    }
}
