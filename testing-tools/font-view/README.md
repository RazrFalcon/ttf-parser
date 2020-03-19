# font-view

A simple tool to preview all glyphs in the font using `ttf-parser`, `freetype` and `harfbuzz`.

## Build

```sh
# build ttf-parser C API first
cargo build --release --manifest-path ../../c-api/Cargo.toml

# build only with ttf-parser support
qmake
make

# or build with freetype support
qmake DEFINES+=WITH_FREETYPE
make

# or build with harfbuzz support
# note that harfbuzz should be built via cmake
qmake DEFINES+=WITH_HARFBUZZ HARFBUZZ_SRC=/path/to/harfbuzz-master/
make

# or with all
qmake DEFINES+=WITH_FREETYPE DEFINES+=WITH_HARFBUZZ HARFBUZZ_SRC=/path/to/harfbuzz-master/
make
```
