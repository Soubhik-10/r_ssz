use crate::ssz::SszTypeInfo;
use crate::{BITS_PER_BYTE, BYTES_PER_LENGTH_OFFSET};
use crate::{
    BitVector, Merkleize, SSZError, SimpleDeserialize, SimpleSerialize,
    merkleization::{merkleize, mix_in_aux},
};
use alloc::vec::Vec;
use alloy_primitives::B256;

#[derive(Debug, Clone, PartialEq)]
pub struct MyStableContainer {
    pub a: Option<u32>,
    pub b: Option<bool>,
    pub c: Option<u64>,
}

pub const N: usize = 4;

impl SimpleSerialize for MyStableContainer {
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        // Create the bitvector
        let mut active_flags_vec =
            alloc::vec![self.a.is_some(), self.b.is_some(), self.c.is_some()];

        active_flags_vec.resize(N, false);

        let active_flags: [bool; N] = active_flags_vec.try_into().unwrap();
        let bitvector = BitVector::<N>::from_bools(&active_flags[..])?;
        // serialize the bitvector to be appended first
        let _ = bitvector.serialize(buffer)?;

        // Collect active values
        let active_values: Vec<(Vec<u8>, bool)> = {
            let mut temp = Vec::new();

            if let Some(a) = self.a {
                let mut buf = Vec::new();
                a.serialize(&mut buf)?;
                temp.push((buf, u32::is_fixed_size()));
            }

            if let Some(b) = self.b {
                let mut buf = Vec::new();
                b.serialize(&mut buf)?;
                temp.push((buf, bool::is_fixed_size()));
            }

            if let Some(c) = self.c {
                let mut buf = Vec::new();
                c.serialize(&mut buf)?;
                temp.push((buf, u64::is_fixed_size()));
            }

            temp
        };

        // Separate fixed-size and variable-size parts
        let mut fixed_lengths = Vec::new();
        let mut fixed_parts = Vec::new();
        let mut variable_parts = Vec::new();

        for (buf, is_fixed) in &active_values {
            if *is_fixed {
                fixed_parts.push(Some(buf.clone()));
                fixed_lengths.push(buf.len());
            } else {
                fixed_parts.push(None);
                fixed_lengths.push(BYTES_PER_LENGTH_OFFSET);
                variable_parts.push(buf.clone());
            }
        }

        // Verify total size
        let variable_lengths: Vec<usize> = variable_parts.iter().map(|v| v.len()).collect();
        let total_len: usize =
            fixed_lengths.iter().sum::<usize>() + variable_lengths.iter().sum::<usize>();

        if total_len >= 1 << (BYTES_PER_LENGTH_OFFSET * BITS_PER_BYTE) {
            return Err(SSZError::OffsetOutOfBounds);
        }

        // Compute and serialize offsets
        let mut variable_offsets = Vec::new();
        let mut offset = fixed_lengths.iter().sum::<usize>();

        for var_len in &variable_lengths {
            let mut offset_buf = Vec::new();
            u32::try_from(offset).unwrap().serialize(&mut offset_buf)?;
            variable_offsets.push(offset_buf);
            offset += var_len;
        }

        // Append fixed parts (offsets interleaved where needed)
        let mut var_offset_index = 0;
        for part in fixed_parts {
            if let Some(data) = part {
                buffer.extend_from_slice(&data);
            } else {
                buffer.extend_from_slice(&variable_offsets[var_offset_index]);
                var_offset_index += 1;
            }
        }

        // Append variable parts
        for var_part in variable_parts {
            buffer.extend_from_slice(&var_part);
        }

        Ok(buffer.len())
    }
}

impl SimpleDeserialize for MyStableContainer {
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        const NUM_FIELDS: usize = 3;

        // Step 1: Deserialize bitvector and validate extra bits
        let mut cursor = 0;
        let bitvector = {
            let bv = BitVector::<N>::deserialize(&data[cursor..])?;
            cursor += (N + 7) / 8; // consume bits
            // Validate unused bits
            for i in NUM_FIELDS..N {
                if bv.get(i).unwrap_or(false) {
                    return Err(SSZError::InvalidBitvector);
                }
            }
            bv
        };

        // Step 2: Deserialize fixed-size fields based on presence
        let mut a = None;
        let mut b = None;
        let mut c = None;

        if bitvector.get(0).unwrap_or(false) {
            a = Some(u32::deserialize(&data[cursor..])?);
            cursor += 4;
        }
        if bitvector.get(1).unwrap_or(false) {
            b = Some(bool::deserialize(&data[cursor..])?);
            cursor += 1;
        }
        if bitvector.get(2).unwrap_or(false) {
            c = Some(u64::deserialize(&data[cursor..])?);
        }

        Ok(Self { a, b, c })
    }
}

impl Merkleize for MyStableContainer {
    fn hash_tree_root(&self) -> Result<B256, SSZError> {
        // Step 1: hash each field or use default
        let a_hash = match self.a {
            Some(x) => x.hash_tree_root(),
            None => Ok(B256::ZERO),
        };
        let b_hash = match self.b {
            Some(x) => x.hash_tree_root(),
            None => Ok(B256::ZERO),
        };
        let c_hash = match self.c {
            Some(x) => x.hash_tree_root(),
            None => Ok(B256::ZERO),
        };

        let field_hashes = alloc::vec![a_hash, b_hash, c_hash];
        let hashes: Vec<[u8; 32]> = field_hashes
            .into_iter()
            .map(|res| res.unwrap().into()) // unwrap() panics on error, into() converts to [u8; 32]
            .collect();
        // Step 2: compute merkle root of fields
        let merkle_root = merkleize(&hashes, None);

        // Step 3: construct active fields bitvector
        let mut bits = BitVector::<3>::default(); // or a custom impl
        if self.a.is_some() {
            bits.set(0, true).unwrap();
        }
        if self.b.is_some() {
            bits.set(1, true).unwrap();
        }
        if self.c.is_some() {
            bits.set(2, true).unwrap();
        }

        let active_root = bits.hash_tree_root();

        // Step 4: mix the auxiliary
        Ok(mix_in_aux(merkle_root?, active_root?))
    }

    fn chunk_count() -> usize {
        3
    }
}
