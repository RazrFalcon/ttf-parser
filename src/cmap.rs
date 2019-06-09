//! The [cmap](https://docs.microsoft.com/en-us/typography/opentype/spec/cmap)
//! table parsing primitives.

use std::ops::Range;

use crate::stream::{Stream, FromData};
use crate::Font;


/// A code point to glyph matching error.
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// No glyph for a specified char was found.
    NoGlyph,

    /// An unsupported table format.
    UnsupportedTableFormat(u16),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::NoGlyph => {
                write!(f, "no glyph for a specified char was found")
            }
            Error::UnsupportedTableFormat(id) => {
                write!(f, "table format {} is not supported", id)
            }
        }
    }
}

impl std::error::Error for Error {}


impl<'a> Font<'a> {
    /// Resolves Glyph ID for code point.
    ///
    /// Returns `Error::NoGlyph` instead of `0` when glyph is not found.
    pub fn glyph_index(&self, c: char) -> Result<u16, Error> {
        let cmap_data = &self.data[self.cmap.range()];
        let mut s = Stream::new(cmap_data);
        s.skip_u16(); // version
        let num_tables = s.read_u16();

        for _ in 0..num_tables {
            s.skip_u16(); // platform_id
            s.skip_u16(); // encoding_id
            let offset = s.read_u32() as usize;

            match parse_subtable(c as u32, &cmap_data[offset..]) {
                Ok(id) => return Ok(id),
                Err(Error::NoGlyph) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(Error::NoGlyph)
    }
}

fn parse_subtable(code_point: u32, data: &[u8]) -> Result<u16, Error> {
    let mut s = Stream::new(data);
    let format = s.read_u16();
    match format {
        0 => {
            let length = s.read_u16();
            s.skip_u16(); // language

            if code_point < (length as u32) {
                s.skip(code_point as usize);
                Ok(s.read_u8() as u16)
            } else {
                Err(Error::NoGlyph)
            }
        }
        4 => {
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

            // A simple binary search.
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
        12 | 13 => {
            s.skip_u16(); // reserved
            s.skip_u32(); // length
            s.skip_u32(); // language
            let num_groups = s.read_u32() as usize;
            let groups = s.read_array::<SequentialMapGroup>(num_groups);
            for group in groups {
                if group.char_code_range.contains(&code_point) {
                    if format == 12 {
                        let id = group.start_glyph_id + code_point - group.char_code_range.start;
                        return Ok(id as u16);
                    } else {
                        return Ok(group.start_glyph_id as u16);
                    }
                }
            }

            Err(Error::NoGlyph)
        }
        _ => Err(Error::UnsupportedTableFormat(format)),
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
