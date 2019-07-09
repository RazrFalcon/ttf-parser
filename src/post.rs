// https://docs.microsoft.com/en-us/typography/opentype/spec/post

use crate::parser::Stream;
use crate::{Font, TableName, LineMetrics, Result, Error};

impl<'a> Font<'a> {
    /// Parses font's underline metrics set in the `post` table.
    pub fn underline_metrics(&self) -> Result<LineMetrics> {
        const UNDERLINE_POSITION_OFFSET: usize = 8;
        const UNDERLINE_THICKNESS_OFFSET: usize = 10;

        let data = self.post.ok_or_else(|| Error::TableMissing(TableName::PostScript))?;
        Ok(LineMetrics {
            position:  Stream::read_at(data, UNDERLINE_POSITION_OFFSET)?,
            thickness: Stream::read_at(data, UNDERLINE_THICKNESS_OFFSET)?,
        })
    }
}
