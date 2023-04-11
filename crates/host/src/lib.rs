//! Simple library for serializing complex types to the wasmtime runtime using serde

mod func;
mod runtime;

pub use bincode::{deserialize, serialize};
pub use func::*;
pub use runtime::*;
pub use wasmtime_serde_host_macro::*;

#[inline(always)]
fn from_bitwise(value: u64) -> (u32, u32) {
	((value << 32 >> 32) as u32, (value >> 32) as u32)
}

#[inline(always)]
fn into_bitwise(a: u32, b: u32) -> u64 {
	(a as u64) | (b as u64) << 32
}
