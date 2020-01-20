// https://docs.microsoft.com/en-us/typography/opentype/spec/vhea

use crate::Font;

impl<'a> Font<'a> {
    /// Returns font's vertical ascender value.
    ///
    /// Returns `None` when `vhea` table is not present.
    #[inline]
    pub fn vertical_ascender(&self) -> Option<i16> {
        self.vhea.map(|table| table.ascender())
    }

    /// Returns font's vertical descender value.
    ///
    /// Returns `None` when `vhea` table is not present.
    #[inline]
    pub fn vertical_descender(&self) -> Option<i16> {
        self.vhea.map(|table| table.descender())
    }

    /// Returns font's vertical height.
    ///
    /// Returns `None` when `vhea` table is not present.
    #[inline]
    pub fn vertical_height(&self) -> Option<i16> {
        Some(self.vertical_ascender()? - self.vertical_descender()?)
    }

    /// Returns font's vertical line gap.
    ///
    /// Returns `None` when `vhea` table is not present.
    #[inline]
    pub fn vertical_line_gap(&self) -> Option<i16> {
        self.vhea.map(|table| table.line_gap())
    }

    #[inline]
    pub(crate) fn number_of_vmetrics(&self) -> Option<u16> {
        self.vhea.map(|table| table.num_of_long_ver_metrics())
    }
}
