// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-0-byte-encoding-table

use crate::parser::{Stream, NumFrom};

pub fn parse(data: &[u8], code_point: u32) -> Option<u16> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    let length: u16 = s.read()?;
    s.skip::<u16>(); // language

    if code_point < u32::from(length) {
        s.advance(usize::num_from(code_point));
        Some(u16::from(s.read::<u8>()?))
    } else {
        None
    }
}
