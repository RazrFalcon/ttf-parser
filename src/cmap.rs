// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap

use std::convert::TryInto;
use std::ops::Range;

use crate::parser::{Stream, FromData, LazyArray, Offset32, SafeStream, TrySlice};
use crate::{Font, GlyphId, TableName, Result, Error, PlatformId};

impl<'a> Font<'a> {
    /// Resolves Glyph ID for code point.
    ///
    /// Returns `Error::NoGlyph` instead of `0` when glyph is not found.
    ///
    /// All subtable formats except Mixed Coverage (8) are supported.
    pub fn glyph_index(&self, c: char) -> Result<GlyphId> {
        let data = self.cmap.ok_or_else(|| Error::TableMissing(TableName::CharacterToGlyphIndexMapping))?;
        let mut s = Stream::new(data);
        s.skip::<u16>(); // version
        let records: LazyArray<EncodingRecord> = s.read_array16()?;
        for record in records {
            let subtable_data = data.try_slice(record.offset as usize..data.len())?;
            let mut s = Stream::new(subtable_data);
            let format = match parse_format(s.read()?) {
                Some(format) => format,
                None => continue,
            };

            if !is_unicode_encoding(format, record.platform_id, record.encoding_id) {
                continue;
            }

            let c = c as u32;
            let glyph = match format {
                Format::ByteEncodingTable => {
                    parse_byte_encoding_table(&mut s, c)
                }
                Format::HighByteMappingThroughTable => {
                    parse_high_byte_mapping_through_table(subtable_data, c)
                }
                Format::SegmentMappingToDeltaValues => {
                    parse_segment_mapping_to_delta_values(subtable_data, c)
                }
                Format::TrimmedTableMapping => {
                    parse_trimmed_table_mapping(&mut s, c)
                }
                Format::MixedCoverage => {
                    // Unsupported.
                    continue;
                }
                Format::TrimmedArray => {
                    parse_trimmed_array(&mut s, c)
                }
                Format::SegmentedCoverage | Format::ManyToOneRangeMappings => {
                    parse_segmented_coverage(&mut s, c, format)
                }
                Format::UnicodeVariationSequences => {
                    // This subtable is used only by glyph_variation_index().
                    continue;
                }
            };

            if let Ok(id) = glyph {
                return Ok(GlyphId(id));
            }
        }

        Err(Error::NoGlyph)
    }

    /// Resolves a variation of a Glyph ID from two code points.
    ///
    /// Implemented according to
    /// [Unicode Variation Sequences](
    /// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences).
    ///
    /// Returns `Error::NoGlyph` instead of `0` when glyph is not found.
    pub fn glyph_variation_index(&self, c: char, variation: char) -> Result<GlyphId> {
        let data = self.cmap.ok_or_else(|| Error::TableMissing(TableName::CharacterToGlyphIndexMapping))?;
        let mut s = Stream::new(data);
        s.skip::<u16>(); // version
        let records: LazyArray<EncodingRecord> = s.read_array16()?;
        for record in records {
            let subtable_data = data.try_slice(record.offset as usize..data.len())?;
            let mut s = Stream::new(subtable_data);
            let format = match parse_format(s.read()?) {
                Some(format) => format,
                None => continue,
            };

            if format != Format::UnicodeVariationSequences {
                continue;
            }

            return self.parse_unicode_variation_sequences(subtable_data, c, variation as u32);
        }

        Err(Error::NoGlyph)
    }

