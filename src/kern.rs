// https://docs.microsoft.com/en-us/typography/opentype/spec/kern

use crate::GlyphId;
use crate::parser::{Stream, FromData, SafeStream};


pub fn glyphs_kerning(kern_table: &[u8], glyph_id1: GlyphId, glyph_id2: GlyphId) -> Option<i16> {
    let mut s = Stream::new(kern_table);

    let version: u16 = s.read()?;
    if version != 0 {
        return None;
    }

    let number_of_subtables: u16 = s.read()?;

    // TODO: Technically, we have to iterate over all tables,
    //       but I'm not sure how exactly this should be implemented.
    //       Also, I have to find a font, that actually has more that one table.
    if number_of_subtables == 0 {
        return None;
    }

    s.skip::<u16>(); // subtable_version
    s.skip::<u16>(); // length
    let coverage: Coverage = s.read()?;

    if !coverage.is_horizontal() {
        return None;
    }

    if coverage.format != 0 {
        return None;
    }

    parse_format1(&mut s, glyph_id1, glyph_id2)
}

fn parse_format1(s: &mut Stream, glyph_id1: GlyphId, glyph_id2: GlyphId) -> Option<i16> {
    let number_of_pairs: u16 = s.read()?;
    s.advance(6); // search_range (u16) + entry_selector (u16) + range_shift (u16)
    let pairs = s.read_array16::<KerningRecord>(number_of_pairs)?;

    let needle = u32::from(glyph_id1.0) << 16 | u32::from(glyph_id2.0);
    pairs.binary_search_by(|v| v.pair.cmp(&needle)).map(|(_, v)| v.value)
}

struct KerningRecord {
    pair: u32,
    value: i16,
}

impl FromData for KerningRecord {
    const SIZE: usize = 6; // Override, since `size_of` will be 8 because of padding.

    #[inline]
    fn parse(data: &[u8]) -> Self {
        let mut s = SafeStream::new(data);
        KerningRecord {
            pair: s.read(),
            value: s.read(),
        }
    }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/kern
struct Coverage {
    coverage: u8,
    format: u8,
}

impl Coverage {
    const HORIZONTAL_BIT: u8 = 0;

    #[inline]
    fn is_horizontal(&self) -> bool {
        (self.coverage >> Coverage::HORIZONTAL_BIT) & 1 == 1
    }
}

impl FromData for Coverage {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        let mut s = SafeStream::new(data);
        Coverage {
            // Reverse order, since we're reading a big-endian u16.
            format: s.read(),
            coverage: s.read(),
        }
    }
}
