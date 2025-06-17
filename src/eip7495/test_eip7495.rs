//! Tests the serialization and merkleization of eip-7495 using spec tests
///
/// See: <https://eips.ethereum.org/EIPS/eip-7495>
///
use crate::ssz::SszTypeInfo;
use crate::{
    BYTES_PER_LENGTH_OFFSET, BitVector, Merkleize, SSZError, SimpleSerialize,
    merkleization::{merkleize, mix_in_aux},
};
use alloc::vec;
use alloc::vec::Vec;
use alloy_primitives::B256;

const N1: usize = 4;
const N2: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shape1 {
    pub side: Option<u16>,
    pub color: Option<u8>,
    pub radius: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shape2 {
    pub side: Option<u16>,
    pub color: Option<u8>,
    pub radius: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shape3 {
    pub side: Option<u16>,
    pub colors: Option<[u8; 2]>,
    pub radius: Option<u16>,
}

macro_rules! impl_stable_container_3 {
    ($name:ident, $n:expr, $($field:ident : $typ:ty),+$(,)?) => {
         impl SimpleSerialize for $name {
            fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
                let mut active_flags_vec = vec![];
                let mut active_fields: Vec<(&str, bool)> = vec![];
                $(
                    let is_some = self.$field.is_some();
                    active_flags_vec.push(is_some);
                    active_fields.push((stringify!($field), is_some));
                )+
                active_flags_vec.resize($n, false);
                let bitvector = BitVector::<$n>::from_bools(&active_flags_vec)?;
                // Collect active field values
                let mut fixed_parts: Vec<Option<Vec<u8>>> = vec![];
                let mut variable_parts: Vec<Vec<u8>> = vec![];

                $(
                    if let Some(val) = &self.$field {
                        let mut ser = vec![];
                        val.serialize(&mut ser)?;

                        if !<$typ>::is_fixed_size() {
                            fixed_parts.push(None);
                            variable_parts.push(ser);
                        } else {
                            fixed_parts.push(Some(ser));
                        }
                    }
                )+

                // Compute fixed_lengths
                let mut fixed_lengths = vec![];
                for part in &fixed_parts {
                    if let Some(p) = part {
                        fixed_lengths.push(p.len());
                    } else {
                        fixed_lengths.push(BYTES_PER_LENGTH_OFFSET);
                    }
                }

                // Compute variable_offsets
                let mut variable_offsets = vec![];
                let mut current_offset = fixed_lengths.iter().sum::<usize>();
                for part in &variable_parts {
                    let mut offset_buf = vec![];
                    (current_offset as u32).serialize(&mut offset_buf)?;
                    variable_offsets.push(offset_buf);
                    current_offset += part.len();
                }

                // Replace None in fixed_parts with variable_offsets
                let mut offset_idx = 0;
                let final_fixed_parts: Vec<Vec<u8>> = fixed_parts
                    .into_iter()
                    .map(|p| {
                        if let Some(v) = p {
                            v
                        } else {
                            let v = variable_offsets[offset_idx].clone();
                            offset_idx += 1;
                            v
                        }
                    })
                    .collect();

                // Final serialization: bitvector + fixed + variable
                let mut full_serialized = vec![];
                bitvector.serialize(&mut full_serialized)?;
                for part in final_fixed_parts {
                    full_serialized.extend(part);
                }
                for part in variable_parts {
                    full_serialized.extend(part);
                }

                buffer.extend(full_serialized);
                Ok(buffer.len())
            }
        }

impl Merkleize for $name {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        const N: usize = $n;

        // 1. Hash each field, or zero if None
        let mut field_hashes: Vec<B256> = vec![B256::ZERO; N];
        let mut active_bits = BitVector::<N>::default();

        let mut idx = 0;
        $(
            if let Some(value) = &self.$field {
                field_hashes[idx] = value.hash_tree_root()?;
                active_bits.set(idx, true).unwrap();
            }
            idx += 1;
        )+

        // 2. Convert field_hashes to [u8; 32] form
        let hashes: Vec<[u8; 32]> = field_hashes.into_iter().map(Into::into).collect();

        // 3. Merkleize and mix in BitVector root
        let data_root = merkleize(&hashes, None)?;
        let bits_root = active_bits.hash_tree_root()?;

        Ok(mix_in_aux(data_root, bits_root))
    }

    fn chunk_count() -> usize {
        $n
    }
}

    };
}

