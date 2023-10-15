//! A [Color Table](
//! https://docs.microsoft.com/en-us/typography/opentype/spec/colr) implementation.

use crate::cpal;
use crate::parser::{FromData, LazyArray16, Offset, Offset32, Stream};
use crate::{GlyphId, RgbaColor};

/// A [base glyph](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyph-and-layer-records).
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

/// A [layer](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyph-and-layer-records).
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
///
/// See [COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr) for details.
pub trait Painter {
    /// Outlines a glyph and stores it until the next paint command.
    fn outline(&mut self, glyph_id: GlyphId);
    /// Paints the current glyph outline using the application provided text foreground color.
    fn paint_foreground(&mut self);
    /// Paints the current glyph outline using the provided color.
    fn paint_color(&mut self, color: RgbaColor);
}

/// A [Color Table](
/// https://docs.microsoft.com/en-us/typography/opentype/spec/colr).
///
/// Currently, only version 0 is supported.
#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    pub(crate) palettes: cpal::Table<'a>,
    base_glyphs: LazyArray16<'a, BaseGlyphRecord>,
    layers: LazyArray16<'a, LayerRecord>,
}

impl<'a> Table<'a> {
    /// Parses a table from raw data.
    pub fn parse(palettes: cpal::Table<'a>, data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version = s.read::<u16>()?;
        if version != 0 {
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

    fn get(&self, glyph_id: GlyphId) -> Option<BaseGlyphRecord> {
        self.base_glyphs
            .binary_search_by(|base| base.glyph_id.cmp(&glyph_id))
            .map(|v| v.1)
    }

    /// Whether the table contains a definition for the given glyph.
    pub fn contains(&self, glyph_id: GlyphId) -> bool {
        self.get(glyph_id).is_some()
    }

    /// Paints the color glyph.
    pub fn paint(&self, glyph_id: GlyphId, palette: u16, painter: &mut dyn Painter) -> Option<()> {
        let base = self.get(glyph_id)?;
        let start = base.first_layer_index;
        let end = start.checked_add(base.num_layers)?;
        let layers = self.layers.slice(start..end)?;

        for layer in layers {
            if layer.palette_index == 0xFFFF {
                // A special case.
                painter.outline(layer.glyph_id);
                painter.paint_foreground();
            } else {
                let color = self.palettes.get(palette, layer.palette_index)?;
                painter.outline(layer.glyph_id);
                painter.paint_color(color);
            }
        }

        Some(())
    }
}
