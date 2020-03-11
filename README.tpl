## {{crate}}
[![Build Status](https://travis-ci.org/RazrFalcon/{{crate}}.svg?branch=master)](https://travis-ci.org/RazrFalcon/{{crate}})
[![Crates.io](https://img.shields.io/crates/v/{{crate}}.svg)](https://crates.io/crates/{{crate}})
[![Documentation](https://docs.rs/{{crate}}/badge.svg)](https://docs.rs/{{crate}})
[![Rust 1.35+](https://img.shields.io/badge/rust-1.35+-orange.svg)](https://www.rust-lang.org)
![](https://img.shields.io/badge/unsafe-forbidden-brightgreen.svg)

{{readme}}

### Alternatives

It's very hard to compare different libraries, so we are using table-based comparison.
There are roughly three types of TrueType tables:

- A table with a list of properties (like `head`, `OS/2`, etc.).<br/>
  If a library tries to parse it at all then we mark it as supported.
- A table that contains a single type of data (`glyf`, `CFF` (kinda), `hmtx`, etc.).<br/>
  Can only be supported or not.
- A table that contains multiple subtables (`cmap`, `kern`, `GPOS`, etc.).<br/>
  Can be partially supported and we note which subtables are actually supported.

| Feature/Library   | ttf-parser             | FreeType            | stb_truetype                   |
| ----------------- | :--------------------: | :-----------------: | :----------------------------: |
| Memory safe       | ✓                      |                     |                                |
| Thread safe       | ✓                      |                     |                                |
| Zero allocation   | ✓                      |                     |                                |
| `CFF `&nbsp;table | ✓                      | ✓                   | ✓                              |
| `cmap` table      | ~ (no 8; Unicode-only) | ✓                   | ~ (no 2,8,10,14; Unicode-only) |
| `gasp` table      |                        | ✓                   |                                |
| `GDEF` table      | ~                      |                     |                                |
| `glyf` table      | ✓                      | ✓                   | ✓                              |
| `GPOS` table      |                        |                     | ~ (only 2)                     |
| `GSUB` table      |                        |                     |                                |
| `head` table      | ✓                      | ✓                   | ✓                              |
| `hhea` table      | ✓                      | ✓                   | ✓                              |
| `hmtx` table      | ✓                      | ✓                   | ✓                              |
| `kern` table      | ~                      | ~                   | ~                              |
| `maxp` table      | ✓                      | ✓                   | ✓                              |
| `name` table      | ✓                      | ✓                   |                                |
| `OS/2` table      | ✓                      | ✓                   |                                |
| `post` table      | ✓                      | ✓                   |                                |
| `SVG `&nbsp;table |                        |                     | ✓                              |
| `vhea` table      | ✓                      | ✓                   |                                |
| `vmtx` table      | ✓                      | ✓                   |                                |
| `VORG` table      | ✓                      | ✓                   |                                |
| Variable fonts    |                        | ✓                   |                                |
| Rendering         |                        | ✓                   | ~<sup>1</sup>                  |
| Language          | Rust + C API           | C                   | C                              |
| Dynamic lib size  | ~250KiB                | ~760KiB<sup>2</sup> | ? (header-only)                |
| Tested version    | 0.4.0                  | 2.9.1               | 1.24                           |
| License           | MIT / Apache-2.0       | FTL/GPLv2           | public domain                  |

Legend:

- ✓ - supported
- ~ - partial
- *nothing* - not supported

Notes:

1. Very primitive.
2. Depends on build flags.

Other Rust alternatives:

- [stb_truetype](https://crates.io/crates/stb_truetype) - Mainly a glyph outline extractor.
  Isn't allocation free. Has less features. Doesn't support CFF. Has a lot of magic numbers.
  Uses `panic` a lot.
- [font](https://github.com/pdf-rs/font) - Very similar to `ttf-parser`, but supports less features.
  Still an alpha. Isn't allocation free.
- [fontdue](https://github.com/mooman219/fontdue) - Parser and rasterizer. In alpha state.
  Allocates all the required data. Doesn't support CFF.
- [font-rs](https://crates.io/crates/font-rs) - Mainly a glyph outline extractor.
  No documentation. Has less features. Doesn't support CFF. Has a lot of magic numbers.
- [truetype](https://crates.io/crates/truetype) - Simply maps TrueType data to the Rust structures.
  Doesn't actually parses the data. Isn't allocation free. Has some **unsafe**. Unmaintained.

(revised on 2019-09-24)

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
