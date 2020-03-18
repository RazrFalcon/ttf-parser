//! Common types for GDEF, GPOS and GSUB tables.

use crate::GlyphId;
use crate::parser::*;


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
                s.read_count_and_array16::<GlyphId>().unwrap().binary_search(&glyph_id).is_some()
            }
            2 => {
                let records = try_opt_or!(s.read_count_and_array16::<crate::raw::gdef::RangeRecord>(), false);
                records.into_iter().any(|r| r.range().contains(&glyph_id))
            }
            _ => false,
        }
    }
}


/// A value of [Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table).
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Class(pub u16);

impl FromData for Class {
    fn parse(data: &[u8]) -> Self {
        Class(SafeStream::new(data).read())
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

                let classes = s.read_count_and_array16::<Class>()?;
                classes.get(glyph_id.0 - start_glyph_id.0)
            }
            2 => {
                let records = s.read_count_and_array16::<crate::raw::gdef::ClassRangeRecord>()?;
                records.into_iter().find(|r| r.range().contains(&glyph_id))
                    .map(|record| Class(record.class()))
            }
            _ => None,
        }
    }
}
