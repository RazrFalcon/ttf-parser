/*!
A collection of [Apple Advanced Typography](
https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6AATIntro.html)
related types.
*/

use core::num::NonZeroU16;

use crate::GlyphId;
use crate::parser::{Stream, FromData, LazyArray16};

/// A [lookup table](
/// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6Tables.html).
///
/// u32 values in Format10 tables will be truncated to u16.
/// u64 values in Format10 tables are not supported.
#[derive(Clone)]
pub struct Lookup<'a> {
    data: LookupInner<'a>,
}

impl<'a> Lookup<'a> {
    /// Parses a lookup table from raw data.
    ///
    /// `number_of_glyphs` is from the `maxp` table.
    #[inline]
    pub fn parse(number_of_glyphs: NonZeroU16, data: &'a [u8]) -> Option<Self> {
        LookupInner::parse(number_of_glyphs, data).map(|data| Self { data })
    }

    /// Returns a value associated with the specified glyph.
    #[inline]
    pub fn value(&self, glyph_id: GlyphId) -> Option<u16> {
        self.data.value(glyph_id)
    }
}

impl core::fmt::Debug for Lookup<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Lookup {{ ... }}")
    }
}


#[derive(Clone)]
enum LookupInner<'a> {
    Format1(LazyArray16<'a, u16>),
    Format2(BinarySearchTable<'a, LookupSegment>),
    Format4(BinarySearchTable<'a, LookupSegment>, &'a [u8]),
    Format6(BinarySearchTable<'a, LookupSingle>),
    Format8 {
        first_glyph: u16,
        values: LazyArray16<'a, u16>
    },
    Format10 {
        value_size: u16,
        first_glyph: u16,
        glyph_count: u16,
        data: &'a [u8],
    },
}

impl<'a> LookupInner<'a> {
    fn parse(number_of_glyphs: NonZeroU16, data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let format = s.read::<u16>()?;
        match format {
            0 => {
                let values = s.read_array16::<u16>(number_of_glyphs.get())?;
                Some(Self::Format1(values))
            }
            2 => {
                let bsearch = BinarySearchTable::<LookupSegment>::parse(s.tail()?)?;
                Some(Self::Format2(bsearch))
            }
            4 => {
                let bsearch = BinarySearchTable::<LookupSegment>::parse(s.tail()?)?;
                Some(Self::Format4(bsearch, data))
            }
            6 => {
                let bsearch = BinarySearchTable::<LookupSingle>::parse(s.tail()?)?;
                Some(Self::Format6(bsearch))
            }
            8 => {
                let first_glyph = s.read::<u16>()?;
                let glyph_count = s.read::<u16>()?;
                let values = s.read_array16::<u16>(glyph_count)?;
                Some(Self::Format8 { first_glyph, values })
            }
            10 => {
                let value_size = s.read::<u16>()?;
                let first_glyph = s.read::<u16>()?;
                let glyph_count = s.read::<u16>()?;
                Some(Self::Format10 { value_size, first_glyph, glyph_count, data: s.tail()? })
            }
            _ => {
                None
            }
        }
    }

    fn value(&self, glyph_id: GlyphId) -> Option<u16> {
        match self {
            Self::Format1(values) => {
                values.get(glyph_id.0)
            }
            Self::Format2(ref bsearch) => {
                bsearch.get(glyph_id).map(|v| v.value)
            }
            Self::Format4(ref bsearch, data) => {
                // In format 4, LookupSegment contains an offset to a list of u16 values.
                // One value for each glyph in the LookupSegment range.
                let segment = bsearch.get(glyph_id)?;
                let index = glyph_id.0.checked_sub(segment.first_glyph)?;
                let offset = usize::from(segment.value) + u16::SIZE * usize::from(index);
                Stream::read_at::<u16>(data, offset)
            }
            Self::Format6(ref bsearch) => {
                bsearch.get(glyph_id).map(|v| v.value)
            }
            Self::Format8 { first_glyph, values } => {
                let idx = glyph_id.0.checked_sub(*first_glyph)?;
                values.get(idx)
            }
            Self::Format10 { value_size, first_glyph, glyph_count, data } => {
                let idx = glyph_id.0.checked_sub(*first_glyph)?;
                let mut s = Stream::new(data);
                match value_size {
                    1 => s.read_array16::<u8>(*glyph_count)?.get(idx).map(u16::from),
                    2 => s.read_array16::<u16>(*glyph_count)?.get(idx),
                    // TODO: we should return u32 here, but this is not supported yet
                    4 => s.read_array16::<u32>(*glyph_count)?.get(idx).map(|n| n as u16),
                    _ => None, // 8 is also supported
                }
            }
        }
    }
}

/// A binary searching table as defined at
/// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6Tables.html
#[derive(Clone)]
struct BinarySearchTable<'a, T: BinarySearchValue> {
    values: LazyArray16<'a, T>,
    len: NonZeroU16, // values length excluding termination segment
}

impl<'a, T: BinarySearchValue + core::fmt::Debug> BinarySearchTable<'a, T> {
    #[inline(never)]
    fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        let segment_size = s.read::<u16>()?;
        let number_of_segments = s.read::<u16>()?;
        s.advance(6); // search_range + entry_selector + range_shift

        if usize::from(segment_size) != T::SIZE {
            return None;
        }

        if number_of_segments == 0 {
            return None;
        }

        let values = s.read_array16::<T>(number_of_segments)?;

        // 'The number of termination values that need to be included is table-specific.
        // The value that indicates binary search termination is 0xFFFF.'
        let mut len = number_of_segments;
        if values.last()?.is_termination() {
            len = len.checked_sub(1)?;
        }

        Some(BinarySearchTable {
            len: NonZeroU16::new(len)?,
            values,
        })
    }

    fn get(&self, key: GlyphId) -> Option<T> {
        let mut min = 0;
        let mut max = (self.len.get() as isize) - 1;
        while min <= max {
            let mid = (min + max) / 2;
            let v = self.values.get(mid as u16)?;
            match v.contains(key) {
                core::cmp::Ordering::Less    => max = mid - 1,
                core::cmp::Ordering::Greater => min = mid + 1,
                core::cmp::Ordering::Equal   => return Some(v),
            }
        }

        None
    }
}


