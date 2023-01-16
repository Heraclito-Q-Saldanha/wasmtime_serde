//! Simple library for serializing complex types to the wasmtime runtime using serde

pub use bincode::{deserialize, serialize};
pub use wasmtime_serde_guest_macro::*;

#[no_mangle]
pub extern "C" fn alloc(len: u32) -> *mut u8 {
	let mut buf = Vec::with_capacity(len as _);
	let ptr = buf.as_mut_ptr();
	std::mem::forget(buf);
	return ptr;
}

#[no_mangle]
pub unsafe extern "C" fn dealloc(value: u64) {
	let (ptr, len) = from_bitwise(value);
	let ptr = std::mem::transmute::<usize, *mut u8>(ptr as _);
	let buffer = Vec::from_raw_parts(ptr, len as _, len as _);
	std::mem::drop(buffer);
}

pub fn write_msg<T: serde::ser::Serialize>(value: &T) -> u64 {
	let mut buffer = bincode::serialize(value).unwrap();
	let len = buffer.len();
	let ptr = buffer.as_mut_ptr();
	std::mem::forget(buffer);
	into_bitwise(ptr as _, len as _)
}

pub unsafe fn read_msg<T: serde::de::DeserializeOwned>(value: u64) -> T {
	let (ptr, len) = from_bitwise(value);
	let ptr = std::mem::transmute::<usize, *mut u8>(ptr as _);
	let buffer = Vec::from_raw_parts(ptr, len as _, len as _);
	bincode::deserialize(&buffer).unwrap()
}

fn from_bitwise(value: u64) -> (u32, u32) {
	((value << 32 >> 32) as u32, (value >> 32) as u32)
}

fn into_bitwise(a: u32, b: u32) -> u64 {
	(a as u64) | (b as u64) << 32
}
