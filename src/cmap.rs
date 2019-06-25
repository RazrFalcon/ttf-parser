use std::ops::Range;

use crate::parser::{Stream, FromData, LazyArray, Offset32};
use crate::{Font, GlyphId, TableName, Result, Error};


impl<'a> Font<'a> {
    /// Resolves Glyph ID for code point.
    ///
    /// Returns `Error::NoGlyph` instead of `0` when glyph is not found.
    pub fn glyph_index(&self, c: char) -> Result<GlyphId> {
        let cmap_data = self.table_data(TableName::CharacterToGlyphIndexMapping)?;
        let mut s = Stream::new(cmap_data);
        s.skip_u16(); // version
        let num_tables = s.read_u16();

        for _ in 0..num_tables {
            s.skip_u16(); // platform_id
            s.skip_u16(); // encoding_id
            let offset = s.read_u32() as usize;

            let subtable_data = &cmap_data[offset..];
            let mut s = Stream::new(subtable_data);
            let format = parse_format(s.read_u16())?;
            if format == Format::UnicodeVariationSequences {
                // UnicodeVariationSequences subtable is used only by glyph_variation_index().
                continue;
            }

            match parse_subtable(subtable_data, format, c as u32) {
                Ok(id) => return Ok(GlyphId(id)),
                Err(Error::NoGlyph) => continue,
                Err(e) => return Err(e),
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
        let cmap_data = self.table_data(TableName::CharacterToGlyphIndexMapping)?;
        let mut s = Stream::new(cmap_data);
        s.skip_u16(); // version
        let num_tables = s.read_u16();

        for _ in 0..num_tables {
            s.skip_u16(); // platform_id
            s.skip_u16(); // encoding_id
            let offset = s.read_u32() as usize;

            let subtable_data = &cmap_data[offset..];
            let mut s = Stream::new(subtable_data);
            let format = parse_format(s.read_u16())?;
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
        s.skip_u16(); // format
        s.skip_u32(); // length
        let num_var_selector_records = s.read_u32() as usize;
        let records = s.read_array::<VariationSelectorRecord>(num_var_selector_records);

        let record = records.binary_search_by(|v| v.variation.cmp(&variation)).ok_or(Error::NoGlyph)?;

        if let Some(offset) = record.default_uvs_offset {
            let mut s = Stream::new(&data[offset.0 as usize..]);
            let count: u32 = s.read(); // numUnicodeValueRanges
            let ranges: LazyArray<UnicodeRangeRecord> = s.read_array(count as usize);
            for range in ranges {
                if range.contains(c) {
                    // This is a default glyph.
                    return self.glyph_index(c);
                }
            }
        }

        if let Some(offset) = record.non_default_uvs_offset {
            let mut s = Stream::new(&data[offset.0 as usize..]);
            let count: u32 = s.read(); // numUVSMappings
            let uvs_mappings: LazyArray<UVSMappingRecord> = s.read_array(count as usize);
            if let Some(mapping) = uvs_mappings.binary_search_by(|v| v.unicode_value.cmp(&cp)) {
                return Ok(mapping.glyph);
            }
        }

        Err(Error::NoGlyph)
    }
}

fn parse_subtable(data: &[u8], format: Format, code_point: u32) -> Result<u16> {
    let mut s = Stream::new(data);
    s.skip_u16(); // format
    match format {
        Format::ByteEncodingTable => {
            let length = s.read_u16();
            s.skip_u16(); // language

            if code_point < (length as u32) {
                s.skip(code_point as usize);
                Ok(s.read_u8() as u16)
            } else {
                Err(Error::NoGlyph)
            }
        }
        Format::SegmentMappingToDeltaValues => {
            // This subtable supports code points only in a u16 range.
            if code_point > 0xffff {
                return Err(Error::NoGlyph);
            }

            let code_point = code_point as u16;

            s.skip_u16(); // length
            s.skip_u16(); // language
            let seg_count_x2 = s.read_u16() as usize;
            let seg_count = seg_count_x2 / 2;
            s.skip_u16(); // searchRange
            s.skip_u16(); // entrySelector
            s.skip_u16(); // rangeShift
            let end_codes = s.read_array::<u16>(seg_count);
            s.skip_u16(); // reservedPad
            let start_codes = s.read_array::<u16>(seg_count);
            let id_deltas = s.read_array::<i16>(seg_count);
            let id_range_offset_pos = s.offset();
            let id_range_offsets = s.read_array::<u16>(seg_count);

            // A custom binary search.
            let mut start = 0;
            let mut end = seg_count;
            while end > start {
                let index = (start + end) / 2;
                let end_value = end_codes.at(index);
                if end_value >= code_point {
                    let start_value = start_codes.at(index);
                    if start_value > code_point {
                        end = index;
                    } else {
                        let id_range_offset = id_range_offsets.at(index);
                        let id_delta = id_deltas.at(index);
                        if id_range_offset == 0 {
                            return Ok(code_point.wrapping_add(id_delta as u16));
                        }

                        let delta = (code_point - start_value) * 2;
                        let id_range_offset_pos = (id_range_offset_pos + index * 2) as u16;
                        let pos = id_range_offset_pos.wrapping_add(delta) + id_range_offset;
                        let glyph_array_value: u16 = Stream::read_at(data, pos as usize);
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

            Err(Error::NoGlyph)
        }
        Format::SegmentedCoverage | Format::ManyToOneRangeMappings => {
            s.skip_u16(); // reserved
            s.skip_u32(); // length
            s.skip_u32(); // language
            let num_groups = s.read_u32() as usize;
            let groups = s.read_array::<SequentialMapGroup>(num_groups);
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
        _ => Err(Error::UnsupportedCharMapFormat(format as u16)),
    }
}


#[derive(Clone, Copy, PartialEq)]
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

fn parse_format(v: u16) -> Result<Format> {
    match v {
         0 => Ok(Format::ByteEncodingTable),
         2 => Ok(Format::HighByteMappingThroughTable),
         4 => Ok(Format::SegmentMappingToDeltaValues),
         6 => Ok(Format::TrimmedTableMapping),
         8 => Ok(Format::MixedCoverage),
        10 => Ok(Format::TrimmedArray),
        12 => Ok(Format::SegmentedCoverage),
        13 => Ok(Format::ManyToOneRangeMappings),
        14 => Ok(Format::UnicodeVariationSequences),
        _ => Err(Error::UnsupportedCharMapFormat(v))
    }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage
// Also, the same as ConstantMapGroup.
struct SequentialMapGroup {
    char_code_range: Range<u32>,
    start_glyph_id: u32,
}

impl FromData for SequentialMapGroup {
    fn parse(data: &[u8]) -> Self {
        let mut s = Stream::new(data);
        SequentialMapGroup {
            char_code_range: s.read_u32()..s.read_u32(),
            start_glyph_id: s.read_u32(),
        }
    }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences
struct VariationSelectorRecord {
    variation: u32,
    default_uvs_offset: Option<Offset32>,
    non_default_uvs_offset: Option<Offset32>,
}

impl FromData for VariationSelectorRecord {
    fn parse(data: &[u8]) -> Self {
        let mut s = Stream::new(data);
        VariationSelectorRecord {
            variation: s.read_u24(),
            default_uvs_offset: s.read(),
            non_default_uvs_offset: s.read(),
        }
    }

    fn size_of() -> usize {
        // variation_selector is u24.
        3 + Offset32::size_of() + Offset32::size_of()
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
    fn parse(data: &[u8]) -> Self {
        let mut s = Stream::new(data);
        UnicodeRangeRecord {
            start_unicode_value: s.read_u24(),
            additional_count: s.read(),
        }
    }

    fn size_of() -> usize {
        // start_unicode_value is u24.
        3 + 1
    }
}


struct UVSMappingRecord {
    unicode_value: u32,
    glyph: GlyphId,
}

impl FromData for UVSMappingRecord {
    fn parse(data: &[u8]) -> Self {
        let mut s = Stream::new(data);
        UVSMappingRecord {
            unicode_value: s.read_u24(),
            glyph: s.read(),
        }
    }

    fn size_of() -> usize {
        // unicode_value is u24.
        3 + GlyphId::size_of()
    }
}
