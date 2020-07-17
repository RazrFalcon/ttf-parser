// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-6-trimmed-table-mapping

use core::convert::TryFrom;

use crate::parser::Stream;

pub fn parse(mut s: Stream, code_point: u32) -> Option<u16> {
    // This subtable supports code points only in a u16 range.
    let code_point = u16::try_from(code_point).ok()?;

    s.skip::<u16>(); // length
    s.skip::<u16>(); // language
    let first_code_point: u16 = s.read()?;
    let count: u16 = s.read()?;
    let glyphs = s.read_array16::<u16>(count)?;

    let idx = code_point.checked_sub(first_code_point)?;
    glyphs.get(idx)
}
