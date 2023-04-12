#[cfg(test)]
mod test {
	use wasmtime_serde_host::*;
	const GUEST_DATA: &[u8] = include_bytes!("guest.wasm");

	#[test]
	fn load_runtime() {
		assert!(Runtime::new(GUEST_DATA, &[]).is_ok())
	}

	#[test]
	fn get_func() {
		let runtime = Runtime::new(GUEST_DATA, &[]).unwrap();
		assert!(runtime.get_func::<(i32, i32), i32>("add").is_ok())
	}

	#[test]
	fn call() {
		let runtime = Runtime::new(GUEST_DATA, &[]).unwrap();
		let add_fn = runtime.get_func::<(i32, i32), i32>("add").unwrap();
		let result = add_fn.call(&(10, 10));
		assert_eq!(result, 20)
	}

	#[test]
	fn checked_call() {
		let runtime = Runtime::new(GUEST_DATA, &[]).unwrap();
		let panic_fn = runtime.get_func::<(), ()>("panic").unwrap();
		let result = panic_fn.checked_call(&());
		assert!(result.is_err())
	}
}
