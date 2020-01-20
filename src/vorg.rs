// https://docs.microsoft.com/en-us/typography/opentype/spec/vorg

use crate::{Font, GlyphId};
use crate::parser::{Stream, LazyArray};
use crate::raw::vorg as raw;


impl<'a> Font<'a> {
    /// Parses a vertical origin of a glyph according to
    /// [Vertical Origin Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vorg).
    ///
    /// Returns `None` when table is missing, malformed or has an unsupported version.
    pub fn glyph_y_origin(&self, glyph: GlyphId) -> Option<i16> {
        let mut s = Stream::new(self.vorg?);

        let major_version: u16 = s.read().ok()?;
        let minor_version: u16 = s.read().ok()?;
        if major_version != 1 && minor_version != 0 {
            return None;
        }

        let default_y: i16 = s.read().ok()?;
        let origins: LazyArray<raw::VertOriginYMetrics> = s.read_array16().ok()?;
        Some(origins.binary_search_by(|m| m.glyph_index().cmp(&glyph))
            .map(|m| m.vert_origin_y())
            .unwrap_or(default_y))
    }
}
