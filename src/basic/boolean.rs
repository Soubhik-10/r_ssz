//! Serialization and deserialization for boolean values.

use crate::ssz::SimpleSerialize;

impl SimpleSerialize for bool {
    /// Serializes a boolean value.
    fn serialize(&self) -> Vec<u8> {
        if *self { vec![1] } else { vec![0] }
    }

    /// Deserializes a boolean value.
    fn deserialize(data: &[u8]) -> Self {
        if data.len() != 1 {
            panic!("Cannot deserialize boolean from empty data");
        }
        match data[0] {
            1 => true,
            0 => false,
            _ => panic!("Invalid byte for boolean deserialization"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool_serialize() {
        assert_eq!(true.serialize(), vec![1]);
        assert_eq!(false.serialize(), vec![0]);
    }

    #[test]
    fn test_bool_deserialize() {
        assert_eq!(bool::deserialize(&[1]), true);
        assert_eq!(bool::deserialize(&[0]), false);
        // Test panic on invalid byte
        let result = std::panic::catch_unwind(|| bool::deserialize(&[2]));
        assert!(result.is_err());
        // Test panic on empty data
        let result = std::panic::catch_unwind(|| bool::deserialize(&[]));
        assert!(result.is_err());
        // Test panic on data with more than one byte
        let result = std::panic::catch_unwind(|| bool::deserialize(&[1, 0]));
        assert!(result.is_err());
    }
}
