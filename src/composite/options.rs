//! Option aka Some and None
use crate::{Merkleize, SSZError, SimpleSerialize, SszTypeInfo};

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

impl<T> Merkleize for Option<T>
where
    T: Merkleize,
{
    fn hash_tree_root(&self) -> Result<alloy_primitives::B256, SSZError> {
        match self {
            Some(value) => value.hash_tree_root(),
            None => Ok(alloy_primitives::B256::ZERO),
        }
    }

    fn chunk_count() -> usize
    where
        Self: Sized,
    {
        T::chunk_count()
    }
}
#[cfg(test)]
mod tests {
    use crate::SimpleSerialize;

    #[test]
    fn test_serialize_none() {
        let none_val: Option<u64> = None;
        assert_eq!(none_val.serialize().unwrap(), vec![0]);
    }

    #[test]
    fn test_serialize_some() {
        let some_val: Option<u64> = Some(0x1122334455667788);
        let mut expected = vec![1]; // prefix for Some
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
}