    fn parse_unicode_variation_sequences(
        &self,
        data: &[u8],
        c: char,
        variation: u32,
    ) -> Result<GlyphId> {
        let cp = c as u32;

        let mut s = Stream::new(data);
        s.skip::<u16>(); // format
        s.skip::<u32>(); // length
        let records: LazyArray<VariationSelectorRecord> = s.read_array32()?;

        let record = records.binary_search_by(|v| v.variation.cmp(&variation)).ok_or(Error::NoGlyph)?;

        if let Some(offset) = record.default_uvs_offset {
            let data = data.try_slice(offset.0 as usize..data.len())?;
            let mut s = Stream::new(data);
            let ranges: LazyArray<UnicodeRangeRecord> = s.read_array32()?;
            for range in ranges {
                if range.contains(c) {
                    // This is a default glyph.
                    return self.glyph_index(c);
                }
            }
        }

        if let Some(offset) = record.non_default_uvs_offset {
            let data = data.try_slice(offset.0 as usize..data.len())?;
            let mut s = Stream::new(data);
            let uvs_mappings: LazyArray<UVSMappingRecord> = s.read_array32()?;
            if let Some(mapping) = uvs_mappings.binary_search_by(|v| v.unicode_value.cmp(&cp)) {
                return Ok(mapping.glyph);
            }
        }

        Err(Error::NoGlyph)
    }
}


struct EncodingRecord {
    platform_id: PlatformId,
    encoding_id: u16,
    offset: u32,
}

impl FromData for EncodingRecord {
    const SIZE: usize = 8; // u16 + u16 + u32

