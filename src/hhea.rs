//! The [hhea](https://docs.microsoft.com/en-us/typography/opentype/spec/hhea)
//! table parsing primitives.

use crate::stream::Stream;
use crate::Font;

impl<'a> Font<'a> {
    /// Returns font's ascender set in the `hhea` table.
    pub fn ascender(&self) -> i16 {
        const ASCENDER_OFFSET: usize = 4;
        Stream::read_at(&self.data[self.hhea.range()], ASCENDER_OFFSET)
    }

    /// Returns font's descender set in the `hhea` table.
    pub fn descender(&self) -> i16 {
        const DESCENDER_OFFSET: usize = 6;
        Stream::read_at(&self.data[self.hhea.range()], DESCENDER_OFFSET)
    }

    /// Returns font's height.
    pub fn height(&self) -> i16 {
        self.ascender() - self.descender()
    }

    /// Returns font's line gap set in the `hhea` table.
    pub fn line_gap(&self) -> i16 {
        const LINEGAP_OFFSET: usize = 8;
        Stream::read_at(&self.data[self.hhea.range()], LINEGAP_OFFSET)
    }

    pub(crate) fn number_of_hmetrics(&self) -> u16 {
        const NUMBER_OF_HMETRICS: usize = 34;
        Stream::read_at(&self.data[self.hhea.range()], NUMBER_OF_HMETRICS)
    }
}
