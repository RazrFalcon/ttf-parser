use crate::parser::Stream;
use crate::{Font, TableName, GlyphId, HorizontalMetrics, Result};


impl<'a> Font<'a> {
    /// Returns glyph's horizontal metrics.
    pub fn glyph_hor_metrics(&self, glyph_id: GlyphId) -> Result<HorizontalMetrics> {
        const HOR_METRIC_RECORD_SIZE: usize = 4;
        const U16_SIZE: usize = 2;

        self.check_glyph_id(glyph_id)?;
        let data = self.table_data(TableName::HorizontalMetrics)?;
        let number_of_hmetrics = self.number_of_hmetrics();

        if glyph_id.0 < number_of_hmetrics {
            // Records are indexed by glyph ID.
            let data = &data[glyph_id.0 as usize * HOR_METRIC_RECORD_SIZE..];
            let mut s = Stream::new(data);
            Ok(HorizontalMetrics {
                advance: s.read(),
                left_side_bearing: s.read(),
            })
        } else {
            assert!(number_of_hmetrics > 0);

            let left_side_bearings_offset = number_of_hmetrics as usize * HOR_METRIC_RECORD_SIZE;

            // 'As an optimization, the number of records can be less than the number of glyphs,
            // in which case the advance width value of the last record applies
            // to all remaining glyph IDs.'
            let offset = left_side_bearings_offset - HOR_METRIC_RECORD_SIZE;
            let advance = Stream::read_at(data, offset);

            let offset = left_side_bearings_offset +
                (glyph_id.0 - number_of_hmetrics) as usize * U16_SIZE;
            let left_side_bearing = Stream::read_at(data, offset);

            Ok(HorizontalMetrics {
                advance,
                left_side_bearing,
            })
        }
    }
}
