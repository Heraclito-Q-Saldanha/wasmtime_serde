//! Simple library for serializing complex types to the wasmtime runtime using serde

mod func;
mod runtime;

pub use bincode::{deserialize, serialize};
pub use func::*;
pub use runtime::*;
pub use wasmtime_serde_host_macro::*;

#[inline(always)]
const fn from_bitwise(value: u64) -> (u32, u32) {
	((value << 32 >> 32) as u32, (value >> 32) as u32)
}

#[inline(always)]
const fn into_bitwise(a: u32, b: u32) -> u64 {
	(a as u64) | (b as u64) << 32
}

#[cfg(test)]
mod test {
	use crate::*;

	#[test]
	fn bitwise() {
		const DATA: (u32, u32) = (10, 20);
		const INTO: u64 = into_bitwise(DATA.0, DATA.1);
		const FROM: (u32, u32) = from_bitwise(INTO);
		assert_eq!(DATA, FROM)
	}
}
