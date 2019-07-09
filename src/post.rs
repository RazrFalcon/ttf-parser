// https://docs.microsoft.com/en-us/typography/opentype/spec/post

use crate::parser::SafeStream;
use crate::{Font, LineMetrics};

// We already checked that `post` table has a valid length,
// so it's safe to use `SafeStream`.

impl<'a> Font<'a> {
    /// Parses font's underline metrics.
    ///
    /// Returns `None` when `post` table is not present.
    #[inline]
    pub fn underline_metrics(&self) -> Option<LineMetrics> {
        const UNDERLINE_POSITION_OFFSET: usize = 8;
        let mut s = SafeStream::new_at(self.post?, UNDERLINE_POSITION_OFFSET);

        // Do not change the order. In the `post` table, line position is set before thickness.
        Some(LineMetrics {
            position: s.read(),
            thickness: s.read(),
        })
    }
}
