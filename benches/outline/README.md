## Build & Run

```sh
cargo build --release --manifest-path ../../c-api/Cargo.toml
meson builddir
ninja -C builddir
builddir/outline
```
