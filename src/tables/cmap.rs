// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap

use core::convert::TryFrom;

use crate::{GlyphId, PlatformId};
use crate::parser::{Stream, FromData, Offset, Offset32, U24, LazyArray16, NumFrom};


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
            platform_id: s.read()?,
            encoding_id: s.read()?,
            offset: s.read()?,
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
        let records = s.read_array16(count)?;

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
        let format = match parse_format(s.read()?) {
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
                parse_byte_encoding_table(s, c)
            }
            Format::HighByteMappingThroughTable => {
                parse_high_byte_mapping_through_table(subtable_data, c)
            }
            Format::SegmentMappingToDeltaValues => {
                parse_segment_mapping_to_delta_values(subtable_data, c)
            }
            Format::TrimmedTableMapping => {
                parse_trimmed_table_mapping(s, c)
            }
            Format::MixedCoverage => {
                // Unsupported.
                continue;
            }
            Format::TrimmedArray => {
                parse_trimmed_array(s, c)
            }
            Format::SegmentedCoverage | Format::ManyToOneRangeMappings => {
                parse_segmented_coverage(s, c, format)
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
        let format = match parse_format(s.read()?) {
            Some(format) => format,
            None => continue,
        };

        if format != Format::UnicodeVariationSequences {
            continue;
        }

        return parse_unicode_variation_sequences(table, subtable_data, c, u32::from(variation));
    }

    None
}


#[derive(Clone, Copy)]
struct VariationSelectorRecord {
    var_selector: u32,
    default_uvs_offset: Option<Offset32>,
    non_default_uvs_offset: Option<Offset32>,
}

impl FromData for VariationSelectorRecord {
    const SIZE: usize = 11;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(VariationSelectorRecord {
            var_selector: s.read::<U24>()?.0,
            default_uvs_offset: s.read()?,
            non_default_uvs_offset: s.read()?,
        })
    }
}


#[derive(Clone, Copy)]
struct UVSMappingRecord {
    unicode_value: u32,
    glyph_id: GlyphId,
}

impl FromData for UVSMappingRecord {
    const SIZE: usize = 5;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(UVSMappingRecord {
            unicode_value: s.read::<U24>()?.0,
            glyph_id: s.read()?,
        })
    }
}


#[derive(Clone, Copy)]
struct UnicodeRangeRecord {
    start_unicode_value: u32,
    additional_count: u8,
}

impl UnicodeRangeRecord {
    fn contains(&self, c: char) -> bool {
        let end = self.start_unicode_value + u32::from(self.additional_count);
        self.start_unicode_value >= u32::from(c) && u32::from(c) < end
    }
}

impl FromData for UnicodeRangeRecord {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(UnicodeRangeRecord {
            start_unicode_value: s.read::<U24>()?.0,
            additional_count: s.read()?,
        })
    }
}


fn parse_unicode_variation_sequences(
    table: &Table,
    data: &[u8],
    c: char,
    variation: u32,
) -> Option<GlyphId> {
    let cp = u32::from(c);

    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u32>(); // length
    let count: u32 = s.read()?;
    let records = s.read_array32::<VariationSelectorRecord>(count)?;

    let (_, record) = match records.binary_search_by(|v| v.var_selector.cmp(&variation)) {
        Some(v) => v,
        None => return None,
    };

    if let Some(offset) = record.default_uvs_offset {
        let data = data.get(offset.to_usize()..)?;
        let mut s = Stream::new(data);
        let count: u32 = s.read()?;
        let ranges = s.read_array32::<UnicodeRangeRecord>(count)?;
        for range in ranges {
            if range.contains(c) {
                // This is a default glyph.
                return glyph_index(table, c);
            }
        }
    }

    if let Some(offset) = record.non_default_uvs_offset {
        let data = data.get(offset.to_usize()..)?;
        let mut s = Stream::new(data);
        let count: u32 = s.read()?;
        let uvs_mappings = s.read_array32::<UVSMappingRecord>(count)?;
        if let Some((_, mapping)) = uvs_mappings.binary_search_by(|v| v.unicode_value.cmp(&cp)) {
            return Some(mapping.glyph_id);
        }
    }

    None
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-0-byte-encoding-table
fn parse_byte_encoding_table(mut s: Stream, code_point: u32) -> Option<u16> {
    let length: u16 = s.read()?;
    s.skip::<u16>(); // language

    if code_point < u32::from(length) {
        s.advance(usize::num_from(code_point));
        Some(u16::from(s.read::<u8>()?))
    } else {
        None
    }
}

#[derive(Clone, Copy)]
struct SubHeaderRecord {
    first_code: u16,
    entry_count: u16,
    id_delta: i16,
    id_range_offset: u16,
}

impl FromData for SubHeaderRecord {
    const SIZE: usize = 8;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(SubHeaderRecord {
            first_code: s.read()?,
            entry_count: s.read()?,
            id_delta: s.read()?,
            id_range_offset: s.read()?,
        })
    }
}

