//! Tests the serialization and merkleization of Profile using eip-7495 using spec tests
///
/// See: <https://eips.ethereum.org/EIPS/eip-7495>
///
use crate::{Merkleize, SSZError, SimpleDeserialize, SimpleSerialize, merkleization::merkleize};
use alloc::vec::Vec;
use alloy_primitives::B256;

///NOTE: Here Square -> Profile[MyProfile]
#[derive(Debug, Clone, PartialEq)]
pub struct Square {
    pub side: u16,
    pub color: u8,
}

/// Serializes Square as per Eip-7495 specs
impl SimpleSerialize for Square {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let mut local_buffer = Vec::new();
        self.side.serialize(&mut local_buffer)?;
        self.color.serialize(&mut local_buffer)?;
        buffer.extend_from_slice(&local_buffer);
        Ok(buffer.len())
    }
}

/// Deserializes Square as per Eip-7495 specs
impl SimpleDeserialize for Square {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.len() < 3 {
            return Err(SSZError::InvalidByteLength {
                got: data.len(),
                expected: 3,
            });
        }

        let side = u16::deserialize(&data[0..2])?;
        let color = u8::deserialize(&data[2..3])?;
        Ok(Self { side, color })
    }
}

/// Merkleizes Square as per Eip-7495 specs
impl Merkleize for Square {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let side_hash = self.side.hash_tree_root()?;
        let color_hash = self.color.hash_tree_root()?;
        merkleize(&[side_hash.into(), color_hash.into()], None)
    }

    fn chunk_count() -> usize {
        2
    }
}

///NOTE: Here Circle -> Profile[MyProfile]
#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    pub color: u8,
    pub radius: u16,
}

/// Serializes Circle as per Eip-7495 specs
impl SimpleSerialize for Circle {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let mut local_buffer = Vec::new();
        self.color.serialize(&mut local_buffer)?;
        self.radius.serialize(&mut local_buffer)?;
        buffer.extend_from_slice(&local_buffer);
        Ok(buffer.len())
    }
}

/// Deserializes Circle as per Eip-7495 specs
impl SimpleDeserialize for Circle {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.len() < 3 {
            return Err(SSZError::InvalidByteLength {
                got: data.len(),
                expected: 3,
            });
        }

        let color = u8::deserialize(&data[0..1])?;
        let radius = u16::deserialize(&data[1..3])?;
        Ok(Self { color, radius })
    }
}

/// Merkleizes Circle as per Eip-7495 specs
impl Merkleize for Circle {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let color_hash = self.color.hash_tree_root()?;
        let radius_hash = self.radius.hash_tree_root()?;
        merkleize(&[color_hash.into(), radius_hash.into()], None)
    }

    fn chunk_count() -> usize {
        2
    }
}

#[cfg(test)]
mod tests {
    use crate::{Circle, SimpleDeserialize, SimpleSerialize, Square};
    use alloc::vec::Vec;
    use alloy_primitives::hex;

    #[test]
    fn test_serialize_deserialize_square() {
        let container = Square {
            side: 0x42,
            color: 1,
        };

        let mut buffer = Vec::new();
        let bytes_written = container.serialize(&mut buffer).unwrap();
        let result = hex::encode(&buffer);
        let expected_result = "420001";
        assert_eq!(result, expected_result);
        assert_eq!(bytes_written, 3);

        let deserialized = Square::deserialize(&buffer).unwrap();
        assert_eq!(container, deserialized);
    }

    #[test]
    fn test_serialize_deserialize_circle() {
        let container = Circle {
            radius: 0x42,
            color: 1,
        };

        let mut buffer = Vec::new();
        let bytes_written = container.serialize(&mut buffer).unwrap();
        let result = hex::encode(&buffer);
        let expected_result = "014200";
        assert_eq!(result, expected_result);
        assert_eq!(bytes_written, 3);

        let deserialized = Circle::deserialize(&buffer).unwrap();
        assert_eq!(container, deserialized);
    }
}