impl_stable_container_3!(Shape1, N1, side: u16, color: u8, radius: u16);
impl_stable_container_3!(Shape2, N2, side: u16, color: u8, radius: u16);

impl SimpleSerialize for Shape3 {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        const N: usize = 8;

        // Bitvector for 3 fields, padded to 8
        let mut active_flags = vec![
            self.side.is_some(),
            self.colors.is_some(),
            self.radius.is_some(),
        ];
        active_flags.resize(N, false);
        let bitvector = BitVector::<N>::from_bools(&active_flags)?;

        // Active fields (order matters!)
        let mut fixed_parts: Vec<Option<Vec<u8>>> = vec![];
        let mut variable_parts: Vec<Vec<u8>> = vec![];

        if let Some(val) = self.side {
            let mut buf = vec![];
            val.serialize(&mut buf)?;
            fixed_parts.push(Some(buf));
        }

        if let Some(val) = self.colors {
            let mut buf = vec![];
            val.serialize(&mut buf)?;
            fixed_parts.push(None); // variable-size placeholder
            variable_parts.push(buf);
        }

        if let Some(val) = self.radius {
            let mut buf = vec![];
            val.serialize(&mut buf)?;
            fixed_parts.push(Some(buf));
        }

        // Compute fixed lengths
        let fixed_lengths: Vec<usize> = fixed_parts
            .iter()
            .map(|p| p.as_ref().map_or(BYTES_PER_LENGTH_OFFSET, |v| v.len()))
            .collect();

        // Compute variable offsets (relative to post-bitvector fields)
        let mut variable_offsets = vec![];
        let mut current_offset = fixed_lengths.iter().sum::<usize>();
        for part in &variable_parts {
            let mut buf = vec![];
            (current_offset as u32).serialize(&mut buf)?;
            variable_offsets.push(buf);
            current_offset += part.len();
        }

        // Finalize fixed section with variable offsets
        let mut offset_idx = 0;
        let final_fixed_parts: Vec<Vec<u8>> = fixed_parts
            .into_iter()
            .map(|part| {
                if let Some(bytes) = part {
                    bytes
                } else {
                    let v = variable_offsets[offset_idx].clone();
                    offset_idx += 1;
                    v
                }
            })
            .collect();

        // Compose final buffer
        let mut result = vec![];
        bitvector.serialize(&mut result)?;
        for part in final_fixed_parts {
            result.extend(part);
        }
        for part in variable_parts {
            result.extend(part);
        }
        buffer.extend(result);
        Ok(buffer.len())
    }
}

impl Merkleize for Shape3 {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        const N: usize = 8;
        let mut chunks: Vec<[u8; 32]> = vec![];

        chunks.push(match &self.side {
            Some(x) => x.hash_tree_root()?.into(),
            None => [0u8; 32],
        });

        chunks.push(match &self.colors {
            Some(x) => x.hash_tree_root()?.into(),
            None => [0u8; 32],
        });

        chunks.push(match &self.radius {
            Some(x) => x.hash_tree_root()?.into(),
            None => [0u8; 32],
        });
        for _ in 3..N {
            chunks.push([0u8; 32]);
        }
        let merkle_root = merkleize(&chunks, None);

        let mut bits = BitVector::<N>::default();
        if self.side.is_some() {
            bits.set(0, true)?;
        }
        if self.colors.is_some() {
            bits.set(1, true)?;
        }
        if self.radius.is_some() {
            bits.set(2, true)?;
        }
        let active_root = bits.hash_tree_root()?;

