// https://docs.microsoft.com/en-us/typography/opentype/spec/loca

use core::num::NonZeroU16;
use core::ops::Range;

use crate::parser::Stream;
use crate::head::IndexToLocationFormat;
use crate::{Font, GlyphId, LazyArray16};

#[derive(Clone, Copy)]
pub enum Table<'a> {
    Short(LazyArray16<'a, u16>),
    Long(LazyArray16<'a, u32>),
}

impl<'a> Table<'a> {
    pub fn parse(
        data: &'a [u8],
        number_of_glyphs: NonZeroU16,
        format: IndexToLocationFormat,
    ) -> Option<Self> {
        // The number of ranges is `maxp.numGlyphs + 1`.
        //
        // Check for overflow first.
        let total = if number_of_glyphs.get() == core::u16::MAX {
            number_of_glyphs.get()
        } else {
            number_of_glyphs.get() + 1
        };

        let mut s = Stream::new(data);
        match format {
            IndexToLocationFormat::Short => {
                Some(Table::Short(s.read_array(total)?))
            }
            IndexToLocationFormat::Long => {
                Some(Table::Long(s.read_array(total)?))
            }
        }
    }

    #[inline]
    fn len(&self) -> u16 {
        match self {
            Table::Short(ref array) => array.len(),
            Table::Long(ref array) => array.len(),
        }
    }
}

impl<'a> Font<'a> {
    pub(crate) fn glyph_range(&self, glyph_id: GlyphId) -> Option<Range<usize>> {
        let table = self.loca?;

        let glyph_id = glyph_id.0;
        if glyph_id == core::u16::MAX {
            return None;
        }

        // Glyph ID must be smaller than total number of values in a `loca` array.
        if glyph_id + 1 >= table.len() {
            return None;
        }

        let range = match table {
            Table::Short(ref array) => {
                // 'The actual local offset divided by 2 is stored.'
                array.at(glyph_id) as usize * 2 .. array.at(glyph_id + 1) as usize * 2
            }
            Table::Long(ref array) => {
                array.at(glyph_id) as usize .. array.at(glyph_id + 1) as usize
            }
        };

        // TODO: use Range::is_empty as soon as it became stable
        if range.start == range.end {
            // No outline.
            None
        } else if range.start > range.end {
            // 'The offsets must be in ascending order.'
            None
        } else {
            Some(range)
        }
    }
}
