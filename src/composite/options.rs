//! Serializes,deserializes and merkleization of options.

use crate::{Merkleize, SSZError, SimpleSerialize, SszTypeInfo, merkleization::mix_in_selector};
use alloy_primitives::B256;

impl<T> SszTypeInfo for Option<T>
where
    T: SszTypeInfo,
{
    /// Indicates that the option is variable-size.
    fn is_fixed_size() -> bool {
        false
    }

    /// Returns None since the size of an option is not fixed.
    fn fixed_size() -> Option<usize> {
        None
    }

    fn is_basic_type() -> bool {
        T::is_basic_type()
    }
}

impl<T> SimpleSerialize for Option<T>
where
    T: SimpleSerialize,
{
    /// Serializes an option, encoding `None` as an empty byte vector and `Some` as the serialized value.
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        match self {
            Some(value) => {
                let mut bytes = vec![1]; // Tag for Some
                bytes.extend(value.serialize()?);
                Ok(bytes)
            }
            None => Ok(vec![0]), // Tag for None
        }
    }

    /// Deserializes an option, interpreting the first byte to determine if it is `Some` or `None`.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.is_empty() {
            return Err(SSZError::InvalidLength {
                expected: 1,
                got: 0,
            });
        }

        match data[0] {
            0 => Ok(None),
            1 => {
                let value = T::deserialize(&data[1..])?;
                Ok(Some(value))
            }
            _ => Err(SSZError::InvalidByte),
        }
    }
}

/// Implements Merkleization for option.
impl<T> Merkleize for Option<T>
where
    T: Merkleize,
{
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        match self {
            Some(value) => {
                let value_root = value.hash_tree_root()?;
                Ok(mix_in_selector(value_root, 1))
            }
            None => Ok(mix_in_selector(B256::ZERO, 0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SimpleSerialize;
    use crate::ssz::Merkleize;

    #[test]
    fn test_serialize_none() {
        let none_val: Option<u64> = None;
        assert_eq!(none_val.serialize().unwrap(), vec![0]);
    }

    #[test]
    fn test_serialize_some() {
        let some_val: Option<u64> = Some(0x1122334455667788);
        let mut expected = vec![1];
        expected.extend_from_slice(&0x1122334455667788u64.to_le_bytes());
        assert_eq!(some_val.serialize().unwrap(), expected);
    }

    #[test]
    fn test_deserialize_none() {
        let result: Option<u64> = Option::deserialize(&[0]).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_deserialize_some() {
        let mut input = vec![1];
        input.extend_from_slice(&42u64.to_le_bytes());
        let result = Option::<u64>::deserialize(&input).unwrap();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_roundtrip_none() {
        let none_val: Option<u64> = None;
        let serialized = none_val.serialize().unwrap();
        let deserialized = Option::<u64>::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, none_val);
    }

    #[test]
    fn test_roundtrip_some() {
        let some_val: Option<u64> = Some(987654321);
        let serialized = some_val.serialize().unwrap();
        let deserialized = Option::<u64>::deserialize(&serialized).unwrap();
        assert_eq!(deserialized, some_val);
    }

    #[test]
    fn check_hash_tree_root_calculation() {
        let a: Option<u8> = Some(4);
        let hashed_tree_root = a.hash_tree_root();
        let expected_root = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0x7063e9add4fb20ab4aee17f218b851e7c814f14ca5c8ec09208d34fe2865cd86"
        ));
        assert_eq!(hashed_tree_root.unwrap(), expected_root);
    }

    #[test]
    fn check_hash_tree_root_calculation_2() {
        let a: Option<bool> = Some(true);
        let hashed_tree_root = a.hash_tree_root();
        let expected_root = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0x56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
        ));
        assert_eq!(hashed_tree_root.unwrap(), expected_root);
    }

    #[test]
    fn check_composite_hash_tree_root() {
        let a: Option<[u8; 3]> = Some([2, 4, 6]);
        let hashed_tree_root = a.hash_tree_root();
        let recovered_tree = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0x53cc7f90b9577c52542200e88688f6646420a9db72bb97d6f8a73f15f9fcd131"
        ));
        assert_eq!(hashed_tree_root.unwrap(), recovered_tree);
    }
}
