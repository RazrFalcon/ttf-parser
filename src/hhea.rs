// https://docs.microsoft.com/en-us/typography/opentype/spec/hhea

use crate::Font;

impl<'a> Font<'a> {
    /// Parses font's ascender value.
    #[inline]
    pub fn ascender(&self) -> i16 {
        if self.is_use_typo_metrics() {
            self.os_2.map(|table| table.s_typo_ascender()).unwrap()
        } else {
            self.hhea.ascender()
        }
    }

    /// Parses font's descender value.
    #[inline]
    pub fn descender(&self) -> i16 {
        if self.is_use_typo_metrics() {
            self.os_2.map(|table| table.s_typo_descender()).unwrap()
        } else {
            self.hhea.descender()
        }
    }

    /// Parses font's height.
    #[inline]
    pub fn height(&self) -> i16 {
        self.ascender() - self.descender()
    }

    /// Parses font's line gap.
    #[inline]
    pub fn line_gap(&self) -> i16 {
        if self.is_use_typo_metrics() {
            self.os_2.map(|table| table.s_typo_line_gap()).unwrap()
        } else {
            self.hhea.line_gap()
        }
    }
}
