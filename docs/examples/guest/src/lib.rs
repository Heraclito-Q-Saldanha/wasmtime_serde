use serde::{Deserialize, Serialize};
use wasmtime_serde_guest::*;

#[derive(Debug, Deserialize, Serialize)]
struct Human{
	name: String,
	age: u8
}

#[export_fn]
fn add(a: i32, b: i32) -> i32 {
	let human = get_human();
	print(format!("{human:?}"));
	a + b
}

import_fn!(
	fn print(msg: String);
	fn get_human() -> Human;
);