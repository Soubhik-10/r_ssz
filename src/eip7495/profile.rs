//! Contains serialization,deserialization and merkleization for `Profile[MyStableContainer]`

use crate::{Merkleize, SSZError, SimpleDeserialize, SimpleSerialize, merkleization::merkleize};
use alloc::vec::Vec;
use alloy_primitives::B256;

#[derive(Debug, Clone, PartialEq)]
pub struct MyProfile {
    pub a: u32,
    pub b: bool,
}

/// Serializes `MyProfile` as per Eip-7495 specs
impl SimpleSerialize for MyProfile {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let mut local_buffer = Vec::new();
        self.a.serialize(&mut local_buffer)?;
        self.b.serialize(&mut local_buffer)?;
        buffer.extend_from_slice(&local_buffer);
        Ok(buffer.len())
    }
}

/// Deserializes `MyProfile` as per Eip-7495 specs
impl SimpleDeserialize for MyProfile {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.len() < 5 {
            return Err(SSZError::InvalidByteLength {
                got: data.len(),
                expected: 5,
            });
        }

        let a = u32::deserialize(&data[0..4])?;
        let b = bool::deserialize(&data[4..5])?;
        Ok(Self { a, b })
    }
}

/// Merkleizes `MyProfile` as per Eip-7495 specs
impl Merkleize for MyProfile {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let a_hash = self.a.hash_tree_root()?;
        let b_hash = self.b.hash_tree_root()?;
        merkleize(&[a_hash.into(), b_hash.into()], None)
    }

    fn chunk_count() -> usize {
        2
    }
}

#[cfg(test)]
mod tests {
    use crate::{MyProfile, SimpleDeserialize, SimpleSerialize};
    use alloc::vec::Vec;

    #[test]
    fn test_serialize_deserialize_all_fields() {
        let container = MyProfile { a: 42, b: true };

        let mut buffer = Vec::new();
        let bytes_written = container.serialize(&mut buffer).unwrap();
        assert_eq!(bytes_written, 5);

        let deserialized = MyProfile::deserialize(&buffer).unwrap();
        assert_eq!(container, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_partial_fields() {
        let container = MyProfile { a: 123, b: false };

        let mut buffer = Vec::new();
        let bytes_written = container.serialize(&mut buffer).unwrap();
        assert_eq!(bytes_written, 5);

        let deserialized = MyProfile::deserialize(&buffer).unwrap();
        assert_eq!(container, deserialized);
    }
}
