# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- `sbix`, `CBLC`, `CBDT` and `SVG` tables support.
- `Font::glyph_image`.

### Removed
- Logging support.

### Fixed
- (`gvar`) Integer overflow.
- (`cmap`) Integer overflow during subtable format 2 parsing.
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
- Initial GSUB/GPOS support.

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

[Unreleased]: https://github.com/RazrFalcon/ttf-parser/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.1.0...v0.2.0
