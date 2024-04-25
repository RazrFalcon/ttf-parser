//! A [Color Table](
//! https://docs.microsoft.com/en-us/typography/opentype/spec/colr) implementation.

use crate::parser::{FromData, LazyArray16, NumFrom, Offset, Offset24, Offset32, Stream, F2DOT14};
use crate::svg::SvgDocument;
use crate::{cpal, Fixed, LazyArray32, Transform};
use crate::{GlyphId, RgbaColor};
use std::vec::Vec;

/// A [base glyph](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyph-and-layer-records).
#[derive(Clone, Copy, Debug)]
struct BaseGlyphRecord {
    glyph_id: GlyphId,
    first_layer_index: u16,
    num_layers: u16,
}

#[derive(Clone, Copy, Debug)]
struct ClipBox {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

/// A [clip record](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyphlist-layerlist-and-cliplist).
#[derive(Clone, Copy, Debug)]
struct ClipRecord {
    /// The first glyph ID for the range covered by this record.
    pub start_glyph_id: GlyphId,
    /// The last glyph ID, *inclusive*, for the range covered by this record.
    pub end_glyph_id: GlyphId,
    pub clip_box_offset: Offset24,
}

impl FromData for ClipRecord {
    const SIZE: usize = 7;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(ClipRecord {
            start_glyph_id: s.read::<GlyphId>()?,
            end_glyph_id: s.read::<GlyphId>()?,
            clip_box_offset: s.read::<Offset24>()?,
        })
    }
}

impl ClipRecord {
    /// Returns the glyphs range.
    pub fn glyphs_range(&self) -> core::ops::RangeInclusive<GlyphId> {
        self.start_glyph_id..=self.end_glyph_id
    }
}

/// A [clip list](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyphlist-layerlist-and-cliplist).
#[derive(Clone, Copy, Debug)]
struct ClipList<'a> {
    data: &'a [u8],
    records: LazyArray32<'a, ClipRecord>,
}

impl Default for ClipList<'_> {
    fn default() -> Self {
        Self {
            data: &[],
            records: LazyArray32::default(),
        }
    }
}

impl<'a> ClipList<'a> {
    #[inline]
    pub fn get(&self, index: u32) -> Option<ClipBox> {
        let record = self.records.get(index)?;
        let offset = record.clip_box_offset.to_usize();
        self.data.get(offset..).and_then(|data| {
            let mut s = Stream::new(data);
            // TODO: Add format 2
            // Format = 1
            s.read::<u8>()?;

            Some(ClipBox {
                x_min: s.read::<i16>()?,
                y_min: s.read::<i16>()?,
                x_max: s.read::<i16>()?,
                y_max: s.read::<i16>()?,
            })
        })
    }

    /// Returns a ClipBox by glyph ID.
    #[inline]
    pub fn find(&self, glyph_id: GlyphId) -> Option<ClipBox> {
        let index = self
            .records
            .into_iter()
            .position(|v| v.glyphs_range().contains(&glyph_id))?;
        self.get(index as u32)
    }
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

/// A [BaseGlyphPaintRecord](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyphlist-layerlist-and-cliplist).
#[derive(Clone, Copy, Debug)]
struct BaseGlyphPaintRecord {
    glyph_id: GlyphId,
    paint_table_offset: Offset32,
}

impl FromData for BaseGlyphPaintRecord {
    const SIZE: usize = 6;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            glyph_id: s.read::<GlyphId>()?,
            paint_table_offset: s.read::<Offset32>()?,
        })
    }
}

/// A [gradient extend](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyphlist-layerlist-and-cliplist).
#[derive(Clone, Copy, Debug)]
pub enum GradientExtend {
    Pad,
    Repeat,
    Reflect,
}

impl FromData for GradientExtend {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        match data[0] {
            0 => Some(Self::Pad),
            1 => Some(Self::Repeat),
            2 => Some(Self::Reflect),
            _ => None,
        }
    }
}

/// A [gradient extend](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyphlist-layerlist-and-cliplist).
#[derive(Clone, Copy, Debug)]
struct ColorStopRaw {
    stop_offset: F2DOT14,
    palette_index: u16,
    alpha: F2DOT14,
}

