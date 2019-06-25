use crate::parser::{Stream, LazyArray};
use crate::{Font, GlyphId, VerticalMetrics, TableName, Result, Error};


impl<'a> Font<'a> {
    /// Returns glyph's vertical metrics.
    pub fn glyph_ver_metrics(&self, glyph_id: GlyphId) -> Result<VerticalMetrics> {
        let data = self.table_data(TableName::VerticalMetrics)?;
        self.check_glyph_id(glyph_id)?;

        let number_of_vmetrics = self.number_of_vmetrics()?;

        let mut s = Stream::new(data);
        let vmetrics_array = s.read_array::<VerticalMetrics>(number_of_vmetrics as usize);

        if glyph_id.0 < number_of_vmetrics {
            vmetrics_array.get(glyph_id.0 as usize).ok_or(Error::NotATrueType)
        } else {
            // 'The number of entries in this array is calculated by subtracting the value of
            // numOfLongVerMetrics from the number of glyphs in the font.'
            let tsb_array_len = self.number_of_glyphs() - number_of_vmetrics;

            // 'This array contains the top sidebearings of glyphs not represented in
            // the first array, and all the glyphs in this array must have the same advance
            // height as the last entry in the vMetrics array.'
            let advance = vmetrics_array.last().advance;

            let array: LazyArray<i16> = s.read_array(tsb_array_len as usize);
            Ok(VerticalMetrics {
                advance,
                top_side_bearing: array.at((glyph_id.0 - number_of_vmetrics) as usize),
            })
        }
    }
}

