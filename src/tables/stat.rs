//! A [Style Attributes Table](https://docs.microsoft.com/en-us/typography/opentype/spec/stat) implementation.

use crate::{
    parser::{Offset, Offset16, Offset32, Stream},
    Fixed, FromData, LazyArray16, Tag,
};

/// Axis-value pairing.
#[derive(Clone, Copy, Debug)]
pub struct AxisValue {
    /// Zero-based index into [`Table::axes`].
    pub axis_index: u16,
    /// Numeric value for this axis.
    pub value: Fixed,
}

impl FromData for AxisValue {
    const SIZE: usize = 6;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let axis_index = s.read::<u16>()?;
        let value = s.read::<Fixed>()?;

        Some(AxisValue { axis_index, value })
    }
}

/// List of axis value tables.
#[derive(Clone, Debug)]
pub struct AxisValueTables<'a> {
    data: Stream<'a>,
    start: Offset32,
    offsets: LazyArray16<'a, Offset16>,
    index: u16,
}

impl<'a> Iterator for AxisValueTables<'a> {
    type Item = AxisValueTable<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.offsets.len() {
            return None;
        }

        let mut s = Stream::new_at(
            self.data.tail()?,
            self.offsets.get(self.index)?.to_usize() + self.start.to_usize(),
        )?;
        self.index += 1;

        let format_variant = s.read::<u16>()?;

        let value = match format_variant {
            1 => {
                let value = s.read::<AxisValueTableFormat1>()?;
                Self::Item::Format1(value)
            }
            2 => {
                let value = s.read::<AxisValueTableFormat2>()?;
                Self::Item::Format2(value)
            }
            3 => {
                let value = s.read::<AxisValueTableFormat3>()?;
                Self::Item::Format3(value)
            }
            4 => {
                let value = AxisValueTableFormat4::parse(s.tail()?)?;
                Self::Item::Format4(value)
            }
            _ => return None,
        };

        Some(value)
    }
}

/// The [axis record](https://learn.microsoft.com/en-us/typography/opentype/spec/stat#axis-records) struct provides information about a single design axis.
#[derive(Clone, Copy, Debug)]
pub struct AxisRecord {
    /// Axis tag.
    pub tag: Tag,
    /// The name ID for entries in the 'name' table that provide a display string for this axis.
    pub name_id: u16,
    /// Sort order for e.g. composing font family or face names.
    pub ordering: u16,
}

/// [Flags](https://learn.microsoft.com/en-us/typography/opentype/spec/stat#flags) for [`AxisValue`].
#[derive(Clone, Copy)]
pub struct AxisValueFlags(u16);

#[rustfmt::skip]
impl AxisValueFlags {
    /// If set, this value also applies to older versions of this font.
    #[inline] pub fn older_sibling_attribute(self) -> bool { self.0 & (1 << 0) != 0 }

    /// If set, this value is the normal (a.k.a. "regular") value for the font family.
    #[inline] pub fn elidable(self) -> bool { self.0 & (1 << 1) != 0 }
}

impl core::fmt::Debug for AxisValueFlags {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut dbg = f.debug_set();

        if self.older_sibling_attribute() {
            dbg.entry(&"OLDER_SIBLING_FONT_ATTRIBUTE");
        }
        if self.elidable() {
            dbg.entry(&"ELIDABLE_AXIS_VALUE_NAME");
        }

        dbg.finish()
    }
}

/// Axis value table format 1
#[derive(Clone, Copy, Debug)]
pub struct AxisValueTableFormat1 {
    /// Zero-based index into [`Table::axes`].
    pub axis_index: u16,
    /// Flags for AxisValue.
    pub flags: AxisValueFlags,
    /// The name ID of the display string.
    pub value_name_id: u16,
    /// Numeric value for this record.
    pub value: Fixed,
}

impl FromData for AxisValueTableFormat1 {
    const SIZE: usize = 10;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(AxisValueTableFormat1 {
            axis_index: s.read::<u16>()?,
            flags: AxisValueFlags(s.read::<u16>()?),
            value_name_id: s.read::<u16>()?,
            value: s.read::<Fixed>()?,
        })
    }
}

