// https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx

use crate::{Font, GlyphId, Result};

impl<'a> Font<'a> {
    /// Parses glyph's vertical advance using
    /// [Vertical Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx).
    pub fn glyph_ver_advance(&self, glyph_id: GlyphId) -> Result<Option<u16>> {
        bail!(self.check_glyph_id(glyph_id));

        let number_of_vmetrics = self.number_of_vmetrics()?;
        if number_of_vmetrics == 0 {
            return Ok(None);
        }

        crate::hmtx::parse_glyph_advance(self.vmtx?, glyph_id, number_of_vmetrics)
    }

    /// Parses glyph's vertical side bearing using
    /// [Vertical Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx).
    pub fn glyph_ver_side_bearing(&self, glyph_id: GlyphId) -> Result<Option<i16>> {
        bail!(self.check_glyph_id(glyph_id));

        let number_of_vmetrics = self.number_of_vmetrics()?;
        if number_of_vmetrics == 0 {
            return Ok(None);
        }

        crate::hmtx::parse_glyph_side_bearing(self.vmtx?, glyph_id, number_of_vmetrics,
                                              self.number_of_glyphs())
    }
}
