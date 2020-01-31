// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap

use crate::parser::{Stream, TrySlice};
use crate::{Font, GlyphId, Result, PlatformId};
use crate::raw::cmap as raw;


impl<'a> Font<'a> {
    /// Resolves Glyph ID for code point.
    ///
    /// Returns `Error::NoGlyph` instead of `0` when glyph is not found.
    ///
    /// All subtable formats except Mixed Coverage (8) are supported.
    pub fn glyph_index(&self, c: char) -> Result<Option<GlyphId>> {
        let data = self.cmap?;
        let mut s = Stream::new(data);
        s.skip::<u16>(); // version
        let records = s.read_array16::<raw::EncodingRecord>()?;
        for record in records {
            let subtable_data = data.try_slice_from(record.offset())?;
            let mut s = Stream::new(subtable_data);
            let format = match parse_format(s.read()?) {
                Some(format) => format,
                None => continue,
            };

            let platform_id = match PlatformId::from_u16(record.platform_id()) {
                Some(v) => v,
                None => continue,
            };

            if !is_unicode_encoding(format, platform_id, record.encoding_id()) {
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

            if let Ok(Some(id)) = glyph {
                return Ok(Some(GlyphId(id)));
            }
        }

        Ok(None)
    }

    /// Resolves a variation of a Glyph ID from two code points.
    ///
    /// Implemented according to
    /// [Unicode Variation Sequences](
    /// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences).
    ///
    /// Returns `Error::NoGlyph` instead of `0` when glyph is not found.
    pub fn glyph_variation_index(&self, c: char, variation: char) -> Result<Option<GlyphId>> {
        let data = self.cmap?;
        let mut s = Stream::new(data);
        s.skip::<u16>(); // version
        let records = s.read_array16::<raw::EncodingRecord>()?;
        for record in records {
            let subtable_data = data.try_slice_from(record.offset())?;
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

        Ok(None)
    }

    fn parse_unicode_variation_sequences(
        &self,
        data: &[u8],
        c: char,
        variation: u32,
    ) -> Result<Option<GlyphId>> {
        let cp = c as u32;

        let mut s = Stream::new(data);
        s.skip::<u16>(); // format
        s.skip::<u32>(); // length
        let records = s.read_array32::<raw::VariationSelectorRecord>()?;

        let (_, record) = match records.binary_search_by(|v| v.var_selector().cmp(&variation)) {
            Some(v) => v,
            None => return Ok(None),
        };

        if let Some(offset) = record.default_uvs_offset() {
            let data = data.try_slice_from(offset)?;
            let mut s = Stream::new(data);
            let ranges = s.read_array32::<raw::UnicodeRangeRecord>()?;
            for range in ranges {
                if range.contains(c) {
                    // This is a default glyph.
                    return self.glyph_index(c);
                }
            }
        }

        if let Some(offset) = record.non_default_uvs_offset() {
            let data = data.try_slice_from(offset)?;
            let mut s = Stream::new(data);
            let uvs_mappings = s.read_array32::<raw::UVSMappingRecord>()?;
            if let Some((_, mapping)) = uvs_mappings.binary_search_by(|v| v.unicode_value().cmp(&cp)) {
                return Ok(Some(mapping.glyph_id()));
            }
        }

        Ok(None)
    }
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-0-byte-encoding-table
fn parse_byte_encoding_table(s: &mut Stream, code_point: u32) -> Result<Option<u16>> {
    let length: u16 = s.read()?;
    s.skip::<u16>(); // language

    if code_point < (length as u32) {
        s.advance(code_point);
        Ok(Some(s.read::<u8>()? as u16))
    } else {
        Ok(None)
    }
}

// This table has a pretty complex parsing algorithm.
// A detailed explanation can be found here:
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
// https://github.com/fonttools/fonttools/blob/a360252709a3d65f899915db0a5bd753007fdbb7/Lib/fontTools/ttLib/tables/_c_m_a_p.py#L360
fn parse_high_byte_mapping_through_table(data: &[u8], code_point: u32) -> Result<Option<u16>> {
    // This subtable supports code points only in a u16 range.
    if code_point > 0xffff {
        return Ok(None);
    }

    let code_point = code_point as u16;
    let high_byte = (code_point >> 8) as u16;
    let low_byte = (code_point & 0x00FF) as u16;

    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let sub_header_keys = s.read_array::<u16, u16>(256)?;
    // The maximum index in a sub_header_keys is a sub_headers count.
    let sub_headers_count = try_ok!(sub_header_keys.into_iter().map(|n| n / 8).max()) + 1;

    // Remember sub_headers offset before reading. Will be used later.
    let sub_headers_offset = s.offset();
    let sub_headers = s.read_array::<raw::SubHeaderRecord, u16>(sub_headers_count)?;

    let i = if code_point < 0xff {
        // 'SubHeader 0 is special: it is used for single-byte character codes.'
        0
    } else {
        // 'Array that maps high bytes to subHeaders: value is subHeader index × 8.'
        try_ok!(sub_header_keys.get(high_byte)) / 8
    };

    let sub_header = try_ok!(sub_headers.get(i));

    let first_code = sub_header.first_code();
    let range_end = first_code + sub_header.entry_count();
    if low_byte < first_code || low_byte > range_end {
        return Ok(None);
    }

    // SubHeaderRecord::id_range_offset points to SubHeaderRecord::first_code
    // in the glyphIndexArray. So we have to advance to our code point.
    let index_offset = (low_byte - first_code) as usize * core::mem::size_of::<u16>();

    // 'The value of the idRangeOffset is the number of bytes
    // past the actual location of the idRangeOffset'.
    let offset =
          sub_headers_offset
        // Advance to required subheader.
        + raw::SubHeaderRecord::SIZE * (i + 1) as usize
        // Move back to idRangeOffset start.
        - core::mem::size_of::<u16>()
        // Use defined offset.
        + sub_header.id_range_offset() as usize
        // Advance to required index in the glyphIndexArray.
        + index_offset;

    let glyph: u16 = Stream::read_at(data, offset)?;
    if glyph == 0 {
        return Ok(None);
    }

    let glyph = ((glyph as i32 + sub_header.id_delta() as i32) % 65536) as u16;
    Ok(Some(glyph))
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-4-segment-mapping-to-delta-values
fn parse_segment_mapping_to_delta_values(data: &[u8], code_point: u32) -> Result<Option<u16>> {
    // This subtable supports code points only in a u16 range.
    if code_point > 0xffff {
        return Ok(None);
    }

    let code_point = code_point as u16;

    let mut s = Stream::new(data);
    s.advance(6 as u32); // format + length + language
    let seg_count_x2: u16 = s.read()?;
    if seg_count_x2 < 2 {
        return Ok(None);
    }

    let seg_count = seg_count_x2 / 2;
    s.advance(6 as u32); // searchRange + entrySelector + rangeShift

    let end_codes = s.read_array::<u16, u16>(seg_count)?;
    s.skip::<u16>(); // reservedPad
    let start_codes = s.read_array::<u16, u16>(seg_count)?;
    let id_deltas = s.read_array::<i16, u16>(seg_count)?;
    let id_range_offset_pos = s.offset();
    let id_range_offsets = s.read_array::<u16, u16>(seg_count)?;

    // A custom binary search.
    let mut start = 0;
    let mut end = seg_count;
    while end > start {
        let index = (start + end) / 2;
        let end_value = try_ok!(end_codes.get(index));
        if end_value >= code_point {
            let start_value = try_ok!(start_codes.get(index));
            if start_value > code_point {
                end = index;
            } else {
                let id_range_offset = try_ok!(id_range_offsets.get(index));
                let id_delta = try_ok!(id_deltas.get(index));
                if id_range_offset == 0 {
                    return Ok(Some(code_point.wrapping_add(id_delta as u16)));
                }

                let delta = (code_point as u32 - start_value as u32) * 2;
                // Check for overflow.
                if delta > core::u16::MAX as u32 {
                    return Ok(None);
                }
                // `delta` must be u16.
                let delta = delta as u16;

                let id_range_offset_pos = (id_range_offset_pos + index as usize * 2) as u16;
                let pos = id_range_offset_pos.wrapping_add(delta);
                let pos = pos.wrapping_add(id_range_offset);
                let glyph_array_value: u16 = Stream::read_at(data, pos as usize)?;
                if glyph_array_value == 0 {
                    return Ok(None);
                }

                let glyph_id = (glyph_array_value as i16).wrapping_add(id_delta);
                return Ok(Some(glyph_id as u16));
            }
        } else {
            start = index + 1;
        }
    }

    Ok(None)
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-6-trimmed-table-mapping
fn parse_trimmed_table_mapping(s: &mut Stream, code_point: u32) -> Result<Option<u16>> {
    // This subtable supports code points only in a u16 range.
    if code_point > 0xffff {
        return Ok(None);
    }

    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let first_code_point: u16 = s.read()?;
    let glyphs = s.read_array16::<u16>()?;

    let code_point = code_point as u16;

    // Check for overflow.
    if code_point < first_code_point {
        return Ok(None);
    }

    let idx = code_point - first_code_point;
    Ok(glyphs.get(idx))
}

// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-10-trimmed-array
fn parse_trimmed_array(s: &mut Stream, code_point: u32) -> Result<Option<u16>> {
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let first_code_point: u32 = s.read()?;
    let glyphs = s.read_array32::<u16>()?;

    // Check for overflow.
    if code_point < first_code_point {
        return Ok(None);
    }

    let idx = code_point - first_code_point;
    Ok(glyphs.get(idx))
}

// + ManyToOneRangeMappings
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-13-many-to-one-range-mappings
fn parse_segmented_coverage(s: &mut Stream, code_point: u32, format: Format) -> Result<Option<u16>> {
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let groups = s.read_array32::<raw::SequentialMapGroup>()?;
    for group in groups {
        let start_char_code = group.start_char_code();
        if code_point >= start_char_code && code_point <= group.end_char_code() {
            if format == Format::SegmentedCoverage {
                let id = group.start_glyph_id() + code_point - start_char_code;
                return Ok(Some(id as u16));
            } else {
                // TODO: what if start_glyph_id is > u16::MAX
                return Ok(Some(group.start_glyph_id() as u16));
            }
        }
    }

    Ok(None)
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

impl raw::UnicodeRangeRecord {
    fn contains(&self, c: char) -> bool {
        let start_unicode_value = self.start_unicode_value();
        let end = start_unicode_value + self.additional_count() as u32;
        start_unicode_value >= (c as u32) && (c as u32) < end
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
