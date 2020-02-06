// https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

use core::num::NonZeroU16;

use crate::{Font, Tag};
use crate::parser::{Stream, Offset16, Offset, LazyArray16};
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


#[derive(Clone, Copy)]
pub(crate) struct Table<'a> {
    axis_count: NonZeroU16,
    axes: LazyArray16<'a, raw::VariationAxisRecord>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let version: u32 = s.read()?;
        if version != 0x00010000 {
            return None;
        }

        let axes_array_offset: Offset16 = s.read()?;
        s.skip::<u16>(); // reserved
        let axis_count: u16 = s.read()?;

        // 'If axisCount is zero, then the font is not functional as a variable font,
        // and must be treated as a non-variable font;
        // any variation-specific tables or data is ignored.'
        let axis_count = NonZeroU16::new(axis_count)?;

        let mut s = Stream::new_at(data, axes_array_offset.to_usize());
        let axes = s.read_array(axis_count.get())?;

        Some(Table {
            axis_count,
            axes,
        })
    }
}


impl<'a> Font<'a> {
    /// Parses a number of variation axes.
    ///
    /// Returns `None` when font is not a variable font.
    /// Number of axis is never 0.
    pub fn variation_axes_count(&self) -> Option<NonZeroU16> {
        self.fvar.map(|fvar| fvar.axis_count)
    }

    /// Parses a variation axis by tag.
    pub fn variation_axis(&self, tag: Tag) -> Option<VariationAxis> {
        let (index, record) = self.fvar?.axes.into_iter()
            .enumerate().find(|(_, r)| r.axis_tag() == tag)?;

        let default_value = record.default_value();
        let min_value = core::cmp::min(default_value, record.min_value());
        let max_value = core::cmp::max(default_value, record.max_value());

        Some(VariationAxis {
            index: index as u16,
            min_value: min_value as f32 / 65536.0,
            default_value: default_value as f32 / 65536.0,
            max_value: max_value as f32 / 65536.0,
            name_id: record.axis_name_id(),
            hidden: (record.flags() >> 3) & 1 == 1,
        })
    }
}
