// https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx

use crate::parser::{Stream, LazyArray};
use crate::{Font, GlyphId};
use crate::raw::hmtx as raw;

impl<'a> Font<'a> {
    /// Parses glyph's vertical advance using
    /// [Vertical Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx).
    pub fn glyph_ver_advance(&self, glyph_id: GlyphId) -> Option<u16> {
        self.check_glyph_id_opt(glyph_id)?;
        let mut s = Stream::new(self.vmtx?);

        let number_of_vmetrics = self.number_of_vmetrics()?;
        if number_of_vmetrics == 0 {
            return None;
        }

        let array: LazyArray<raw::HorizontalMetrics> = s.read_array(number_of_vmetrics).ok()?;
        if let Some(metrics) = array.get(glyph_id.0) {
            Some(metrics.advance_width())
        } else {
            array.last().map(|m| m.advance_width())
        }
    }

    /// Parses glyph's vertical side bearing using
    /// [Vertical Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx).
    pub fn glyph_ver_side_bearing(&self, glyph_id: GlyphId) -> Option<i16> {
        self.check_glyph_id_opt(glyph_id)?;
        let mut s = Stream::new(self.vmtx?);

        let number_of_vmetrics = self.number_of_vmetrics()?;
        if number_of_vmetrics == 0 {
            return None;
        }

        let array: LazyArray<raw::HorizontalMetrics> = s.read_array(number_of_vmetrics).ok()?;
        if let Some(metrics) = array.get(glyph_id.0) {
            Some(metrics.lsb())
        } else {
            // 'The number of entries in this array is calculated by subtracting the value of
            // numOfLongVerMetrics from the number of glyphs in the font.'

            // Check for overflow first.
            if self.number_of_glyphs() < number_of_vmetrics {
                return None;
            }

            let tsb_array_len = self.number_of_glyphs() - number_of_vmetrics;

            // 'This array contains the top sidebearings of glyphs not represented in
            // the first array, and all the glyphs in this array must have the same advance
            // height as the last entry in the vMetrics array.'
            let array: LazyArray<i16> = s.read_array(tsb_array_len).ok()?;
            array.get(glyph_id.0 - number_of_vmetrics)
        }
    }
}

