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
- (`kern`) Retrieving a glyphs pair kerning using [glyphs_kerning()] method.
- (`maxp`) Retrieving a total number of glyphs using [number_of_glyphs()] method.
- (`name`) Listing all name records using [names()] method.
- (`name`) Retrieving a font's family name using [family_name()] method.
- (`name`) Retrieving a font's PostScript name using [post_script_name()] method.
- (`post`) Retrieving a font's underline metrics name using [underline_metrics()] method.
- (`head`) Retrieving a font's units per EM value using [units_per_em()] method.
- (`hhea`) Retrieving a generic font info using: [ascender()], [descender()], [height()]
  and [line_gap()] methods.

[glyph_index()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_index
[glyph_variation_index()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_variation_index
[outline_glyph()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.outline_glyph
[glyph_hor_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_hor_metrics
[glyph_ver_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_ver_metrics
[glyphs_kerning()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyphs_kerning
[number_of_glyphs()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.number_of_glyphs
[names()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.names
[family_name()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.family_name
[post_script_name()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.post_script_name
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

### Methods' computational complexity

TrueType fonts designed for fast querying, so most of the methods are very fast.
The main exception is glyph outlining. Glyphs can be stored using two different methods:
using [Glyph Data](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) format
and [Compact Font Format](http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5176.CFF.pdf) (pdf).
The first one is fairly simple which makes it faster to process.
The second one is basically a tiny language with a stack-based VM, which makes it way harder to process.
Currently, it takes 40% more time to outline all glyphs in
*SourceSansPro-Regular.otf* (which uses CFF) rather than in *SourceSansPro-Regular.ttf*.

```
test outline_cff  ... bench:   1,651,557 ns/iter (+/- 2,751)
test outline_glyf ... bench:     977,046 ns/iter (+/- 4,973)
```

Here is some methods benchmarks:

```
test outline_glyph_276_from_cff  ... bench:       1,247 ns/iter (+/- 2)
test outline_glyph_276_from_glyf ... bench:         817 ns/iter (+/- 15)
test outline_glyph_8_from_cff    ... bench:         521 ns/iter (+/- 2)
test family_name                 ... bench:         445 ns/iter (+/- 4)
test from_data_otf               ... bench:         435 ns/iter (+/- 1)
test outline_glyph_8_from_glyf   ... bench:         360 ns/iter (+/- 7)
test from_data_ttf               ... bench:         133 ns/iter (+/- 0)
```

Some methods are too fast, so we execute them **1000 times** to get better measurements.

```
test glyph_index_u41     ... bench:      24,648 ns/iter (+/- 256)
test glyph_2_hor_metrics ... bench:       8,421 ns/iter (+/- 18)
test units_per_em        ... bench:         564 ns/iter (+/- 2)
test x_height            ... bench:         568 ns/iter (+/- 1)
test strikeout_metrics   ... bench:         564 ns/iter (+/- 0)
test width               ... bench:         422 ns/iter (+/- 0)
test ascender            ... bench:         279 ns/iter (+/- 1)
test subscript_metrics   ... bench:         279 ns/iter (+/- 0)
```

`family_name` is expensive, because it allocates a `String` and the original data
is stored as UTF-16 BE.

### Safety

- The library must not panic. Any panic considered as a critical bug and should be reported.
- The library forbids `unsafe` code.

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
