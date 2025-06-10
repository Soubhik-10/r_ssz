//! This module provides a simple serialization and deserialization mechanism for data structures.

use crate::SSZError;
use alloy_primitives::B256;

/// The `SimpleSerialize` trait defines methods for serializing and deserializing data structures
pub trait SimpleSerialize: Sized {
    /// Serializes the data structure into a byte vector.
    fn serialize(&self) -> Result<Vec<u8>, SSZError>;
    /// Deserializes the data structure from a byte slice.
    fn deserialize(data: &[u8]) -> Result<Self, SSZError>;
}

/// The `SszTypeInfo` trait provides information about the size characteristics of a type.
pub trait SszTypeInfo {
    /// If Some(size), then type is fixed-size with known size in bytes.
    /// If None, then it's variable-size (e.g. Vec<u8>, String, etc).
    fn is_fixed_size() -> bool;
    fn fixed_size() -> Option<usize>;
    fn is_basic_type()->bool{
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
