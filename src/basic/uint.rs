//! Serialization and deserialzation for uint values.

use crate::Merkleize;
use crate::SszTypeInfo;
use crate::constants::BYTES;
use crate::{SSZError, SimpleSerialize};
use alloy_primitives::B256;
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
        if data.len() != BYTES {
            return Err(SSZError::InvalidLength {
                expected: BYTES,
                got: data.len(),
            });
        }
        let mut bytes = [0u8; BYTES];
        bytes.copy_from_slice(data);
        Ok(U256::from_le_bytes(bytes))
    }
}

macro_rules! impl_uint_typeinfo {
    ($type:ty, $bytes:expr) => {
        impl SszTypeInfo for $type {
            /// Returns true if the type is fixed-size.
            fn is_fixed_size() -> bool {
                true
            }

            /// Returns the fixed size in bytes.
            fn fixed_size() -> Option<usize> {
                Some($bytes)
            }
        }
    };
}
impl_uint_typeinfo!(u8, 1);
impl_uint_typeinfo!(u16, 2);
impl_uint_typeinfo!(u32, 4);
impl_uint_typeinfo!(u64, 8);
impl_uint_typeinfo!(u128, 16);

impl SszTypeInfo for U256 {
    /// Returns true if the type is fixed-size.
    fn is_fixed_size() -> bool {
        true
    }

    /// Returns the fixed size in bytes.
    fn fixed_size() -> Option<usize> {
        Some(BYTES)
    }
}
macro_rules! impl_uint_merkleize {
    ($type:ty, $bytes:expr) => {
        impl Merkleize for $type {
            /// returns `hash_tree_root` for uint
            fn hash_tree_root(&self) -> Result<B256, SSZError> {
                let bytes = self.to_le_bytes();
                let mut buf = [0u8; 32];
                buf[..$bytes].copy_from_slice(&bytes);
                Ok(B256::from(buf))
            }
        }
    };
}

impl_uint_merkleize!(u8, 1);
impl_uint_merkleize!(u16, 2);
impl_uint_merkleize!(u32, 4);
impl_uint_merkleize!(u64, 8);
impl_uint_merkleize!(u128, 16);

impl Merkleize for U256 {
    /// returns `hash_tree_root` for u256
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        let bytes: [u8; BYTES] = self.to_le_bytes();
        let hash = B256::from_slice(&bytes);
        Ok(hash)
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex;

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
    #[test]
    fn test_uint_hash_tree_root() {
        // Test u8
        let value_u8: u8 = 0xFF;
        let root_u8 = value_u8.hash_tree_root().unwrap();
        assert_eq!(
            root_u8,
            B256::from(hex!(
                "ff00000000000000000000000000000000000000000000000000000000000000"
            ))
        );

        // Test u16
        let value_u16: u16 = 0xFFFF;
        let root_u16 = value_u16.hash_tree_root().unwrap();
        assert_eq!(
            root_u16,
            B256::from(hex!(
                "ffff000000000000000000000000000000000000000000000000000000000000"
            ))
        );

        // Test u32
        let value_u32: u32 = 0xFFFFFFFF;
        let root_u32 = value_u32.hash_tree_root().unwrap();
        assert_eq!(
            root_u32,
            B256::from(hex!(
                "ffffffff00000000000000000000000000000000000000000000000000000000"
            ))
        );

        // Test U256
        let value_u256 = U256::MAX;
        let root_u256 = value_u256.hash_tree_root().unwrap();
        assert_eq!(
            root_u256,
            B256::from(hex!(
                "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
            ))
        );
    }

    #[test]
    fn test_uint_hash_tree_root_zero() {
        // Test zero values are properly padded
        let zero_u64 = 0u64;
        let root = zero_u64.hash_tree_root().unwrap();
        assert_eq!(
            root,
            B256::from(hex!(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ))
        );
    }
}
