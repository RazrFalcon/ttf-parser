// https://docs.microsoft.com/en-us/typography/opentype/spec/fvar

use core::num::NonZeroU16;

use crate::{Tag, NormalizedCoordinate};
use crate::parser::{Stream, FromData, Fixed, Offset16, Offset, LazyArray16, LazyArrayIter16, f32_bound};


/// A [variation axis](https://docs.microsoft.com/en-us/typography/opentype/spec/fvar#variationaxisrecord).
#[allow(missing_docs)]
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VariationAxis {
    pub tag: Tag,
    pub min_value: f32,
    pub def_value: f32,
    pub max_value: f32,
    /// An axis name in the `name` table.
    pub name_id: u16,
    pub hidden: bool,
}

impl VariationAxis {
    /// Returns a normalized variation coordinate for this axis.
    pub(crate) fn normalized_value(&self, mut v: f32) -> NormalizedCoordinate {
        // Based on
        // https://docs.microsoft.com/en-us/typography/opentype/spec/avar#overview

        v = f32_bound(self.min_value, v, self.max_value);
        if v == self.def_value {
            v = 0.0;
        } else if v < self.def_value {
            v = (v - self.def_value) / (self.def_value - self.min_value);
        } else {
            v = (v - self.def_value) / (self.max_value - self.def_value);
        }

        NormalizedCoordinate::from(v)
    }
}


#[derive(Clone, Copy)]
pub(crate) struct Table<'a> {
    axes: LazyArray16<'a, VariationAxisRecord>,
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

        let mut s = Stream::new_at(data, axes_array_offset.to_usize())?;
        let axes = s.read_array16::<VariationAxisRecord>(axis_count.get())?;

        Some(Table { axes })
    }

    pub fn axes(&self) -> VariationAxes<'a> {
        VariationAxes { iter: self.axes.into_iter() }
    }

    // TODO: add axis_by_tag
}


/// An iterator over variation axes.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy, Default)]
pub struct VariationAxes<'a> {
    iter: LazyArrayIter16<'a, VariationAxisRecord>,
}

impl<'a> Iterator for VariationAxes<'a> {
    type Item = VariationAxis;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let record = self.iter.next()?;

        let def_value = record.def_value;
        let min_value = def_value.min(record.min_value);
        let max_value = def_value.max(record.max_value);

        Some(VariationAxis {
            tag: record.axis_tag,
            min_value,
            def_value,
            max_value,
            name_id: record.axis_name_id,
            hidden: (record.flags >> 3) & 1 == 1,
        })
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }
}


#[derive(Clone, Copy)]
struct VariationAxisRecord {
    axis_tag: Tag,
    min_value: f32,
    def_value: f32,
    max_value: f32,
    flags: u16,
    axis_name_id: u16,
}

impl FromData for VariationAxisRecord {
    const SIZE: usize = 20;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(VariationAxisRecord {
            axis_tag: s.read::<Tag>()?,
            min_value: s.read::<Fixed>()?.0,
            def_value: s.read::<Fixed>()?.0,
            max_value: s.read::<Fixed>()?.0,
            flags: s.read::<u16>()?,
            axis_name_id: s.read::<u16>()?,
        })
    }
}
