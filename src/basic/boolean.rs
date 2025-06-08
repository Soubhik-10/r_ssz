//! Serialization and deserialization for boolean values.

use crate::{
    SSZError::{self, *},
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_serialize() {
        assert_eq!(true.serialize(), Ok(vec![1]));
        assert_eq!(false.serialize(), Ok(vec![0]));
    }

    #[test]
    fn test_bool_deserialize() {
        assert_eq!(bool::deserialize(&[1]), Ok(true));
        assert_eq!(bool::deserialize(&[0]), Ok(false));
        // Test panic on invalid byte
        // Deserialize invalid byte
        assert_eq!(bool::deserialize(&[2]), Err(SSZError::InvalidBooleanByte));

        // Deserialize empty slice
        assert_eq!(
            bool::deserialize(&[]),
            Err(SSZError::InvalidLength {
                expected: 1,
                got: 0
            })
        );

        // Deserialize too many bytes
        assert_eq!(
            bool::deserialize(&[1, 0]),
            Err(SSZError::InvalidLength {
                expected: 1,
                got: 2
            })
        );
    }
}
