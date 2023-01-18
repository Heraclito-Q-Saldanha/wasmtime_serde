use crate::*;
use std::{cell::RefCell, rc::Rc};

pub struct Func<P: serde::ser::Serialize, R: serde::de::DeserializeOwned> {
	pub(crate) wasm_fn: wasmtime::TypedFunc<u64, u64>,
	pub(crate) store: Rc<RefCell<wasmtime::Store<Option<RuntimeCaller>>>>,
	pub(crate) par: std::marker::PhantomData<P>,
	pub(crate) rtn: std::marker::PhantomData<R>,
}

impl<P: serde::ser::Serialize, R: serde::de::DeserializeOwned> Func<P, R> {
	pub fn call(&self, value: &P) -> R {
		let RuntimeCaller { memory, alloc_fn, .. } = self.store.borrow().data().unwrap();
		let buffer = serialize(value).unwrap();
		let len = buffer.len() as _;
		let ptr = alloc_fn.call(&mut *self.store.borrow_mut(), len).unwrap();
		memory.write(&mut *self.store.borrow_mut(), ptr as _, &buffer).unwrap();
		let ptr = self.wasm_fn.call(&mut *self.store.borrow_mut(), into_bitwise(ptr, len)).unwrap();
		let (ptr, len) = from_bitwise(ptr);
		let mut buffer = vec![0u8; len as _];
		memory.read(& *self.store.borrow(), ptr as _, &mut buffer).unwrap();
		deserialize(&buffer).unwrap()
	}
}
