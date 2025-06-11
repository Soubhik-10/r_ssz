// ! Serializes and deserializes container

use crate::error::SSZError;
use crate::ssz::SimpleSerialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Foo {
    pub a: u32,
    pub b: u8,
}

impl SimpleSerialize for Foo {
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        let mut bytes = Vec::new();
        bytes.extend(self.a.serialize()?);
        bytes.extend(self.b.serialize()?);
        Ok(bytes)
    }

    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.len() < 5 {
            return Err(SSZError::ExpectedFurtherInput);
        }

        let a = u32::deserialize(&data[0..4])?;
        let b = u8::deserialize(&data[4..5])?;
        Ok(Foo { a, b })
    }
}

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

#[cfg(test)]
mod test {
    use alloy_primitives::B256;
    use alloy_primitives::hex;

    use crate::container::Foo;
    use crate::ssz::Merkleize;
    use crate::ssz::SimpleSerialize;

    #[test]
    pub fn test_container_roundtrip() {
        let original = super::Foo { a: 12, b: 6 };
        let serialized = original.serialize().expect("Serialization failed");
        let deserialized = super::Foo::deserialize(&serialized).expect("Deserialization failed");
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
}
