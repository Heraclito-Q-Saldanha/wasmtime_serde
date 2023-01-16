use crate::*;

pub struct Func<'a, P: serde::ser::Serialize, R: serde::de::DeserializeOwned> {
	pub(crate) wasm_fn: wasmtime::TypedFunc<u64, u64>,
	pub(crate) runtime: &'a Runtime,
	pub(crate) par: std::marker::PhantomData<P>,
	pub(crate) rtn: std::marker::PhantomData<R>,
}

impl<'a, P: serde::ser::Serialize, R: serde::de::DeserializeOwned> Func<'a, P, R> {
	pub fn call(&self, value: &P) -> R {
		let ptr = self.runtime.write_msg(value).unwrap();
		let ptr = self.runtime.call(&self.wasm_fn, ptr).unwrap();
		self.runtime.read_msg(ptr).unwrap()
	}
}
