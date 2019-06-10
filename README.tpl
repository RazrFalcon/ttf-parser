## {{crate}}
[![Build Status](https://travis-ci.org/RazrFalcon/{{crate}}.svg?branch=master)](https://travis-ci.org/RazrFalcon/{{crate}})
[![Crates.io](https://img.shields.io/crates/v/{{crate}}.svg)](https://crates.io/crates/{{crate}})
[![Documentation](https://docs.rs/{{crate}}/badge.svg)](https://docs.rs/{{crate}})

{{readme}}

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
