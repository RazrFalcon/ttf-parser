// https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

use crate::{Font, Tag};
use crate::parser::{Stream, Offset, LazyArray};
use crate::raw::fvar as raw;


/// A [variation axis](https://docs.microsoft.com/en-us/typography/opentype/spec/fvar#variationaxisrecord).
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VariationAxis {
    /// Axis index in `fvar` table.
    pub index: u16,
    pub min_value: f32,
    pub default_value: f32,
    pub max_value: f32,
    /// Axis name in `name` table.
    pub name_id: u16,
    pub hidden: bool,
}


impl<'a> Font<'a> {
    /// Parses a number of variation axes.
    ///
    /// Returns `None` when font is not a variable font.
    /// Number of axis is never 0.
    pub fn variation_axes_count(&self) -> Option<u16> {
        match self.fvar?.axis_count() {
            0 => None,
            n => Some(n),
        }
    }

    /// Parses a variation axis by tag.
    pub fn variation_axis(&self, tag: Tag) -> Option<VariationAxis> {
        let table = self.fvar?;
        let offset = table.axes_array_offset();
        let mut s = Stream::new_at(table.data, offset.to_usize());
        let axes: LazyArray<raw::VariationAxisRecord> = s.read_array(table.axis_count()).ok()?;
        if let Some(index) = axes.into_iter().position(|r| r.axis_tag() == tag).map(|i| i as u16) {
            let record = axes.at(index);

            let default_value = record.default_value();
            let min_value = core::cmp::min(default_value, record.min_value());
            let max_value = core::cmp::max(default_value, record.max_value());

            return Some(VariationAxis {
                index,
                min_value: min_value as f32 / 65536.0,
                default_value: default_value as f32 / 65536.0,
                max_value: max_value as f32 / 65536.0,
                name_id: record.axis_name_id(),
                hidden: (record.flags() >> 3) & 1 == 1,
            })
        }

        None
    }
}
