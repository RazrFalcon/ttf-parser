// https://docs.microsoft.com/en-us/typography/opentype/spec/vhea

use crate::Font;

impl<'a> Font<'a> {
    #[inline]
    pub(crate) fn number_of_vmetrics(&self) -> Option<u16> {
        self.vhea.map(|table| table.num_of_long_ver_metrics())
    }
}
