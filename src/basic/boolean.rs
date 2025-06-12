//! Serialization deserialization and merkleization for boolean values.

use alloy_primitives::B256;

use crate::{
    Merkleize,
    SSZError::{self, *},
    SszTypeInfo,
    ssz::SimpleSerialize,
};

impl SimpleSerialize for bool {
    /// Serializes a boolean value.
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        if *self { Ok(vec![1]) } else { Ok(vec![0]) }
    }

    /// Deserializes a boolean value.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.len() != 1 {
            return Err(InvalidLength {
                expected: 1,
                got: data.len(),
            });
        }
        match data[0] {
            1 => Ok(true),
            0 => Ok(false),
            _ => Err(InvalidBooleanByte),
        }
    }
}

impl SszTypeInfo for bool {
    /// Indicates that the boolean type is fixed-size.
    fn is_fixed_size() -> bool {
        true
    }

    /// Returns the fixed size of a boolean value in bytes.
    fn fixed_size() -> Option<usize> {
        Some(1)
    }

    ///Returns true since it is basic type.
    fn is_basic_type() -> bool {
        true
    }
}

impl Merkleize for bool {
    /// Calculates the hash tree root of a boolean value.
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let mut hash = B256::default();
        if *self {
            hash[0] = 1;
        }
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use alloy_primitives::hex::FromHex;

    #[test]
    fn test_bool_serialize() {
        assert_eq!(true.serialize(), Ok(vec![1]));
        assert_eq!(false.serialize(), Ok(vec![0]));
    }

    #[test]
    fn test_bool_deserialize() {
        assert_eq!(bool::deserialize(&[1]), Ok(true));
        assert_eq!(bool::deserialize(&[0]), Ok(false));
        assert_eq!(bool::deserialize(&[2]), Err(SSZError::InvalidBooleanByte));
        assert_eq!(
            bool::deserialize(&[]),
            Err(SSZError::InvalidLength {
                expected: 1,
                got: 0
            })
        );
        assert_eq!(
            bool::deserialize(&[1, 0]),
            Err(SSZError::InvalidLength {
                expected: 1,
                got: 2
            })
        );
    }

    #[test]
    fn test_bool_roundtrip() {
        let original_true = true;
        let serialized = original_true.serialize().expect("can serialize true");
        let recovered_true = bool::deserialize(&serialized).expect("can deserialize true");
        assert_eq!(original_true, recovered_true);

        let original_false = false;
        let serialized = original_false.serialize().expect("can serialize false");
        let recovered_false = bool::deserialize(&serialized).expect("can deserialize false");
        assert_eq!(original_false, recovered_false);
    }
    #[test]
    fn test_bool_hash_tree_root() {
        let root_true = true.hash_tree_root().expect("can merkleize true");
        assert_eq!(
            root_true,
            B256::from_hex("0100000000000000000000000000000000000000000000000000000000000000")
                .expect("valid hex")
        );

        let root_false = false.hash_tree_root().expect("can merkleize false");
        assert_eq!(
            root_false,
            B256::from_hex("0000000000000000000000000000000000000000000000000000000000000000")
                .expect("valid hex")
        );

        assert_ne!(root_true, root_false);
    }
}
