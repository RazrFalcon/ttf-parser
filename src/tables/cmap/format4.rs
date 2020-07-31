// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-4-segment-mapping-to-delta-values

use core::convert::TryFrom;

use crate::parser::Stream;

pub fn parse(data: &[u8], code_point: u32) -> Option<u16> {
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

                // 0 indicates missing glyph.
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

pub fn codepoints(data: &[u8], mut f: impl FnMut(u32)) -> Option<()> {
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

    for (start, end) in start_codes.into_iter().zip(end_codes) {
        for code_point in start..=end {
            f(u32::from(code_point));
        }
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use super::{parse, codepoints};

    #[test]
    fn single_glyph() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        assert_eq!(parse(data, 0x41), Some(1));
        assert_eq!(parse(data, 0x42), None);
    }

    #[test]
    fn continuous_range() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x49, // char code [0]: 73
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        assert_eq!(parse(data, 0x40), None);
        assert_eq!(parse(data, 0x41), Some(1));
        assert_eq!(parse(data, 0x42), Some(2));
        assert_eq!(parse(data, 0x43), Some(3));
        assert_eq!(parse(data, 0x44), Some(4));
        assert_eq!(parse(data, 0x45), Some(5));
        assert_eq!(parse(data, 0x46), Some(6));
        assert_eq!(parse(data, 0x47), Some(7));
        assert_eq!(parse(data, 0x48), Some(8));
        assert_eq!(parse(data, 0x49), Some(9));
        assert_eq!(parse(data, 0x4A), None);
    }

    #[test]
    fn multiple_ranges() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x30, // subtable size: 48
            0x00, 0x00, // language ID: 0
            0x00, 0x08, // 2 x segCount: 8
            0x00, 0x04, // search range: 4
            0x00, 0x01, // entry selector: 1
            0x00, 0x04, // range shift: 4
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0x00, 0x45, // char code [1]: 69
            0x00, 0x49, // char code [2]: 73
            0xFF, 0xFF, // char code [3]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0x00, 0x43, // char code [1]: 67
            0x00, 0x47, // char code [2]: 71
            0xFF, 0xFF, // char code [3]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0xFF, 0xBF, // delta [1]: -65
            0xFF, 0xBE, // delta [2]: -66
            0x00, 0x01, // delta [3]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
            0x00, 0x00, // offset [2]: 0
            0x00, 0x00, // offset [3]: 0
        ];

        assert_eq!(parse(data, 0x40), None);
        assert_eq!(parse(data, 0x41), Some(1));
        assert_eq!(parse(data, 0x42), None);
        assert_eq!(parse(data, 0x43), Some(2));
        assert_eq!(parse(data, 0x44), Some(3));
        assert_eq!(parse(data, 0x45), Some(4));
        assert_eq!(parse(data, 0x46), None);
        assert_eq!(parse(data, 0x47), Some(5));
        assert_eq!(parse(data, 0x48), Some(6));
        assert_eq!(parse(data, 0x49), Some(7));
        assert_eq!(parse(data, 0x4A), None);
    }

    #[test]
    fn unordered_ids() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x2A, // subtable size: 42
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x45, // char code [0]: 69
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0x00, 0x00, // delta [0]: 0
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x04, // offset [0]: 4
            0x00, 0x00, // offset [1]: 0
            // Glyph index array
            0x00, 0x01, // glyph ID [0]: 1
            0x00, 0x0A, // glyph ID [1]: 10
            0x00, 0x64, // glyph ID [2]: 100
            0x03, 0xE8, // glyph ID [3]: 1000
            0x27, 0x10, // glyph ID [4]: 10000
        ];

        assert_eq!(parse(data, 0x40), None);
        assert_eq!(parse(data, 0x41), Some(1));
        assert_eq!(parse(data, 0x42), Some(10));
        assert_eq!(parse(data, 0x43), Some(100));
        assert_eq!(parse(data, 0x44), Some(1000));
        assert_eq!(parse(data, 0x45), Some(10000));
        assert_eq!(parse(data, 0x46), None);
    }

    #[test]
    fn unordered_chars_and_ids() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x40, // subtable size: 64
            0x00, 0x00, // language ID: 0
            0x00, 0x0C, // 2 x segCount: 12
            0x00, 0x08, // search range: 8
            0x00, 0x02, // entry selector: 2
            0x00, 0x04, // range shift: 4
            // End character codes
            0x00, 0x50, // char code [0]: 80
            0x01, 0x00, // char code [1]: 256
            0x01, 0x50, // char code [2]: 336
            0x02, 0x00, // char code [3]: 512
            0x02, 0x50, // char code [4]: 592
            0xFF, 0xFF, // char code [5]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x50, // char code [0]: 80
            0x01, 0x00, // char code [1]: 256
            0x01, 0x50, // char code [2]: 336
            0x02, 0x00, // char code [3]: 512
            0x02, 0x50, // char code [4]: 592
            0xFF, 0xFF, // char code [5]: 65535
            // Deltas
            0xFF, 0xB1, // delta [0]: -79
            0xFF, 0x0A, // delta [1]: -246
            0xFF, 0x14, // delta [2]: -236
            0x01, 0xE8, // delta [3]: 488
            0x24, 0xC0, // delta [4]: 9408
            0x00, 0x01, // delta [5]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
            0x00, 0x00, // offset [2]: 0
            0x00, 0x00, // offset [3]: 0
            0x00, 0x00, // offset [4]: 0
            0x00, 0x00, // offset [5]: 0
        ];

        assert_eq!(parse(data, 0x40),  None);
        assert_eq!(parse(data, 0x50),  Some(1));
        assert_eq!(parse(data, 0x100), Some(10));
        assert_eq!(parse(data, 0x150), Some(100));
        assert_eq!(parse(data, 0x200), Some(1000));
        assert_eq!(parse(data, 0x250), Some(10000));
        assert_eq!(parse(data, 0x300), None);
    }

    #[test]
    fn no_end_codes() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 28
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x49, // char code [0]: 73
            // 0xFF, 0xFF, // char code [1]: 65535 <-- removed
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            // 0xFF, 0xFF, // char code [1]: 65535 <-- removed
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        assert_eq!(parse(data, 0x40), None);
        assert_eq!(parse(data, 0x41), None);
        assert_eq!(parse(data, 0x42), None);
        assert_eq!(parse(data, 0x43), None);
        assert_eq!(parse(data, 0x44), None);
        assert_eq!(parse(data, 0x45), None);
        assert_eq!(parse(data, 0x46), None);
        assert_eq!(parse(data, 0x47), None);
        assert_eq!(parse(data, 0x48), None);
        assert_eq!(parse(data, 0x49), None);
        assert_eq!(parse(data, 0x4A), None);
    }

    #[test]
    fn invalid_segment_count() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x01, // 2 x segCount: 1 <-- must be more than 1
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        assert_eq!(parse(data, 0x41), None);
    }

    #[test]
    fn only_end_segments() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x02, // 2 x segCount: 2
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        // Should not loop forever.
        assert_eq!(parse(data, 0x41), None);
    }

    #[test]
    fn invalid_length() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x10, // subtable size: 16 <-- the size should be 32, but we don't check it anyway
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        assert_eq!(parse(data, 0x41), Some(1));
        assert_eq!(parse(data, 0x42), None);
    }

    #[test]
    fn codepoint_out_of_range() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x20, // subtable size: 32
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0xFF, 0xC0, // delta [0]: -64
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x00, // offset [0]: 0
            0x00, 0x00, // offset [1]: 0
        ];

        // Format 4 support only u16 codepoints, so we have to bail immediately otherwise.
        assert_eq!(parse(data, 0x1FFFF), None);
    }

    #[test]
    fn zero() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x2A, // subtable size: 42
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x45, // char code [0]: 69
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x41, // char code [0]: 65
            0xFF, 0xFF, // char code [1]: 65535
            // Deltas
            0x00, 0x00, // delta [0]: 0
            0x00, 0x01, // delta [1]: 1
            // Offsets into Glyph index array
            0x00, 0x04, // offset [0]: 4
            0x00, 0x00, // offset [1]: 0
            // Glyph index array
            0x00, 0x00, // glyph ID [0]: 0 <-- indicates missing glyph
            0x00, 0x0A, // glyph ID [1]: 10
            0x00, 0x64, // glyph ID [2]: 100
            0x03, 0xE8, // glyph ID [3]: 1000
            0x27, 0x10, // glyph ID [4]: 10000
        ];

        assert_eq!(parse(data, 0x41), None);
    }

    #[test]
    fn collect_codepoints() {
        let data = &[
            0x00, 0x04, // format: 4
            0x00, 0x18, // subtable size: 24
            0x00, 0x00, // language ID: 0
            0x00, 0x04, // 2 x segCount: 4
            0x00, 0x02, // search range: 2
            0x00, 0x00, // entry selector: 0
            0x00, 0x02, // range shift: 2
            // End character codes
            0x00, 0x22, // char code [0]: 34
            0xFF, 0xFF, // char code [1]: 65535
            0x00, 0x00, // reserved: 0
            // Start character codes
            0x00, 0x1B, // char code [0]: 27
            0xFF, 0xFD, // char code [1]: 65533
            // codepoints does not care about glyph ids
        ];

        let mut vec = vec![];
        codepoints(data, |c| vec.push(c));
        assert_eq!(vec, [27, 28, 29, 30, 31, 32, 33, 34, 65533, 65534, 65535]);
    }
}
