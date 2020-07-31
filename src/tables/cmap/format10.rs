// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-10-trimmed-array

use crate::parser::Stream;

pub fn parse(data: &[u8], code_point: u32) -> Option<u16> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let first_code_point: u32 = s.read()?;
    let count: u32 = s.read()?;
    let glyphs = s.read_array32::<u16>(count)?;

    let idx = code_point.checked_sub(first_code_point)?;
    glyphs.get(idx)
}

pub fn codepoints(data: &[u8], mut f: impl FnMut(u32)) -> Option<()> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let first_code_point: u32 = s.read()?;
    let count: u32 = s.read()?;

    for i in 0..count {
        let code_point = first_code_point.checked_add(i)?;
        f(code_point);
    }

    Some(())
}
