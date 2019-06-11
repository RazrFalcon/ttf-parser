//! The [post](https://docs.microsoft.com/en-us/typography/opentype/spec/post)
//! table parsing primitives.

use crate::stream::Stream;
use crate::{Font, LineMetrics};


impl<'a> Font<'a> {
    /// Parses font's underline metrics set in the `post` table.
    pub fn underline_metrics(&self) -> LineMetrics {
        const UNDERLINE_POSITION_OFFSET: usize = 8;
        const UNDERLINE_THICKNESS_OFFSET: usize = 10;

        let data = &self.data[self.post.range()];
        LineMetrics {
            position:  Stream::read_at(data, UNDERLINE_POSITION_OFFSET),
            thickness: Stream::read_at(data, UNDERLINE_THICKNESS_OFFSET),
        }
    }
}
