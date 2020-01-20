// https://docs.microsoft.com/en-us/typography/opentype/spec/kern

use crate::parser::{Stream, FromData, SafeStream, LazyArray};
use crate::{Font, GlyphId, TableName, Result, Error};

impl<'a> Font<'a> {
    /// Returns a glyphs pair kerning.
    ///
    /// Only horizontal kerning is supported.
    pub fn glyphs_kerning(&self, glyph_id1: GlyphId, glyph_id2: GlyphId) -> Result<i16> {
        self.check_glyph_id(glyph_id1)?;
        self.check_glyph_id(glyph_id2)?;
        let data = self.kern.ok_or_else(|| Error::TableMissing(TableName::Kerning))?;

        let mut s = Stream::new(data);
        let version: u16 = s.read()?;

        if version != 0 {
            return Err(Error::UnsupportedTableVersion(TableName::Kerning, version));
        }

        let number_of_subtables: u16 = s.read()?;

        // TODO: Technically, we have to iterate over all tables,
        //       but I'm not sure how exactly this should be implemented.
        //       Also, I have to find a font, that actually has more that one table.
        if number_of_subtables == 0 {
            return Err(Error::NoKerning);
        }

        s.skip::<u16>(); // subtable_version
        s.skip::<u16>(); // length
        let coverage: Coverage = s.read()?;

        if !coverage.is_horizontal() {
            return Err(Error::NoKerning);
        }

        if coverage.format != 0 {
            return Err(Error::NoKerning);
        }

        parse_format1(&mut s, glyph_id1, glyph_id2)
    }
}

fn parse_format1(s: &mut Stream, glyph_id1: GlyphId, glyph_id2: GlyphId) -> Result<i16> {
    let number_of_pairs: u16 = s.read()?;
    s.advance(6u32); // search_range (u16) + entry_selector (u16) + range_shift (u16)
    let pairs: LazyArray<KerningRecord> = s.read_array(number_of_pairs)?;

    let needle = (glyph_id1.0 as u32) << 16 | glyph_id2.0 as u32;
    match pairs.binary_search_by(|v| v.pair.cmp(&needle)) {
        Some(v) => Ok(v.value),
        None => Err(Error::NoKerning),
    }
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
