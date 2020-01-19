# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- Basic CFF2 support.
- `Font::glyph_bounding_box`
- `Font::glyph_name`
- `Font::has_glyph_classes`
- `Font::glyph_class`
- `Font::glyph_mark_attachment_class`
- `Font::is_mark_glyph`
- `Font::variation_axes_count`
- `Font::variation_axis`
- `Font::map_variation_coordinates`
- `Tag` type.

### Changed
- `Font::outline_glyph` not accepts `&mut dyn OutlineBuilder` and not `&mut impl OutlineBuilder`.

### Removed
- `Error::InvalidGlyphClass`, because unused.

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

[Unreleased]: https://github.com/RazrFalcon/ttf-parser/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.2...v0.3.0
[0.2.2]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.1.0...v0.2.0
