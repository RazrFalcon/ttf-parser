// https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx

use crate::{Font, GlyphId, Result};
use crate::parser::Stream;
use crate::raw::hmtx as raw;


impl<'a> Font<'a> {
    /// Parses glyph's horizontal advance using
    /// [Horizontal Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx).
    pub fn glyph_hor_advance(&self, glyph_id: GlyphId) -> Result<Option<u16>> {
        bail!(self.check_glyph_id(glyph_id));

        let number_of_hmetrics = self.number_of_hmetrics();
        if number_of_hmetrics == 0 {
            return Ok(None);
        }

        parse_glyph_advance(self.hmtx?, glyph_id, number_of_hmetrics)
    }

    /// Parses glyph's horizontal side bearing using
    /// [Horizontal Metrics Table](https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx).
    pub fn glyph_hor_side_bearing(&self, glyph_id: GlyphId) -> Result<Option<i16>> {
        bail!(self.check_glyph_id(glyph_id));

        let number_of_hmetrics = self.number_of_hmetrics();
        if number_of_hmetrics == 0 {
            return Ok(None);
        }

        parse_glyph_side_bearing(self.hmtx?, glyph_id, number_of_hmetrics, self.number_of_glyphs())
    }
}

#[inline]
pub fn parse_glyph_advance(
    data: &[u8],
    glyph_id: GlyphId,
    number_of_hmetrics: u16,
) -> Result<Option<u16>> {
    let mut s = Stream::new(data);
    let array = s.read_array::<raw::HorizontalMetrics, u16>(number_of_hmetrics)?;
    if let Some(metrics) = array.get(glyph_id.0) {
        Ok(Some(metrics.advance_width()))
    } else {
        // 'As an optimization, the number of records can be less than the number of glyphs,
        // in which case the advance width value of the last record applies
        // to all remaining glyph IDs.'
        Ok(array.last().map(|m| m.advance_width()))
    }
}

#[inline]
pub fn parse_glyph_side_bearing(
    data: &[u8],
    glyph_id: GlyphId,
    number_of_hmetrics: u16,
    number_of_glyphs: u16,
) -> Result<Option<i16>> {
    let mut s = Stream::new(data);
    let array = s.read_array::<raw::HorizontalMetrics, u16>(number_of_hmetrics)?;
    if let Some(metrics) = array.get(glyph_id.0) {
        Ok(Some(metrics.lsb()))
    } else {
        // 'If the number_of_hmetrics is less than the total number of glyphs,
        // then that array is followed by an array for the left side bearing values
        // of the remaining glyphs.'

        // Check for overflow.
        if number_of_glyphs < number_of_hmetrics {
            return Ok(None);
        }

        let count = number_of_glyphs - number_of_hmetrics;
        let left_side_bearings = s.read_array::<i16, u16>(count)?;
        // Overflow is not possible, because when `glyph_id` is smaller than `number_of_hmetrics`
        // this branch will not be executed.
        Ok(left_side_bearings.get(glyph_id.0 - number_of_hmetrics))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;
    use crate::writer;
    use writer::TtfType::*;

    #[test]
    fn simple_case() {
        let data = writer::convert(&[
            UInt16(1), // advanceWidth[0]
            Int16(2), // sideBearing[0]
        ]);

        assert_eq!(parse_glyph_advance(&data, GlyphId(0), 1).unwrap(), Some(1));
        assert_eq!(parse_glyph_side_bearing(&data, GlyphId(0), 1, 1).unwrap(), Some(2));
    }

    #[test]
    fn empty() {
        assert_eq!(parse_glyph_advance(&[], GlyphId(0), 1).unwrap_err().to_string(),
                   "an attempt to slice out of bounds");
        assert_eq!(parse_glyph_side_bearing(&[], GlyphId(0), 1, 1).unwrap_err().to_string(),
                   "an attempt to slice out of bounds");
    }

    #[test]
    fn smaller_than_glyphs_count() {
        let data = writer::convert(&[
            UInt16(1), // advanceWidth[0]
            Int16(2), // sideBearing[0]
            Int16(3), // sideBearing[1]
        ]);

        assert_eq!(parse_glyph_advance(&data, GlyphId(0), 1).unwrap(), Some(1));
        assert_eq!(parse_glyph_side_bearing(&data, GlyphId(0), 1, 2).unwrap(), Some(2));
        assert_eq!(parse_glyph_advance(&data, GlyphId(1), 1).unwrap(), Some(1));
        assert_eq!(parse_glyph_side_bearing(&data, GlyphId(1), 1, 2).unwrap(), Some(3));
    }

    #[test]
    fn less_metrics_than_glyphs() {
        let data = writer::convert(&[
            UInt16(1), // advanceWidth[0]
            Int16(2), // sideBearing[0]
            UInt16(3), // advanceWidth[1]
            Int16(4), // sideBearing[1]
            Int16(5), // sideBearing[2]
        ]);

        assert_eq!(parse_glyph_side_bearing(&data, GlyphId(0), 2, 1).unwrap(), Some(2));
        assert_eq!(parse_glyph_side_bearing(&data, GlyphId(1), 2, 1).unwrap(), Some(4));
        assert_eq!(parse_glyph_side_bearing(&data, GlyphId(2), 2, 1).unwrap(), None);
    }
}
