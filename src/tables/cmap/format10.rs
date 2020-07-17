// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-10-trimmed-array

use crate::parser::Stream;

pub fn parse(mut s: Stream, code_point: u32) -> Option<u16> {
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let first_code_point: u32 = s.read()?;
    let count: u32 = s.read()?;
    let glyphs = s.read_array32::<u16>(count)?;

    let idx = code_point.checked_sub(first_code_point)?;
    glyphs.get(idx)
}
