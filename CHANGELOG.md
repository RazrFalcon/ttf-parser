# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Fixed
- (cmap) Incorrectly returning glyph ID `0` instead of `None` for format 0

## [0.8.1] - 2020-07-29
### Added
- `Face::is_monospaced`
- `Face::italic_angle`
- `Face::typographic_ascender`
- `Face::typographic_descender`
- `Face::typographic_line_gap`
- `Face::captial_height`

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

[Unreleased]: https://github.com/RazrFalcon/ttf-parser/compare/v0.8.1...HEAD
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
