//! The [head](https://docs.microsoft.com/en-us/typography/opentype/spec/head)
//! table parsing primitives.

use crate::stream::Stream;
use crate::Font;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum IndexToLocationFormat {
    Short,
    Long,
}

impl<'a> Font<'a> {
    pub(crate) fn index_to_location_format(&self) -> Option<IndexToLocationFormat> {
        const INDEX_TO_LOC_FORMAT_OFFSET: usize = 50;

        let num: i16 = Stream::read_at(
            &self.data[self.head.range()], INDEX_TO_LOC_FORMAT_OFFSET
        );
        match num {
            0 => Some(IndexToLocationFormat::Short),
            1 => Some(IndexToLocationFormat::Long),
            _ => None,
        }
    }

    /// Returns font's units per EM set in the `head` table.
    ///
    /// Returns `None` if value is not in a 16..16384 range.
    pub fn units_per_em(&self) -> Option<u16> {
        const UNITS_PER_EM_OFFSET: usize = 18;

        let num: u16 = Stream::read_at(&self.data[self.head.range()], UNITS_PER_EM_OFFSET);
        if num >= 16 && num <= 16384 {
            Some(num)
        } else {
            None
        }
    }
}
