// https://docs.microsoft.com/en-us/typography/opentype/spec/post

use crate::{Font, LineMetrics};


impl<'a> Font<'a> {
    /// Parses font's underline metrics.
    ///
    /// Returns `None` when `post` table is not present.
    #[inline]
    pub fn underline_metrics(&self) -> Option<LineMetrics> {
        let table = self.post?;
        Some(LineMetrics {
            position: table.underline_position(),
            thickness: table.underline_thickness(),
        })
    }
}
