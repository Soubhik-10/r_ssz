// ! Serializes,deserializes and merkleization of union

use crate::SimpleDeserialize;
use crate::{Merkleize, SSZError, SimpleSerialize, SszTypeInfo, merkleization::mix_in_selector};
use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::B256;

/// Basic container for serialization,deserialization and merkleization.
#[derive(Debug, PartialEq)]
pub enum MyUnion {
    None,
    U32(u32),
    ByteList(Vec<u8>),
}

impl SszTypeInfo for MyUnion {
    /// Returns false since `MyUnion` is not fixed size.
    fn is_fixed_size() -> bool {
        false
    }

    /// Returns `None` since `MyUnion` is not fixed size.
    fn fixed_size() -> Option<usize> {
        None
    }
}

impl SimpleSerialize for MyUnion {
    /// Serializes `MyUnion`.
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let start_len = buffer.len();

        match self {
            MyUnion::None => {
                buffer.push(0);
            }

            MyUnion::U32(val) => {
                buffer.push(1);
                val.serialize(buffer)?;
            }

            MyUnion::ByteList(vec) => {
                buffer.push(2);

                if vec.is_empty() {
                    // Handle empty list specially if needed
                    buffer.extend(vec![0; 4]); // Empty list offset
                } else {
                    // For non-empty variable-length data, we need offset + data
                    let offset_pos = buffer.len();
                    buffer.extend(vec![0; 4]); // Placeholder for offset
                    let data_start = buffer.len();
                    vec.serialize(buffer)?;

                    // Now fill in the offset (relative to start of union)
                    let offset = (data_start - start_len) as u32;
                    buffer[offset_pos..offset_pos + 4].copy_from_slice(&offset.to_le_bytes());
                }
            }
        }

        Ok(buffer.len() - start_len)
    }
}

impl SimpleDeserialize for MyUnion {
    /// Deserializes `MyUnion`.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.is_empty() {
            return Err(SSZError::ExpectedFurtherInput);
        }

        let selector = data[0];
        let payload = &data[1..];

        match selector {
            0 => {
                if !payload.is_empty() {
                    return Err(SSZError::InvalidByteLength {
                        got: payload.len(),
                        expected: 0,
                    });
                }
                Ok(MyUnion::None)
            }

            1 => {
                let val = u32::deserialize(payload)?;
                Ok(MyUnion::U32(val))
            }

            2 => {
                let vec = Vec::<u8>::deserialize(payload)?;
                Ok(MyUnion::ByteList(vec))
            }

            sel if sel > 127 => Err(SSZError::InvalidSelector {
                selector: sel.into(),
                reason: "Selector value above 127 is reserved for forward compatibility".into(),
            }),

            sel => Err(SSZError::InvalidSelector {
                selector: sel.into(),
                reason: "Unknown selector".into(),
            }),
        }
    }
}

/// Implements `Merkleization` for `MyUnion`.
impl Merkleize for MyUnion {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        match self {
            MyUnion::None => Ok(mix_in_selector(B256::ZERO, 0)),
            MyUnion::U32(val) => {
                let root = val.hash_tree_root()?;
                Ok(mix_in_selector(root, 1))
            }
            MyUnion::ByteList(vec) => {
                let root = vec.hash_tree_root()?;
                Ok(mix_in_selector(root, 2))
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum BadUnion {
    None,
    NothingAgain,
    Reserved(u8),
}

impl SszTypeInfo for BadUnion {
    fn is_fixed_size() -> bool {
        false
    }

    fn fixed_size() -> Option<usize> {
        None
    }
}

impl SimpleSerialize for BadUnion {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let start_len = buffer.len();

        match self {
            BadUnion::None => {
                buffer.push(0); // Type tag 0 for None
            }

            BadUnion::NothingAgain => {
                buffer.push(1); // Type tag 1 for NothingAgain
            }

            BadUnion::Reserved(byte) => {
                buffer.push(2); // Using standard sequential tags (not 200)

                // For fixed-size values, serialize directly
                buffer.push(*byte);
            }
        }

        Ok(buffer.len() - start_len)
    }
}

impl SimpleDeserialize for BadUnion {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.is_empty() {
            return Err(SSZError::ExpectedFurtherInput);
        }

