use crate::*;
use std::cell::RefCell;

pub struct Runtime {
	instance: wasmtime::Instance,
	store: RefCell<wasmtime::Store<Option<RuntimeCaller>>>,
}

#[derive(Clone, Copy)]
struct RuntimeCaller {
	memory: wasmtime::Memory,
	alloc_fn: wasmtime::TypedFunc<u32, u32>,
	dealloc_fn: wasmtime::TypedFunc<u64, ()>,
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
		Ok(Self { instance, store: RefCell::new(store) })
	}
	pub fn get_func<P: serde::ser::Serialize, R: serde::de::DeserializeOwned>(&self, name: &str) -> anyhow::Result<Func<P, R>> {
		let wasm_fn = self.instance.get_typed_func::<u64, u64>(&mut *self.store.borrow_mut(), &format!("_wasm_guest_{name}"))?;
		Ok(Func {
			wasm_fn,
			runtime: self,
			par: std::marker::PhantomData::<P>,
			rtn: std::marker::PhantomData::<R>,
		})
	}
	pub fn write_msg<T: serde::ser::Serialize>(&self, value: &T) -> anyhow::Result<u64> {
		let buffer = serialize(value).unwrap();
		let offset = self.alloc(buffer.len() as _)?;
		let memory = self.store.borrow().data().unwrap().memory;
		memory.write(&mut *self.store.borrow_mut(), offset as _, &buffer)?;
		Ok(into_bitwise(offset, buffer.len() as _))
	}
	pub fn read_msg<T: serde::de::DeserializeOwned>(&self, value: u64) -> anyhow::Result<T> {
		let (ptr, len) = from_bitwise(value);
		let mut buffer = vec![0u8; len as _];
		let memory = self.store.borrow().data().unwrap().memory;
		memory.read(&mut *self.store.borrow_mut(), ptr as _, &mut buffer)?;
		self.dealloc(value)?;
		Ok(deserialize(&buffer)?)
	}
	pub(crate) fn call(&self, func: &wasmtime::TypedFunc<u64, u64>, ptr: u64) -> anyhow::Result<u64> {
		Ok(func.call(&mut *self.store.borrow_mut(), ptr)?)
	}
	pub(crate) fn alloc(&self, size: u32) -> anyhow::Result<u32> {
		let alloc_fn = self.store.borrow().data().unwrap().alloc_fn;
		Ok(alloc_fn.call(&mut *self.store.borrow_mut(), size)?)
	}
	pub(crate) fn dealloc(&self, ptr: u64) -> anyhow::Result<()> {
		let dealloc_fn = self.store.borrow().data().unwrap().dealloc_fn;
		Ok(dealloc_fn.call(&mut *self.store.borrow_mut(), ptr)?)
	}
}
