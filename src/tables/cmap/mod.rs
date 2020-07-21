// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap

use crate::{GlyphId, PlatformId};
use crate::parser::{Stream, FromData, LazyArray16, NumFrom};

mod format0;
mod format2;
mod format4;
mod format6;
mod format10;
mod format12;
mod format13;
mod format14;

#[derive(Clone, Copy)]
struct EncodingRecord {
    platform_id: u16,
    encoding_id: u16,
    offset: u32,
}

impl FromData for EncodingRecord {
    const SIZE: usize = 8;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(EncodingRecord {
            platform_id: s.read::<u16>()?,
            encoding_id: s.read::<u16>()?,
            offset: s.read::<u32>()?,
        })
    }
}


#[derive(Clone, Copy)]
pub struct Table<'a> {
    data: &'a [u8],
    records: LazyArray16<'a, EncodingRecord>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        s.skip::<u16>(); // version
        let count: u16 = s.read()?;
        let records = s.read_array16::<EncodingRecord>(count)?;

        Some(Table {
            data,
            records,
        })
    }
}

pub fn glyph_index(table: &Table, c: char) -> Option<GlyphId> {
    for record in table.records {
        let subtable_data = table.data.get(usize::num_from(record.offset)..)?;
        let mut s = Stream::new(subtable_data);
        let format = match parse_format(s.read::<u16>()?) {
            Some(format) => format,
            None => continue,
        };

        let platform_id = match PlatformId::from_u16(record.platform_id) {
            Some(v) => v,
            None => continue,
        };

        if !is_unicode_encoding(format, platform_id, record.encoding_id) {
            continue;
        }

        let c = u32::from(c);
        let glyph = match format {
            Format::ByteEncodingTable => {
                format0::parse(s, c)
            }
            Format::HighByteMappingThroughTable => {
                format2::parse(subtable_data, c)
            }
            Format::SegmentMappingToDeltaValues => {
                format4::parse(subtable_data, c)
            }
            Format::TrimmedTableMapping => {
                format6::parse(s, c)
            }
            Format::MixedCoverage => {
                // Unsupported.
                continue;
            }
            Format::TrimmedArray => {
                format10::parse(s, c)
            }
            Format::SegmentedCoverage => {
                format12::parse(s, c)
            }
            Format::ManyToOneRangeMappings => {
                format13::parse(s, c)
            }
            Format::UnicodeVariationSequences => {
                // This subtable is used only by glyph_variation_index().
                continue;
            }
        };

        if let Some(id) = glyph {
            return Some(GlyphId(id));
        }
    }

    None
}

pub fn glyph_variation_index(table: &Table, c: char, variation: char) -> Option<GlyphId> {
    for record in table.records {
        let subtable_data = table.data.get(usize::num_from(record.offset)..)?;
        let mut s = Stream::new(subtable_data);
        let format = match parse_format(s.read::<u16>()?) {
            Some(format) => format,
            None => continue,
        };

        if format != Format::UnicodeVariationSequences {
            continue;
        }

        return format14::parse(table, subtable_data, c, u32::from(variation));
    }

    None
}


#[derive(Clone, Copy, PartialEq, Debug)]
enum Format {
    ByteEncodingTable = 0,
    HighByteMappingThroughTable = 2,
    SegmentMappingToDeltaValues = 4,
    TrimmedTableMapping = 6,
    MixedCoverage = 8,
    TrimmedArray = 10,
    SegmentedCoverage = 12,
    ManyToOneRangeMappings = 13,
    UnicodeVariationSequences = 14,
}

fn parse_format(v: u16) -> Option<Format> {
    match v {
         0 => Some(Format::ByteEncodingTable),
         2 => Some(Format::HighByteMappingThroughTable),
         4 => Some(Format::SegmentMappingToDeltaValues),
         6 => Some(Format::TrimmedTableMapping),
         8 => Some(Format::MixedCoverage),
        10 => Some(Format::TrimmedArray),
        12 => Some(Format::SegmentedCoverage),
        13 => Some(Format::ManyToOneRangeMappings),
        14 => Some(Format::UnicodeVariationSequences),
        _ => None,
    }
}

#[inline]
fn is_unicode_encoding(format: Format, platform_id: PlatformId, encoding_id: u16) -> bool {
    // https://docs.microsoft.com/en-us/typography/opentype/spec/name#windows-encoding-ids
    const WINDOWS_UNICODE_BMP_ENCODING_ID: u16 = 1;
    const WINDOWS_UNICODE_FULL_REPERTOIRE_ENCODING_ID: u16 = 10;

    match platform_id {
        PlatformId::Unicode => true,
        PlatformId::Windows if encoding_id == WINDOWS_UNICODE_BMP_ENCODING_ID => true,
        PlatformId::Windows => {
            // "Fonts that support Unicode supplementary-plane characters (U+10000 to U+10FFFF)
            // on the Windows platform must have a format 12 subtable for platform ID 3,
            // encoding ID 10."
               encoding_id == WINDOWS_UNICODE_FULL_REPERTOIRE_ENCODING_ID
            && format == Format::SegmentedCoverage
        }
        _ => false,
    }
}
