use core::convert::{TryFrom, TryInto};
use core::ops::Range;

use crate::parser::{Stream, FromData, LazyArray};
use crate::{GlyphId, Font, TableName, Result, Error};


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
    type Error = Error;

    fn try_from(value: u16) -> Result<Self> {
        match value {
            1 => Ok(GlyphClass::Base),
            2 => Ok(GlyphClass::Ligature),
            3 => Ok(GlyphClass::Mark),
            4 => Ok(GlyphClass::Component),
            _ => Err(Error::InvalidGlyphClass(value)),
        }
    }
}


struct ClassRangeRecord {
    range: Range<GlyphId>,
    class: u16,
}

impl FromData for ClassRangeRecord {
    fn parse(data: &[u8]) -> Self {
        let mut s = Stream::new(data);
        ClassRangeRecord {
            // Make the upper bound inclusive.
            range: s.read()..GlyphId(s.read::<u16>() + 1),
            class: s.read(),
        }
    }
}


impl<'a> Font<'a> {
    /// Returns glyph's class.
    pub fn glyph_class(&self, glyph_id: GlyphId) -> Result<GlyphClass> {
        const GLYPH_CLASS_DEF_OFFSET_OFFSET: usize = 4;
        self.parse_glyph_class_def_table(glyph_id, GLYPH_CLASS_DEF_OFFSET_OFFSET)
            .and_then(|c| c.try_into())
    }

    /// Returns glyph's mark attachment class.
    pub fn glyph_mark_attachment_class(&self, glyph_id: GlyphId) -> Result<u16> {
        const MARK_ATTACH_CLASS_DEF_OFFSET_OFFSET: usize = 10;
        self.parse_glyph_class_def_table(glyph_id, MARK_ATTACH_CLASS_DEF_OFFSET_OFFSET)
    }

    fn parse_glyph_class_def_table(&self, glyph_id: GlyphId, offset: usize) -> Result<u16> {
        self.check_glyph_id(glyph_id)?;
        let data = self.table_data(TableName::GlyphDefinition)?;
        let offset: u16 = Stream::read_at(data, offset);
        Self::parse_glyph_class_table(&data[offset as usize ..], glyph_id)
    }

    fn parse_glyph_class_table(data: &[u8], glyph_id: GlyphId) -> Result<u16> {
        let mut s = Stream::new(data);
        let class_format: u16 = s.read();
        match class_format {
            1 => Self::parse_glyph_class_table_1(&mut s, glyph_id),
            2 => Self::parse_glyph_class_table_2(&mut s, glyph_id),
            _ => Ok(0),
        }
    }

    fn parse_glyph_class_table_1(s: &mut Stream, glyph_id: GlyphId) -> Result<u16> {
        let start_glyph_id: GlyphId = s.read();
        let glyph_count: u16 = s.read();
        let class_values: LazyArray<u16> = s.read_array(glyph_count);

        // Prevent overflow.
        if glyph_id < start_glyph_id {
            return Ok(0);
        }

        Ok(class_values.get(glyph_id.0 - start_glyph_id.0).unwrap_or(0))
    }

    fn parse_glyph_class_table_2(s: &mut Stream, glyph_id: GlyphId) -> Result<u16> {
        let class_range_count: u16 = s.read();
        let records: LazyArray<ClassRangeRecord> = s.read_array(class_range_count);
        for record in records {
            if record.range.contains(&glyph_id) {
                return Ok(record.class);
            }
        }

        Ok(0)
    }
}
