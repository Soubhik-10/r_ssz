// ! Serialization and deserialization for bitlist

use crate::{SSZError, SimpleSerialize, SszTypeInfo};

#[derive(Debug, PartialEq)]
pub struct BitList<const N: usize> {
    bits: Vec<bool>,
}

impl<const N: usize> Default for BitList<N> {
    fn default() -> Self {
        Self { bits: vec![] }
    }
}

impl<const N: usize> TryFrom<&[bool]> for BitList<N> {
    type Error = SSZError;

    fn try_from(slice: &[bool]) -> Result<Self, Self::Error> {
        BitList::from_vec(slice.to_vec())
    }
}

impl<const N: usize> BitList<N> {
    pub fn new() -> Self {
        Self { bits: vec![] }
    }

    pub fn from_vec(bits: Vec<bool>) -> Result<Self, SSZError> {
        if bits.len() > N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: bits.len(),
            });
        }
        Ok(Self { bits })
    }

    pub fn push(&mut self, bit: bool) -> Result<(), SSZError> {
        if self.bits.len() >= N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: self.bits.len() + 1,
            });
        }
        self.bits.push(bit);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }
}

impl<const N: usize> SszTypeInfo for BitList<N> {
    fn is_fixed_size() -> bool {
        false
    }

    fn fixed_size() -> Option<usize> {
        None
    }
}

impl<const N: usize> SimpleSerialize for BitList<N> {
    /// Serializes a bit list.
    fn serialize(&self) -> Result<Vec<u8>, SSZError> {
        let bit_len = self.bits.len();
        if bit_len > N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: bit_len,
            });
        }

        let byte_len = bit_len.div_ceil(8) + 1;
        let mut bytes = vec![0u8; byte_len];

        for (i, &bit) in self.bits.iter().enumerate() {
            if bit {
                bytes[i / 8] |= 1 << (i % 8);
            }
        }

        let dbyte = bit_len / 8;
        let dbit = bit_len % 8;
        bytes[dbyte] |= 1 << dbit;

        Ok(bytes)
    }

    /// Deserializes a bit list.    
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if data.is_empty() {
            return Err(SSZError::InvalidLength {
                expected: 1,
                got: 0,
            });
        }

        let last = data[data.len() - 1];
        if last == 0 {
            return Err(SSZError::OffsetOutOfBounds);
        }

        let mut bits = Vec::new();
        let total_bits = data.len() * 8;
        let mut _found_delimiter = false;
        let mut logical_bits = 0;

        #[allow(unused_labels)]
        'outer: for (i, byte) in data.iter().enumerate() {
            for j in 0..8 {
                let global_bit_index = i * 8 + j;
                if global_bit_index >= total_bits {
                    break;
                }
                if (byte >> j) & 1 != 0 {
                    logical_bits = global_bit_index;
                }
            }
        }

        for i in 0..logical_bits {
            let byte = data[i / 8];
            let bit = (byte >> (i % 8)) & 1;
            bits.push(bit == 1);
        }

        if bits.len() > N {
            return Err(SSZError::InvalidLength {
                expected: N,
                got: bits.len(),
            });
        }

        Ok(Self { bits })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bitlist_serialize() {
        let value: BitList<10> = BitList::default();
        let encoding = (value).serialize().expect("can encode");
        let expected = [1u8];
        assert_eq!(encoding, expected);

        let mut value: BitList<32> = BitList::default();
        let _ = value.push(false);
        let _ = value.push(true);
        let encoding = (value).serialize().expect("can encode");
        let expected = [6u8, 0u8];
        assert_eq!(encoding, expected);

        let mut value: BitList<32> = BitList::default();
        let _ = value.push(false);
        let _ = value.push(false);
        let _ = value.push(false);
        let _ = value.push(true);
        let _ = value.push(true);
        let _ = value.push(false);
        let _ = value.push(false);
        let _ = value.push(false);

        let encoding = (value).serialize().expect("can encode");
        let expected = [24u8, 1u8];
        assert_eq!(encoding, expected);
    }

    #[test]
    fn decode_bitlist() {
        let bytes = vec![1u8];
        let result = BitList::<32>::deserialize(&bytes).expect("test data is correct");
        let expected = BitList::<32>::default();
        assert_eq!(result, expected);

        let bytes = vec![24u8, 1u8];
        let result = BitList::<32>::deserialize(&bytes).expect("test data is correct");
        let expected =
            BitList::try_from([false, false, false, true, true, false, false, false].as_ref())
                .unwrap();
        assert_eq!(result, expected);

        let bytes = vec![24u8, 2u8];
        let result = BitList::<32>::deserialize(&bytes).expect("test data is correct");
        let expected = BitList::try_from(
            [false, false, false, true, true, false, false, false, false].as_ref(),
        )
        .unwrap();
        assert_eq!(result, expected);
        let bytes = vec![24u8, 3u8];
        let result = BitList::<32>::deserialize(&bytes).expect("test data is correct");
        let expected = BitList::try_from(
            [false, false, false, true, true, false, false, false, true].as_ref(),
        )
        .unwrap();
        assert_eq!(result, expected);

        let bytes = vec![24u8, 0u8];
        let result = BitList::<32>::deserialize(&bytes).expect_err("test data is incorrect");
        let expected = SSZError::OffsetOutOfBounds;
        assert_eq!(result.to_string(), expected.to_string());
    }

    #[test]
    fn roundtrip_bitlist() {
        let input = BitList::<32>::try_from(
            [
                false, false, false, true, true, false, false, false, false, false, false, false,
                false, false, false, true, true, false, false, false, false, false, false, false,
                false, false, false, true, true, false, false, false,
            ]
            .as_ref(),
        )
        .unwrap();
        let buffer = input.serialize().expect("can serialize");
        let recovered = BitList::<32>::deserialize(&buffer).expect("can decode");
        assert_eq!(input, recovered);
    }
}
