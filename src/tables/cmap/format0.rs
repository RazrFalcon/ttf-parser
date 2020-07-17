// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-0-byte-encoding-table

use crate::parser::{Stream, NumFrom};

pub fn parse(mut s: Stream, code_point: u32) -> Option<u16> {
    let length: u16 = s.read()?;
    s.skip::<u16>(); // language

    if code_point < u32::from(length) {
        s.advance(usize::num_from(code_point));
        Some(u16::from(s.read::<u8>()?))
    } else {
        None
    }
}
