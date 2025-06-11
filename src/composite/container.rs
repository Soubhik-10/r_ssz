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

#[derive(Debug, Clone, PartialEq)]
pub struct TestComposite {
    pub name: bool,
    pub value: u32,
}

impl SimpleSerialize for TestComposite {
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        let mut bytes = Vec::new();
        bytes.extend(self.name.serialize()?);
        bytes.extend(self.value.serialize()?);
        Ok(bytes)
    }

    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        // Deserialize name (bool)
        let (name, rest) = {
            let name = bool::deserialize(&data[0..1])?;
            (name, &data[1..])
        };
        // Deserialize value (u32)
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
    use alloy_primitives::B256;
    use alloy_primitives::hex;

    use crate::container::Foo;
    use crate::container::TestComposite;
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

    #[test]
    pub fn test_composite_serialization() {
        use crate::ssz::Merkleize;
        use crate::ssz::SimpleSerialize;

        let original = super::TestComposite {
            value: 4,
            name: true,
        };
        let root = TestComposite::hash_tree_root(&original).expect("Hash tree root failed");
        println!("Hash tree root: {:?}", root);
        // Serialize
        let serialized = original.serialize().expect("Serialization failed");
        println!("Serialized: {:?}", serialized);

        // Deserialize
        let deserialized =
            super::TestComposite::deserialize(&serialized).expect("Deserialization failed");
        println!("Deserialized: {:?}", deserialized);
        // Check equality
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.value, deserialized.value);
        let a: u16 = 56;
        let opa: Option<u16> = Some(a);
        let opasr = opa.serialize().unwrap();
        let asr = a.serialize().unwrap();
        println!("a: {asr:?}");
        println!("opa: {opasr:?}");
        let root = u16::hash_tree_root(&a).unwrap();
        let op_root = Option::<u16>::hash_tree_root(&opa).unwrap();
        println!("root:{}, op_root{}", root, op_root);
    }
}
