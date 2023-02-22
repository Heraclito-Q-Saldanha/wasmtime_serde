# examples

add wasm target

```
rustup target add wasm32-unknown-unknown
```

compile guest
```
(cd guest; cargo build)
```

compile and run host
```
(cd host; cargo run)
```