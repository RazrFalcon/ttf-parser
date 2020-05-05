#!/usr/bin/env bash

set -e

# build without logging
cargo build --no-default-features
# build with logging
cargo build

cargo test


cd c-api

cargo build
gcc test.c -o test -L./target/debug/ -lttfparser -Werror -fsanitize=address
env LD_LIBRARY_PATH=./target/debug/ ./test

# test WASM too
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release

cd ..


cd benches
cargo bench dummy # `cargo build` will not actually build it
