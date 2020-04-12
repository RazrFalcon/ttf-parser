// https://docs.microsoft.com/en-us/typography/opentype/spec/maxp

use core::num::NonZeroU16;

use crate::parser::Stream;

// We care only about `numGlyphs`.
pub fn parse(data: &[u8]) -> Option<NonZeroU16> {
    let mut s = Stream::new(data);
    let version: u32 = s.read()?;
    if !(version == 0x00005000 || version == 0x00010000) {
        return None;
    }

    let n: u16 = s.read()?;
    NonZeroU16::new(n)
}
