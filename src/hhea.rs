// https://docs.microsoft.com/en-us/typography/opentype/spec/hhea

use crate::parser::SafeStream;
use crate::Font;

// This is a mandatory table, so we already know that this table exists
// and has a valid size. So we can use SafeStream.

impl<'a> Font<'a> {
    /// Returns font's ascender value.
    #[inline]
    pub fn ascender(&self) -> i16 {
        const ASCENDER_OFFSET: usize = 4;
        SafeStream::read_at(self.hhea, ASCENDER_OFFSET)
    }

    /// Returns font's descender value.
    #[inline]
    pub fn descender(&self) -> i16 {
        const DESCENDER_OFFSET: usize = 6;
        SafeStream::read_at(self.hhea, DESCENDER_OFFSET)
    }

    /// Returns font's height.
    #[inline]
    pub fn height(&self) -> i16 {
        self.ascender() - self.descender()
    }

    /// Returns font's line gap.
    #[inline]
    pub fn line_gap(&self) -> i16 {
        const LINEGAP_OFFSET: usize = 8;
        SafeStream::read_at(self.hhea, LINEGAP_OFFSET)
    }

    #[inline]
    pub(crate) fn number_of_hmetrics(&self) -> u16 {
        const NUMBER_OF_HMETRICS_OFFSET: usize = 34;
        SafeStream::read_at(self.hhea, NUMBER_OF_HMETRICS_OFFSET)
    }
}
