// https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx

use core::num::NonZeroU16;

use crate::GlyphId;
use crate::parser::{Stream, FromData, LazyArray16};


#[derive(Clone, Copy)]
struct HorizontalMetrics {
    advance_width: u16,
    lsb: i16,
}

impl FromData for HorizontalMetrics {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(HorizontalMetrics {
            advance_width: s.read::<u16>()?,
            lsb: s.read::<i16>()?,
        })
    }
}


#[derive(Clone, Copy)]
pub struct Table<'a> {
    metrics: LazyArray16<'a, HorizontalMetrics>,
    bearings: Option<LazyArray16<'a, i16>>,
    number_of_metrics: u16, // Sum of long metrics + bearings.
}

impl<'a> Table<'a> {
    pub fn parse(
        data: &'a [u8],
        number_of_hmetrics: NonZeroU16,
        number_of_glyphs: NonZeroU16,
    ) -> Option<Self> {
        let mut s = Stream::new(data);
        let metrics = s.read_array16::<HorizontalMetrics>(number_of_hmetrics.get())?;

        let mut number_of_metrics = number_of_hmetrics.get();

        // 'If the number_of_hmetrics is less than the total number of glyphs,
        // then that array is followed by an array for the left side bearing values
        // of the remaining glyphs.'
        let bearings_count = number_of_glyphs.get().checked_sub(number_of_hmetrics.get());
        let bearings = if let Some(count) = bearings_count {
            number_of_metrics += count;
            s.read_array16::<i16>(count)
        } else {
            None
        };

        Some(Table {
            metrics,
            bearings,
            number_of_metrics,
        })
    }

    #[inline]
    pub fn advance(&self, glyph_id: GlyphId) -> Option<u16> {
        if glyph_id.0 >= self.number_of_metrics {
            return None;
        }

        if let Some(metrics) = self.metrics.get(glyph_id.0) {
            Some(metrics.advance_width)
        } else {
            // 'As an optimization, the number of records can be less than the number of glyphs,
            // in which case the advance width value of the last record applies
            // to all remaining glyph IDs.'
            self.metrics.last().map(|m| m.advance_width)
        }
    }

    #[inline]
    pub fn side_bearing(&self, glyph_id: GlyphId) -> Option<i16> {
        if let Some(metrics) = self.metrics.get(glyph_id.0) {
            Some(metrics.lsb)
        } else if let Some(bearings) = self.bearings {
            // 'If the number_of_hmetrics is less than the total number of glyphs,
            // then that array is followed by an array for the left side bearing values
            // of the remaining glyphs.'

            let number_of_hmetrics = self.metrics.len();

            // Check for overflow.
            if glyph_id.0 < number_of_hmetrics {
                return None;
            }

            bearings.get(glyph_id.0 - number_of_hmetrics)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! nzu16 {
        ($n:expr) => { NonZeroU16::new($n).unwrap() };
    }

    #[test]
    fn simple_case() {
        let data = &[
            0x00, 0x01, // advance width [0]: 1
            0x00, 0x02, // side bearing [0]: 2
        ];

        let table = Table::parse(data, nzu16!(1), nzu16!(1)).unwrap();
        assert_eq!(table.advance(GlyphId(0)), Some(1));
        assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
    }

    #[test]
    fn empty() {
        assert!(Table::parse(&[], nzu16!(1), nzu16!(1)).is_none());
    }

    #[test]
    fn smaller_than_glyphs_count() {
        let data = &[
            0x00, 0x01, // advance width [0]: 1
            0x00, 0x02, // side bearing [0]: 2

            0x00, 0x03, // side bearing [1]: 3
        ];

        let table = Table::parse(data, nzu16!(1), nzu16!(2)).unwrap();
        assert_eq!(table.advance(GlyphId(0)), Some(1));
        assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
        assert_eq!(table.advance(GlyphId(1)), Some(1));
        assert_eq!(table.side_bearing(GlyphId(1)), Some(3));
    }

    #[test]
    fn less_metrics_than_glyphs() {
        let data = &[
            0x00, 0x01, // advance width [0]: 1
            0x00, 0x02, // side bearing [0]: 2

            0x00, 0x03, // advance width [1]: 3
            0x00, 0x04, // side bearing [1]: 4

            0x00, 0x05, // side bearing [2]: 5
        ];

        let table = Table::parse(data, nzu16!(2), nzu16!(1)).unwrap();
        assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
        assert_eq!(table.side_bearing(GlyphId(1)), Some(4));
        assert_eq!(table.side_bearing(GlyphId(2)), None);
    }

    #[test]
    fn glyph_out_of_bounds_0() {
        let data = &[
            0x00, 0x01, // advance width [0]: 1
            0x00, 0x02, // side bearing [0]: 2
        ];

        let table = Table::parse(data, nzu16!(1), nzu16!(1)).unwrap();
        assert_eq!(table.advance(GlyphId(0)), Some(1));
        assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
        assert_eq!(table.advance(GlyphId(1)), None);
        assert_eq!(table.side_bearing(GlyphId(1)), None);
    }

    #[test]
    fn glyph_out_of_bounds_1() {
        let data = &[
            0x00, 0x01, // advance width [0]: 1
            0x00, 0x02, // side bearing [0]: 2

            0x00, 0x03, // side bearing [1]: 3
        ];

        let table = Table::parse(data, nzu16!(1), nzu16!(2)).unwrap();
        assert_eq!(table.advance(GlyphId(1)), Some(1));
        assert_eq!(table.side_bearing(GlyphId(1)), Some(3));
        assert_eq!(table.advance(GlyphId(2)), None);
        assert_eq!(table.side_bearing(GlyphId(2)), None);
    }
}