trait BinarySearchValue: FromData {
    fn is_termination(&self) -> bool;
    fn contains(&self, glyph_id: GlyphId) -> core::cmp::Ordering;
}


#[derive(Clone, Copy, Debug)]
struct LookupSegment {
    last_glyph: u16,
    first_glyph: u16,
    value: u16,
}

impl FromData for LookupSegment {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(LookupSegment {
            last_glyph: s.read::<u16>()?,
            first_glyph: s.read::<u16>()?,
            value: s.read::<u16>()?,
        })
    }
}

impl BinarySearchValue for LookupSegment {
    #[inline]
    fn is_termination(&self) -> bool {
        self.last_glyph == 0xFFFF && self.first_glyph == 0xFFFF
    }

    #[inline]
    fn contains(&self, id: GlyphId) -> core::cmp::Ordering {
        if id.0 < self.first_glyph {
            core::cmp::Ordering::Less
        } else if id.0 <= self.last_glyph {
            core::cmp::Ordering::Equal
        } else {
            core::cmp::Ordering::Greater
        }
    }
}


#[derive(Clone, Copy, Debug)]
struct LookupSingle {
    glyph: u16,
    value: u16,
}

impl FromData for LookupSingle {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(LookupSingle {
            glyph: s.read::<u16>()?,
            value: s.read::<u16>()?,
        })
    }
}

impl BinarySearchValue for LookupSingle {
    #[inline]
    fn is_termination(&self) -> bool {
        self.glyph == 0xFFFF
    }

    #[inline]
    fn contains(&self, id: GlyphId) -> core::cmp::Ordering {
        id.0.cmp(&self.glyph)
    }
}
