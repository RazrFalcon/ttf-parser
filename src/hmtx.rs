//! The [hmtx](https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx)
//! table parsing primitives.

use crate::stream::Stream;
use crate::{Font, GlyphId};


/// A horizontal metrics of a glyph.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct HorizontalMetrics {
    /// A horizontal advance.
    pub advance: u16,

    /// Left side bearing.
    pub left_side_bearing: i16,
}

impl<'a> Font<'a> {
    /// Returns glyph's horizontal metrics.
    ///
    /// Returns `None` when font doesn't have such `glyph_id`.
    pub fn glyph_hor_metrics(&self, glyph_id: GlyphId) -> Option<HorizontalMetrics> {
        const HOR_METRIC_RECORD_SIZE: usize = 4;
        const U16_SIZE: usize = 2;

        if glyph_id >= self.number_of_glyphs {
            return None;
        }

        let number_of_hmetrics = self.number_of_hmetrics();
        let data = &self.data[self.hmtx.range()];

        if glyph_id.0 < number_of_hmetrics {
            // Records are indexed by glyph ID.
            let data = &data[glyph_id.0 as usize * HOR_METRIC_RECORD_SIZE..];
            let mut s = Stream::new(data);
            Some(HorizontalMetrics {
                advance: s.read_u16(),
                left_side_bearing: s.read_i16(),
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

            Some(HorizontalMetrics {
                advance,
                left_side_bearing,
            })
        }
    }
}
