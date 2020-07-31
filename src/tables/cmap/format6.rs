// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-6-trimmed-table-mapping

use core::convert::TryFrom;

use crate::parser::Stream;

pub fn parse(data: &[u8], code_point: u32) -> Option<u16> {
    // This subtable supports code points only in a u16 range.
    let code_point = u16::try_from(code_point).ok()?;

    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let first_code_point: u16 = s.read()?;
    let count: u16 = s.read()?;
    let glyphs = s.read_array16::<u16>(count)?;

    let idx = code_point.checked_sub(first_code_point)?;
    glyphs.get(idx)
}

pub fn codepoints(data: &[u8], mut f: impl FnMut(u32)) -> Option<()> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let first_code_point: u16 = s.read()?;
    let count: u16 = s.read()?;

    for i in 0..count {
        let code_point = first_code_point.checked_add(i)?;
        f(u32::from(code_point));
    }

    Some(())
}
