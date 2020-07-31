// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-0-byte-encoding-table

use crate::parser::{Stream, NumFrom};

pub fn parse(data: &[u8], code_point: u32) -> Option<u16> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // length
    s.skip::<u16>(); // language

    s.advance(usize::num_from(code_point));
    let glyph_id: u8 = s.read()?;

    // Make sure that the glyph is not zero, the array always has length 256,
    // but some codepoints may be mapped to zero.
    if glyph_id != 0 {
        Some(u16::from(glyph_id))
    } else {
        None
    }
}

pub fn codepoints(data: &[u8], mut f: impl FnMut(u32)) -> Option<()> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // length
    s.skip::<u16>(); // language

    for code_point in 0..256 {
        // In contrast to every other format, here we take a look at the glyph
        // id and check whether it is zero because otherwise this method would
        // always simply call `f` for `0..256` which would be kind of pointless
        // (this array always has length 256 even when the face has only fewer
        // glyphs).
        let glyph_id: u8 = s.read()?;
        if glyph_id != 0 {
            f(code_point);
        }
    }

    Some(())
}

#[cfg(test)]
mod tests {
    use super::{parse, codepoints};

    #[test]
    fn maps_not_all_256_codepoints() {
        let mut data = vec![
            0x00, 0x00, // format: 0
            0x01, 0x06, // subtable size: 262
            0x00, 0x00, // language ID: 0
        ];

        // Map (only) codepoint 0x40 to 100.
        data.extend(std::iter::repeat(0).take(256));
        data[6 + 0x40] = 100;

        assert_eq!(parse(&data, 0), None);
        assert_eq!(parse(&data, 0x40), Some(100));
        assert_eq!(parse(&data, 100), None);

        let mut vec = vec![];
        codepoints(&data, |c| vec.push(c));
        assert_eq!(vec, [0x40]);
    }
}
