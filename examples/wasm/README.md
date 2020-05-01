== ttf-parser as WebAssembly module ==

Build:

```
rustup target add wasm32-unknown-unknown

cargo build --target wasm32-unknown-unknown --release --manifest-path ../../c-api/Cargo.toml
cp ../../c-api/target/wasm32-unknown-unknown/release/ttfparser.wasm .
```