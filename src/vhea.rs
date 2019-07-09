// https://docs.microsoft.com/en-us/typography/opentype/spec/vhea

use crate::parser::SafeStream;
use crate::Font;

// We already checked that `vhea` table has a valid length,
// so it's safe to use `SafeStream`.

impl<'a> Font<'a> {
    #[inline]
    pub(crate) fn number_of_vmetrics(&self) -> Option<u16> {
        const NUMBER_OF_VMETRICS_OFFSET: usize = 34;
        Some(SafeStream::read_at(self.vhea?, NUMBER_OF_VMETRICS_OFFSET))
    }
}
