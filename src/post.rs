// https://docs.microsoft.com/en-us/typography/opentype/spec/post

use crate::{Font, LineMetrics, GlyphId, Result};
use crate::parser::{Stream, Fixed, LazyArray};


// https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6post.html
const MACINTOSH_NAMES: &[&str] = &[
    ".notdef",
    ".null",
    "nonmarkingreturn",
    "space",
    "exclam",
    "quotedbl",
    "numbersign",
    "dollar",
    "percent",
    "ampersand",
    "quotesingle",
    "parenleft",
    "parenright",
    "asterisk",
    "plus",
    "comma",
    "hyphen",
    "period",
    "slash",
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "colon",
    "semicolon",
    "less",
    "equal",
    "greater",
    "question",
    "at",
    "A",
    "B",
    "C",
    "D",
    "E",
    "F",
    "G",
    "H",
    "I",
    "J",
    "K",
    "L",
    "M",
    "N",
    "O",
    "P",
    "Q",
    "R",
    "S",
    "T",
    "U",
    "V",
    "W",
    "X",
    "Y",
    "Z",
    "bracketleft",
    "backslash",
    "bracketright",
    "asciicircum",
    "underscore",
    "grave",
    "a",
    "b",
    "c",
    "d",
    "e",
    "f",
    "g",
    "h",
    "i",
    "j",
    "k",
    "l",
    "m",
    "n",
    "o",
    "p",
    "q",
    "r",
    "s",
    "t",
    "u",
    "v",
    "w",
    "x",
    "y",
    "z",
    "braceleft",
    "bar",
    "braceright",
    "asciitilde",
    "Adieresis",
    "Aring",
    "Ccedilla",
    "Eacute",
    "Ntilde",
    "Odieresis",
    "Udieresis",
    "aacute",
    "agrave",
    "acircumflex",
    "adieresis",
    "atilde",
    "aring",
    "ccedilla",
    "eacute",
    "egrave",
    "ecircumflex",
    "edieresis",
    "iacute",
    "igrave",
    "icircumflex",
    "idieresis",
    "ntilde",
    "oacute",
    "ograve",
    "ocircumflex",
    "odieresis",
    "otilde",
    "uacute",
    "ugrave",
    "ucircumflex",
    "udieresis",
    "dagger",
    "degree",
    "cent",
    "sterling",
    "section",
    "bullet",
    "paragraph",
    "germandbls",
    "registered",
    "copyright",
    "trademark",
    "acute",
    "dieresis",
    "notequal",
    "AE",
    "Oslash",
    "infinity",
    "plusminus",
    "lessequal",
    "greaterequal",
    "yen",
    "mu",
    "partialdiff",
    "summation",
    "product",
    "pi",
    "integral",
    "ordfeminine",
    "ordmasculine",
    "Omega",
    "ae",
    "oslash",
    "questiondown",
    "exclamdown",
    "logicalnot",
    "radical",
    "florin",
    "approxequal",
    "Delta",
    "guillemotleft",
    "guillemotright",
    "ellipsis",
    "nonbreakingspace",
    "Agrave",
    "Atilde",
    "Otilde",
    "OE",
    "oe",
    "endash",
    "emdash",
    "quotedblleft",
    "quotedblright",
    "quoteleft",
    "quoteright",
    "divide",
    "lozenge",
    "ydieresis",
    "Ydieresis",
    "fraction",
    "currency",
    "guilsinglleft",
    "guilsinglright",
    "fi",
    "fl",
    "daggerdbl",
    "periodcentered",
    "quotesinglbase",
    "quotedblbase",
    "perthousand",
    "Acircumflex",
    "Ecircumflex",
    "Aacute",
    "Edieresis",
    "Egrave",
    "Iacute",
    "Icircumflex",
    "Idieresis",
    "Igrave",
    "Oacute",
    "Ocircumflex",
    "apple",
    "Ograve",
    "Uacute",
    "Ucircumflex",
    "Ugrave",
    "dotlessi",
    "circumflex",
    "tilde",
    "macron",
    "breve",
    "dotaccent",
    "ring",
    "cedilla",
    "hungarumlaut",
    "ogonek",
    "caron",
    "Lslash",
    "lslash",
    "Scaron",
    "scaron",
    "Zcaron",
    "zcaron",
    "brokenbar",
    "Eth",
    "eth",
    "Yacute",
    "yacute",
    "Thorn",
    "thorn",
    "minus",
    "multiply",
    "onesuperior",
    "twosuperior",
    "threesuperior",
    "onehalf",
    "onequarter",
    "threequarters",
    "franc",
    "Gbreve",
    "gbreve",
    "Idotaccent",
    "Scedilla",
    "scedilla",
    "Cacute",
    "cacute",
    "Ccaron",
    "ccaron",
    "dcroat",
];


impl<'a> Font<'a> {
    /// Parses font's underline metrics.
    #[inline]
    pub fn underline_metrics(&self) -> Result<LineMetrics> {
        let mut s = Stream::new_at(self.post?, 8); // TODO: to raw
        Ok(LineMetrics {
            position: s.read()?,
            thickness: s.read()?,
        })
    }

    /// Parses glyph's name.
    ///
    /// Uses the `post` table as a source.
    ///
    /// Returns `Ok(None)` when no name is associated with a `glyph`.
    #[inline]
    pub fn glyph_name(&self, glyph: GlyphId) -> Result<Option<&str>> {
        let mut s = Stream::new(self.post?);
        let version: Fixed = s.read()?;

        // In case of version 1.0 we are using predefined set of names.
        if version.0 == 1.0 {
            return if (glyph.0 as usize) < MACINTOSH_NAMES.len() {
                Ok(Some(MACINTOSH_NAMES[glyph.0 as usize]))
            } else {
                Ok(None)
            };
        }

        // Only version 2.0 of the table has data at the end.
        if version.0 != 2.0 || s.at_end() {
            return Ok(None);
        }

        s.advance(28_u32); // Jump to the end of the base table.
        let name_indexes: LazyArray<u16> = s.read_array16()?;
        let mut index = try_ok!(name_indexes.get(glyph.0));

        // 'If the name index is between 0 and 257, treat the name index
        // as a glyph index in the Macintosh standard order.'
        if (index as usize) < MACINTOSH_NAMES.len() {
            Ok(Some(MACINTOSH_NAMES[index as usize]))
        } else {
            // 'If the name index is between 258 and 65535, then subtract 258 and use that
            // to index into the list of Pascal strings at the end of the table.'
            index -= MACINTOSH_NAMES.len() as u16;

            let mut i = 0;
            while !s.at_end() && i < core::u16::MAX {
                let len: u8 = s.read()?;

                if i == index {
                    if len == 0 {
                        // Empty name is an error.
                        break;
                    } else {
                        let name = s.read_bytes(len as u16)?;
                        return match core::str::from_utf8(name) {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None), // TODO: custom error
                        };
                    }
                } else {
                    s.advance(len as u16);
                }

                i += 1;
            }

            Ok(None)
        }
    }
}
