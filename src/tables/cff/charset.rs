use crate::GlyphId;
use crate::parser::{Stream, FromData, LazyArray16};
use super::StringId;


#[derive(Clone, Copy, Debug)]
pub(crate) struct Format1Range {
    first: StringId,
    left: u8,
}

impl FromData for Format1Range {
    const SIZE: usize = 3;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Format1Range {
            first: s.read()?,
            left: s.read()?,
        })
    }
}


#[derive(Clone, Copy, Debug)]
pub(crate) struct Format2Range {
    first: StringId,
    left: u16,
}

impl FromData for Format2Range {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Format2Range {
            first: s.read()?,
            left: s.read()?,
        })
    }
}


#[derive(Clone, Copy, Debug)]
pub(crate) enum Charset<'a> {
    ISOAdobe,
    Expert,
    ExpertSubset,
    Format0(LazyArray16<'a, StringId>),
    Format1(LazyArray16<'a, Format1Range>),
    Format2(LazyArray16<'a, Format2Range>),
}

impl Charset<'_> {
    pub fn sid_to_gid(&self, sid: StringId) -> Option<GlyphId> {
        if sid.0 == 0 {
            return Some(GlyphId(0));
        }

        match self {
            Charset::ISOAdobe | Charset::Expert | Charset::ExpertSubset => None,
            Charset::Format0(ref array) => {
                // First glyph is omitted, so we have to add 1.
                array.into_iter().position(|n| n == sid).map(|n| GlyphId(n as u16 + 1))
            }
            Charset::Format1(array) => {
                let mut glyph_id = GlyphId(1);
                for range in *array {
                    let last = u32::from(range.first.0) + u32::from(range.left);
                    if range.first <= sid && u32::from(sid.0) <= last {
                        glyph_id.0 += sid.0 - range.first.0;
                        return Some(glyph_id)
                    }

                    glyph_id.0 += u16::from(range.left) + 1;
                }

                None
            }
            Charset::Format2(array) => {
                // The same as format 1, but Range::left is u16.
                let mut glyph_id = GlyphId(1);
                for range in *array {
                    let last = u32::from(range.first.0) + u32::from(range.left);
                    if sid >= range.first && u32::from(sid.0) <= last {
                        glyph_id.0 += sid.0 - range.first.0;
                        return Some(glyph_id)
                    }

                    glyph_id.0 += range.left + 1;
                }

                None
            }
        }
    }
}

pub(crate) fn parse_charset<'a>(number_of_glyphs: u16, s: &mut Stream<'a>) -> Option<Charset<'a>> {
    if number_of_glyphs < 2 {
        return None;
    }

    // -1 everywhere, since `.notdef` is omitted.
    let format: u8 = s.read()?;
    match format {
        0 => Some(Charset::Format0(s.read_array16(number_of_glyphs - 1)?)),
        1 => {
            // The number of ranges is not defined, so we have to
            // read until no glyphs are left.
            let mut count = 0;
            {
                let mut s = s.clone();
                let mut total_left = number_of_glyphs - 1;
                while total_left > 0 {
                    s.skip::<StringId>(); // first
                    let left: u8 = s.read()?;
                    total_left = total_left.checked_sub(u16::from(left) + 1)?;
                    count += 1;
                }
            }

            s.read_array16(count).map(Charset::Format1)
        }
        2 => {
            // The same as format 1, but Range::left is u16.
            let mut count = 0;
            {
                let mut s = s.clone();
                let mut total_left = number_of_glyphs - 1;
                while total_left > 0 {
                    s.skip::<StringId>(); // first
                    let left: u16 = s.read()?;
                    let left = left.checked_add(1)?;
                    total_left = total_left.checked_sub(left)?;
                    count += 1;
                }
            }

            s.read_array16(count).map(Charset::Format2)
        }
        _ => None,
    }
}
