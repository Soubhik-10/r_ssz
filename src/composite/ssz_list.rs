use crate::{
    Merkleize, SSZError, SimpleDeserialize, SimpleSerialize, SszTypeInfo,
    merkleization::{merkleize, mix_in_length, pack},
};
use alloc::vec::Vec;
use alloy_primitives::B256;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct List<T, const N: usize> {
    elements: Vec<T>,
    _phantom: PhantomData<[T; N]>, // enforce max length at compile time
}

impl<T, const N: usize> List<T, N> {
    pub fn new(elements: Vec<T>) -> Result<Self, SSZError> {
        if elements.len() > N {
            Err(SSZError::InvalidLength {
                expected: N,
                got: elements.len(),
            })
        } else {
            Ok(Self {
                elements,
                _phantom: PhantomData,
            })
        }
    }

    pub fn into_inner(self) -> Vec<T> {
        self.elements
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

// Optional: allow treating List<T, N> like a Vec<T>
impl<T, const N: usize> Deref for List<T, N> {
    type Target = [T];
    fn deref(&self) -> &Self::Target {
        &self.elements
    }
}
impl<T, const N: usize> DerefMut for List<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.elements
    }
}

impl<T, const N: usize> SszTypeInfo for List<T, N>
where
    T: SszTypeInfo,
{
    fn is_fixed_size() -> bool {
        false
    }

    fn fixed_size() -> Option<usize> {
        None
    }
}

impl<T, const N: usize> SimpleSerialize for List<T, N>
where
    T: SimpleSerialize + SszTypeInfo,
{
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        if self.len() > N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: self.len(),
            });
        }
        self.elements.serialize(buffer)
    }
}

impl<T, const N: usize> SimpleDeserialize for List<T, N>
where
    T: SimpleDeserialize + SszTypeInfo,
{
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        let vec = Vec::<T>::deserialize(data)?;
        if vec.len() > N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: vec.len(),
            });
        }
        Ok(List {
            elements: vec,
            _phantom: PhantomData,
        })
    }
}

impl<T, const N: usize> Merkleize for List<T, N>
where
    T: Merkleize + SimpleSerialize + SszTypeInfo,
{
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let chunks = if T::is_basic_type() {
            let mut serialized = Vec::new();
            self.serialize(&mut serialized)?;
            pack(&serialized)
        } else {
            self.elements
                .iter()
                .map(|e| e.hash_tree_root().map(|h| h.0))
                .collect::<Result<Vec<_>, _>>()?
        };

        let root = merkleize(&chunks, None)?; // list: no forced chunk count
        Ok(mix_in_length(root, self.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::List;
    use crate::{Merkleize, SimpleDeserialize, SimpleSerialize};
    
    use alloy_primitives::{
        B256,
        hex::{self, FromHex},
    };

    #[test]
    fn test_serialize_deserialize_list_u64() {
        let list = List::<u64, 3>::new(vec![10, 20, 30]).unwrap();
        let mut buffer = vec![];
        list.serialize(&mut buffer).unwrap();
        let deserialized = List::<u64, 3>::deserialize(&buffer).unwrap();
        assert_eq!(list, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_list_option_u64() {
        let list = List::<Option<u64>, 3>::new(vec![Some(42), None, Some(99)]).unwrap();
        let mut buffer = vec![];
        list.serialize(&mut buffer).unwrap();
        let deserialized = List::<Option<u64>, 3>::deserialize(&buffer).unwrap();
        assert_eq!(list, deserialized);
    }

    #[test]
    fn test_deserialize_invalid_length_list() {
        // 10 bytes is not a multiple of u64 (8 bytes), and overflows length 1
        let bad_data = vec![0u8; 10];
        let result = List::<u64, 1>::deserialize(&bad_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_exceeding_capacity_should_fail() {
        let too_long = vec![1u8; 5];
        let result = List::<u8, 4>::new(too_long);
        assert!(result.is_err());
    }

    #[test]
    fn test_list_within_capacity() {
        let valid = vec![1u8; 4];
        let result = List::<u8, 4>::new(valid.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 4);
    }

    #[test]
    fn test_ssz_merkle_list_root() {
        let list = List::<u16, 10>::new(vec![1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        let root = list.hash_tree_root().expect("can compute root");

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

    #[test]
    fn test_list_merkle_root_differs_by_length() {
        let l1 = List::<u8, 10>::new(vec![1, 2, 3]).unwrap();
        let l2 = List::<u8, 10>::new(vec![1, 2, 3, 4]).unwrap();
        assert_ne!(l1.hash_tree_root().unwrap(), l2.hash_tree_root().unwrap());
    }
}
