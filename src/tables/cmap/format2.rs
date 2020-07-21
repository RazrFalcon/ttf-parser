// This table has a pretty complex parsing algorithm.
// A detailed explanation can be found here:
// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table
// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
// https://github.com/fonttools/fonttools/blob/a360252709a3d65f899915db0a5bd753007fdbb7/Lib/fontTools/ttLib/tables/_c_m_a_p.py#L360

use core::convert::TryFrom;

use crate::parser::{Stream, FromData};

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
            first_code: s.read::<u16>()?,
            entry_count: s.read::<u16>()?,
            id_delta: s.read::<i16>()?,
            id_range_offset: s.read::<u16>()?,
        })
    }
}

pub fn parse(data: &[u8], code_point: u32) -> Option<u16> {
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
