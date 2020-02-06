// https://docs.microsoft.com/en-us/typography/opentype/spec/head

use crate::Font;

#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IndexToLocationFormat {
    Short,
    Long,
}

impl<'a> Font<'a> {
    /// Parses glyphs index to location format.
    #[inline]
    pub fn index_to_location_format(&self) -> Option<IndexToLocationFormat> {
        match self.head.index_to_loc_format() {
            0 => Some(IndexToLocationFormat::Short),
            1 => Some(IndexToLocationFormat::Long),
            _ => None,
        }
    }

    /// Parses font's units per EM.
    ///
    /// Returns `None` when value is not in a 16..=16384 range.
    #[inline]
    pub fn units_per_em(&self) -> Option<u16> {
        let num = self.head.units_per_em();
        if num >= 16 && num <= 16384 {
            Some(num)
        } else {
            None
        }
    }
}
