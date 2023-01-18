use crate::*;
use std::{cell::RefCell, rc::Rc};

pub struct Runtime {
	instance: wasmtime::Instance,
	store: Rc<RefCell<wasmtime::Store<Option<RuntimeCaller>>>>,
}

#[derive(Clone, Copy)]
pub(crate) struct RuntimeCaller {
	pub(crate) memory: wasmtime::Memory,
	pub(crate) alloc_fn: wasmtime::TypedFunc<u32, u32>,
	pub(crate) dealloc_fn: wasmtime::TypedFunc<u64, ()>,
}

impl Runtime {
	pub fn load_from_file<P: AsRef<std::path::Path>>(path: P, imports: &'static [(&'static str, fn(&[u8]) -> Vec<u8>)]) -> anyhow::Result<Self> {
		let engine = wasmtime::Engine::default();
		let module = wasmtime::Module::from_file(&engine, path)?;
		let mut store = wasmtime::Store::new(&engine, None);
		let mut linker = wasmtime::Linker::new(&engine);
		for (name, callback) in imports {
			linker.func_wrap("env", name, |mut caller: wasmtime::Caller<Option<RuntimeCaller>>, ptr: u64| -> u64 {
				let RuntimeCaller { memory, alloc_fn, dealloc_fn } = caller.data().unwrap();
				let (ptr, len) = from_bitwise(ptr);
				let mut buffer = vec![0u8; len as _];
				memory.read(&caller, ptr as _, &mut buffer).unwrap();
				dealloc_fn.call(&mut caller, into_bitwise(ptr, len)).unwrap();
				let buffer = (callback)(&buffer);
				let ptr = alloc_fn.call(&mut caller, buffer.len() as _).unwrap();
				memory.write(&mut caller, ptr as _, &buffer).unwrap();
				into_bitwise(ptr, buffer.len() as _)
			})?;
		}
		let instance = linker.instantiate(&mut store, &module)?;
		let memory = instance.get_memory(&mut store, "memory").unwrap();
		let alloc_fn = instance.get_typed_func(&mut store, "alloc")?;
		let dealloc_fn = instance.get_typed_func(&mut store, "dealloc")?;
		*store.data_mut() = Some(RuntimeCaller { memory, alloc_fn, dealloc_fn });
		Ok(Self { instance, store: Rc::new(RefCell::new(store)) })
	}
	pub fn get_func<P: serde::ser::Serialize, R: serde::de::DeserializeOwned>(&self, name: &str) -> anyhow::Result<Func<P, R>> {
		let wasm_fn = self.instance.get_typed_func::<u64, u64>(&mut *self.store.borrow_mut(), &format!("_wasm_guest_{name}"))?;
		Ok(Func {
			wasm_fn,
			store: self.store.clone(),
			par: std::marker::PhantomData::<P>,
			rtn: std::marker::PhantomData::<R>,
		})
	}
}
