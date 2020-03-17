#!/usr/bin/env bash

set -e

# build without logging
cargo build --no-default-features
# build with logging
cargo build

cargo test

cd c-api
cargo build
gcc test.c -o test -L./target/debug/ -lttfparser -Werror
env LD_LIBRARY_PATH=./target/debug/ ./test
cd ..

cd benches
cargo build
