//! Implements the serialization and merkleization of eip-7495 using spec tests
///
/// See: <https://eips.ethereum.org/EIPS/eip-7916>
///
use alloc::vec::Vec;
use alloy_primitives::B256;

use crate::{
    Merkleize, SSZError, SimpleDeserialize, SimpleSerialize, SszTypeInfo,
    merkleization::{merkleize_progressive_list, mix_in_length, pack},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgressiveList<T> {
    pub elements: Vec<T>,
}

impl<T> ProgressiveList<T> {
    pub fn new(elements: Vec<T>) -> Self {
        Self { elements }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

/// Serializes a progressive list
impl<T> SimpleSerialize for ProgressiveList<T>
where
    T: SimpleSerialize + SszTypeInfo,
{
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        self.elements.serialize(buffer)
    }
}

/// Desrializes a progressive list
impl<T> SimpleDeserialize for ProgressiveList<T>
where
    T: SimpleDeserialize + SszTypeInfo,
{
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        let elements = Vec::<T>::deserialize(data)?;
        Ok(ProgressiveList::new(elements))
    }
}

/// Merkleizes a progressive list
impl<T> Merkleize for ProgressiveList<T>
where
    T: Merkleize + SszTypeInfo + SimpleSerialize,
{
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let chunks: Vec<[u8; 32]> = if T::is_basic_type() {
            // Pack serialized bytes into 32-byte chunks
            let mut bytes = Vec::new();
            for item in &self.elements {
                item.serialize(&mut bytes)?;
            }
            pack(&bytes)
        } else {
            // Composite: hash_tree_root each element into a chunk
            self.elements
                .iter()
                .map(|e| e.hash_tree_root().map(|h| h.0))
                .collect::<Result<Vec<_>, _>>()?
        };

        let root = merkleize_progressive_list(&chunks, 1, 4)?;
        Ok(mix_in_length(root, self.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_fixed_size_u8() {
        let list = ProgressiveList::new(vec![1u8, 2, 3, 4]);

        // Test serialization
        let mut buffer = Vec::new();
        list.serialize(&mut buffer).unwrap();
        assert_eq!(buffer, vec![1, 2, 3, 4]);

        // Test deserialization
        let deserialized = ProgressiveList::<u8>::deserialize(&buffer).unwrap();
        assert_eq!(list, deserialized);
    }

    #[test]
    fn test_fixed_size_u32() {
        let list = ProgressiveList::new(vec![0x11223344u32, 0x55667788]);

        let mut buffer = Vec::new();
        list.serialize(&mut buffer).unwrap();
        assert_eq!(buffer, vec![0x44, 0x33, 0x22, 0x11, 0x88, 0x77, 0x66, 0x55]);

        let deserialized = ProgressiveList::<u32>::deserialize(&buffer).unwrap();
        assert_eq!(list, deserialized);
    }

    #[test]
    fn test_variable_size_vec() {
        let list = ProgressiveList::new(vec![
            vec![1u8, 2, 3],  // [1,2,3]
            vec![4, 5],       // [4,5]
            vec![6, 7, 8, 9], // [6,7,8,9]
        ]);

        let mut buffer = Vec::new();
        list.serialize(&mut buffer).unwrap();

        // Offsets = 12 + len(elem1) + len(elem2)
        assert_eq!(buffer.len(), 12 + 3 + 2 + 4); // 21

        let offset1 = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
        let offset2 = u32::from_le_bytes(buffer[4..8].try_into().unwrap());
        let offset3 = u32::from_le_bytes(buffer[8..12].try_into().unwrap());

        assert_eq!(offset1, 12);
        assert_eq!(offset2, 15); // 12 + 3
        assert_eq!(offset3, 17); // 15 + 2

        assert_eq!(&buffer[12..15], &[1, 2, 3]);
        assert_eq!(&buffer[15..17], &[4, 5]);
        assert_eq!(&buffer[17..21], &[6, 7, 8, 9]);

        let deserialized = ProgressiveList::<Vec<u8>>::deserialize(&buffer).unwrap();
        assert_eq!(list, deserialized);
    }

    #[test]
    fn test_empty_list() {
        let list: ProgressiveList<u8> = ProgressiveList::new(vec![]);

        let mut buffer = Vec::new();
        list.serialize(&mut buffer).unwrap();
        assert!(buffer.is_empty());

        let deserialized = ProgressiveList::<u8>::deserialize(&buffer).unwrap();
        assert_eq!(list, deserialized);
    }

    #[test]
    fn test_length_mix_in() {
        // Test that length is properly mixed into hash_tree_root
        let list1 = ProgressiveList::new(vec![1u8, 2, 3]);
        let list2 = ProgressiveList::new(vec![1u8, 2, 3, 4]);

        let root1 = list1.hash_tree_root().unwrap();
        let root2 = list2.hash_tree_root().unwrap();

        assert_ne!(root1, root2); // different lengths should produce different roots
    }
}
