// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-13-many-to-one-range-mappings

use core::convert::TryFrom;

use crate::parser::Stream;

pub fn parse(data: &[u8], code_point: u32) -> Option<u16> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let count: u32 = s.read()?;
    let groups = s.read_array32::<super::format12::SequentialMapGroup>(count)?;
    for group in groups {
        let start_char_code = group.start_char_code;
        if code_point >= start_char_code && code_point <= group.end_char_code {
            return u16::try_from(group.start_glyph_id).ok();
        }
    }

    None
}

pub fn codepoints(data: &[u8], f: impl FnMut(u32)) -> Option<()> {
    // Only the glyph id mapping differs for this table. The code points are the
    // same as for format 12.
    super::format12::codepoints(data, f)
}
