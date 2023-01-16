use serde::{Deserialize, Serialize};
use wasmtime_serde_host::*;

#[derive(Debug, Deserialize, Serialize)]
struct Human{
	name: String,
	age: u8
}

#[export_fn]
fn print(msg: String){
	println!("{msg}")
}

#[export_fn]
fn get_human() -> Human{
	Human{
		name: "Ferros".to_string(),
		age: 192
	}
}

fn main() {
	let host_fns = host_funcs![
		print,
		get_human
	];
	let runtime = Runtime::load_from_file("../guest/target/wasm32-unknown-unknown/debug/guest.wasm", host_fns).unwrap();
	let add_fn = runtime.get_func::<(i32, i32), i32>("add").unwrap();
	let result = add_fn.call(&(1, 2));
	println!("{result}");
}