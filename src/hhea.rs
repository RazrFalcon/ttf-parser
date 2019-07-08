// https://docs.microsoft.com/en-us/typography/opentype/spec/hhea

use crate::parser::Stream;
use crate::{Font, TableName};

// This is a mandatory table, so we already know that this table exists
// and has a valid size. So unwrapping is safe.

impl<'a> Font<'a> {
    /// Returns font's ascender set in the `hhea` table.
    #[inline(never)]
    pub fn ascender(&self) -> i16 {
        const ASCENDER_OFFSET: usize = 4;
        let data = self.table_data(TableName::HorizontalHeader).unwrap();
        Stream::read_at(data, ASCENDER_OFFSET).unwrap()
    }

    /// Returns font's descender set in the `hhea` table.
    #[inline(never)]
    pub fn descender(&self) -> i16 {
        const DESCENDER_OFFSET: usize = 6;
        let data = self.table_data(TableName::HorizontalHeader).unwrap();
        Stream::read_at(data, DESCENDER_OFFSET).unwrap()
    }

    /// Returns font's height.
    pub fn height(&self) -> i16 {
        self.ascender() - self.descender()
    }

    /// Returns font's line gap set in the `hhea` table.
    #[inline(never)]
    pub fn line_gap(&self) -> i16 {
        const LINEGAP_OFFSET: usize = 8;
        let data = self.table_data(TableName::HorizontalHeader).unwrap();
        Stream::read_at(data, LINEGAP_OFFSET).unwrap()
    }

    pub(crate) fn number_of_hmetrics(&self) -> u16 {
        const NUMBER_OF_HMETRICS_OFFSET: usize = 34;
        let data = self.table_data(TableName::HorizontalHeader).unwrap();
        Stream::read_at(data, NUMBER_OF_HMETRICS_OFFSET).unwrap()
    }
}
