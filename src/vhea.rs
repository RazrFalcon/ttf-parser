//! The [vhea](https://docs.microsoft.com/en-us/typography/opentype/spec/vhea)
//! table parsing primitives.

use crate::parser::Stream;


/// Handle to a `vhea` table.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Table<'a> {
    pub(crate) data: &'a [u8],
}

impl<'a> Table<'a> {
    pub(crate) fn number_of_vmetrics(&self) -> u16 {
        const NUMBER_OF_VMETRICS_OFFSET: usize = 34;
        Stream::read_at(self.data, NUMBER_OF_VMETRICS_OFFSET)
    }
}
