// https://docs.microsoft.com/en-us/typography/opentype/spec/vorg

use crate::{Font, GlyphId, Result, Error};
use crate::parser::Stream;
use crate::raw::vorg as raw;


impl<'a> Font<'a> {
    /// Parses a vertical origin of a glyph according to
    /// [Vertical Origin Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vorg).
    pub fn glyph_y_origin(&self, glyph: GlyphId) -> Result<i16> {
        let mut s = Stream::new(self.vorg?);

        let major_version: u16 = s.read()?;
        let minor_version: u16 = s.read()?;
        if !(major_version == 1 && minor_version == 0) {
            return Err(Error::UnsupportedTableVersion);
        }

        let default_y: i16 = s.read()?;
        let origins = s.read_array16::<raw::VertOriginYMetrics>()?;
        Ok(origins.binary_search_by(|m| m.glyph_index().cmp(&glyph))
            .map(|(_, m)| m.vert_origin_y())
            .unwrap_or(default_y))
    }
}