impl FromData for ColorStopRaw {
    const SIZE: usize = 6;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            stop_offset: s.read::<F2DOT14>()?,
            palette_index: s.read::<u16>()?,
            alpha: s.read::<F2DOT14>()?,
        })
    }
}

#[derive(Clone)]
struct ColorLine<'a> {
    extend: GradientExtend,
    colors: LazyArray16<'a, ColorStopRaw>,
    palettes: cpal::Table<'a>,
}

impl ColorLine<'_> {
    fn get(&self, palette: u16, foreground_color: RgbaColor, index: u16) -> Option<ColorStop> {
        let info = self.colors.get(index)?;

        let mut color = if info.palette_index == u16::MAX {
            foreground_color
        } else {
            self.palettes.get(palette, info.palette_index)?
        };

        color.apply_alpha(info.alpha.to_f32());
        Some(ColorStop {
            stop_offset: info.stop_offset.to_f32(),
            color,
        })
    }
}

/// A [gradient extend](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyphlist-layerlist-and-cliplist).
#[derive(Clone, Copy, Debug)]
pub struct ColorStop {
    pub stop_offset: f32,
    pub color: RgbaColor,
}

#[derive(Clone)]
pub struct LinearGradient<'a> {
    pub x0: i16,
    pub y0: i16,
    pub x1: i16,
    pub y1: i16,
    pub x2: i16,
    pub y2: i16,
    pub extend: GradientExtend,
    color_line: ColorLine<'a>,
}

impl<'a> core::fmt::Debug for LinearGradient<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("LinearGradient")
            .field("x0", &self.x0)
            .field("y0", &self.y0)
            .field("x1", &self.x1)
            .field("y1", &self.y1)
            .field("x2", &self.x2)
            .field("y2", &self.y2)
            .field("extend", &self.extend)
            // TODO: Avoid hardcoding foregrounf color here?
            .field("stops", &self.stops(0, RgbaColor::new(0, 0, 0, 255)))
            .finish()
    }
}

impl<'a> LinearGradient<'a> {
    pub fn stops<'b>(
        &'b self,
        palette: u16,
        foreground_color: RgbaColor,
    ) -> GradientStopsIter<'a, 'b> {
        GradientStopsIter {
            color_line: &self.color_line,
            foreground_color,
            palette,
            index: 0,
        }
    }
}

#[derive(Clone)]
pub struct RadialGradient<'a> {
    pub x0: i16,
    pub y0: i16,
    pub r0: u16,
    pub r1: u16,
    pub x1: i16,
    pub y1: i16,
    pub extend: GradientExtend,
    color_line: ColorLine<'a>,
}

impl<'a> core::fmt::Debug for RadialGradient<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RadialGradient")
            .field("x0", &self.x0)
            .field("y0", &self.y0)
            .field("r0", &self.r0)
            .field("r1", &self.r1)
            .field("x1", &self.x1)
            .field("y1", &self.y1)
            .field("extend", &self.extend)
            // TODO: Avoid hardcoding foregrounf color here?
            .field("stops", &self.stops(0, RgbaColor::new(0, 0, 0, 255)))
            .finish()
    }
}

impl<'a> RadialGradient<'a> {
    pub fn stops<'b>(
        &'b self,
        palette: u16,
        foreground_color: RgbaColor,
    ) -> GradientStopsIter<'a, 'b> {
        GradientStopsIter {
            color_line: &self.color_line,
            palette,
            foreground_color,
            index: 0,
        }
    }
}

#[derive(Clone)]
pub struct SweepGradient<'a> {
    pub center_x: i16,
    pub center_y: i16,
    pub start_angle: f32,
    pub end_angle: f32,
    pub extend: GradientExtend,
    color_line: ColorLine<'a>,
}

impl<'a> core::fmt::Debug for SweepGradient<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SweepGradient")
            .field("center_x", &self.center_x)
            .field("center_y", &self.center_y)
            .field("start_angle", &self.start_angle)
            .field("end_angle", &self.end_angle)
            .field("extend", &self.extend)
            // TODO: Avoid hardcoding foregrounf color here?
            .field("stops", &self.stops(0, RgbaColor::new(0, 0, 0, 255)))
            .finish()
    }
}

