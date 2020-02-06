// https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx

use crate::{Font, GlyphId};

impl<'a> Font<'a> {
    /// Parses glyph's vertical advance using
    /// [Vertical Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx).
    #[inline]
    pub fn glyph_ver_advance(&self, glyph_id: GlyphId) -> Option<u16> {
        self.vmtx.and_then(|vmtx| vmtx.advance(glyph_id))
    }

    /// Parses glyph's vertical side bearing using
    /// [Vertical Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx).
    #[inline]
    pub fn glyph_ver_side_bearing(&self, glyph_id: GlyphId) -> Option<i16> {
        self.vmtx.and_then(|vmtx| vmtx.side_bearing(glyph_id))
    }
}
