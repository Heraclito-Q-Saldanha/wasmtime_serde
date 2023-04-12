use wasmtime_serde_host::*;

#[test]
fn load_runtime() {
	const DATA: &[u8] = include_bytes!("guest.wasm");
	assert!(Runtime::new(DATA, &[]).is_ok())
}

#[test]
fn get_func() {
	const DATA: &[u8] = include_bytes!("guest.wasm");
	let runtime = Runtime::new(DATA, &[]).unwrap();
	assert!(runtime.get_func::<(i32, i32), i32>("add").is_ok())
}

#[test]
fn call() {
	const DATA: &[u8] = include_bytes!("guest.wasm");
	let runtime = Runtime::new(DATA, &[]).unwrap();
	let add_fn = runtime.get_func::<(i32, i32), i32>("add").unwrap();
	let result = add_fn.call(&(10, 10));
	assert_eq!(result, 20)
}

#[test]
fn checked_call() {
	const DATA: &[u8] = include_bytes!("guest.wasm");
	let runtime = Runtime::new(DATA, &[]).unwrap();
	let panic_fn = runtime.get_func::<(), ()>("panic").unwrap();
	let result = panic_fn.checked_call(&());
	assert!(result.is_err())
}
