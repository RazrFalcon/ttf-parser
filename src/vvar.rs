// https://docs.microsoft.com/en-us/typography/opentype/spec/vvar

use crate::{Font, GlyphId};


impl<'a> Font<'a> {
    /// Parses glyph's variation offset for vertical advance using
    /// [Vertical Metrics Variations Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vvar).
    ///
    /// Note: coordinates should be converted from fixed point 2.14 to i32
    /// by multiplying each coordinate by 16384.
    ///
    /// Number of `coordinates` should be the same as number of variation axes in the font.
    ///
    /// Returns `None` when `VVAR` table is not present or invalid.
    pub fn glyph_ver_advance_variation(
        &self,
        glyph_id: GlyphId,
        coordinates: &[i32],
    ) -> Option<f32> {
        crate::hvar::glyph_advance_variation(self.vvar?, glyph_id, coordinates)
    }

    /// Parses glyph's variation offset for vertical side bearing using
    /// [Vertical Metrics Variations Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vvar).
    ///
    /// Note: coordinates should be converted from fixed point 2.14 to i32
    /// by multiplying each coordinate by 16384.
    ///
    /// Number of `coordinates` should be the same as number of variation axes in the font.
    ///
    /// Returns `None` when `VVAR` table is not present or invalid.
    pub fn glyph_ver_side_bearing_variation(
        &self,
        glyph_id: GlyphId,
        coordinates: &[i32],
    ) -> Option<f32> {
        crate::hvar::glyph_side_bearing_variation(self.vvar?, glyph_id, coordinates)
    }
}
