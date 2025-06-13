//! Serialization and deserialization for FixedList.

use crate::{BYTES_PER_LENGTH_OFFSET, SSZError, SimpleDeserialize, SimpleSerialize, SszTypeInfo};
use alloc::vec::Vec;

pub struct FixedList<T, const N: usize>([T; N]);

impl<T, const N: usize> TryFrom<Vec<T>> for FixedList<T, N> {
    type Error = ();

    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        vec.try_into().map(FixedList).map_err(|_| ())
    }
}

/// Implements serialization for `FixedList`.
impl<T, const N: usize> SimpleSerialize for FixedList<T, N>
where
    T: SimpleSerialize + Clone + SszTypeInfo,
{
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError> {
        let start_len = buffer.len();

        if T::is_fixed_size() {
            for item in self.0.iter() {
                item.serialize(buffer)?;
            }
        } else {
            let offset_bytes_len = N * BYTES_PER_LENGTH_OFFSET;
            let mut parts = Vec::with_capacity(N);

            for item in self.0.iter() {
                let mut part = Vec::new();
                item.serialize(&mut part)?;
                parts.push(part);
            }

            let mut offset = offset_bytes_len;
            for part in &parts {
                buffer.extend(&(offset as u32).to_le_bytes());
                offset += part.len();
            }

            for part in parts {
                buffer.extend(part);
            }
        }

        Ok(buffer.len() - start_len)
    }
}

/// Implements deserialization for `FixedList`.
impl<T, const N: usize> SimpleDeserialize for FixedList<T, N>
where
    T: SimpleDeserialize + Clone + SszTypeInfo,
{
    fn deserialize(data: &[u8]) -> Result<Self, SSZError> {
        if T::is_fixed_size() {
            let size = T::fixed_size().ok_or(SSZError::InvalidByte)?;
            let total = size * N;

            if data.len() != total {
                return Err(SSZError::InvalidLength {
                    expected: total,
                    got: data.len(),
                });
            }

            let mut out_fixed: Vec<T> = Vec::with_capacity(N);
            for i in 0..N {
                let start = i * size;
                let end = start + size;
                let item = T::deserialize(&data[start..end])?;
                out_fixed.push(item);
            }

            out_fixed
                .clone()
                .try_into()
                .map_err(|_| SSZError::InvalidLength {
                    expected: N,
                    got: out_fixed.len(),
                })
        } else {
            let offset_bytes_len = BYTES_PER_LENGTH_OFFSET * N;
            if data.len() < offset_bytes_len {
                return Err(SSZError::InvalidLength {
                    expected: offset_bytes_len,
                    got: data.len(),
                });
            }

            let mut offsets = Vec::with_capacity(N);
            for i in 0..N {
                let start = i * BYTES_PER_LENGTH_OFFSET;
                let end = start + BYTES_PER_LENGTH_OFFSET;
                let offset = u32::from_le_bytes(data[start..end].try_into().unwrap()) as usize;
                if offset > data.len() {
                    return Err(SSZError::OffsetOutOfBounds);
                }
                offsets.push(offset);
            }

            let mut out_var: Vec<T> = Vec::with_capacity(N);
            for i in 0..N {
                let start = offsets[i];
                let end = if i + 1 < N {
                    offsets[i + 1]
                } else {
                    data.len()
                };
                if start > end || end > data.len() {
                    return Err(SSZError::InvalidOffsetRange { start, end });
                }
                let item = T::deserialize(&data[start..end])?;
                out_var.push(item);
            }

            out_var
                .clone()
                .try_into()
                .map_err(|_| SSZError::InvalidLength {
                    expected: N,
                    got: out_var.len(),
                })
        }
    }
}
