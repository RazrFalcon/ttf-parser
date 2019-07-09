// https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx

use crate::parser::{Stream, LazyArray};
use crate::{Font, GlyphId, VerticalMetrics, TableName, Result, Error};

impl<'a> Font<'a> {
    /// Returns glyph's vertical metrics.
    pub fn glyph_ver_metrics(&self, glyph_id: GlyphId) -> Result<VerticalMetrics> {
        self.check_glyph_id(glyph_id)?;
        let data = self.vmtx.ok_or_else(|| Error::TableMissing(TableName::VerticalMetrics))?;
        let mut s = Stream::new(data);

        let number_of_vmetrics = self.number_of_vmetrics()
            .ok_or_else(|| Error::NoHorizontalMetrics)?;

        if number_of_vmetrics == 0 {
            return Err(Error::NoHorizontalMetrics);
        }

        let glyph_id = glyph_id.0;

        let array: LazyArray<VerticalMetrics> = s.read_array(number_of_vmetrics)?;

        if let Some(metrics) = array.get(glyph_id) {
            Ok(metrics)
        } else {
            let advance = array.last().ok_or_else(|| Error::NoVerticalMetrics)?.advance;

            // 'The number of entries in this array is calculated by subtracting the value of
            // numOfLongVerMetrics from the number of glyphs in the font.'

            // Check for overflow first.
            if self.number_of_glyphs() < number_of_vmetrics {
                return Err(Error::NoVerticalMetrics);
            }

            let tsb_array_len = self.number_of_glyphs() - number_of_vmetrics;

            // 'This array contains the top sidebearings of glyphs not represented in
            // the first array, and all the glyphs in this array must have the same advance
            // height as the last entry in the vMetrics array.'
            let array: LazyArray<i16> = s.read_array(tsb_array_len)?;
            let top_side_bearing = array
                .get(glyph_id - number_of_vmetrics)
                .ok_or_else(|| Error::NoVerticalMetrics)?;

            Ok(VerticalMetrics {
                advance,
                top_side_bearing,
            })
        }
    }
}

