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
    if low_byte < first_code || low_byte >= range_end {
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

pub fn codepoints(data: &[u8], mut f: impl FnMut(u32)) -> Option<()> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let sub_header_keys = s.read_array16::<u16>(256)?;

    // The maximum index in a sub_header_keys is a sub_headers count.
    let sub_headers_count = sub_header_keys.into_iter().map(|n| n / 8).max()? + 1;
    let sub_headers = s.read_array16::<SubHeaderRecord>(sub_headers_count)?;

    for first_byte in 0u16..256 {
        let i = sub_header_keys.get(first_byte)? / 8;
        let sub_header = sub_headers.get(i)?;
        let first_code = sub_header.first_code;

        if i == 0 {
            // This is a single byte code.
            let range_end = first_code.checked_add(sub_header.entry_count)?;
            if first_byte >= first_code && first_byte < range_end {
                f(u32::from(first_byte));
            }
        } else {
            // This is a two byte code.
            let base = first_code.checked_add(first_byte << 8)?;
            for k in 0..sub_header.entry_count {
                let code_point = base.checked_add(k)?;
                f(u32::from(code_point));
            }
        }
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use crate::parser::FromData;
    use super::{parse, codepoints};

    #[test]
    fn collect_codepoints() {
        let mut data = vec![
            0x00, 0x02, // format: 2
            0x02, 0x16, // subtable size: 534
            0x00, 0x00, // language ID: 0
        ];

        // Make only high byte 0x28 multi-byte.
        data.extend(std::iter::repeat(0x00).take(256 * u16::SIZE));
        data[6 + 0x28 * u16::SIZE + 1] = 0x08;

        data.extend(&[
            // First sub header (for single byte mapping)
            0x00, 0xFE, // first code: 254
            0x00, 0x02, // entry count: 2
            0x00, 0x00, // id delta: uninteresting
            0x00, 0x00, // id range offset: uninteresting
            // Second sub header (for high byte 0x28)
            0x00, 0x10, // first code: (0x28 << 8) + 0x10 = 10256,
            0x00, 0x03, // entry count: 3
            0x00, 0x00, // id delta: uninteresting
            0x00, 0x00, // id range offset: uninteresting
        ]);

        // Now only glyph ID's would follow. Not interesting for codepoints.

        let mut vec = vec![];
        codepoints(&data, |c| vec.push(c));
        assert_eq!(vec, [10256, 10257, 10258, 254, 255]);
    }

    #[test]
    fn codepoint_at_range_end() {
        let mut data = vec![
            0x00, 0x02, // format: 2
            0x02, 0x14, // subtable size: 532
            0x00, 0x00, // language ID: 0
        ];

        // Only single bytes.
        data.extend(std::iter::repeat(0x00).take(256 * u16::SIZE));
        data.extend(&[
            // First sub header (for single byte mapping)
            0x00, 0x28, // first code: 40
            0x00, 0x02, // entry count: 2
            0x00, 0x00, // id delta: 0
            0x00, 0x02, // id range offset: 2
            // Glyph index
            0x00, 0x64, // glyph ID [0]: 100
            0x03, 0xE8, // glyph ID [1]: 1000
            0x03, 0xE8, // glyph ID [2]: 10000 (unused)
        ]);

        assert_eq!(parse(&data, 39), None);
        assert_eq!(parse(&data, 40), Some(100));
        assert_eq!(parse(&data, 41), Some(1000));
        assert_eq!(parse(&data, 42), None);
    }
}
