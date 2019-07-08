// https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx

use crate::parser::LazyArray;
use crate::{Font, TableName, GlyphId, HorizontalMetrics, Result, Error};


impl<'a> Font<'a> {
    /// Returns glyph's horizontal metrics.
    pub fn glyph_hor_metrics(&self, glyph_id: GlyphId) -> Result<HorizontalMetrics> {
        self.check_glyph_id(glyph_id)?;
        let mut s = self.table_stream(TableName::HorizontalMetrics)?;

        let number_of_hmetrics = self.number_of_hmetrics()?;
        if number_of_hmetrics == 0 {
            return Err(Error::NoHorizontalMetrics);
        }

        let glyph_id = glyph_id.0;

        let array: LazyArray<HorizontalMetrics> = s.read_array(number_of_hmetrics)?;

        if let Some(metrics) = array.get(glyph_id) {
            Ok(metrics)
        } else {
            // 'If the number_of_hmetrics is less than the total number of glyphs,
            // then that array is followed by an array for the left side bearing values
            // of the remaining glyphs.'

            // Check for overflow.
            if self.number_of_glyphs() < number_of_hmetrics {
                return Err(Error::NoHorizontalMetrics);
            }

            let count = self.number_of_glyphs() - number_of_hmetrics;
            let left_side_bearings: LazyArray<i16> = s.read_array(count)?;
            let left_side_bearing = left_side_bearings.get(glyph_id)
                .ok_or_else(|| Error::NoHorizontalMetrics)?;

            // 'As an optimization, the number of records can be less than the number of glyphs,
            // in which case the advance width value of the last record applies
            // to all remaining glyph IDs.'
            let last_metric = array.last().ok_or_else(|| Error::NoHorizontalMetrics)?;

            Ok(HorizontalMetrics {
                advance: last_metric.advance,
                left_side_bearing,
            })
        }
    }
}
