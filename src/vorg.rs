// https://docs.microsoft.com/en-us/typography/opentype/spec/vorg

use crate::{Font, GlyphId};
use crate::parser::{Stream, LazyArray16};
use crate::raw::vorg as raw;

#[derive(Clone, Copy)]
pub struct Table<'a> {
    default_y: i16,
    origins: LazyArray16<'a, raw::VertOriginYMetrics>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let major_version: u16 = s.read()?;
        let minor_version: u16 = s.read()?;
        if !(major_version == 1 && minor_version == 0) {
            return None;
        }

        Some(Table {
            default_y: s.read()?,
            origins: s.read_array16()?,
        })
    }
}

impl<'a> Font<'a> {
    /// Parses a vertical origin of a glyph according to
    /// [Vertical Origin Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vorg).
    pub fn glyph_y_origin(&self, glyph: GlyphId) -> Option<i16> {
        let table = self.vorg?;
        Some(table.origins.binary_search_by(|m| m.glyph_index().cmp(&glyph))
            .map(|(_, m)| m.vert_origin_y())
            .unwrap_or(table.default_y))
    }
}
