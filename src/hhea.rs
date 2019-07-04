use crate::parser::Stream;
use crate::{Font, TableName, Result};

impl<'a> Font<'a> {
    /// Returns font's ascender set in the `hhea` table.
    #[inline(never)]
    pub fn ascender(&self) -> Result<i16> {
        const ASCENDER_OFFSET: usize = 4;
        let data = self.table_data(TableName::HorizontalHeader)?;
        Stream::read_at(data, ASCENDER_OFFSET)
    }

    /// Returns font's descender set in the `hhea` table.
    #[inline(never)]
    pub fn descender(&self) -> Result<i16> {
        const DESCENDER_OFFSET: usize = 6;
        let data = self.table_data(TableName::HorizontalHeader)?;
        Stream::read_at(data, DESCENDER_OFFSET)
    }

    /// Returns font's height.
    pub fn height(&self) -> Result<i16> {
        Ok(self.ascender()? - self.descender()?)
    }

    /// Returns font's line gap set in the `hhea` table.
    #[inline(never)]
    pub fn line_gap(&self) -> Result<i16> {
        const LINEGAP_OFFSET: usize = 8;
        let data = self.table_data(TableName::HorizontalHeader)?;
        Stream::read_at(data, LINEGAP_OFFSET)
    }

    pub(crate) fn number_of_hmetrics(&self) -> Result<u16> {
        const NUMBER_OF_HMETRICS_OFFSET: usize = 34;
        let data = self.table_data(TableName::HorizontalHeader)?;
        Stream::read_at(data, NUMBER_OF_HMETRICS_OFFSET)
    }
}
