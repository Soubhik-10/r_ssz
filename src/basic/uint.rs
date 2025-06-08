//! Serialization and deserialzation for uint values.

use crate::constants::BYTES;
use crate::{SSZError, SimpleSerialize};
use alloy_primitives::U256;

macro_rules! impl_uint {
    ($type:ty, $bytes:expr) => {
        impl SimpleSerialize for $type {
            /// Implements serialization for unsigned integers.
            fn serialize(&self) -> Result<Vec<u8>, SSZError> {
                Ok(self.to_le_bytes().to_vec())
            }

            /// Implements the deserialization trait for unsigned integers.
            fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
                if data.len() != $bytes {
                    return Err(SSZError::InvalidLength {
                        expected: $bytes,
                        got: data.len(),
                    });
                }
                let mut bytes = [0u8; $bytes];
                bytes.copy_from_slice(data);
                Ok(Self::from_le_bytes(bytes))
            }
        }
    };
}

impl_uint!(u8, 1);
impl_uint!(u16, 2);
impl_uint!(u32, 4);
impl_uint!(u64, 8);
impl_uint!(u128, 16);

impl SimpleSerialize for U256 {
    /// Implements serialization for U256.
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        Ok(self.to_le_bytes::<{ BYTES }>().to_vec())
    }

    /// Implements the deserialization trait for U256.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.len() != 32 {
            return Err(SSZError::InvalidLength {
                expected: 32,
                got: data.len(),
            });
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(data);
        Ok(U256::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uint_serialize() {
        assert_eq!(42u8.serialize(), Ok(vec![42]));
        assert_eq!(300u16.serialize(), Ok(vec![44, 1]));
        assert_eq!(65536u32.serialize(), Ok(vec![0, 0, 1, 0]));
        assert_eq!(
            U256::from(65536).serialize(),
            Ok(vec![
                0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0
            ])
        );
    }

    #[test]
    fn test_uint_deserialize() {
        assert_eq!(u8::deserialize(&[42]), Ok(42));
        assert_eq!(u16::deserialize(&[44, 1]), Ok(300));
        assert_eq!(u32::deserialize(&[0, 0, 1, 0]), Ok(65536));
        assert_eq!(U256::deserialize(&[0xffu8; 32]), Ok(U256::MAX));

        // Test invalid lengths
        assert!(u8::deserialize(&[0, 0]).is_err());
        assert!(u16::deserialize(&[0]).is_err());
        assert!(u32::deserialize(&[0, 0, 0]).is_err());
    }

    #[test]
    fn round_trip_uint() {
        let values: Vec<u64> = vec![0, 1, 255, 256, 65535, 65536, 4294967295];
        for &value in &values {
            let serialized = value.serialize().unwrap();
            let deserialized = u64::deserialize(&serialized).unwrap();
            assert_eq!(value, deserialized);
        }
    }
}