    #[inline]
    fn parse(s: &mut SafeStream) -> Self {
        EncodingRecord {
            // Fallback to `Custom` on error. This isn't that correct,
            // but since `FromData` can't return an error this is the next best thing.
            platform_id: s.read::<u16>().try_into().unwrap_or(PlatformId::Custom),
            encoding_id: s.read(),
            offset: s.read(),
        }
    }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-0-byte-encoding-table
fn parse_byte_encoding_table(s: &mut Stream, code_point: u32) -> Result<u16> {
    let length: u16 = s.read()?;
    s.skip::<u16>(); // language

    if code_point < (length as u32) {
        s.skip_len(code_point);
        Ok(s.read::<u8>()? as u16)
    } else {
        Err(Error::NoGlyph)
    }
}

// This table has a pretty complex parsing algorithm.
// A detailed explanation can be found here:
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
// https://github.com/fonttools/fonttools/blob/a360252709a3d65f899915db0a5bd753007fdbb7/Lib/fontTools/ttLib/tables/_c_m_a_p.py#L360
fn parse_high_byte_mapping_through_table(data: &[u8], code_point: u32) -> Result<u16> {
    // This subtable supports code points only in a u16 range.
    if code_point > 0xffff {
        return Err(Error::NoGlyph);
    }

    let code_point = code_point as u16;
    let high_byte = (code_point >> 8) as u16;
    let low_byte = (code_point & 0x00FF) as u16;

    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let sub_header_keys: LazyArray<u16> = s.read_array(256_u32)?;
    // The maximum index in a sub_header_keys is a sub_headers count.
    let sub_headers_count = sub_header_keys.into_iter().map(|n| n / 8).max()
        .ok_or_else(|| Error::NoGlyph)? + 1;

    // Remember sub_headers offset before reading. Will be used later.
    let sub_headers_offset = s.offset();
    let sub_headers: LazyArray<SubHeaderRecord> = s.read_array(sub_headers_count)?;

    let i = if code_point < 0xff {
        // 'SubHeader 0 is special: it is used for single-byte character codes.'
        0
    } else {
        // 'Array that maps high bytes to subHeaders: value is subHeader index × 8.'
        sub_header_keys.get(high_byte).ok_or_else(|| Error::NoGlyph)? / 8
    };

    let sub_header = sub_headers.get(i).ok_or_else(|| Error::NoGlyph)?;

    let range_end = sub_header.first_code + sub_header.entry_count;
    if low_byte < sub_header.first_code || low_byte > range_end {
        return Err(Error::NoGlyph);
    }

    // SubHeaderRecord::id_range_offset points to SubHeaderRecord::first_code
    // in the glyphIndexArray. So we have to advance to our code point.
    let index_offset = (low_byte - sub_header.first_code) as usize * u16::SIZE;

    // 'The value of the idRangeOffset is the number of bytes
    // past the actual location of the idRangeOffset'.
    let offset =
          sub_headers_offset
        // Advance to required subheader.
        + SubHeaderRecord::SIZE * (i + 1) as usize
        // Move back to idRangeOffset start.
        - u16::SIZE
        // Use defined offset.
        + sub_header.id_range_offset as usize
        // Advance to required index in the glyphIndexArray.
        + index_offset;

    let glyph: u16 = Stream::read_at(data, offset)?;
    if glyph == 0 {
        return Err(Error::NoGlyph);
    }

    let glyph = ((glyph as i32 + sub_header.id_delta as i32) % 65536) as u16;
    Ok(glyph)
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-4-segment-mapping-to-delta-values
fn parse_segment_mapping_to_delta_values(data: &[u8], code_point: u32) -> Result<u16> {
    // This subtable supports code points only in a u16 range.
    if code_point > 0xffff {
        return Err(Error::NoGlyph);
    }

    let code_point = code_point as u16;

    let mut s = Stream::new(data);
    s.skip_len(6 as u32); // format + length + language
    let seg_count_x2: u16 = s.read()?;
    if seg_count_x2 < 2 {
        return Err(Error::NoGlyph);
    }

    let seg_count = seg_count_x2 / 2;
    s.skip_len(6 as u32); // searchRange + entrySelector + rangeShift
    let end_codes: LazyArray<u16> = s.read_array(seg_count)?;
    s.skip::<u16>(); // reservedPad
    let start_codes: LazyArray<u16> = s.read_array(seg_count)?;
    let id_deltas: LazyArray<i16> = s.read_array(seg_count)?;
    let id_range_offset_pos = s.offset();
    let id_range_offsets: LazyArray<u16> = s.read_array(seg_count)?;

    // A custom binary search.
    let mut start = 0;
    let mut end = seg_count;
    while end > start {
        let index = (start + end) / 2;
        let end_value = end_codes.get(index).ok_or_else(|| Error::NoGlyph)?;
        if end_value >= code_point {
            let start_value = start_codes.get(index).ok_or_else(|| Error::NoGlyph)?;
            if start_value > code_point {
                end = index;
            } else {
                let id_range_offset = id_range_offsets.get(index).ok_or_else(|| Error::NoGlyph)?;
                let id_delta = id_deltas.get(index).ok_or_else(|| Error::NoGlyph)?;
                if id_range_offset == 0 {
                    return Ok(code_point.wrapping_add(id_delta as u16));
                }

                let delta = (code_point as u32 - start_value as u32) * 2;
                // Check for overflow.
                if delta > std::u16::MAX as u32 {
                    return Err(Error::NoGlyph);
                }
                // `delta` must be u16.
                let delta = delta as u16;

                let id_range_offset_pos = (id_range_offset_pos + index as usize * 2) as u16;
                let pos = id_range_offset_pos.wrapping_add(delta);
                let pos = pos.wrapping_add(id_range_offset);
                let glyph_array_value: u16 = Stream::read_at(data, pos as usize)?;
                if glyph_array_value == 0 {
                    return Err(Error::NoGlyph);
                }

                let glyph_id = (glyph_array_value as i16).wrapping_add(id_delta);
                return Ok(glyph_id as u16);
            }
        } else {
            start = index + 1;
        }
    }

    return Err(Error::NoGlyph);
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-6-trimmed-table-mapping
fn parse_trimmed_table_mapping(s: &mut Stream, code_point: u32) -> Result<u16> {
    // This subtable supports code points only in a u16 range.
    if code_point > 0xffff {
        return Err(Error::NoGlyph);
    }

    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let first_code_point: u16 = s.read()?;
    let glyphs: LazyArray<u16> = s.read_array16()?;

    let code_point = code_point as u16;

    // Check for overflow.
    if code_point < first_code_point {
        return Err(Error::NoGlyph);
    }

    let idx = code_point - first_code_point;
    glyphs.get(idx).ok_or_else(|| Error::NoGlyph)
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-10-trimmed-array
fn parse_trimmed_array(s: &mut Stream, code_point: u32) -> Result<u16> {
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let first_code_point: u32 = s.read()?;
    let glyphs: LazyArray<u16> = s.read_array32()?;

    // Check for overflow.
    if code_point < first_code_point {
        return Err(Error::NoGlyph);
    }

    let idx = code_point - first_code_point;
    glyphs.get(idx).ok_or_else(|| Error::NoGlyph)
}

// + ManyToOneRangeMappings
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-13-many-to-one-range-mappings
fn parse_segmented_coverage(s: &mut Stream, code_point: u32, format: Format) -> Result<u16> {
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let groups: LazyArray<SequentialMapGroup> = s.read_array32()?;
    for group in groups {
        if group.char_code_range.contains(&code_point) {
            if format == Format::SegmentedCoverage {
                let id = group.start_glyph_id + code_point - group.char_code_range.start;
                return Ok(id as u16);
            } else {
                return Ok(group.start_glyph_id as u16);
            }
        }
    }

    Err(Error::NoGlyph)
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


#[derive(Clone, Copy, Debug)]
struct SubHeaderRecord {
    first_code: u16,
    entry_count: u16,
    id_delta: i16,
    id_range_offset: u16,
}

impl FromData for SubHeaderRecord {
    fn parse(s: &mut SafeStream) -> Self {
        SubHeaderRecord {
            first_code: s.read(),
            entry_count: s.read(),
            id_delta: s.read(),
            id_range_offset: s.read(),
        }
    }
}


// Also, the same as ConstantMapGroup.
struct SequentialMapGroup {
    char_code_range: Range<u32>,
    start_glyph_id: u32,
}

impl FromData for SequentialMapGroup {
    fn parse(s: &mut SafeStream) -> Self {
        let start: u32 = s.read();
        let mut end: u32 = s.read();

        if end != std::u32::MAX {
            // +1 makes the upper bound inclusive.
            end += 1;
        }

        SequentialMapGroup {
            char_code_range: start..end,
            start_glyph_id: s.read(),
        }
    }
}


struct VariationSelectorRecord {
    variation: u32,
    default_uvs_offset: Option<Offset32>,
    non_default_uvs_offset: Option<Offset32>,
}

impl FromData for VariationSelectorRecord {
    const SIZE: usize = 3 + Offset32::SIZE + Offset32::SIZE; // variation_selector is u24

    fn parse(s: &mut SafeStream) -> Self {
        VariationSelectorRecord {
            variation: s.read_u24(),
            default_uvs_offset: s.read(),
            non_default_uvs_offset: s.read(),
        }
    }
}


struct UnicodeRangeRecord {
    start_unicode_value: u32,
    additional_count: u8,
}

impl UnicodeRangeRecord {
    fn contains(&self, c: char) -> bool {
        let end = self.start_unicode_value + self.additional_count as u32;
        self.start_unicode_value >= (c as u32) && (c as u32) < end
    }
}

impl FromData for UnicodeRangeRecord {
    const SIZE: usize = 3 + 1; // start_unicode_value is u24.

    fn parse(s: &mut SafeStream) -> Self {
        UnicodeRangeRecord {
            start_unicode_value: s.read_u24(),
            additional_count: s.read(),
        }
    }
}


struct UVSMappingRecord {
    unicode_value: u32,
    glyph: GlyphId,
}

impl FromData for UVSMappingRecord {
    const SIZE: usize = 3 + GlyphId::SIZE; // unicode_value is u24

    fn parse(s: &mut SafeStream) -> Self {
        UVSMappingRecord {
            unicode_value: s.read_u24(),
            glyph: s.read(),
        }
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
