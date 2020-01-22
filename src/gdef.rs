// https://docs.microsoft.com/en-us/typography/opentype/spec/gdef

use crate::{Font, GlyphId, Result};
use crate::parser::{Stream, SafeStream, FromData, TrySlice, Offset, Offset16, Offset32};
use crate::raw::gdef as raw;


/// A [glyph class](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#glyph-class-definition-table).
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum GlyphClass {
    Base      = 1,
    Ligature  = 2,
    Mark      = 3,
    Component = 4,
}


/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#coverage-table
struct CoverageTable<'a> {
    data: &'a [u8],
}

impl<'a> CoverageTable<'a> {
    fn new(data: &'a [u8]) -> Self {
        CoverageTable { data }
    }

    fn contains(&self, glyph_id: GlyphId) -> bool {
        let mut s = Stream::new(self.data);
        let format: u16 = match s.read() {
            Ok(v) => v,
            Err(_) => return false,
        };

        match format {
            1 => {
                s.read_array16::<GlyphId>().unwrap().binary_search(&glyph_id).is_some()
            }
            2 => {
                let records = s.read_array16::<raw::RangeRecord>().unwrap();
                records.into_iter().any(|r| r.range().contains(&glyph_id))
            }
            _ => false,
        }
    }
}


/// A value of [Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Class(pub u16);

impl FromData for Class {
    fn parse(data: &[u8]) -> Self {
        Class(SafeStream::new(data).read())
    }
}


/// https://docs.microsoft.com/en-us/typography/opentype/spec/chapter2#class-definition-table
struct ClassDefinitionTable<'a> {
    data: &'a [u8],
}

impl<'a> ClassDefinitionTable<'a> {
    fn new(data: &'a [u8]) -> Self {
        ClassDefinitionTable { data }
    }

    /// Any glyph not included in the range of covered glyph IDs automatically belongs to Class 0.
    fn get(&self, glyph_id: GlyphId) -> Result<Class> {
        let mut s = Stream::new(self.data);
        let format: u16 = s.read()?;
        match format {
            1 => {
                let start_glyph_id: GlyphId = s.read()?;

                // Prevent overflow.
                if glyph_id < start_glyph_id {
                    return Ok(Class(0));
                }

                let classes = s.read_array16::<Class>()?;
                Ok(classes.get(glyph_id.0 - start_glyph_id.0).unwrap_or(Class(0)))
            }
            2 => {
                let records = s.read_array16::<raw::ClassRangeRecord>()?;
                Ok(match records.into_iter().find(|r| r.range().contains(&glyph_id)) {
                    Some(record) => Class(record.class()),
                    None => Class(0),
                })
            }
            _ => Ok(Class(0)),
        }
    }
}


impl<'a> Font<'a> {
    /// Checks that font has
    /// [Glyph Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#glyph-class-definition-table).
    pub fn has_glyph_classes(&self) -> bool {
        if let Ok(table) = self.gdef {
            if let Some(offset) = table.glyph_class_def_offset() {
                return table.data.try_slice_from(offset).is_ok();
            }
        }

        false
    }

    /// Returns glyph's class according to
    /// [Glyph Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#glyph-class-definition-table).
    ///
    /// Returns `Ok(None)` when *Glyph Class Definition Table* is not set
    /// or glyph class is not set or invalid.
    pub fn glyph_class(&self, glyph_id: GlyphId) -> Result<Option<GlyphClass>> {
        let table = self.gdef?;
        let data = match table.glyph_class_def_offset() {
            Some(offset) => table.data.try_slice_from(offset)?,
            None => return Ok(None),
        };

        match ClassDefinitionTable::new(data).get(glyph_id)?.0 {
            1 => Ok(Some(GlyphClass::Base)),
            2 => Ok(Some(GlyphClass::Ligature)),
            3 => Ok(Some(GlyphClass::Mark)),
            4 => Ok(Some(GlyphClass::Component)),
            _ => Ok(None),
        }
    }

    /// Returns glyph's mark attachment class according to
    /// [Mark Attachment Class Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#mark-attachment-class-definition-table).
    ///
    /// All glyphs not assigned to a class fall into Class 0.
    pub fn glyph_mark_attachment_class(&self, glyph_id: GlyphId) -> Result<Class> {
        let table = self.gdef?;
        match table.mark_attach_class_def_offset() {
            Some(offset) => {
                let data = table.data.try_slice_from(offset)?;
                let table = ClassDefinitionTable::new(data);
                table.get(glyph_id)
            }
            None => Ok(Class(0)),
        }
    }

    /// Checks that glyph is a mark according to
    /// [Mark Glyph Sets Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#mark-glyph-sets-table).
    ///
    /// `set_index` allows checking a specific glyph coverage set.
    /// Otherwise all sets will be checked.
    ///
    /// Returns `Ok(false)` when *Mark Glyph Sets Table* is not set.
    pub fn is_mark_glyph(&self, glyph_id: GlyphId, set_index: Option<u32>) -> Result<bool> {
        let table = self.gdef?;

        // `markGlyphSetsDefOffset` is present only in table version >= 1.2
        if !(table.major_version() == 1 && table.minor_version() == 2) {
            return Ok(false);
        }

        // Offset can be NULL.
        let offset: Option<Offset16>
            = Stream::read_at(table.data, raw::MARK_GLYPH_SETS_DEF_OFFSET_OFFSET)?;
        let offset = match offset {
            Some(v) => v,
            None => return Ok(false),
        };

        let data = &table.data[offset.to_usize()..];
        let mut s = Stream::new(data);
        let format: u16 = s.read()?;
        if format != 1 {
            // Unsupported version.
            return Ok(false);
        }

        let offsets = s.read_array16::<Offset32>()?;

        if let Some(set_index) = set_index {
            if let Some(offset) = offsets.get(set_index) {
                let table = CoverageTable::new(data.try_slice_from(offset)?);
                if table.contains(glyph_id) {
                    return Ok(true);
                }
            }
        } else {
            for offset in s.read_array16::<Offset32>()? {
                let table = CoverageTable::new(data.try_slice_from(offset)?);
                if table.contains(glyph_id) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }
}