// This table has a pretty complex parsing algorithm.
// A detailed explanation can be found here:
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
// https://github.com/fonttools/fonttools/blob/a360252709a3d65f899915db0a5bd753007fdbb7/Lib/fontTools/ttLib/tables/_c_m_a_p.py#L360
fn parse_high_byte_mapping_through_table(data: &[u8], code_point: u32) -> Option<u16> {
    // This subtable supports code points only in a u16 range.
    let code_point = u16::try_from(code_point).ok()?;

    let code_point = code_point;
    let high_byte = code_point >> 8;
    let low_byte = code_point & 0x00FF;

    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let sub_header_keys = s.read_array16::<u16>(256)?;
    // The maximum index in a sub_header_keys is a sub_headers count.
    let sub_headers_count = sub_header_keys.into_iter().map(|n| n / 8).max()? + 1;

    // Remember sub_headers offset before reading. Will be used later.
    let sub_headers_offset = s.offset();
    let sub_headers = s.read_array16::<SubHeaderRecord>(sub_headers_count)?;

    let i = if code_point < 0xff {
        // 'SubHeader 0 is special: it is used for single-byte character codes.'
        0
    } else {
        // 'Array that maps high bytes to subHeaders: value is subHeader index × 8.'
        sub_header_keys.get(high_byte)? / 8
    };

    let sub_header = sub_headers.get(i)?;

    let first_code = sub_header.first_code;
    let range_end = first_code.checked_add(sub_header.entry_count)?;
    if low_byte < first_code || low_byte > range_end {
        return None;
    }

    // SubHeaderRecord::id_range_offset points to SubHeaderRecord::first_code
    // in the glyphIndexArray. So we have to advance to our code point.
    let index_offset = usize::from(low_byte.checked_sub(first_code)?) * u16::SIZE;

    // 'The value of the idRangeOffset is the number of bytes
    // past the actual location of the idRangeOffset'.
    let offset =
          sub_headers_offset
        // Advance to required subheader.
        + SubHeaderRecord::SIZE * usize::from(i + 1)
        // Move back to idRangeOffset start.
        - u16::SIZE
        // Use defined offset.
        + usize::from(sub_header.id_range_offset)
        // Advance to required index in the glyphIndexArray.
        + index_offset;

    let glyph: u16 = Stream::read_at(data, offset)?;
    if glyph == 0 {
        return None;
    }

    u16::try_from((i32::from(glyph) + i32::from(sub_header.id_delta)) % 65536).ok()
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-4-segment-mapping-to-delta-values
fn parse_segment_mapping_to_delta_values(data: &[u8], code_point: u32) -> Option<u16> {
    // This subtable supports code points only in a u16 range.
    let code_point = u16::try_from(code_point).ok()?;

    let mut s = Stream::new(data);
    s.advance(6); // format + length + language
    let seg_count_x2: u16 = s.read()?;
    if seg_count_x2 < 2 {
        return None;
    }

    let seg_count = seg_count_x2 / 2;
    s.advance(6); // searchRange + entrySelector + rangeShift

    let end_codes = s.read_array16::<u16>(seg_count)?;
    s.skip::<u16>(); // reservedPad
    let start_codes = s.read_array16::<u16>(seg_count)?;
    let id_deltas = s.read_array16::<i16>(seg_count)?;
    let id_range_offset_pos = s.offset();
    let id_range_offsets = s.read_array16::<u16>(seg_count)?;

    // A custom binary search.
    let mut start = 0;
    let mut end = seg_count;
    while end > start {
        let index = (start + end) / 2;
        let end_value = end_codes.get(index)?;
        if end_value >= code_point {
            let start_value = start_codes.get(index)?;
            if start_value > code_point {
                end = index;
            } else {
                let id_range_offset = id_range_offsets.get(index)?;
                let id_delta = id_deltas.get(index)?;
                if id_range_offset == 0 {
                    return Some(code_point.wrapping_add(id_delta as u16));
                }

                let delta = (u32::from(code_point) - u32::from(start_value)) * 2;
                let delta = u16::try_from(delta).ok()?;

                let id_range_offset_pos = (id_range_offset_pos + usize::from(index) * 2) as u16;
                let pos = id_range_offset_pos.wrapping_add(delta);
                let pos = pos.wrapping_add(id_range_offset);
                let glyph_array_value: u16 = Stream::read_at(data, usize::from(pos))?;
                if glyph_array_value == 0 {
                    return None;
                }

                let glyph_id = (glyph_array_value as i16).wrapping_add(id_delta);
                return u16::try_from(glyph_id).ok();
            }
        } else {
            start = index + 1;
        }
    }

    None
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-6-trimmed-table-mapping
fn parse_trimmed_table_mapping(mut s: Stream, code_point: u32) -> Option<u16> {
    // This subtable supports code points only in a u16 range.
    let code_point = u16::try_from(code_point).ok()?;

    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let first_code_point: u16 = s.read()?;
    let count: u16 = s.read()?;
    let glyphs = s.read_array16::<u16>(count)?;

    let idx = code_point.checked_sub(first_code_point)?;
    glyphs.get(idx)
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-10-trimmed-array
fn parse_trimmed_array(mut s: Stream, code_point: u32) -> Option<u16> {
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let first_code_point: u32 = s.read()?;
    let count: u32 = s.read()?;
    let glyphs = s.read_array32::<u16>(count)?;

    let idx = code_point.checked_sub(first_code_point)?;
    glyphs.get(idx)
}

#[derive(Clone, Copy)]
struct SequentialMapGroup {
    start_char_code: u32,
    end_char_code: u32,
    start_glyph_id: u32,
}

impl FromData for SequentialMapGroup {
    const SIZE: usize = 12;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(SequentialMapGroup {
            start_char_code: s.read()?,
            end_char_code: s.read()?,
            start_glyph_id: s.read()?,
        })
    }
}

// + ManyToOneRangeMappings
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-13-many-to-one-range-mappings
fn parse_segmented_coverage(mut s: Stream, code_point: u32, format: Format) -> Option<u16> {
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let count: u32 = s.read()?;
    let groups = s.read_array32::<SequentialMapGroup>(count)?;
    for group in groups {
        let start_char_code = group.start_char_code;
        if code_point >= start_char_code && code_point <= group.end_char_code {
            let id = if format == Format::SegmentedCoverage {
                group.start_glyph_id.checked_add(code_point)?.checked_sub(start_char_code)?
            } else {
                group.start_glyph_id
            };

            return u16::try_from(id).ok();
        }
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
