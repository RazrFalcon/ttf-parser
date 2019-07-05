# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased]
### Added
- CFF support.
- Basic kerning support.
- High Byte Mapping Through Table (2) `cmap` subtable support.
- Unicode Variation Sequences (14) `cmap` subtable support.
- Vertical metrics querying from the `vmtx` table.
- OpenType fonts are allowed now.

### Changed
- A major rewrite. TrueType tables are no longer public.
- Use `GlyphId` instead of `u16`.

### Removed
- `GDEF` table parsing.

### Fixed
- Panic during a glyph outlining when contour has only one point.

[Unreleased]: https://github.com/RazrFalcon/ttf-parser/compare/v0.1.0...HEAD