/// Axis value table format 2
#[derive(Clone, Copy, Debug)]
pub struct AxisValueTableFormat2 {
    /// Zero-based index into [`Table::axes`].
    pub axis_index: u16,
    /// Flags for AxisValue.
    pub flags: AxisValueFlags,
    /// The name ID of the display string.
    pub value_name_id: u16,
    /// Nominal numeric value for this record.
    pub nominal_value: Fixed,
    /// The minimum value for this record.
    pub range_min_value: Fixed,
    /// The maximum value for this record.
    pub range_max_value: Fixed,
}

impl FromData for AxisValueTableFormat2 {
    const SIZE: usize = 18;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(AxisValueTableFormat2 {
            axis_index: s.read::<u16>()?,
            flags: AxisValueFlags(s.read::<u16>()?),
            value_name_id: s.read::<u16>()?,
            nominal_value: s.read::<Fixed>()?,
            range_min_value: s.read::<Fixed>()?,
            range_max_value: s.read::<Fixed>()?,
        })
    }
}

/// Axis value table format 3
#[derive(Clone, Copy, Debug)]
pub struct AxisValueTableFormat3 {
    /// Zero-based index into [`Table::axes`].
    pub axis_index: u16,
    /// Flags for AxisValue.
    pub flags: AxisValueFlags,
    /// The name ID of the display string.
    pub value_name_id: u16,
    /// Numeric value for this record.
    pub value: Fixed,
    /// Numeric value for a style-linked mapping.
    pub linked_value: Fixed,
}

impl FromData for AxisValueTableFormat3 {
    const SIZE: usize = 14;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(AxisValueTableFormat3 {
            axis_index: s.read::<u16>()?,
            flags: AxisValueFlags(s.read::<u16>()?),
            value_name_id: s.read::<u16>()?,
            value: s.read::<Fixed>()?,
            linked_value: s.read::<Fixed>()?,
        })
    }
}

/// Axis value table format 4
#[derive(Clone, Copy, Debug)]
pub struct AxisValueTableFormat4<'a> {
    /// Flags for AxisValue.
    pub flags: u16,
    /// The name ID of the display string.
    pub value_name_id: u16,
    /// List of axis-value pairings.
    pub values: LazyArray16<'a, AxisValue>,
}

impl<'a> AxisValueTableFormat4<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let axis_count = s.read::<u16>()?;
        let flags = s.read::<u16>()?;
        let value_name_id = s.read::<u16>()?;
        let values = s.read_array16::<AxisValue>(axis_count)?;

        Some(AxisValueTableFormat4 {
            flags,
            value_name_id,
            values,
        })
    }
}

/// An [axis value table](https://learn.microsoft.com/en-us/typography/opentype/spec/stat#axis-value-tables).
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug)]
pub enum AxisValueTable<'a> {
    Format1(AxisValueTableFormat1),
    Format2(AxisValueTableFormat2),
    Format3(AxisValueTableFormat3),
    Format4(AxisValueTableFormat4<'a>),
}

impl FromData for AxisRecord {
    const SIZE: usize = 8;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(AxisRecord {
            tag: s.read::<Tag>()?,
            name_id: s.read::<u16>()?,
            ordering: s.read::<u16>()?,
        })
    }
}

/// A [Style Attributes Table](https://docs.microsoft.com/en-us/typography/opentype/spec/stat).
#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    /// List of axes
    pub axes: LazyArray16<'a, AxisRecord>,
    data: &'a [u8],
    value_lookup_start: Offset32,
    value_offsets: LazyArray16<'a, Offset16>,
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let major = s.read::<u16>()?;
        let minor = s.read::<u16>()?;

        match (major, minor) {
            (1, 0) | (1, 1) | (1, 2) => {}
            _ => return None,
        }

        let _axis_size = s.read::<u16>()?;
        let axis_count = s.read::<u16>()?;
        let axis_offset = s.read::<Offset32>()?.to_usize();

        let value_count = s.read::<u16>()?;
        let value_lookup_start = s.read::<Offset32>()?;

        let mut s = Stream::new_at(data, axis_offset)?;
        let axes = s.read_array16::<AxisRecord>(axis_count)?;

        let mut s = Stream::new_at(data, value_lookup_start.to_usize())?;
        let value_offsets = s.read_array16::<Offset16>(value_count)?;

        Some(Self {
            axes,
            data,
            value_lookup_start,
            value_offsets,
        })
    }

    /// Iterator over the collection of axis value tables.
    pub fn tables(&self) -> AxisValueTables<'a> {
        AxisValueTables {
            data: Stream::new(self.data),
            start: self.value_lookup_start,
            offsets: self.value_offsets,
            index: 0,
        }
    }
}
