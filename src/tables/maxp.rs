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


#[cfg(test)]
mod tests {
    #[test]
    fn version_05() {
        let num_glyphs = super::parse(&[
            0x00, 0x00, 0x50, 0x00, // version: 0.3125
            0x00, 0x01, // number of glyphs: 1
        ]).map(|n| n.get());
        assert_eq!(num_glyphs, Some(1));
    }

    #[test]
    fn version_1_full() {
        let num_glyphs = super::parse(&[
            0x00, 0x01, 0x00, 0x00, // version: 1
            0x00, 0x01, // number of glyphs: 1
            0x00, 0x00, // maximum points in a non-composite glyph: 0
            0x00, 0x00, // maximum contours in a non-composite glyph: 0
            0x00, 0x00, // maximum points in a composite glyph: 0
            0x00, 0x00, // maximum contours in a composite glyph: 0
            0x00, 0x00, // maximum zones: 0
            0x00, 0x00, // maximum twilight points: 0
            0x00, 0x00, // number of Storage Area locations: 0
            0x00, 0x00, // number of FDEFs: 0
            0x00, 0x00, // number of IDEFs: 0
            0x00, 0x00, // maximum stack depth: 0
            0x00, 0x00, // maximum byte count for glyph instructions: 0
            0x00, 0x00, // maximum number of components: 0
            0x00, 0x00, // maximum levels of recursion: 0
        ]).map(|n| n.get());
        assert_eq!(num_glyphs, Some(1));
    }

    #[test]
    fn version_1_trimmed() {
        // We don't really care about the data after the number of glyphs.
        let num_glyphs = super::parse(&[
            0x00, 0x01, 0x00, 0x00, // version: 1
            0x00, 0x01, // number of glyphs: 1
        ]).map(|n| n.get());
        assert_eq!(num_glyphs, Some(1));
    }

    #[test]
    fn unknown_version() {
        let num_glyphs = super::parse(&[
            0x00, 0x00, 0x00, 0x00, // version: 0
            0x00, 0x01, // number of glyphs: 1
        ]).map(|n| n.get());
        assert_eq!(num_glyphs, None);
    }

    #[test]
    fn zero_glyphs() {
        let num_glyphs = super::parse(&[
            0x00, 0x00, 0x50, 0x00, // version: 0.3125
            0x00, 0x00, // number of glyphs: 0
        ]).map(|n| n.get());
        assert_eq!(num_glyphs, None);
    }

    // TODO: what to do when the number of glyphs is 0xFFFF?
    //       we're actually checking this in loca
}
