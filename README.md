## ttf-parser
![Build Status](https://github.com/RazrFalcon/ttf-parser/workflows/Rust/badge.svg)
[![Crates.io](https://img.shields.io/crates/v/ttf-parser.svg)](https://crates.io/crates/ttf-parser)
[![Documentation](https://docs.rs/ttf-parser/badge.svg)](https://docs.rs/ttf-parser)
[![Rust 1.35+](https://img.shields.io/badge/rust-1.35+-orange.svg)](https://www.rust-lang.org)
![](https://img.shields.io/badge/unsafe-forbidden-brightgreen.svg)

A high-level, safe, zero-allocation TrueType font parser.

Can be used as Rust and as C library.

### Features

- A high-level API, for people who doesn't know how TrueType works internally.
  Basically, no direct access to font tables.
- A [C API](./c-api).
- Zero heap allocations.
- Zero unsafe.
- Zero dependencies.
- `no_std`/WASM compatible.
- Fast. See the *Performance* section.
- Stateless. No mutable parsing methods.
- Simple and maintainable code (no magic numbers).

### Safety

- The library must not panic. Any panic considered as a critical bug and should be reported.
- The library forbids the unsafe code.
- No heap allocations, so crash due to OOM is not possible.
- All recursive methods have a depth limit.
- Technically, should use less than 64KiB of stack in the worst case scenario.
- Most of arithmetic operations are checked.
- Most of numeric casts are checked.

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
| Thread safe       | ✓                      |                     | ~ (mostly reentrant)           |
| Zero allocation   | ✓                      |                     |                                |
| Variable fonts    | ✓                      | ✓                   |                                |
| Rendering         |                        | ✓                   | ~ (very primitive)             |
| `avar` table      | ✓                      | ✓                   |                                |
| `bdat` table      |                        | ✓                   |                                |
| `bloc` table      |                        | ✓                   |                                |
| `CBDT` table      | ✓                      | ✓                   |                                |
| `CBLC` table      | ✓                      | ✓                   |                                |
| `CFF `&nbsp;table | ✓                      | ✓                   | ~ (no `seac` support)          |
| `CFF2` table      | ✓                      | ✓                   |                                |
| `cmap` table      | ~ (no 8)               | ✓                   | ~ (no 2,8,10,14; Unicode-only) |
| `EBDT` table      |                        | ✓                   |                                |
| `EBLC` table      |                        | ✓                   |                                |
| `fvar` table      | ✓                      | ✓                   |                                |
| `gasp` table      |                        | ✓                   |                                |
| `GDEF` table      | ~                      |                     |                                |
| `glyf` table      | ~<sup>1</sup>          | ✓                   | ~<sup>1</sup>                  |
| `GPOS` table      |                        |                     | ~ (only 2)                     |
| `GSUB` table      |                        |                     |                                |
| `gvar` table      | ✓                      | ✓                   |                                |
| `head` table      | ✓                      | ✓                   | ✓                              |
| `hhea` table      | ✓                      | ✓                   | ✓                              |
| `hmtx` table      | ✓                      | ✓                   | ✓                              |
| `HVAR` table      | ✓                      | ✓                   |                                |
| `kern` table      | ✓                      | ~ (only 0)          | ~ (only 0)                     |
| `maxp` table      | ✓                      | ✓                   | ✓                              |
| `MVAR` table      | ✓                      | ✓                   |                                |
| `name` table      | ✓                      | ✓                   |                                |
| `OS/2` table      | ✓                      | ✓                   |                                |
| `post` table      | ✓                      | ✓                   |                                |
| `sbix` table      | ~ (PNG only)           | ~ (PNG only)        |                                |
| `SVG `&nbsp;table | ✓                      |                     | ✓                              |
| `vhea` table      | ✓                      | ✓                   |                                |
| `vmtx` table      | ✓                      | ✓                   |                                |
| `VORG` table      | ✓                      | ✓                   |                                |
| `VVAR` table      | ✓                      | ✓                   |                                |
| Language          | Rust + C API           | C                   | C                              |
| Dynamic lib size  | <300KiB<sup>2</sup>    | ~760KiB<sup>3</sup> | ? (header-only)                |
| Tested version    | 0.8.0                  | 2.9.1               | 1.24                           |
| License           | MIT / Apache-2.0       | FTL / GPLv2         | public domain                  |

Legend:

- ✓ - supported
- ~ - partial
- *nothing* - not supported

Notes:

1. Matching points are not supported.
2. When using from Rust, the library binary overhead depends on used methods
   and can vary from 10KiB up to 100KiB.<br/>
   When using from C, we have to include the Rust's std too, which blows up the size.
3. Depends on build flags.

### Performance

TrueType fonts designed for fast querying, so most of the methods are very fast.
The main exception is glyph outlining. Glyphs can be stored using two different methods:
using [Glyph Data](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) format
and [Compact Font Format](http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5176.CFF.pdf) (pdf).
The first one is fairly simple which makes it faster to process.
The second one is basically a tiny language with a stack-based VM, which makes it way harder to process.

The [benchmark](./benches/outline/) tests how long it takes to outline all glyphs in the font.

| Table/Library | ttf-parser     | FreeType   | stb_truetype   |
| ------------- | -------------: | ---------: | -------------: |
| `glyf`        |   `0.835 ms`   | `1.194 ms` | **`0.695 ms`** |
| `gvar`        | **`3.158 ms`** | `3.594 ms` |              - |
| `CFF`         | **`1.114 ms`** | `5.946 ms` |   `2.862 ms`   |
| `CFF2`        | **`1.763 ms`** | `7.001 ms` |              - |

**Note:** FreeType is surprisingly slow, so I'm worried that I've messed something up.

And here are some methods benchmarks:

```text
test outline_glyph_276_from_cff2 ... bench:         778 ns/iter (+/- 15)
test from_data_otf_cff           ... bench:         760 ns/iter (+/- 13)
test from_data_otf_cff2          ... bench:         709 ns/iter (+/- 25)
test outline_glyph_276_from_cff  ... bench:         678 ns/iter (+/- 41)
test outline_glyph_276_from_glyf ... bench:         649 ns/iter (+/- 11)
test outline_glyph_8_from_cff2   ... bench:         476 ns/iter (+/- 14)
test from_data_ttf               ... bench:         352 ns/iter (+/- 11)
test glyph_name_post_276         ... bench:         223 ns/iter (+/- 5)
test outline_glyph_8_from_cff    ... bench:         261 ns/iter (+/- 13)
test outline_glyph_8_from_glyf   ... bench:         281 ns/iter (+/- 5)
test family_name                 ... bench:         183 ns/iter (+/- 102)
test glyph_name_cff_276          ... bench:         109 ns/iter (+/- 1)
test glyph_index_u41             ... bench:          16 ns/iter (+/- 0)
test glyph_name_cff_8            ... bench:           7 ns/iter (+/- 0)
test glyph_name_post_8           ... bench:           2 ns/iter (+/- 0)
test subscript_metrics           ... bench:           2 ns/iter (+/- 0)
test glyph_hor_advance           ... bench:           2 ns/iter (+/- 0)
test glyph_hor_side_bearing      ... bench:           2 ns/iter (+/- 0)
test glyph_name_8                ... bench:           1 ns/iter (+/- 0)
test ascender                    ... bench:           1 ns/iter (+/- 0)
test underline_metrics           ... bench:           1 ns/iter (+/- 0)
test strikeout_metrics           ... bench:           1 ns/iter (+/- 0)
test x_height                    ... bench:           1 ns/iter (+/- 0)
test units_per_em                ... bench:         0.5 ns/iter (+/- 0)
test width                       ... bench:         0.2 ns/iter (+/- 0)
```

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
