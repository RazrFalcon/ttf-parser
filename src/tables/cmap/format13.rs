// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-13-many-to-one-range-mappings

use core::convert::TryFrom;

use crate::parser::Stream;

pub fn parse(mut s: Stream, code_point: u32) -> Option<u16> {
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
