// ! Serialization and deserialization for BitVector

use crate::{SSZError, SimpleSerialize};

#[derive(Debug, PartialEq)]
pub struct BitVector<const N: usize> {
    bits: Vec<bool>,
}

impl<const N: usize> BitVector<N> {
    pub fn new() -> Self {
        Self {
            bits: vec![false; N],
        }
    }

    pub fn set(&mut self, index: usize, value: bool) -> Result<(), SSZError> {
        if index >= N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: index,
            });
        }
        self.bits[index] = value;
        Ok(())
    }
}

impl<const N: usize> SimpleSerialize for BitVector<N> {
    /// Serializes a  bit vector.
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        let byte_length = (N + 7) / 8;
        let mut bytes = vec![0u8; byte_length];

        for (i, &bit) in self.bits.iter().enumerate() {
            if bit {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        Ok(bytes)
    }

    /// Deserializes a bit vector.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        let expected_bytes = (N + 7) / 8;
        if data.len() != expected_bytes {
            return Err(SSZError::InvalidLength {
                expected: expected_bytes,
                got: data.len(),
            });
        }

        let mut bv = Self::new();
        for i in 0..N {
            let byte = data[i / 8];
            let bit = (byte >> (i % 8)) & 1 == 1;
            bv.bits[i] = bit;
        }

        Ok(bv)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitvector_serialize() {
        let mut bv = BitVector::<8>::new();
        bv.set(3, true).unwrap();
        bv.set(4, true).unwrap();
        assert_eq!(bv.serialize(), Ok(vec![24u8]));
    }

    #[test]
    fn test_bitvector_deserialize() {
        let data = vec![24u8, 1u8];
        let bv = BitVector::<12>::deserialize(&data).unwrap();
        assert_eq!(
            bv.bits,
            vec![
                false, false, false, true, true, false, false, false, true, false, false, false
            ]
        );
    }

    #[test]
    fn test_invalid_length() {
        assert!(BitVector::<8>::deserialize(&[0, 0]).is_err());
        let mut bv = BitVector::<8>::new();
        assert!(bv.set(8, true).is_err());
    }

    #[test]
    fn roundtrip_test() {
        let input = vec![24u8, 1u8];
        let bv = BitVector::<16>::deserialize(&input).unwrap();
        let expected = vec![
            false, false, false, true, true, false, false, false, true, false, false, false, false,
            false, false, false,
        ];
        assert_eq!(bv.bits, expected);
        let serialized = bv.serialize().unwrap();
        assert_eq!(serialized, input);
    }
}
