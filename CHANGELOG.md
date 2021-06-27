# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]

## [0.12.3] - 2021-06-27
### Changed
- (`glyf`) Always use a calculated bbox.

## [0.12.2] - 2021-06-11
### Fixed
- `Face::glyph_bounding_box` for variable `glyf`.
- (`glyf`) Do not skip glyphs with zero-sized bbox.

## [0.12.1] - 2021-05-24
### Added
- Support Format 13 subtables in `cmap::Subtable::is_unicode`.
  Thanks to [csmulhern](https://github.com/csmulhern)
- Derive more traits by default. Thanks to [dhardy](https://github.com/dhardy)

## [0.12.0] - 2021-02-14
### Changed
- `Face::ascender` and `Face::descender` will use
  [usWinAscent](https://docs.microsoft.com/en-us/typography/opentype/spec/os2#uswinascent) and
  [usWinDescent](https://docs.microsoft.com/en-us/typography/opentype/spec/os2#uswindescent)
  when `USE_TYPO_METRICS` flag is not set in `OS/2` table.
  Previously, those values were ignored and
  [hhea::ascender](https://docs.microsoft.com/en-us/typography/opentype/spec/hhea#ascender) and
  [hhea::descender](https://docs.microsoft.com/en-us/typography/opentype/spec/hhea#descender)
  were used. Now `hhea` table values will be used only when `OS/2` table is not present.
- `Face::outline_glyph` and `Face::glyph_bounding_box` in case of a `glyf` table
  can fallback to a calculated bbox when the embedded bbox is malformed now.

## [0.11.0] - 2021-02-04
### Added
- `FaceTables`, which allowed to load `Face` not only from a single chunk of data,
  but also in a per-table way. Which is useful for WOFF parsing.
  No changes to the API.
  Thanks to [fschutt](https://github.com/fschutt)

## [0.10.1] - 2021-01-21
### Changed
- Update a font used for tests.

## [0.10.0] - 2021-01-16
### Added
- `variable-fonts` build feature. Enabled by default.
  By disabling it you can reduce `ttf-parser` binary size overhead almost twice.

### Changed
- (`gvar`) Increase the maximum number of variation tuples from 16 to 32.
  Increases stack usage and makes `gvar` parsing 10% slower now.

### Fixed
- (`CFF`) Fix `seac` processing. Thanks to [wezm](https://github.com/wezm)

## [0.9.0] - 2020-12-05
### Removed
- `kern` AAT subtable 1 aka `kern::state_machine`.
  Mainly because it's useless without a proper shaping.

## [0.8.3] - 2020-11-15
### Added
- `Face::glyph_variation_delta`

### Fixed
- `Iterator::nth` implementation for `cmap::Subtables` and `Names`.

## [0.8.2] - 2020-07-31
### Added
- `cmap::Subtable::codepoints`

### Fixed
- (cmap) Incorrectly returning glyph ID `0` instead of `None` for format 0
- (cmap) Possible invalid glyph mapping for format 2

## [0.8.1] - 2020-07-29
### Added
- `Face::is_monospaced`
- `Face::italic_angle`
- `Face::typographic_ascender`
- `Face::typographic_descender`
- `Face::typographic_line_gap`
- `Face::capital_height`

## [0.8.0] - 2020-07-21
### Added
- Allow `true` magic.
- `FaceParsingError`
- `NormalizedCoordinate`
- `Face::variation_coordinates`
- `Face::has_non_default_variation_coordinates`
- `Face::glyph_name` can lookup CFF names too.
- `Face::table_data`
- `Face::character_mapping_subtables`

### Changed
- (CFF,CFF2) 10% faster parsing.
- `Face::from_slice` returns `Result` now.
- `Name::platform_id` returns `PlatformId` instead of `Option<PlatformId>` now.
- The `cmap` module became public.

### Fixed
- `Face::width` parsing.
- Possible u32 overflow on 32-bit platforms during `Face::from_slice`.
- (cmap) `Face::glyph_variation_index` processing when the encoding table has only one glyph.

## [0.7.0] - 2020-07-16
### Added
- (CFF) CID fonts support.
- (CFF) `seac` support.
- `Font::global_bounding_box`

### Changed
- Rename `Font` to `Face`, because this is what it actually is.
- Rename `Font::from_data` to `Font::from_slice` to match serde and other libraries.
- Rename `Name::name_utf8` to `Name::to_string`.

### Removed
- `Font::family_name` and `Font::post_script_name`. They were a bit confusing.
  Prefer:
  ```
  face.names().find(|name| name.name_id() == name_id::FULL_NAME).and_then(|name| name.to_string())
  ```

## [0.6.2] - 2020-07-02
### Added
- `Name::is_unicode`
- `Font::family_name` will load names with Windows Symbol encoding now.

### Fixed
- `Font::glyph_bounding_box` will apply variation in case of `gvar` fonts.

## [0.6.1] - 2020-05-19
### Fixed
- (`kern`) Support fonts that ignore the subtable size limit.

## [0.6.0] - 2020-05-18
### Added
- `sbix`, `CBLC`, `CBDT` and `SVG` tables support.
- `Font::glyph_raster_image` and `Font::glyph_svg_image`.
- `Font::kerning_subtables` with subtable formats 0..3 support.

### Changed
- (c-api) The library doesn't allocate `ttfp_font` anymore. All allocations should be
  handled by the caller from now.

### Removed
- `Font::glyphs_kerning`. Use `Font::kerning_subtables` instead.
- (c-api) `ttfp_create_font` and `ttfp_destroy_font`.
  Use `ttfp_font_size_of` + `ttfp_font_init` instead.
  ```c
  ttfp_font *font = (ttfp_font*)alloca(ttfp_font_size_of());
  ttfp_font_init(font_data, font_data_size, 0, font);
  ```
- Logging support. We haven't used it anyway.

### Fixed
- (`gvar`) Integer overflow.
- (`cmap`) Integer overflow during subtable format 2 parsing.
- (`CFF`, `CFF2`) DICT number parsing.
- `Font::glyph_*_advance` will return `None` when glyph ID
  is larger than the number of metrics in the table.
- Ignore variation offset in `Font::glyph_*_advance` and `Font::glyph_*_side_bearing`
  when `HVAR`/`VVAR` tables are missing.
  Previously returned `None` which is incorrect.

## [0.5.0] - 2020-03-19
### Added
- Variable fonts support.
- C API.
- `gvar`, `CFF2`, `avar`, `fvar`, `HVAR`, `VVAR` and `MVAR` tables support.
- `Font::variation_axes`
- `Font::set_variation`
- `Font::is_variable`
- `Tag` type.

### Fixed
- Multiple issues due to arithmetic overflow.

## [0.4.0] - 2020-02-24

**A major rewrite.**

### Added
- `Font::glyph_bounding_box`
- `Font::glyph_name`
- `Font::has_glyph_classes`
- `Font::glyph_class`
- `Font::glyph_mark_attachment_class`
- `Font::is_mark_glyph`
- `Font::glyph_y_origin`
- `Font::vertical_ascender`
- `Font::vertical_descender`
- `Font::vertical_height`
- `Font::vertical_line_gap`
- Optional `log` dependency.

### Changed
- `Font::outline_glyph` now accepts `&mut dyn OutlineBuilder` and not `&mut impl OutlineBuilder`.
- `Font::ascender`, `Font::descender` and `Font::line_gap` will check `USE_TYPO_METRICS`
  flag in OS/2 table now.
- `glyph_hor_metrics` was split into `glyph_hor_advance` and `glyph_hor_side_bearing`.
- `glyph_ver_metrics` was split into `glyph_ver_advance` and `glyph_ver_side_bearing`.
- `CFFError` is no longer public.

### Removed
- `Error` enum. All methods will return `Option<T>` now.
- All `unsafe`.

### Fixed
- `glyph_hor_side_bearing` parsing when the number of metrics is less than the total number of glyphs.
- Multiple CFF parsing fixes. The parser is more strict now.

## [0.3.0] - 2019-09-26
### Added
- `no_std` compatibility.

### Changed
- The library has one `unsafe` block now.
- 35% faster `family_name()` method.
- 25% faster `from_data()` method for TrueType fonts.
- The `Name` struct has a new API. Public fields became public functions
  and data is parsed on demand and not beforehand.

## [0.2.2] - 2019-08-12
### Fixed
- Allow format 12 subtables with *Unicode full repertoire* in `cmap`.

## [0.2.1] - 2019-08-12
### Fixed
- Check that `cmap` subtable encoding is Unicode.

## [0.2.0] - 2019-07-10
### Added
- CFF support.
- Basic kerning support.
- All `cmap` subtable formats except Mixed Coverage (8) are supported.
- Vertical metrics querying from the `vmtx` table.
- OpenType fonts are allowed now.

### Changed
- A major rewrite. TrueType tables are no longer public.
- Use `GlyphId` instead of `u16`.

### Removed
- `GDEF` table parsing.

[Unreleased]: https://github.com/RazrFalcon/ttf-parser/compare/v0.12.3...HEAD
[0.12.3]: https://github.com/RazrFalcon/ttf-parser/compare/v0.12.2...v0.12.3
[0.12.2]: https://github.com/RazrFalcon/ttf-parser/compare/v0.12.1...v0.12.2
[0.12.1]: https://github.com/RazrFalcon/ttf-parser/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.11.0...v0.12.0
[0.11.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.10.1...v0.11.0
[0.10.1]: https://github.com/RazrFalcon/ttf-parser/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.8.3...v0.9.0
[0.8.3]: https://github.com/RazrFalcon/ttf-parser/compare/v0.8.2...v0.8.3
[0.8.2]: https://github.com/RazrFalcon/ttf-parser/compare/v0.8.1...v0.8.2
[0.8.1]: https://github.com/RazrFalcon/ttf-parser/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.6.2...v0.7.0
[0.6.2]: https://github.com/RazrFalcon/ttf-parser/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/RazrFalcon/ttf-parser/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.1.0...v0.2.0
