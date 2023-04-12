//! An [Embedded Bitmap Data Table](
//! https://docs.microsoft.com/en-us/typography/opentype/spec/ebdt) implementation.
use super::{cbdt, eblc};
use crate::{GlyphId, RasterGlyphImage};

// CBDT is defined as a backward compatible extension to EBDT, so any valid EBDT is also a valid
// CBDT. Thus, we can just re-use the CBDT table parsing code for EBDT.

/// An [Embedded Bitmap Data Table](
/// https://docs.microsoft.com/en-us/typography/opentype/spec/ebdt).
#[derive(Clone, Copy)]
pub struct Table<'a>(cbdt::Table<'a>);

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(locations: eblc::Table<'a>, data: &'a [u8]) -> Option<Self> {
        cbdt::Table::parse(locations.0, data).map(Self)
    }

    /// Returns a raster image for the glyph.
    pub fn get(&self, glyph_id: GlyphId, pixels_per_em: u16) -> Option<RasterGlyphImage<'a>> {
        self.0.get(glyph_id, pixels_per_em)
    }
}

impl core::fmt::Debug for Table<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}