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
- Stateless.
- Simple and maintainable code (no magic numbers).

### Supported TrueType features

- (`cmap`) Character to glyph index mapping using [glyph_index()] method.
  <br/>All subtable formats except Mixed Coverage (8) are supported.
- (`cmap`) Character variation to glyph index mapping using [glyph_variation_index()] method.
- (`glyf`) Glyph outlining using [outline_glyph()] method.
- (`hmtx`) Retrieving a glyph's horizontal metrics using [glyph_hor_metrics()] method.
- (`vmtx`) Retrieving a glyph's vertical metrics using [glyph_ver_metrics()] method.
- (`maxp`) Retrieving a total number of glyphs using [number_of_glyphs()] method.
- (`name`) Listing all name records using [names()] method.
- (`name`) Retrieving a font's family name using [family_name()] method.
- (`name`) Retrieving a font's PostScript name using [post_stript_name()] method.
- (`post`) Retrieving a font's underline metrics name using [underline_metrics()] method.
- (`head`) Retrieving a font's units per EM value using [units_per_em()] method.
- (`hhea`) Retrieving a generic font info using: [ascender()], [descender()], [height()]
  and [line_gap()] methods.

[glyph_index()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_index
[glyph_variation_index()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_variation_index
[outline_glyph()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.outline_glyph
[glyph_hor_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_hor_metrics
[glyph_ver_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_ver_metrics
[number_of_glyphs()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.number_of_glyphs
[names()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.names
[family_name()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.family_name
[post_stript_name()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.post_stript_name
[underline_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.underline_metrics
[units_per_em()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.units_per_em
[ascender()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.ascender
[descender()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.descender
[height()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.height
[line_gap()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.line_gap

### Supported OpenType features

- (`CFF `) Glyph outlining using [outline_glyph()] method.
- (`OS/2`) Retrieving a font kind using [is_regular()], [is_italic()],
  [is_bold()] and [is_oblique()] methods.
- (`OS/2`) Retrieving a font's weight using [weight()] method.
- (`OS/2`) Retrieving a font's width using [width()] method.
- (`OS/2`) Retrieving a font's X height using [x_height()] method.
- (`OS/2`) Retrieving a font's strikeout metrics using [strikeout_metrics()] method.
- (`OS/2`) Retrieving a font's subscript metrics using [subscript_metrics()] method.
- (`OS/2`) Retrieving a font's superscript metrics using [superscript_metrics()] method.
- (`GDEF`) Retrieving a glyph's class using [glyph_class()] method.
- (`GDEF`) Retrieving a glyph's mark attachment class using [glyph_mark_attachment_class()] method.

[is_regular()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.is_regular
[is_italic()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.is_italic
[is_bold()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.is_bold
[is_oblique()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.is_oblique
[weight()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.weight
[width()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.width
[x_height()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.x_height
[strikeout_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.strikeout_metrics
[subscript_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.subscript_metrics
[superscript_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.superscript_metrics
[glyph_class()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_class
[glyph_mark_attachment_class()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_mark_attachment_class

### Safety

- The library relies heavily on Rust's bounds checking and assumes that font is well-formed.
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
