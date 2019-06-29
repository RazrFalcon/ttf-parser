use std::ops::Range;

use crate::parser::Stream;
use crate::{Font, GlyphId, TableName, Result, Error};


impl<'a> Font<'a> {
    pub(crate) fn glyph_range(&self, glyph_id: GlyphId) -> Result<Range<usize>> {
        use crate::head::IndexToLocationFormat as Format;
        const U16_LEN: u32 = 2;
        const U32_LEN: u32 = 4;

        self.check_glyph_id(glyph_id)?;

        let format = self.index_to_location_format().ok_or(Error::NoGlyph)?;
        let data = self.table_data(TableName::IndexToLocation)?;
        let mut s = Stream::new(data);

        let (start, end) = match format {
            Format::Short => {
                s.skip_len(glyph_id.0 as u32 * U16_LEN);
                // 'The actual local offset divided by 2 is stored.'
                (s.read::<u16>() as u32 * 2, s.read::<u16>() as u32 * 2)
            }
            Format::Long  => {
                s.skip_len(glyph_id.0 as u32 * U32_LEN);
                (s.read::<u32>(), s.read::<u32>())
            }
        };

        if start == end {
            // No outline.
            Err(Error::NoOutline)
        } else if start > end {
            // 'The offsets must be in ascending order.'
            Err(Error::NoGlyph)
        } else {
            Ok(start as usize .. end as usize)
        }
    }
}
