// https://docs.microsoft.com/en-us/typography/opentype/spec/hhea

use crate::Font;

impl<'a> Font<'a> {
    /// Returns font's ascender value.
    #[inline]
    pub fn ascender(&self) -> i16 {
        self.hhea.ascender()
    }

    /// Returns font's descender value.
    #[inline]
    pub fn descender(&self) -> i16 {
        self.hhea.descender()
    }

    /// Returns font's height.
    #[inline]
    pub fn height(&self) -> i16 {
        self.ascender() - self.descender()
    }

    /// Returns font's line gap.
    #[inline]
    pub fn line_gap(&self) -> i16 {
        self.hhea.line_gap()
    }

    #[inline]
    pub(crate) fn number_of_hmetrics(&self) -> u16 {
        self.hhea.number_of_h_metrics()
    }
}
