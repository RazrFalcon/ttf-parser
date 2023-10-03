//! A [Color Table](
//! https://docs.microsoft.com/en-us/typography/opentype/spec/colr) implementation.

use crate::cpal::{self, Color};
use crate::parser::{FromData, LazyArray16, Offset, Offset32, Stream};
use crate::GlyphId;

/// A [base glyph](https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyph-and-layer-records).
#[derive(Clone, Copy, Debug)]
struct BaseGlyphRecord {
    glyph_id: GlyphId,
    first_layer_index: u16,
    num_layers: u16,
}

impl FromData for BaseGlyphRecord {
    const SIZE: usize = 6;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            glyph_id: s.read::<GlyphId>()?,
            first_layer_index: s.read::<u16>()?,
            num_layers: s.read::<u16>()?,
        })
    }
}

/// A [layer](https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyph-and-layer-records).
#[derive(Clone, Copy, Debug)]
struct LayerRecord {
    glyph_id: GlyphId,
    palette_index: u16,
}

impl FromData for LayerRecord {
    const SIZE: usize = 4;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            glyph_id: s.read::<GlyphId>()?,
            palette_index: s.read::<u16>()?,
        })
    }
}

/// A trait for color glyph painting.
pub trait ColorGlyphPainter {
    /// Paints an outline glyph in the given color.
    fn glyph(&mut self, id: GlyphId, color: Color);
}

/// A [Color Table](
/// https://docs.microsoft.com/en-us/typography/opentype/spec/colr).
///
/// Currently, only version 0 is supported.
#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    palettes: cpal::Table<'a>,
    base_glyphs: LazyArray16<'a, BaseGlyphRecord>,
    layers: LazyArray16<'a, LayerRecord>,
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(palettes: cpal::Table<'a>, data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version = s.read::<u16>()?;
        if version > 1 {
            return None;
        }

        let num_base_glyphs = s.read::<u16>()?;
        let base_glyphs_offset = s.read::<Offset32>()?;
        let layers_offset = s.read::<Offset32>()?;
        let num_layers = s.read::<u16>()?;

        let base_glyphs = Stream::new_at(data, base_glyphs_offset.to_usize())?
            .read_array16::<BaseGlyphRecord>(num_base_glyphs)?;

        let layers = Stream::new_at(data, layers_offset.to_usize())?
            .read_array16::<LayerRecord>(num_layers)?;

        Some(Self {
            palettes,
            base_glyphs,
            layers,
        })
    }

    /// Paints the color glyph.
    pub fn paint(
        &self,
        glyph_id: GlyphId,
        palette: u16,
        painter: &mut dyn ColorGlyphPainter,
    ) -> Option<()> {
        let (_, base) = self
            .base_glyphs
            .binary_search_by(|base| base.glyph_id.cmp(&glyph_id))?;

        let start = base.first_layer_index;
        let end = start.checked_add(base.num_layers)?;
        let layers = self.layers.slice(start..end)?;

        for layer in layers {
            let color = self.palettes.get(palette, layer.palette_index)?;
            painter.glyph(layer.glyph_id, color);
        }

        Some(())
    }
}
