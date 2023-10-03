//! A [Color Palette Table](
//! https://docs.microsoft.com/en-us/typography/opentype/spec/cpal) implementation.

use crate::parser::{FromData, LazyArray16, Offset, Offset32, Stream};

/// A [Color Palette Table](
/// https://docs.microsoft.com/en-us/typography/opentype/spec/cpal).
#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    color_indices: LazyArray16<'a, u16>,
    colors: LazyArray16<'a, Color>,
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version = s.read::<u16>()?;
        if version > 1 {
            return None;
        }

        s.skip::<u16>(); // number of palette entries

        let num_palettes = s.read::<u16>()?;
        let num_colors = s.read::<u16>()?;
        let color_records_offset = s.read::<Offset32>()?;
        let color_indices = s.read_array16::<u16>(num_palettes)?;

        let colors = Stream::new_at(data, color_records_offset.to_usize())?
            .read_array16::<Color>(num_colors)?;

        Some(Self {
            color_indices,
            colors,
        })
    }

    /// Returns the color at the given index into the given palette.
    pub fn get(&self, palette: u16, palette_entry: u16) -> Option<Color> {
        let index = self
            .color_indices
            .get(palette)?
            .checked_add(palette_entry)?;
        self.colors.get(index)
    }
}

/// A BGRA color in sRGB.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Color {
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub alpha: u8,
}

impl FromData for Color {
    const SIZE: usize = 4;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            blue: s.read::<u8>()?,
            green: s.read::<u8>()?,
            red: s.read::<u8>()?,
            alpha: s.read::<u8>()?,
        })
    }
}