        let selector = data[0];
        let payload = &data[1..];
        match selector {
            sel if sel > 127 => Err(SSZError::InvalidSelector {
                selector: sel.into(),
                reason: "Selector above 127 is reserved".into(),
            }),
            0 => {
                if !payload.is_empty() {
                    return Err(SSZError::InvalidByteLength {
                        got: payload.len(),
                        expected: 0,
                    });
                }
                Ok(BadUnion::None)
            }

            1 => {
                if !payload.is_empty() {
                    return Err(SSZError::InvalidByteLength {
                        got: payload.len(),
                        expected: 0,
                    });
                }
                Ok(BadUnion::NothingAgain)
            }

            200 => {
                if payload.len() != 1 {
                    return Err(SSZError::InvalidByteLength {
                        got: payload.len(),
                        expected: 1,
                    });
                }
                Ok(BadUnion::Reserved(payload[0]))
            }

            sel => Err(SSZError::InvalidSelector {
                selector: sel.into(),
                reason: "Unknown selector".into(),
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Foo {
    A(u32),
    B(u8),
}
impl SszTypeInfo for Foo {
    fn is_fixed_size() -> bool {
        false
    }

    fn fixed_size() -> Option<usize> {
        None
    }
}

impl SimpleSerialize for Foo {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let start_len = buffer.len();

        match self {
            Foo::A(val) => {
                buffer.push(0); // Variant discriminator
                val.serialize(buffer)?;
            }
            Foo::B(val) => {
                buffer.push(1); // Variant discriminator
                val.serialize(buffer)?;
            }
        }

        Ok(buffer.len() - start_len)
    }
}

impl SimpleDeserialize for Foo {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.is_empty() {
            return Err(SSZError::ExpectedFurtherInput);
        }

        let selector = data[0];
        let payload = &data[1..];

        match selector {
            0 => {
                let val = u32::deserialize(payload)?;
                Ok(Foo::A(val))
            }

            1 => {
                let val = u8::deserialize(payload)?;
                Ok(Foo::B(val))
            }

            sel if sel > 127 => Err(SSZError::InvalidSelector {
                selector: sel.into(),
                reason: "Selector value above 127 is reserved for forward compatibility".into(),
            }),

            sel => Err(SSZError::InvalidSelector {
                selector: sel.into(),
                reason: "Unknown selector".into(),
            }),
        }
    }
}

impl Merkleize for Foo {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        match self {
            Foo::A(val) => {
                let root = val.hash_tree_root()?;
                Ok(mix_in_selector(root, 0))
            }
            Foo::B(val) => {
                let root = val.hash_tree_root()?;
                Ok(mix_in_selector(root, 1))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_myunion_roundtrip_none() {
        let original = MyUnion::None;
        let mut buffer = vec![];
        original
            .serialize(&mut buffer)
            .expect("Serialization failed");
        let decoded = MyUnion::deserialize(&mut buffer).expect("Deserialization failed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_myunion_roundtrip_u32() {
        let original = MyUnion::U32(42);
        let mut buffer = vec![];
        original
            .serialize(&mut buffer)
            .expect("Serialization failed");
        let decoded = MyUnion::deserialize(&mut buffer).expect("Deserialization failed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_myunion_roundtrip_bytelist() {
        let original = MyUnion::ByteList(vec![1, 2, 3, 4, 5]);
        let mut buffer = vec![];
        original
            .serialize(&mut buffer)
            .expect("Serialization failed");
        let decoded = MyUnion::deserialize(&mut buffer).expect("Deserialization failed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_badunion_roundtrip_valid() {
        let original = BadUnion::NothingAgain;
        let mut buffer = vec![];
        original
            .serialize(&mut buffer)
            .expect("Serialization failed");
        let decoded = BadUnion::deserialize(&mut buffer).expect("Deserialization failed");
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_badunion_reserved_selector_violation() {
        let original = BadUnion::Reserved(42);
        let mut buffer = vec![];
        original
            .serialize(&mut buffer)
            .expect("Serialization failed");
        let decoded = BadUnion::deserialize(&mut buffer);
        match decoded {
            Err(SSZError::InvalidSelector { selector, reason }) => {
                assert_eq!(selector, 200);
                assert_eq!(reason, "Selector above 127 is reserved");
            }
            other => panic!("Expected InvalidSelector error, got {:?}", other),
        }
    }

    #[test]
    fn encode_union() {
        let value = Foo::A(12u32);
        let mut buffer = vec![];
        let _ = value.serialize(&mut buffer);
        let expected = [0u8, 12u8, 0u8, 0u8, 0u8];
        assert_eq!(buffer, expected);
        let value = Foo::B(6u8);
        let mut buffer = vec![];
        let _ = value.serialize(&mut buffer);
        let expected = [1u8, 6u8];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn check_union_hash_tree_root() {
        let value = Foo::A(12u32);
        let value_merkle = value.hash_tree_root();
        let expected_root = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0xf66cb566864e46e968c4c34b1a1ceb2b3cf1d7f4ba6b74a990553dfc06d89a17"
        ));
        assert_eq!(value_merkle.unwrap(), expected_root);
        let value = Foo::B(6u8);
        let value_merkle = value.hash_tree_root();
        let expected_root = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0x38c4d95f59831265092af466f948752e1d954ee38898636e08932ee33a19b74a"
        ));
        assert_eq!(value_merkle.unwrap(), expected_root);
    }

    #[test]
    fn check_union_hash_tree_root_2() {
        let original = MyUnion::None;
        let original_merkle = original.hash_tree_root();
        let expected_root = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
        ));
        assert_eq!(original_merkle.unwrap(), expected_root);
    }
}
