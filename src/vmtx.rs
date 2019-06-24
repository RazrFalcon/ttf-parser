//! The [vmtx](https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx)
//! table parsing primitives.

use crate::parser::{Stream, FromData, LazyArray};
use crate::GlyphId;


/// A vertical metrics of a glyph.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VerticalMetrics {
    /// A vertical advance.
    pub advance: u16,

    /// Top side bearing.
    pub top_side_bearing: i16,
}

impl FromData for VerticalMetrics {
    fn parse(data: &[u8]) -> Self {
        let mut s = Stream::new(data);
        VerticalMetrics {
            advance: s.read(),
            top_side_bearing: s.read(),
        }
    }
}


/// Handle to a `vmtx` table.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Table<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) number_of_glyphs: u16,
    pub(crate) number_of_vmetrics: u16,
}

impl<'a> Table<'a> {
    /// Returns glyph's vertical metrics.
    ///
    /// Returns `None` when font doesn't have such `glyph_id`.
    pub fn glyph_ver_metrics(&self, glyph_id: GlyphId) -> Option<VerticalMetrics> {
        if glyph_id.0 >= self.number_of_glyphs {
            return None;
        }

        let mut s = Stream::new(self.data);
        let vmetrics_array = s.read_array::<VerticalMetrics>(self.number_of_vmetrics as usize);

        if glyph_id.0 < self.number_of_vmetrics {
            vmetrics_array.get(glyph_id.0 as usize)
        } else {
            // 'The number of entries in this array is calculated by subtracting the value of
            // numOfLongVerMetrics from the number of glyphs in the font.'
            let tsb_array_len = self.number_of_glyphs - self.number_of_vmetrics;

            // 'This array contains the top sidebearings of glyphs not represented in
            // the first array, and all the glyphs in this array must have the same advance
            // height as the last entry in the vMetrics array.'
            let advance = vmetrics_array.last().advance;

            let array: LazyArray<i16> = s.read_array(tsb_array_len as usize);
            Some(VerticalMetrics {
                advance,
                top_side_bearing: array.at((glyph_id.0 - self.number_of_vmetrics) as usize),
            })
        }
    }
}

