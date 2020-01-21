// https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx

use crate::parser::{Stream, LazyArray};
use crate::{Font, GlyphId};
use crate::raw::hmtx as raw;

impl<'a> Font<'a> {
    /// Parses glyph's horizontal advance using
    /// [Horizontal Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx).
    pub fn glyph_hor_advance(&self, glyph_id: GlyphId) -> Option<u16> {
        self.check_glyph_id_opt(glyph_id)?;
        let mut s = Stream::new(self.hmtx?);

        let number_of_hmetrics = self.number_of_hmetrics();
        if number_of_hmetrics == 0 {
            return None;
        }

        let array: LazyArray<raw::HorizontalMetrics> = s.read_array(number_of_hmetrics).ok()?;
        if let Some(metrics) = array.get(glyph_id.0) {
            Some(metrics.advance_width())
        } else {
            // 'As an optimization, the number of records can be less than the number of glyphs,
            // in which case the advance width value of the last record applies
            // to all remaining glyph IDs.'
            array.last().map(|m| m.advance_width())
        }
    }

    /// Parses glyph's horizontal side bearing using
    /// [Horizontal Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx).
    pub fn glyph_hor_side_bearing(&self, glyph_id: GlyphId) -> Option<i16> {
        self.check_glyph_id_opt(glyph_id)?;
        let mut s = Stream::new(self.hmtx?);

        let number_of_hmetrics = self.number_of_hmetrics();
        if number_of_hmetrics == 0 {
            return None;
        }

        let array: LazyArray<raw::HorizontalMetrics> = s.read_array(number_of_hmetrics).ok()?;
        if let Some(metrics) = array.get(glyph_id.0) {
            Some(metrics.lsb())
        } else {
            // 'If the number_of_hmetrics is less than the total number of glyphs,
            // then that array is followed by an array for the left side bearing values
            // of the remaining glyphs.'

            // Check for overflow.
            if self.number_of_glyphs() < number_of_hmetrics {
                return None;
            }

            let count = self.number_of_glyphs() - number_of_hmetrics;
            let left_side_bearings: LazyArray<i16> = s.read_array(count).ok()?;
            left_side_bearings.get(glyph_id.0)
        }
    }
}
