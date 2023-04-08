# wasmtime serde
Simple library for serializing complex types to the wasmtime runtime using serde

### using

```Rust
// guest
use wasmtime_serde_guest::*;

#[export_fn]
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// host
use wasmtime_serde_host::*;

fn main(){
    let runtime = Runtime::from_file("file.wasm", &[]).unwrap();
    let add_fn = runtime.get_func::<(i32, i32), i32>("add").unwrap();
    let result = add_fn.call(&(1, 2));
    println!("{result}");
}

```

See the [example code](examples)

Dual-licensed under [MIT](../LICENSE-MIT) or the [UNLICENSE](../UNLICENSE).