impl<'a> SweepGradient<'a> {
    pub fn stops<'b>(
        &'b self,
        palette: u16,
        foreground_color: RgbaColor,
    ) -> GradientStopsIter<'a, 'b> {
        GradientStopsIter {
            color_line: &self.color_line,
            palette,
            foreground_color,
            index: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GradientStopsIter<'a, 'b> {
    color_line: &'b ColorLine<'a>,
    palette: u16,
    foreground_color: RgbaColor,
    index: u16,
}

impl Iterator for GradientStopsIter<'_, '_> {
    type Item = ColorStop;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.color_line.colors.len() {
            return None;
        }

        let index = self.index;
        self.index = self.index.checked_add(1)?;
        self.color_line
            .get(self.palette, self.foreground_color, index)
    }
}

impl core::fmt::Debug for GradientStopsIter<'_, '_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(*self).finish()
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CompositeMode {
    Clear,
    Source,
    Destination,
    SourceOver,
    DestinationOver,
    SourceIn,
    DestinationIn,
    SourceOut,
    DestinationOut,
    SourceAtop,
    DestinationAtop,
    Xor,
    Plus,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Multiply,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

impl FromData for CompositeMode {
    const SIZE: usize = 1;

    fn parse(data: &[u8]) -> Option<Self> {
        match data[0] {
            0 => Some(Self::Clear),
            1 => Some(Self::Source),
            2 => Some(Self::Destination),
            3 => Some(Self::SourceOver),
            4 => Some(Self::DestinationOver),
            5 => Some(Self::SourceIn),
            6 => Some(Self::DestinationIn),
            7 => Some(Self::SourceOut),
            8 => Some(Self::DestinationOut),
            9 => Some(Self::SourceAtop),
            10 => Some(Self::DestinationAtop),
            11 => Some(Self::Xor),
            12 => Some(Self::Plus),
            13 => Some(Self::Screen),
            14 => Some(Self::Overlay),
            15 => Some(Self::Darken),
            16 => Some(Self::Lighten),
            17 => Some(Self::ColorDodge),
            18 => Some(Self::ColorBurn),
            19 => Some(Self::HardLight),
            20 => Some(Self::SoftLight),
            21 => Some(Self::Difference),
            22 => Some(Self::Exclusion),
            23 => Some(Self::Multiply),
            24 => Some(Self::Hue),
            25 => Some(Self::Saturation),
            26 => Some(Self::Color),
            27 => Some(Self::Luminosity),
            _ => None,
        }
    }
}

/// A trait for color glyph painting.
///
/// See [COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr) for details.
pub trait Painter<'a> {
    /// Outlines a glyph and stores it until the next paint command.
    fn outline(&mut self, glyph_id: GlyphId);
    /// Paints the current glyph outline using the application provided text foreground color.
    fn paint_foreground(&mut self);
    /// Paints the current glyph outline using the provided color.
    fn paint_color(&mut self, color: RgbaColor);
    fn paint_linear_gradient(&mut self, gradient: LinearGradient<'a>);
    fn paint_radial_gradient(&mut self, gradient: RadialGradient<'a>);
    fn paint_sweep_gradient(&mut self, gradient: SweepGradient<'a>);

    fn clip(&mut self);

    fn push_clip_box(&mut self, x_min: i16, y_min: i16, x_max: i16, y_max: i16);

    fn pop_clip_box(&mut self);

    fn push_isolate(&mut self);
    fn pop_isolate(&mut self);

    fn push_group(&mut self, mode: CompositeMode);
    fn pop_group(&mut self);

    fn translate(&mut self, tx: f32, ty: f32);
    fn scale(&mut self, sx: f32, sy: f32);
    /// Explain why.
    fn rotate(&mut self, angle: f32);
    fn skew(&mut self, skew_x: f32, skew_y: f32);
    fn transform(&mut self, transform: Transform);
    fn pop_transform(&mut self);
}

/// A [Color Table](
/// https://docs.microsoft.com/en-us/typography/opentype/spec/colr).
///
/// Currently, only version 0 is supported.
#[derive(Clone, Copy, Debug)]
pub struct Table<'a> {
    pub(crate) palettes: cpal::Table<'a>,
    data: &'a [u8],
    version: u8,
    // v0
    base_glyphs: LazyArray16<'a, BaseGlyphRecord>,
    layers: LazyArray16<'a, LayerRecord>,
    // v1
    base_glyph_paints_offset: Offset32,
    base_glyph_paints: LazyArray32<'a, BaseGlyphPaintRecord>,
    layer_paint_offsets_offset: Offset32,
    layer_paint_offsets: LazyArray32<'a, Offset32>,
    clip_list_offsets_offset: Offset32,
    clip_list: ClipList<'a>,
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

        let mut table = Self {
            version: version as u8,
            data,
            palettes,
            base_glyphs,
            layers,
            base_glyph_paints_offset: Offset32(0), // the actual value doesn't matter
            base_glyph_paints: LazyArray32::default(),
            layer_paint_offsets_offset: Offset32(0),
            layer_paint_offsets: LazyArray32::default(),
            clip_list_offsets_offset: Offset32(0),
            clip_list: ClipList::default(),
        };

        if version == 0 {
            return Some(table);
        }

        table.base_glyph_paints_offset = s.read::<Offset32>()?;
        let layer_list_offset = s.read::<Option<Offset32>>()?;
        let clip_list_offset = s.read::<Option<Offset32>>()?;

        {
            let mut s = Stream::new_at(data, table.base_glyph_paints_offset.to_usize())?;
            let count = s.read::<u32>()?;
            table.base_glyph_paints = s.read_array32::<BaseGlyphPaintRecord>(count)?;
        }

        if let Some(offset) = layer_list_offset {
            table.layer_paint_offsets_offset = offset;
            let mut s = Stream::new_at(data, offset.to_usize())?;
            let count = s.read::<u32>()?;
            table.layer_paint_offsets = s.read_array32::<Offset32>(count)?;
        }

        if let Some(offset) = clip_list_offset {
            table.clip_list_offsets_offset = offset;
            let clip_data = data.get(offset.to_usize()..)?;
            let mut s = Stream::new(clip_data);
            // Format
            s.read::<u8>()?;
            let count = s.read::<u32>()?;
            table.clip_list = ClipList {
                data: clip_data,
                records: s.read_array32::<ClipRecord>(count)?,
            };
        }

        Some(table)
    }

    /// Returns `true` if the current table has version 0.
    ///
    /// A simple table can only emit `outline`, `paint_foreground` and `paint_color`
    /// [`Painter`] methods.
    pub fn is_simple(&self) -> bool {
        self.version == 0
    }

    fn get_v0(&self, glyph_id: GlyphId) -> Option<BaseGlyphRecord> {
        self.base_glyphs
            .binary_search_by(|base| base.glyph_id.cmp(&glyph_id))
            .map(|v| v.1)
    }

    fn get_v1(&self, glyph_id: GlyphId) -> Option<BaseGlyphPaintRecord> {
        self.base_glyph_paints
            .binary_search_by(|base| base.glyph_id.cmp(&glyph_id))
            .map(|v| v.1)
    }

    /// Whether the table contains a definition for the given glyph.
    pub fn contains(&self, glyph_id: GlyphId) -> bool {
        self.get_v1(glyph_id).is_some() || self.get_v0(glyph_id).is_some()
    }

    /// Paints the color glyph.
    pub fn paint(
        &self,
        glyph_id: GlyphId,
        palette: u16,
        painter: &mut dyn Painter<'a>,
    ) -> Option<()> {
        if let Some(base) = self.get_v1(glyph_id) {
            self.paint_v1(base, palette, painter)
        } else if let Some(base) = self.get_v0(glyph_id) {
            self.paint_v0(base, palette, painter)
        } else {
            None
        }
    }

    fn paint_v0(
        &self,
        base: BaseGlyphRecord,
        palette: u16,
        painter: &mut dyn Painter,
    ) -> Option<()> {
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

    fn paint_v1(
        &self,
        base: BaseGlyphPaintRecord,
        palette: u16,
        painter: &mut dyn Painter<'a>,
    ) -> Option<()> {
        let clip_box = self.clip_list.find(base.glyph_id);
        if let Some(clip_box) = clip_box {
            painter.push_clip_box(
                clip_box.x_min,
                clip_box.y_min,
                clip_box.x_max,
                clip_box.y_max,
            );
        }

        self.parse_paint(
            self.base_glyph_paints_offset.to_usize() + base.paint_table_offset.to_usize(),
            palette,
            painter,
        )?;

        if clip_box.is_some() {
            painter.pop_clip_box();
        }

        Some(())
    }

    fn parse_paint(
        &self,
        offset: usize,
        palette: u16,
        painter: &mut dyn Painter<'a>,
    ) -> Option<()> {
        let mut s = Stream::new_at(self.data, offset)?;
        let format = s.read::<u8>()?;
        match format {
            1 => {
                // PaintColrLayers
                let layers_count = s.read::<u8>()?;
                let first_layer_index = s.read::<u32>()?;

                for i in 0..layers_count {
                    let index = first_layer_index.checked_add(u32::from(i))?;
                    let paint_offset = self.layer_paint_offsets.get(index)?;
                    self.parse_paint(
                        self.layer_paint_offsets_offset.to_usize() + paint_offset.to_usize(),
                        palette,
                        painter,
                    );
                }
            }
            2 => {
                // PaintSolid
                let palette_index = s.read::<u16>()?;
                let alpha = s.read::<F2DOT14>()?;
                let mut color = self.palettes.get(palette, palette_index)?;
                color.apply_alpha(alpha.to_f32());
                painter.paint_color(color);
            }
            4 => {
                // PaintLinearGradient
                let color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_color_line(offset + color_line_offset.to_usize())?;
                painter.paint_linear_gradient(LinearGradient {
                    x0: s.read::<i16>()?,
                    y0: s.read::<i16>()?,
                    x1: s.read::<i16>()?,
                    y1: s.read::<i16>()?,
                    x2: s.read::<i16>()?,
                    y2: s.read::<i16>()?,
                    extend: color_line.extend,
                    color_line,
                })
            }
            6 => {
                // PaintRadialGradient
                let color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_color_line(offset + color_line_offset.to_usize())?;
                painter.paint_radial_gradient(RadialGradient {
                    x0: s.read::<i16>()?,
                    y0: s.read::<i16>()?,
                    r0: s.read::<u16>()?,
                    x1: s.read::<i16>()?,
                    y1: s.read::<i16>()?,
                    r1: s.read::<u16>()?,
                    extend: color_line.extend,
                    color_line,
                })
            }
            8 => {
                // PaintSweepGradient
                let color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_color_line(offset + color_line_offset.to_usize())?;
                painter.paint_sweep_gradient(SweepGradient {
                    center_x: s.read::<i16>()?,
                    center_y: s.read::<i16>()?,
                    start_angle: s.read::<F2DOT14>()?.to_f32(),
                    end_angle: s.read::<F2DOT14>()?.to_f32(),
                    extend: color_line.extend,
                    color_line,
                })
            }
            10 => {
                // PaintGlyph
                let paint_offset = s.read::<Offset24>()?;
                let glyph_id = s.read::<GlyphId>()?;
                painter.outline(glyph_id);
                painter.clip();

                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
            }
            11 => {
                // PaintColrGlyph
                let glyph_id = s.read::<GlyphId>()?;
                self.paint(glyph_id, palette, painter);
            }
            12 => {
                // PaintTransform
                let paint_offset = s.read::<Offset24>()?;
                let ts_offset = s.read::<Offset24>()?;
                let mut s = Stream::new_at(self.data, offset + ts_offset.to_usize())?;
                let ts = Transform {
                    a: s.read::<Fixed>().map(|n| n.0)?,
                    b: s.read::<Fixed>().map(|n| n.0)?,
                    c: s.read::<Fixed>().map(|n| n.0)?,
                    d: s.read::<Fixed>().map(|n| n.0)?,
                    e: s.read::<Fixed>().map(|n| n.0)?,
                    f: s.read::<Fixed>().map(|n| n.0)?,
                };

                painter.transform(ts);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
            }
            14 => {
                // PaintTranslate
                let paint_offset = s.read::<Offset24>()?;
                let tx = f32::from(s.read::<i16>()?);
                let ty = f32::from(s.read::<i16>()?);

                painter.translate(tx, ty);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
            }
            16 => {
                // PaintScale
                let paint_offset = s.read::<Offset24>()?;
                let sx = s.read::<F2DOT14>()?.to_f32();
                let sy = s.read::<F2DOT14>()?.to_f32();

                painter.scale(sx, sy);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
            }
            18 => {
                // PaintScaleAroundCenter
                let paint_offset = s.read::<Offset24>()?;
                let sx = s.read::<F2DOT14>()?.to_f32();
                let sy = s.read::<F2DOT14>()?.to_f32();
                let center_x = f32::from(s.read::<i16>()?);
                let center_y = f32::from(s.read::<i16>()?);

                painter.translate(center_x, center_y);
                painter.scale(sx, sy);
                painter.translate(-center_x, -center_y);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            20 => {
                // PaintScaleUniform
                let paint_offset = s.read::<Offset24>()?;
                let scale = s.read::<F2DOT14>()?.to_f32();

                painter.scale(scale, scale);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
            }
            22 => {
                // PaintScaleUniformAroundCenter
                let paint_offset = s.read::<Offset24>()?;
                let scale = s.read::<F2DOT14>()?.to_f32();
                let center_x = f32::from(s.read::<i16>()?);
                let center_y = f32::from(s.read::<i16>()?);

                painter.translate(center_x, center_y);
                painter.scale(scale, scale);
                painter.translate(-center_x, -center_y);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            24 => {
                // PaintRotate
                let paint_offset = s.read::<Offset24>()?;
                let angle = s.read::<F2DOT14>()?.to_f32();

                painter.rotate(angle);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
            }
            26 => {
                // PaintRotate
                let paint_offset = s.read::<Offset24>()?;
                let angle = s.read::<F2DOT14>()?.to_f32();
                let center_x = f32::from(s.read::<i16>()?);
                let center_y = f32::from(s.read::<i16>()?);

                painter.translate(center_x, center_y);
                painter.rotate(angle);
                painter.translate(-center_x, -center_y);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            28 => {
                // PaintSkew
                let paint_offset = s.read::<Offset24>()?;
                let skew_x = s.read::<F2DOT14>()?.to_f32();
                let skew_y = s.read::<F2DOT14>()?.to_f32();

                painter.skew(skew_x, skew_y);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
            }
            30 => {
                // PaintSkewAroundCenter
                let paint_offset = s.read::<Offset24>()?;
                let skew_x = s.read::<F2DOT14>()?.to_f32();
                let skew_y = s.read::<F2DOT14>()?.to_f32();
                let center_x = f32::from(s.read::<i16>()?);
                let center_y = f32::from(s.read::<i16>()?);

                painter.translate(center_x, center_y);
                painter.skew(skew_x, skew_y);
                painter.translate(-center_x, -center_y);
                self.parse_paint(offset + paint_offset.to_usize(), palette, painter);
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            32 => {
                // PaintComposite
                let source_paint_offset = s.read::<Offset24>()?;
                let composite_mode = s.read::<CompositeMode>()?;
                let backdrop_paint_offset = s.read::<Offset24>()?;

                painter.push_isolate();
                self.parse_paint(offset + backdrop_paint_offset.to_usize(), palette, painter);
                painter.push_group(composite_mode);
                self.parse_paint(offset + source_paint_offset.to_usize(), palette, painter);
                painter.pop_group();
                painter.pop_isolate();
            }
            _ => {}
        }

        Some(())
    }

    fn parse_color_line(&self, offset: usize) -> Option<ColorLine<'a>> {
        let mut s = Stream::new_at(self.data, offset)?;
        let extend = s.read::<GradientExtend>()?;
        let count = s.read::<u16>()?;
        let colors = s.read_array16::<ColorStopRaw>(count)?;
        Some(ColorLine {
            extend,
            colors,
            palettes: self.palettes,
        })
    }
}