        Ok(mix_in_aux(merkle_root?, active_root))
    }

    fn chunk_count() -> usize {
        8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::hex;

    fn hash(hex: &str) -> B256 {
        B256::from_slice(&hex::decode(hex).expect("Invalid hex string"))
    }

    #[test]
    fn test_shape1_cases() {
        let test_cases = [
            (
                Shape1 {
                    side: Some(0x42),
                    color: Some(1),
                    radius: Some(0x42),
                },
                "074200014200",
                hash("37b28eab19bc3e246e55d2e2b2027479454c27ee006d92d4847c84893a162e6d"),
            ),
            (
                Shape1 {
                    side: Some(0x42),
                    color: Some(1),
                    radius: None,
                },
                "03420001",
                hash("bfdb6fda9d02805e640c0f5767b8d1bb9ff4211498a5e2d7c0f36e1b88ce57ff"),
            ),
            (
                Shape1 {
                    side: None,
                    color: Some(1),
                    radius: None,
                },
                "0201",
                hash("522edd7309c0041b8eb6a218d756af558e9cf4c816441ec7e6eef42dfa47bb98"),
            ),
            (
                Shape1 {
                    side: None,
                    color: Some(1),
                    radius: Some(0x42),
                },
                "06014200",
                hash("f66d2c38c8d2afbd409e86c529dff728e9a4208215ca20ee44e49c3d11e145d8"),
            ),
        ];
        for (val, expected_ser, expected_root) in test_cases {
            let mut buffer = Vec::new();
            val.serialize(&mut buffer).unwrap();
            assert_eq!(hex::encode(&buffer), expected_ser);
            assert_eq!(val.hash_tree_root().unwrap(), expected_root);
        }
    }

    #[test]
    fn test_shape2_cases() {
        let test_cases = [
            (
                Shape2 {
                    side: Some(0x42),
                    color: Some(1),
                    radius: Some(0x42),
                },
                "074200014200",
                hash("0792fb509377ee2ff3b953dd9a88eee11ac7566a8df41c6c67a85bc0b53efa4e"),
            ),
            (
                Shape2 {
                    side: Some(0x42),
                    color: Some(1),
                    radius: None,
                },
                "03420001",
                hash("ddc7acd38ae9d6d6788c14bd7635aeb1d7694768d7e00e1795bb6d328ec14f28"),
            ),
            (
                Shape2 {
                    side: None,
                    color: Some(1),
                    radius: None,
                },
                "0201",
                hash("9893ecf9b68030ff23c667a5f2e4a76538a8e2ab48fd060a524888a66fb938c9"),
            ),
            (
                Shape2 {
                    side: None,
                    color: Some(1),
                    radius: Some(0x42),
                },
                "06014200",
                hash("e823471310312d52aa1135d971a3ed72ba041ade3ec5b5077c17a39d73ab17c5"),
            ),
        ];

        for (val, expected_ser, expected_root) in test_cases {
            let mut buffer = Vec::new();
            val.serialize(&mut buffer).unwrap();
            assert_eq!(hex::encode(&buffer), expected_ser);
            assert_eq!(val.hash_tree_root().unwrap(), expected_root);
        }
    }

    #[test]
    fn test_shape3_cases() {
        let test_cases = [
            (
                Shape3 {
                    side: Some(0x42),
                    colors: Some([1, 2]),
                    radius: Some(0x42),
                },
                "0742000800000042000102",
                hash("1093b0f1d88b1b2b458196fa860e0df7a7dc1837fe804b95d664279635cb302f"),
            ),
            (
                Shape3 {
                    side: Some(0x42),
                    colors: None,
                    radius: None,
                },
                "014200",
                hash("28df3f1c3eebd92504401b155c5cfe2f01c0604889e46ed3d22a3091dde1371f"),
            ),
            (
                Shape3 {
                    side: None,
                    colors: Some([1, 2]),
                    radius: None,
                },
                "02040000000102",
                hash("659638368467b2c052ca698fcb65902e9b42ce8e94e1f794dd5296ceac2dec3e"),
            ),
            (
                Shape3 {
                    side: None,
                    colors: None,
                    radius: Some(0x42),
                },
                "044200",
                hash("d585dd0561c718bf4c29e4c1bd7d4efd4a5fe3c45942a7f778acb78fd0b2a4d2"),
            ),
            (
                Shape3 {
                    side: None,
                    colors: Some([1, 2]),
                    radius: Some(0x42),
                },
                "060600000042000102",
                hash("00fc0cecc200a415a07372d5d5b8bc7ce49f52504ed3da0336f80a26d811c7bf"),
            ),
        ];

        for (val, expected_ser, expected_root) in test_cases {
            let mut buffer = Vec::new();
            val.serialize(&mut buffer).unwrap();
            assert_eq!(hex::encode(&buffer), expected_ser);
            assert_eq!(val.hash_tree_root().unwrap(), expected_root);
        }
    }
}
