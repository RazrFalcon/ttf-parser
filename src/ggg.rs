//! Common types for GDEF, GPOS and GSUB tables.

use crate::GlyphId;
use crate::parser::*;


#[derive(Clone, Copy)]
struct RangeRecord {
    start_glyph_id: GlyphId,
    end_glyph_id: GlyphId,
    value: u16,
}

impl RangeRecord {
    fn range(&self) -> core::ops::RangeInclusive<GlyphId> {
        self.start_glyph_id..=self.end_glyph_id
    }
}

impl FromData for RangeRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(RangeRecord {
            start_glyph_id: s.read::<GlyphId>()?,
            end_glyph_id: s.read::<GlyphId>()?,
            value: s.read::<u16>()?,
        })
    }
}


/// A [Coverage Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-table).
#[derive(Clone, Copy, Debug)]
pub(crate) struct CoverageTable<'a> {
    data: &'a [u8],
}

impl<'a> CoverageTable<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        CoverageTable { data }
    }

    pub fn contains(&self, glyph_id: GlyphId) -> bool {
        let mut s = Stream::new(self.data);
        let format: u16 = try_opt_or!(s.read(), false);

        match format {
            1 => {
                let count = try_opt_or!(s.read::<u16>(), false);
                s.read_array16::<GlyphId>(count).unwrap().binary_search(&glyph_id).is_some()
            }
            2 => {
                let count = try_opt_or!(s.read::<u16>(), false);
                let records = try_opt_or!(s.read_array16::<RangeRecord>(count), false);
                records.into_iter().any(|r| r.range().contains(&glyph_id))
            }
            _ => false,
        }
    }
}


/// A value of [Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table).
#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub struct Class(pub u16);

impl FromData for Class {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(Class)
    }
}


/// A [Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table).
#[derive(Clone, Copy)]
pub(crate) struct ClassDefinitionTable<'a> {
    data: &'a [u8],
}

impl<'a> ClassDefinitionTable<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        ClassDefinitionTable { data }
    }

    /// Any glyph not included in the range of covered glyph IDs automatically belongs to Class 0.
    pub fn get(&self, glyph_id: GlyphId) -> Class {
        self.get_impl(glyph_id).unwrap_or(Class(0))
    }

    fn get_impl(&self, glyph_id: GlyphId) -> Option<Class> {
        let mut s = Stream::new(self.data);
        let format: u16 = s.read()?;
        match format {
            1 => {
                let start_glyph_id: GlyphId = s.read()?;

                // Prevent overflow.
                if glyph_id < start_glyph_id {
                    return None;
                }

                let count: u16 = s.read()?;
                let classes = s.read_array16::<Class>(count)?;
                classes.get(glyph_id.0 - start_glyph_id.0)
            }
            2 => {
                let count: u16 = s.read()?;
                let records = s.read_array16::<RangeRecord>(count)?;
                records.into_iter().find(|r| r.range().contains(&glyph_id))
                    .map(|record| Class(record.value))
            }
            _ => None,
        }
    }
}
