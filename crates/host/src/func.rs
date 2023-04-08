use crate::*;
use std::{cell::RefCell, rc::Rc};

pub struct Func<P: serde::ser::Serialize, R: serde::de::DeserializeOwned> {
	pub(crate) wasm_fn: wasmtime::TypedFunc<u64, u64>,
	pub(crate) store: Rc<RefCell<wasmtime::Store<Option<RuntimeCaller>>>>,
	pub(crate) par: std::marker::PhantomData<P>,
	pub(crate) rtn: std::marker::PhantomData<R>,
}

impl<P: serde::ser::Serialize, R: serde::de::DeserializeOwned> Func<P, R> {
	/// a more ergonomic version of the check_call function, which panic if it fails, using an analogy to an array, if checked_call were array.get(i), call would be array\[i\]
	pub fn call(&self, value: &P) -> R {
		self.checked_call(value).unwrap()
	}
	/// fail if the function in the guest panic and does not return
	pub fn checked_call(&self, value: &P) -> anyhow::Result<R> {
		let RuntimeCaller { memory, alloc_fn, .. } = self.store.borrow().data().unwrap();
		let buffer = serialize(value)?;
		let len = buffer.len() as _;
		let ptr = alloc_fn.call(&mut *self.store.borrow_mut(), len)?;
		memory.write(&mut *self.store.borrow_mut(), ptr as _, &buffer)?;
		let ptr = self.wasm_fn.call(&mut *self.store.borrow_mut(), into_bitwise(ptr, len))?;
		let (ptr, len) = from_bitwise(ptr);
		let mut buffer = vec![0u8; len as _];
		memory.read(&*self.store.borrow(), ptr as _, &mut buffer)?;
		Ok(deserialize(&buffer)?)
	}
}
