//! The [loca](https://docs.microsoft.com/en-us/typography/opentype/spec/loca)
//! table parsing primitives.

use crate::stream::Stream;
use crate::{Font, Range32, GlyphId};

/// A glyph location resolving error.
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Font doesn't have a glyph with such ID.
    OutOfRange,

    /// Glyph doesn't have an outline.
    NoOutline,

    /// Malformed `loca` table data.
    InvalidRange,

    /// An invalid *index to location format* set in the `head` table.
    InvalidVersion,

    /// An invalid table length.
    LengthMismatch,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::OutOfRange => {
                write!(f, "glyph is out of range")
            }
            Error::NoOutline => {
                write!(f, "glyph has no outline")
            }
            Error::InvalidRange => {
                write!(f, "glyph offsets are not in an ascending order")
            }
            Error::InvalidVersion => {
                write!(f, "an invalid index to location format")
            }
            Error::LengthMismatch => {
                write!(f, "table's length doesn't match maxp.numGlyphs")
            }
        }
    }
}

impl std::error::Error for Error {}


impl<'a> Font<'a> {
    pub(crate) fn glyph_range(&self, glyph_id: GlyphId) -> Result<Range32, Error> {
        use crate::head::IndexToLocationFormat as Format;
        const U16_LEN: u32 = 2;
        const U32_LEN: u32 = 4;

        if glyph_id >= self.number_of_glyphs {
            return Err(Error::OutOfRange);
        }

        let format = self.index_to_location_format().ok_or(Error::InvalidVersion)?;

        // 'There is an extra entry after the last valid index.'
        // That's why we have `+ 1`.
        let expected_len = match format {
            Format::Short => (self.number_of_glyphs.0 as u32 + 1) * U16_LEN,
            Format::Long  => (self.number_of_glyphs.0 as u32 + 1) * U32_LEN,
        };

        // 'Most routines will look at the 'maxp' table to determine
        // the number of glyphs in the font, but the value
        // in the 'loca' table must agree.'
        if self.loca.length != expected_len {
            return Err(Error::LengthMismatch);
        }

        let mut s = Stream::new(&self.data[self.loca.range()]);

        let (start, end) = match format {
            Format::Short => {
                s.skip((glyph_id.0 as u32 * U16_LEN) as usize);
                // 'The actual local offset divided by 2 is stored.'
                (s.read_u16() as u32 * 2, s.read_u16() as u32 * 2)
            }
            Format::Long  => {
                s.skip((glyph_id.0 as u32 * U32_LEN) as usize);
                (s.read_u32(), s.read_u32())
            }
        };

        if start == end {
            // No outline.
            Err(Error::NoOutline)
        } else if start > end {
            // 'The offsets must be in ascending order.'
            Err(Error::InvalidRange)
        } else {
            Ok(start..end)
        }
    }
}
