// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-12-segmented-coverage

use core::convert::TryFrom;

use crate::parser::{Stream, FromData};

#[derive(Clone, Copy)]
pub struct SequentialMapGroup {
    pub start_char_code: u32,
    pub end_char_code: u32,
    pub start_glyph_id: u32,
}

impl FromData for SequentialMapGroup {
    const SIZE: usize = 12;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(SequentialMapGroup {
            start_char_code: s.read::<u32>()?,
            end_char_code: s.read::<u32>()?,
            start_glyph_id: s.read::<u32>()?,
        })
    }
}

pub fn parse(data: &[u8], code_point: u32) -> Option<u16> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let count: u32 = s.read()?;
    let groups = s.read_array32::<SequentialMapGroup>(count)?;
    for group in groups {
        let start_char_code = group.start_char_code;
        if code_point >= start_char_code && code_point <= group.end_char_code {
            let id = group.start_glyph_id.checked_add(code_point)?.checked_sub(start_char_code)?;
            return u16::try_from(id).ok();
        }
    }

    None
}

pub fn codepoints(data: &[u8], mut f: impl FnMut(u32)) -> Option<()> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u16>(); // reserved
    s.skip::<u32>(); // length
    s.skip::<u32>(); // language
    let count: u32 = s.read()?;
    let groups = s.read_array32::<SequentialMapGroup>(count)?;
    for group in groups {
        for code_point in group.start_char_code..=group.end_char_code {
            f(code_point);
        }
    }

    Some(())
}
