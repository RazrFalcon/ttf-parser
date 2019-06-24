## ttf-parser
[![Build Status](https://travis-ci.org/RazrFalcon/ttf-parser.svg?branch=master)](https://travis-ci.org/RazrFalcon/ttf-parser)
[![Crates.io](https://img.shields.io/crates/v/ttf-parser.svg)](https://crates.io/crates/ttf-parser)
[![Documentation](https://docs.rs/ttf-parser/badge.svg)](https://docs.rs/ttf-parser)

A high-level, safe, zero-allocation TrueType font parser.

### Features

- A high-level API.
- Zero allocations.
- Zero `unsafe`.
- Zero dependencies.
- Fast.
- Simple and maintainable code (no magic numbers).

### Limitations

- Non [ARGS_ARE_XY_VALUES] transform is not supported yet.
- Only 0, 4, 12 and 13 formats of `cmap` table are supported.

[ARGS_ARE_XY_VALUES]: https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#composite-glyph-description

### Safety

- The library heavily relies on Rust's bounds checking and assumes that font is well-formed.
  You can invoke a checksums checking manually.
- The library uses per table slices, so it can't read data outside the specified TrueType table.
- The library forbids `unsafe` code.

### Alternatives

- [font-rs](https://crates.io/crates/font-rs) - Mainly a glyph outline extractor.
  No documentation. Has less features. A lot of magic numbers.
- [truetype](https://crates.io/crates/truetype) - Isn't allocation free.
  Does a little postprocessing (parses most of the data as is). Has some **unsafe**.
- [stb_truetype](https://crates.io/crates/stb_truetype) - Mainly a glyph outline extractor.
  Isn't allocation free. Has less features. Uses `panic` a lot.

### License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
