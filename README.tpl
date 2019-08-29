## {{crate}}
[![Build Status](https://travis-ci.org/RazrFalcon/{{crate}}.svg?branch=master)](https://travis-ci.org/RazrFalcon/{{crate}})
[![Crates.io](https://img.shields.io/crates/v/{{crate}}.svg)](https://crates.io/crates/{{crate}})
[![Documentation](https://docs.rs/{{crate}}/badge.svg)](https://docs.rs/{{crate}})
[![Rust 1.35+](https://img.shields.io/badge/rust-1.35+-orange.svg)](https://www.rust-lang.org)

{{readme}}

### Alternatives

- [font-rs](https://crates.io/crates/font-rs) - Mainly a glyph outline extractor.
  No documentation. Has less features. Doesn't support CFF. Has a lot of magic numbers.
- [stb_truetype](https://crates.io/crates/stb_truetype) - Mainly a glyph outline extractor.
  Isn't allocation free. Has less features. Doesn't support CFF. Has a lot of magic numbers.
  Uses `panic` a lot.
- [truetype](https://crates.io/crates/truetype) - Simply maps TrueType data to the Rust structures.
  Doesn't actually parses the data. Isn't allocation free. Has some **unsafe**.

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
