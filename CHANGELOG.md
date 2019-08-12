# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
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

[Unreleased]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/RazrFalcon/ttf-parser/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/RazrFalcon/ttf-parser/compare/v0.1.0...v0.2.0
