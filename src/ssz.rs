//! This module provides a simple serialization and deserialization mechanism for data structures.

use crate::SSZError;
use alloc::vec::Vec;
use alloy_primitives::B256;

/// The `SimpleSerialize` trait defines methods for serializing data structures
pub trait SimpleSerialize: Sized {
    /// Serializes the data structure into a byte vector.
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SSZError>;
}

/// The `SimpleDeserialize` trait defines methods for deserializing data structures
pub trait SimpleDeserialize: Sized {
    /// Deserializes the data structure from a byte slice.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError>;
}

/// The `SszTypeInfo` trait provides information about the size characteristics of a type.
pub trait SszTypeInfo {
    /// If Some(size), then type is fixed-size with known size in bytes.
    /// If None, then it's variable-size (e.g. Vec<u8>, String, etc).
    fn is_fixed_size() -> bool;
    /// If fixed-size, returns the size in bytes.
    fn fixed_size() -> Option<usize>;
    /// Returns true if this type is a basic type (e.g. u8, u16, etc).
    fn is_basic_type() -> bool {
        false
    }
}

/// Merkleization trait for SSZ types
pub trait Merkleize {
    /// Calculate the hash tree root of this value
    fn hash_tree_root(&self) -> Result<B256, SSZError>;

    /// Get the chunk count for merkleization
    fn chunk_count() -> usize
    where
        Self: Sized,
    {
        1 // Default for basic types
    }
}
