// https://docs.microsoft.com/en-us/typography/opentype/spec/loca

use core::num::NonZeroU16;
use core::ops::Range;

use crate::{GlyphId, IndexToLocationFormat};
use crate::parser::{Stream, LazyArray16, NumFrom};

#[derive(Clone, Copy)]
pub(crate) enum Table<'a> {
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
                Some(Table::Short(s.read_array16::<u16>(total)?))
            }
            IndexToLocationFormat::Long => {
                Some(Table::Long(s.read_array16::<u32>(total)?))
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

    #[inline]
    pub fn glyph_range(&self, glyph_id: GlyphId) -> Option<Range<usize>> {
        let glyph_id = glyph_id.0;
        if glyph_id == core::u16::MAX {
            return None;
        }

        // Glyph ID must be smaller than total number of values in a `loca` array.
        if glyph_id + 1 >= self.len() {
            return None;
        }

        let range = match self {
            Table::Short(ref array) => {
                // 'The actual local offset divided by 2 is stored.'
                usize::from(array.get(glyph_id)?) * 2 .. usize::from(array.get(glyph_id + 1)?) * 2
            }
            Table::Long(ref array) => {
                usize::num_from(array.get(glyph_id)?) .. usize::num_from(array.get(glyph_id + 1)?)
            }
        };

        if range.start >= range.end {
            // 'The offsets must be in ascending order.'
            // And range cannot be empty.
            None
        } else {
            Some(range)
        }
    }
}
