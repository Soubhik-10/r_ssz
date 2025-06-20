//! Contains the base modules.
// #![no_std]
#![allow(unused_assignments)]
extern crate alloc;

pub mod basic;
pub use basic::*;

pub mod composite;
pub use composite::*;

pub mod ssz;
pub use ssz::*;

pub mod error;
pub use error::*;

pub mod constants;
pub use constants::*;

pub mod merkleization;

pub mod eip7495;
pub use eip7495::*;
