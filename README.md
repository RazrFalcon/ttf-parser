## ttf-parser
[![Build Status](https://travis-ci.org/RazrFalcon/ttf-parser.svg?branch=master)](https://travis-ci.org/RazrFalcon/ttf-parser)
[![Crates.io](https://img.shields.io/crates/v/ttf-parser.svg)](https://crates.io/crates/ttf-parser)
[![Documentation](https://docs.rs/ttf-parser/badge.svg)](https://docs.rs/ttf-parser)
[![Rust 1.35+](https://img.shields.io/badge/rust-1.35+-orange.svg)](https://www.rust-lang.org)

A high-level, safe, zero-allocation TrueType font parser.

### Features

- A high-level API.
- Zero allocations.
- Zero dependencies.
- `no_std` compatible.
- Fast.
- Stateless.
- Simple and maintainable code (no magic numbers).

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
- (`post`) Retrieving font's underline metrics name using [underline_metrics()] method.
- (`post`) Retrieving glyph's name using [glyph_name()] method.
- (`head`) Retrieving font's units per EM value using [units_per_em()] method.
- (`hhea`) Retrieving generic font info using: [ascender()], [descender()], [height()]
  and [line_gap()] methods.

[glyph_index()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_index
[glyph_variation_index()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_variation_index
[outline_glyph()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.outline_glyph
[glyph_hor_advance()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_hor_advance
[glyph_hor_side_bearing()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_hor_side_bearing
[glyph_ver_advance()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_ver_advance
[glyph_ver_side_bearing()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_ver_side_bearing
[glyphs_kerning()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyphs_kerning
[number_of_glyphs()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.number_of_glyphs
[names()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.names
[family_name()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.family_name
[post_script_name()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.post_script_name
[underline_metrics()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.underline_metrics
[glyph_name()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_name
[units_per_em()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.units_per_em
[ascender()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.ascender
[descender()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.descender
[height()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.height
[line_gap()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.line_gap

### Supported OpenType features

- (`CFF `) Glyph outlining using [outline_glyph()] method.
- (`CFF2`) Glyph outlining using [outline_glyph()] method.
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
- (`avar`) Variation coordinates normalization using [map_variation_coordinates()] method.
- (`fvar`) Variation axis parsing using [variation_axis()] method.
- (`VORG`) Retrieving glyph's vertical origin using [glyph_y_origin()] method.
- (`MVAR`) Retrieving font's metrics variation using [metrics_variation()] method.
- (`HVAR`) Retrieving glyph's variation offset for horizontal advance using [glyph_hor_advance_variation()] method.
- (`HVAR`) Retrieving glyph's variation offset for horizontal side bearing using [glyph_hor_side_bearing_variation()] method.
- (`VVAR`) Retrieving glyph's variation offset for vertical advance using [glyph_ver_advance_variation()] method.
- (`VVAR`) Retrieving glyph's variation offset for vertical side bearing using [glyph_ver_side_bearing_variation()] method.

[is_regular()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_regular
[is_italic()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_italic
[is_bold()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_bold
[is_oblique()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_oblique
[weight()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.weight
[width()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.width
[x_height()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.x_height
[strikeout_metrics()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.strikeout_metrics
[subscript_metrics()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.subscript_metrics
[superscript_metrics()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.superscript_metrics
[glyph_class()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_class
[glyph_mark_attachment_class()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_mark_attachment_class
[is_mark_glyph()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_mark_glyph
[map_variation_coordinates()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.map_variation_coordinates
[variation_axis()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.variation_axis
[glyph_y_origin()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_y_origin
[metrics_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.metrics_variation
[glyph_hor_advance_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_hor_advance_variation
[glyph_hor_side_bearing_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_hor_side_bearing_variation
[glyph_ver_advance_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_ver_advance_variation
[glyph_ver_side_bearing_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_ver_side_bearing_variation

### Error handling

The library uses `Result<Option<T>, Error>` pattern, where `Error` indicates a parsing error
and `Ok(None)` a not set value.
This is a bit verbose, but allows us to separate malformed files and not set values.
For example, if a font doesn't have a glyph for a specified character - it's not an error.
And error will be emitted only in two cases: on a malformed file or bug in implementation.

### Methods' computational complexity

TrueType fonts designed for fast querying, so most of the methods are very fast.
The main exception is glyph outlining. Glyphs can be stored using two different methods:
using [Glyph Data](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) format
and [Compact Font Format](http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5176.CFF.pdf) (pdf).
The first one is fairly simple which makes it faster to process.
The second one is basically a tiny language with a stack-based VM, which makes it way harder to process.

```
test outline_cff  ... bench:   1,010,120 ns/iter (+/- 11,517)
test outline_cff2 ... bench:   1,385,488 ns/iter (+/- 21,411)
test outline_glyf ... bench:     717,052 ns/iter (+/- 5,907)
```

Here is some methods benchmarks:

```
test outline_glyph_276_from_cff  ... bench:   745.0 ns/iter (+/- 31)
test from_data_otf_cff2          ... bench:   673.0 ns/iter (+/- 9)
test outline_glyph_276_from_cff2 ... bench:   595.0 ns/iter (+/- 24)
test outline_glyph_276_from_glyf ... bench:   564.0 ns/iter (+/- 6)
test from_data_otf_cff           ... bench:   485.0 ns/iter (+/- 11)
test outline_glyph_8_from_cff2   ... bench:   371.0 ns/iter (+/- 54)
test outline_glyph_8_from_glyf   ... bench:   249.0 ns/iter (+/- 2)
test outline_glyph_8_from_cff    ... bench:   243.0 ns/iter (+/- 7)
test glyph_name_276              ... bench:   216.0 ns/iter (+/- 0)
test from_data_ttf               ... bench:   200.0 ns/iter (+/- 3)
test family_name                 ... bench:   161.0 ns/iter (+/- 5)
test glyph_index_u41             ... bench:    14.0 ns/iter (+/- 1)
test hor_advance                 ... bench:     3.0 ns/iter (+/- 0)
test hor_side_bearing            ... bench:     3.0 ns/iter (+/- 0)
test glyph_name_8                ... bench:     2.0 ns/iter (+/- 0)
test ascender                    ... bench:     0.6 ns/iter (+/- 0)
test x_height                    ... bench:     0.5 ns/iter (+/- 0)
test underline_metrics           ... bench:     0.5 ns/iter (+/- 0)
test strikeout_metrics           ... bench:     0.5 ns/iter (+/- 0)
test units_per_em                ... bench:     0.5 ns/iter (+/- 0)
test subscript_metrics           ... bench:     0.2 ns/iter (+/- 0)
test width                       ... bench:     0.2 ns/iter (+/- 0)
```

`family_name` is expensive, because it allocates a `String` and the original data
is stored as UTF-16 BE.

`glyph_name_8` is faster that `glyph_name_276`, because for glyph indexes lower than 258
we are using predefined names, so no parsing is involved.

### Safety

- The library must not panic. Any panic considered as a critical bug and should be reported.
- The library has a single `unsafe` block for array casting.

### Alternatives

- [font-rs](https://crates.io/crates/font-rs) - Mainly a glyph outline extractor.
  No documentation. Has less features. Doesn't support CFF. Has a lot of magic numbers.
- [stb_truetype](https://crates.io/crates/stb_truetype) - Mainly a glyph outline extractor.
  Isn't allocation free. Has less features. Doesn't support CFF. Has a lot of magic numbers.
  Uses `panic` a lot.
- [truetype](https://crates.io/crates/truetype) - Simply maps TrueType data to the Rust structures.
  Doesn't actually parses the data. Isn't allocation free. Has some **unsafe**. Unmaintained.
- [font](https://github.com/pdf-rs/font) - Very similar to `ttf-parser`, but supports less features.
  Still an alpha. Isn't allocation free.
- [fontdue](https://github.com/mooman219/fontdue) - Parser and rasterizer. In alpha state.
  Allocates all the required data. Doesn't support CFF.

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
