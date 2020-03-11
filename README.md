## ttf-parser
[![Build Status](https://travis-ci.org/RazrFalcon/ttf-parser.svg?branch=master)](https://travis-ci.org/RazrFalcon/ttf-parser)
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
- Zero required dependencies. Logging is enabled by default.
- `no_std` compatible.
- Fast. Set the *Performance* section.
- Stateless. No mutable methods.
- Simple and maintainable code (no magic numbers).

### Safety

- The library must not panic. Any panic considered as a critical bug and should be reported.
- The library forbids the unsafe code.
- No heap allocations, so crash due to OOM is not possible.
- All recursive methods have a depth limit.
- Technically, should use less than 64KiB of stack in worst case scenario.
- Most of arithmetic operations are checked.

### Supported TrueType features

- (`cmap`) Character to glyph index mapping using [glyph_index()] method.
  <br/>All subtable formats except Mixed Coverage (8) are supported.
- (`cmap`) Character variation to glyph index mapping using [glyph_variation_index()] method.
- (`glyf`) Glyph outlining using [outline_glyph()] method.
- (`hmtx`) Retrieving glyph's horizontal metrics using [glyph_hor_advance()] and [glyph_hor_side_bearing()] methods.
- (`vmtx`) Retrieving glyph's vertical metrics using [glyph_ver_advance()] and [glyph_ver_side_bearing()] methods.
- (`kern`) Retrieving glyphs pair kerning using [glyphs_kerning()] method.
- (`maxp`) Retrieving total number of glyphs using [number_of_glyphs()] method.
- (`name`) Listing all name records using [names()] method.
- (`name`) Retrieving font's family name using [family_name()] method.
- (`name`) Retrieving font's PostScript name using [post_script_name()] method.
- (`post`) Retrieving font's underline metrics using [underline_metrics()] method.
- (`post`) Retrieving glyph's name using [glyph_name()] method.
- (`head`) Retrieving font's units per EM value using [units_per_em()] method.
- (`hhea`) Retrieving generic font info using: [ascender()], [descender()], [height()]
  and [line_gap()] methods.

[glyph_index()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_index
[glyph_variation_index()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_variation_index
[outline_glyph()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.outline_glyph
[glyph_hor_advance()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_hor_advance
[glyph_hor_side_bearing()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_hor_side_bearing
[glyph_ver_advance()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_ver_advance
[glyph_ver_side_bearing()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_ver_side_bearing
[glyphs_kerning()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyphs_kerning
[number_of_glyphs()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.number_of_glyphs
[names()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.names
[family_name()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.family_name
[post_script_name()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.post_script_name
[underline_metrics()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.underline_metrics
[glyph_name()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_name
[units_per_em()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.units_per_em
[ascender()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.ascender
[descender()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.descender
[height()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.height
[line_gap()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.line_gap

### Supported OpenType features

- (`CFF `) Glyph outlining using [outline_glyph()] method.
- (`OS/2`) Retrieving font's kind using [is_regular()], [is_italic()],
  [is_bold()] and [is_oblique()] methods.
- (`OS/2`) Retrieving font's weight using [weight()] method.
- (`OS/2`) Retrieving font's width using [width()] method.
- (`OS/2`) Retrieving font's X height using [x_height()] method.
- (`OS/2`) Retrieving font's strikeout metrics using [strikeout_metrics()] method.
- (`OS/2`) Retrieving font's subscript metrics using [subscript_metrics()] method.
- (`OS/2`) Retrieving font's superscript metrics using [superscript_metrics()] method.
- (`GDEF`) Retrieving glyph's class using [glyph_class()] method.
- (`GDEF`) Retrieving glyph's mark attachment class using [glyph_mark_attachment_class()] method.
- (`GDEF`) Checking that glyph is a mark using [is_mark_glyph()] method.
- (`VORG`) Retrieving glyph's vertical origin using [glyph_y_origin()] method.

[is_regular()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.is_regular
[is_italic()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.is_italic
[is_bold()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.is_bold
[is_oblique()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.is_oblique
[weight()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.weight
[width()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.width
[x_height()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.x_height
[strikeout_metrics()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.strikeout_metrics
[subscript_metrics()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.subscript_metrics
[superscript_metrics()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.superscript_metrics
[glyph_class()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_class
[glyph_mark_attachment_class()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_mark_attachment_class
[is_mark_glyph()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.is_mark_glyph
[glyph_y_origin()]: https://docs.rs/ttf-parser/0.4.0/ttf_parser/struct.Font.html#method.glyph_y_origin

### Error handling

`ttf-parser` is designed to parse well-formed fonts, so it does not have an `Error` enum.
It doesn't mean that it will crash or panic on malformed fonts, only that the
error handling will boil down to `Option::None`. So you will not get a detailed cause of an error.
By doing so we can simplify an API quite a lot since otherwise, we will have to use
`Result<Option<T>, Error>`.

Some methods may print warnings, when the `logging` feature is enabled.

### Methods' computational complexity

TrueType fonts designed for fast querying, so most of the methods are very fast.
The main exception is glyph outlining. Glyphs can be stored using two different methods:
using [Glyph Data](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) format
and [Compact Font Format](http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5176.CFF.pdf) (pdf).
The first one is fairly simple which makes it faster to process.
The second one is basically a tiny language with a stack-based VM, which makes it way harder to process.

The [benchmark](./benches/outline/) tests how long it takes to outline all glyphs in the font.

```
stb_truetype_outline_glyf     695873 ns
ttf_parser_outline_glyf       765007 ns
freetype_outline_glyf        1194395 ns

ttf_parser_outline_cff       1165904 ns
stb_truetype_outline_cff     2862264 ns
freetype_outline_cff         5806994 ns
```

**Note:** FreeType is surprisingly slow, so I'm worried that I've messed something up.

And here are some methods benchmarks:

```
test outline_glyph_276_from_cff  ... bench:         841 ns/iter (+/- 53)
test outline_glyph_276_from_glyf ... bench:         674 ns/iter (+/- 15)
test from_data_otf_cff           ... bench:         403 ns/iter (+/- 3)
test outline_glyph_8_from_cff    ... bench:         339 ns/iter (+/- 44)
test outline_glyph_8_from_glyf   ... bench:         295 ns/iter (+/- 16)
test glyph_name_276              ... bench:         214 ns/iter (+/- 1)
test from_data_ttf               ... bench:         169 ns/iter (+/- 3)
test family_name                 ... bench:         155 ns/iter (+/- 5)
test glyph_index_u41             ... bench:          16 ns/iter (+/- 0)
test glyph_name_8                ... bench:           1 ns/iter (+/- 0)
test underline_metrics           ... bench:         0.5 ns/iter (+/- 0)
test units_per_em                ... bench:         0.5 ns/iter (+/- 0)
test strikeout_metrics           ... bench:         0.5 ns/iter (+/- 0)
test x_height                    ... bench:         0.4 ns/iter (+/- 0)
test ascender                    ... bench:         0.2 ns/iter (+/- 0)
test hor_advance                 ... bench:         0.2 ns/iter (+/- 0)
test hor_side_bearing            ... bench:         0.2 ns/iter (+/- 0)
test subscript_metrics           ... bench:         0.2 ns/iter (+/- 0)
test width                       ... bench:         0.2 ns/iter (+/- 0)
```

`family_name` is expensive, because it allocates a `String` and the original data
is stored as UTF-16 BE.

`glyph_name_8` is faster than `glyph_name_276`, because for glyph indexes lower than 258
we are using predefined names, so no parsing is involved.

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
