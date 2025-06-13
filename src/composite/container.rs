//! Serializes , deserializes and merkleization of container.

use crate::SimpleDeserialize;
use crate::error::SSZError;
use crate::ssz::SimpleSerialize;
use alloc::vec::Vec;

/// Basic container for testing.
#[derive(Debug, Clone, PartialEq)]
pub struct Foo {
    pub a: u32,
    pub b: u8,
}

/// Serialization of `Foo`.
impl SimpleSerialize for Foo {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let mut written = 0;
        written += self.a.serialize(buffer)?;
        written += self.b.serialize(buffer)?;
        Ok(written)
    }
}

/// Deserialization of `Foo`.
impl SimpleDeserialize for Foo {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.len() < 5 {
            return Err(SSZError::ExpectedFurtherInput);
        }

        let a = u32::deserialize(&data[0..4])?;
        let b = u8::deserialize(&data[4..5])?;
        Ok(Foo { a, b })
    }
}

/// `SszTypeInfo` for `Foo`.
impl crate::ssz::SszTypeInfo for Foo {
    fn is_fixed_size() -> bool {
        true
    }

    fn fixed_size() -> Option<usize> {
        Some(5)
    }

    fn is_basic_type() -> bool {
        false
    }
}

/// Merkleization of `Foo`.
impl crate::ssz::Merkleize for Foo {
    fn hash_tree_root(&self) -> Result<alloy_primitives::B256, SSZError> {
        let a_root = self.a.hash_tree_root()?;
        let b_root = self.b.hash_tree_root()?;
        crate::merkleization::merkleize(&[*a_root, *b_root], None)
    }
    fn chunk_count() -> usize {
        1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestComposite {
    pub name: bool,
    pub value: u32,
}

impl SimpleSerialize for TestComposite {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let mut written = 0;
        written += self.name.serialize(buffer)?;
        written += self.value.serialize(buffer)?;
        Ok(written)
    }
}

impl SimpleDeserialize for TestComposite {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        let (name, rest) = {
            let name = bool::deserialize(&data[0..1])?;
            (name, &data[1..])
        };
        let value = u32::deserialize(rest)?;
        Ok(TestComposite { name, value })
    }
}

impl crate::ssz::SszTypeInfo for TestComposite {
    fn is_fixed_size() -> bool {
        true
    }

    fn fixed_size() -> Option<usize> {
        Some(5)
    }

    fn is_basic_type() -> bool {
        false
    }
}
impl crate::ssz::Merkleize for TestComposite {
    fn hash_tree_root(&self) -> Result<alloy_primitives::B256, SSZError> {
        let name_root = self.name.hash_tree_root()?;
        let value_root = self.value.hash_tree_root()?;
        crate::merkleization::merkleize(&[*name_root, *value_root], None)
    }
    fn chunk_count() -> usize {
        1
    }
}

#[cfg(test)]
mod test {
    use crate::SimpleDeserialize;
    use crate::container::Foo;
    use crate::container::TestComposite;
    use crate::ssz::Merkleize;
    use crate::ssz::SimpleSerialize;
    use alloc::vec;
    use alloy_primitives::B256;
    use alloy_primitives::hex;

    #[test]
    pub fn test_container_roundtrip() {
        let mut buffer = vec![];
        let original = super::Foo { a: 12, b: 6 };
        original
            .serialize(&mut buffer)
            .expect("Serialization failed");
        let deserialized = super::Foo::deserialize(&buffer).expect("Deserialization failed");
        assert_eq!(original.a, deserialized.a);
        assert_eq!(original.b, deserialized.b);
    }

    #[test]
    pub fn test_container_merkleize() {
        let original = super::Foo { a: 12, b: 6 };
        let root = Foo::hash_tree_root(&original).expect("Hash tree root failed");
        let expected_root = B256::from(hex!(
            "0xe922cefc3d48d862e694c6c4615f407767d46ca09b4d476302f852fe9b5e8ce1"
        ));
        assert_eq!(root, expected_root);
    }

    #[test]
    pub fn test_composite_serialization() {
        use crate::ssz::Merkleize;
        use crate::ssz::SimpleSerialize;

        let original = super::TestComposite {
            value: 4,
            name: true,
        };
        let root = TestComposite::hash_tree_root(&original).expect("Hash tree root failed");
        let expected_root = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0xf9c5ada16029ed1580188989686f19e749c006b2eac37d3ef087b824b31ba997"
        ));
        assert_eq!(root, expected_root);
        let mut buffer = vec![];
        original
            .serialize(&mut buffer)
            .expect("Serialization failed");
        let deserialized =
            super::TestComposite::deserialize(&buffer).expect("Deserialization failed");
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.value, deserialized.value);
        let a: u16 = 56;
        let opa: Option<u16> = Some(a);
        let root = u16::hash_tree_root(&a).unwrap();
        let op_root = Option::<u16>::hash_tree_root(&opa).unwrap();
        let expected_a_root = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0x3800000000000000000000000000000000000000000000000000000000000000"
        ));
        let expected_opa_root = alloy_primitives::B256::from(alloy_primitives::hex!(
            "0x7e8c63098a2af54deed7308a992a823805d46b97d91a5b750fb38c98da142e59"
        ));
        assert_eq!(root, expected_a_root);
        assert_eq!(op_root, expected_opa_root);
    }
}
