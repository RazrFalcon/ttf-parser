//! The [GDEF](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef)
//! table parsing primitives.

use std::convert::{TryFrom, TryInto};
use std::ops::Range;

use crate::stream::{Stream, FromData};


/// A [glyph class](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#glyph-class-definition-table).
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum GlyphClass {
    Base      = 1,
    Ligature  = 2,
    Mark      = 3,
    Component = 4,
}

impl TryFrom<u16> for GlyphClass {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(GlyphClass::Base),
            2 => Ok(GlyphClass::Ligature),
            3 => Ok(GlyphClass::Mark),
            4 => Ok(GlyphClass::Component),
            _ => Err(()),
        }
    }
}


struct ClassRangeRecord {
    range: Range<u16>,
    class: u16,
}

impl FromData for ClassRangeRecord {
    fn parse(data: &[u8]) -> Self {
        let mut s = Stream::new(data);
        ClassRangeRecord {
            range: s.read_u16()..s.read_u16(),
            class: s.read_u16(),
        }
    }
}


/// Handle to a `GDEF` table.
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Table<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) number_of_glyphs: u16,
}

impl<'a> Table<'a> {
    /// Returns glyph's class.
    ///
    /// Returns `None` when font doesn't have such glyph ID.
    pub fn glyph_class(&self, glyph_id: u16) -> Option<GlyphClass> {
        const GLYPH_CLASS_DEF_OFFSET_OFFSET: usize = 4;
        self.parse_glyph_class_def_table(glyph_id, GLYPH_CLASS_DEF_OFFSET_OFFSET)
            .and_then(|c| c.try_into().ok())
    }

    /// Returns glyph's mark attachment class.
    ///
    /// Returns `None` when font doesn't have such glyph ID.
    pub fn glyph_mark_attachment_class(&self, glyph_id: u16) -> Option<u16> {
        const MARK_ATTACH_CLASS_DEF_OFFSET_OFFSET: usize = 10;
        self.parse_glyph_class_def_table(glyph_id, MARK_ATTACH_CLASS_DEF_OFFSET_OFFSET)
    }

    fn parse_glyph_class_def_table(&self, glyph_id: u16, offset: usize) -> Option<u16> {
        if glyph_id >= self.number_of_glyphs {
            return None;
        }

        let offset: u16 = Stream::read_at(self.data, offset);
        Self::parse_glyph_class_table(&self.data[offset as usize ..], glyph_id)
    }

    fn parse_glyph_class_table(data: &[u8], glyph_id: u16) -> Option<u16> {
        let mut s = Stream::new(data);
        let class_format = s.read_u16();
        match class_format {
            1 => Self::parse_glyph_class_table_1(&mut s, glyph_id),
            2 => Self::parse_glyph_class_table_2(&mut s, glyph_id),
            _ => None,
        }
    }

    fn parse_glyph_class_table_1(s: &mut Stream, glyph_id: u16) -> Option<u16> {
        let start_glyph_id = s.read_u16();
        let glyph_count = s.read_u16() as usize;
        let class_values = s.read_array::<u16>(glyph_count);

        // Prevent overflow.
        if glyph_id < start_glyph_id {
            return None;
        }

        class_values.get((glyph_id - start_glyph_id) as usize)
    }

    fn parse_glyph_class_table_2(s: &mut Stream, glyph_id: u16) -> Option<u16> {
        let class_range_count = s.read_u16() as usize;
        let records = s.read_array::<ClassRangeRecord>(class_range_count);
        for record in records {
            if record.range.contains(&glyph_id) {
                return Some(record.class);
            }
        }

        None
    }
}
