//! A [Color Table](
//! https://docs.microsoft.com/en-us/typography/opentype/spec/colr) implementation.

// NOTE: Parts of the implementation have been inspired by
// [skrifa](https://github.com/googlefonts/fontations/tree/main/skrifa).

use crate::hvar::DeltaSetIndexMap;
use crate::parser::{FromData, LazyArray16, Offset, Offset24, Offset32, Stream, F2DOT14};
use crate::var_store::ItemVariationStore;
use crate::{cpal, Fixed, LazyArray32, NormalizedCoordinate, Transform};
use crate::{GlyphId, RgbaColor};

/// A [base glyph](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyph-and-layer-records).
#[derive(Clone, Copy, Debug)]
struct BaseGlyphRecord {
    glyph_id: GlyphId,
    first_layer_index: u16,
    num_layers: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct ClipBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

#[derive(Clone, Debug)]
pub enum Paint<'a> {
    Solid(RgbaColor),
    LinearGradient(LinearGradient<'a>),
    RadialGradient(RadialGradient<'a>),
    SweepGradient(SweepGradient<'a>),
}

/// A [clip record](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#baseglyphlist-layerlist-and-cliplist).
#[derive(Clone, Copy, Debug)]
struct ClipRecord {
    /// The first glyph ID for the range covered by this record.
    pub start_glyph_id: GlyphId,
    /// The last glyph ID, *inclusive*, for the range covered by this record.
    pub end_glyph_id: GlyphId,
    /// The offset to the clip box.
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
    pub fn get(&self, index: u32, variation_data: &VariationData, coords: &[NormalizedCoordinate]) -> Option<ClipBox> {
        let record = self.records.get(index)?;
        let offset = record.clip_box_offset.to_usize();
        self.data.get(offset..).and_then(|data| {
            let mut s = Stream::new(data);
            let format = s.read::<u8>()?;

            let deltas = if format == 2 {
                let mut var_s = s.clone();
                var_s.advance(8);
                let var_index_base = var_s.read::<u32>()?;

                variation_data
                    .read_deltas::<4>(var_index_base, coords)
            }   else {
                [0.0, 0.0, 0.0, 0.0]
            };

            Some(ClipBox {
                x_min: s.read::<i16>()? as f32 + deltas[0],
                y_min: s.read::<i16>()? as f32 + deltas[0],
                x_max: s.read::<i16>()? as f32 + deltas[0],
                y_max: s.read::<i16>()? as f32 + deltas[0],
            })
        })
    }

    /// Returns a ClipBox by glyph ID.
    #[inline]
    pub fn find(&self, glyph_id: GlyphId, variation_data: &VariationData, coords: &[NormalizedCoordinate]) -> Option<ClipBox> {
        let index = self
            .records
            .into_iter()
            .position(|v| v.glyphs_range().contains(&glyph_id))?;
        self.get(index as u32, variation_data, coords)
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

/// A [color stop](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#color-references-colorstop-and-colorline).
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

/// A [var color stop](
/// https://learn.microsoft.com/en-us/typography/opentype/spec/colr#color-references-colorstop-and-colorline).
#[derive(Clone, Copy, Debug)]
struct VarColorStopRaw {
    stop_offset: F2DOT14,
    palette_index: u16,
    alpha: F2DOT14,
    var_index_base: u32,
}

impl FromData for VarColorStopRaw {
    const SIZE: usize = 10;

    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(Self {
            stop_offset: s.read::<F2DOT14>()?,
            palette_index: s.read::<u16>()?,
            alpha: s.read::<F2DOT14>()?,
            var_index_base: s.read::<u32>()?,
        })
    }
}

#[derive(Clone)]
struct NonVarColorLine<'a> {
    extend: GradientExtend,
    colors: LazyArray16<'a, ColorStopRaw>,
    palettes: cpal::Table<'a>,
    foreground_color: RgbaColor,
}

impl NonVarColorLine<'_> {
    // TODO: Color stops should be sorted, but hard to do without allocations
    fn get(&self, palette: u16, index: u16) -> Option<ColorStop> {
        let info = self.colors.get(index)?;

        let mut color = if info.palette_index == u16::MAX {
            self.foreground_color
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

impl VarColorLine<'_> {
    // TODO: Color stops should be sorted, but hard to do without allocations
    fn get(
        &self,
        palette: u16,
        index: u16,
        variation_data: VariationData,
        coordinates: &[NormalizedCoordinate],
    ) -> Option<ColorStop> {
        let info = self.colors.get(index)?;

        let mut color = if info.palette_index == u16::MAX {
            self.foreground_color
        } else {
            self.palettes.get(palette, info.palette_index)?
        };

        let deltas = variation_data.read_deltas::<2>(info.var_index_base, coordinates);
        let stop_offset = info.stop_offset.apply_float_delta(deltas[0]);
        color.apply_alpha(info.alpha.apply_float_delta(deltas[1]));

        Some(ColorStop { stop_offset, color })
    }
}

#[derive(Clone)]
struct VarColorLine<'a> {
    extend: GradientExtend,
    colors: LazyArray16<'a, VarColorStopRaw>,
    palettes: cpal::Table<'a>,
    foreground_color: RgbaColor,
}

#[derive(Clone)]
enum ColorLine<'a> {
    VarColorLine(VarColorLine<'a>),
    NonVarColorLine(NonVarColorLine<'a>),
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
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
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
            // .field("stops", &self.stops(0))
            .finish()
    }
}

impl<'a> LinearGradient<'a> {
    pub fn stops<'b>(
        &'b self,
        palette: u16,
        coords: &'b [NormalizedCoordinate],
        variation_data: VariationData<'a>,
    ) -> GradientStopsIter<'a, 'b> {
        GradientStopsIter {
            color_line: &self.color_line,
            palette,
            index: 0,
            variation_data,
            coords,
        }
    }
}

#[derive(Clone)]
pub struct RadialGradient<'a> {
    pub x0: f32,
    pub y0: f32,
    pub r0: f32,
    pub r1: f32,
    pub x1: f32,
    pub y1: f32,
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
            // .field("stops", &self.stops(0))
            .finish()
    }
}

impl<'a> RadialGradient<'a> {
    pub fn stops<'b>(
        &'b self,
        palette: u16,
        variation_data: VariationData<'a>,
        coords: &'a [NormalizedCoordinate],
    ) -> GradientStopsIter<'a, 'b> {
        GradientStopsIter {
            color_line: &self.color_line,
            palette,
            index: 0,
            variation_data,
            coords,
        }
    }
}

#[derive(Clone)]
pub struct SweepGradient<'a> {
    pub center_x: f32,
    pub center_y: f32,
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
            // .field("stops", &self.stops(0))
            .finish()
    }
}

impl<'a> SweepGradient<'a> {
    pub fn stops<'b>(
        &'b self,
        palette: u16,
        variation_data: VariationData<'a>,
        coords: &'a [NormalizedCoordinate],
    ) -> GradientStopsIter<'a, 'b> {
        GradientStopsIter {
            color_line: &self.color_line,
            palette,
            index: 0,
            variation_data,
            coords,
        }
    }
}

#[derive(Clone, Copy)]
pub struct GradientStopsIter<'a, 'b> {
    color_line: &'b ColorLine<'a>,
    palette: u16,
    index: u16,
    variation_data: VariationData<'a>,
    coords: &'b [NormalizedCoordinate],
}

impl Iterator for GradientStopsIter<'_, '_> {
    type Item = ColorStop;

    fn next(&mut self) -> Option<Self::Item> {
        let len = match self.color_line {
            ColorLine::VarColorLine(vcl) => vcl.colors.len(),
            ColorLine::NonVarColorLine(nvcl) => nvcl.colors.len(),
        };

        if self.index == len {
            return None;
        }

        let index = self.index;
        self.index = self.index.checked_add(1)?;

        match self.color_line {
            ColorLine::VarColorLine(vcl) => {
                vcl.get(self.palette, index, self.variation_data, self.coords)
            }
            ColorLine::NonVarColorLine(nvcl) => nvcl.get(self.palette, index),
        }
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
    fn outline_glyph(&mut self, glyph_id: GlyphId);
    /// Paints the current glyph outline using the provided color.
    fn paint(&mut self, paint: Paint<'a>);

    fn push_clip(&mut self);

    fn push_clip_box(&mut self, clipbox: ClipBox);
    fn pop_clip(&mut self);

    fn push_layer(&mut self, mode: CompositeMode);
    fn pop_layer(&mut self);

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
    #[cfg(feature = "variable-fonts")]
    var_index_map: Option<DeltaSetIndexMap<'a>>,
    #[cfg(feature = "variable-fonts")]
    item_variation_store: Option<ItemVariationStore<'a>>,
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
            item_variation_store: None,
            var_index_map: None,
        };

        if version == 0 {
            return Some(table);
        }

        table.base_glyph_paints_offset = s.read::<Offset32>()?;
        let layer_list_offset = s.read::<Option<Offset32>>()?;
        let clip_list_offset = s.read::<Option<Offset32>>()?;
        let var_index_map_offset = s.read::<Option<Offset32>>()?;
        let item_variation_offset = s.read::<Option<Offset32>>()?;

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

        if let Some(offset) = item_variation_offset {
            let item_var_data = data.get(offset.to_usize()..)?;
            let s = Stream::new(item_var_data);
            let var_store = ItemVariationStore::parse(s)?;
            table.item_variation_store = Some(var_store);
        }

        if let Some(offset) = var_index_map_offset {
            let var_index_map_data = data.get(offset.to_usize()..)?;
            let var_index_map = DeltaSetIndexMap::new(var_index_map_data);
            table.var_index_map = Some(var_index_map);
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

    pub fn variation_data(&self) -> VariationData {
        VariationData {
            variation_store: self.item_variation_store,
            delta_map: self.var_index_map,
        }
    }

    /// Whether the table contains a definition for the given glyph.
    pub fn contains(&self, glyph_id: GlyphId) -> bool {
        self.get_v1(glyph_id).is_some() || self.get_v0(glyph_id).is_some()
    }

    // This method should only be called from outside, not from within `colr.rs`.
    // From inside, you always should call paint_impl, so that the recursion stack can
    // be passed on and any kind of recursion can be prevented.
    /// Paints the color glyph.
    pub fn paint(
        &self,
        glyph_id: GlyphId,
        palette: u16,
        painter: &mut dyn Painter<'a>,
        coords: &[NormalizedCoordinate],
        foreground_color: RgbaColor
    ) -> Option<()> {
        let mut recursion_stack = RecursionStack {
            stack: [0; 64],
            len: 0,
        };

        self.paint_impl(glyph_id, palette, painter, &mut recursion_stack, coords, foreground_color)
    }

    fn paint_impl(
        &self,
        glyph_id: GlyphId,
        palette: u16,
        painter: &mut dyn Painter<'a>,
        recusion_stack: &mut RecursionStack,
        coords: &[NormalizedCoordinate],
        foreground_color: RgbaColor
    ) -> Option<()> {
        if let Some(base) = self.get_v1(glyph_id) {
            self.paint_v1(base, palette, painter, recusion_stack, coords, foreground_color)
        } else if let Some(base) = self.get_v0(glyph_id) {
            self.paint_v0(base, palette, painter, foreground_color)
        } else {
            None
        }
    }

    fn paint_v0(
        &self,
        base: BaseGlyphRecord,
        palette: u16,
        painter: &mut dyn Painter,
        foreground_color: RgbaColor
    ) -> Option<()> {
        let start = base.first_layer_index;
        let end = start.checked_add(base.num_layers)?;
        let layers = self.layers.slice(start..end)?;

        for layer in layers {
            if layer.palette_index == 0xFFFF {
                // A special case.
                painter.outline_glyph(layer.glyph_id);
                painter.paint(Paint::Solid(foreground_color));
            } else {
                let color = self.palettes.get(palette, layer.palette_index)?;
                painter.outline_glyph(layer.glyph_id);
                painter.paint(Paint::Solid(color));
            }
        }

        Some(())
    }

    fn paint_v1(
        &self,
        base: BaseGlyphPaintRecord,
        palette: u16,
        painter: &mut dyn Painter<'a>,
        recursion_stack: &mut RecursionStack,
        coords: &[NormalizedCoordinate],
        foreground_color: RgbaColor
    ) -> Option<()> {
        let clip_box = self.clip_list.find(base.glyph_id, &self.variation_data(), coords);
        if let Some(clip_box) = clip_box {
            painter.push_clip_box(clip_box);
        }

        self.parse_paint(
            self.base_glyph_paints_offset.to_usize() + base.paint_table_offset.to_usize(),
            palette,
            painter,
            recursion_stack,
            coords,
            foreground_color
        );

        if clip_box.is_some() {
            painter.pop_clip();
        }

        Some(())
    }

    fn parse_paint(
        &self,
        offset: usize,
        palette: u16,
        painter: &mut dyn Painter<'a>,
        recursion_stack: &mut RecursionStack,
        coords: &[NormalizedCoordinate],
        foreground_color: RgbaColor
    ) -> Option<()> {
        let mut s = Stream::new_at(self.data, offset)?;
        let format = s.read::<u8>()?;

        // Cycle detected
        if recursion_stack.contains(offset) {
            return None;
        }

        recursion_stack.push(offset).ok()?;
        let result = self.parse_paint_impl(
            offset,
            palette,
            painter,
            recursion_stack,
            &mut s,
            format,
            coords,
            foreground_color
        );
        recursion_stack.pop();

        result
    }

    fn parse_paint_impl(
        &self,
        offset: usize,
        palette: u16,
        painter: &mut dyn Painter<'a>,
        recursion_stack: &mut RecursionStack,
        s: &mut Stream,
        format: u8,
        coords: &[NormalizedCoordinate],
        foreground_color: RgbaColor
    ) -> Option<()> {
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
                        recursion_stack,
                        coords,
                        foreground_color
                    );
                }
            }
            2 => {
                // PaintSolid
                let palette_index = s.read::<u16>()?;
                let alpha = s.read::<F2DOT14>()?;

                let mut color = if palette_index == u16::MAX {
                    foreground_color
                } else {
                    self.palettes.get(palette, palette_index)?
                };

                color.apply_alpha(alpha.to_f32());
                painter.paint(Paint::Solid(color));
            }
            3 => {
                // PaintVarSolid
                let palette_index = s.read::<u16>()?;
                let alpha = s.read::<F2DOT14>()?;
                let var_index_base = s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<1>(var_index_base, coords);

                let mut color = if palette_index == u16::MAX {
                    foreground_color
                } else {
                    self.palettes.get(palette, palette_index)?
                };

                color.apply_alpha(alpha.apply_float_delta(deltas[0]));
                painter.paint(Paint::Solid(color));
            }
            4 => {
                // PaintLinearGradient
                let color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_color_line(
                    offset + color_line_offset.to_usize(),
                    foreground_color
                )?;

                painter.paint(Paint::LinearGradient(LinearGradient {
                    x0: s.read::<i16>()? as f32,
                    y0: s.read::<i16>()? as f32,
                    x1: s.read::<i16>()? as f32,
                    y1: s.read::<i16>()? as f32,
                    x2: s.read::<i16>()? as f32,
                    y2: s.read::<i16>()? as f32,
                    extend: color_line.extend,
                    color_line: ColorLine::NonVarColorLine(color_line),
                }))
            }
            5 => {
                // PaintVarLinearGradient
                let var_color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_var_color_line(
                    offset + var_color_line_offset.to_usize(),
                    foreground_color
                )?;
                let mut var_s = s.clone();
                var_s.advance(12);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<6>(var_index_base, coords);

                painter.paint(Paint::LinearGradient(LinearGradient {
                    x0: s.read::<i16>()? as f32 + deltas[0],
                    y0: s.read::<i16>()? as f32 + deltas[1],
                    x1: s.read::<i16>()? as f32 + deltas[2],
                    y1: s.read::<i16>()? as f32 + deltas[3],
                    x2: s.read::<i16>()? as f32 + deltas[4],
                    y2: s.read::<i16>()? as f32 + deltas[5],
                    extend: color_line.extend,
                    color_line: ColorLine::VarColorLine(color_line),
                }))
            }
            6 => {
                // PaintRadialGradient
                let color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_color_line(
                    offset + color_line_offset.to_usize(),
                    foreground_color
                )?;
                painter.paint(Paint::RadialGradient(RadialGradient {
                    x0: s.read::<i16>()? as f32,
                    y0: s.read::<i16>()? as f32,
                    r0: s.read::<u16>()? as f32,
                    x1: s.read::<i16>()? as f32,
                    y1: s.read::<i16>()? as f32,
                    r1: s.read::<u16>()? as f32,
                    extend: color_line.extend,
                    color_line: ColorLine::NonVarColorLine(color_line),
                }))
            }
            7 => {
                // PaintVarRadialGradient
                let color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_var_color_line(
                    offset + color_line_offset.to_usize(),
                    foreground_color
                )?;

                let mut var_s = s.clone();
                var_s.advance(12);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<6>(var_index_base, coords);

                painter.paint(Paint::RadialGradient(RadialGradient {
                    x0: s.read::<i16>()? as f32 + deltas[0],
                    y0: s.read::<i16>()? as f32 + deltas[1],
                    r0: s.read::<u16>()? as f32 + deltas[2],
                    x1: s.read::<i16>()? as f32 + deltas[3],
                    y1: s.read::<i16>()? as f32 + deltas[4],
                    r1: s.read::<u16>()? as f32 + deltas[5],
                    extend: color_line.extend,
                    color_line: ColorLine::VarColorLine(color_line),
                }))
            }
            8 => {
                // PaintSweepGradient
                let color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_color_line(
                    offset + color_line_offset.to_usize(),
                    foreground_color
                )?;
                painter.paint(Paint::SweepGradient(SweepGradient {
                    center_x: s.read::<i16>()? as f32,
                    center_y: s.read::<i16>()? as f32,
                    start_angle: s.read::<F2DOT14>()?.to_f32(),
                    end_angle: s.read::<F2DOT14>()?.to_f32(),
                    extend: color_line.extend,
                    color_line: ColorLine::NonVarColorLine(color_line),
                }))
            }
            9 => {
                // PaintVarSweepGradient
                let color_line_offset = s.read::<Offset24>()?;
                let color_line = self.parse_var_color_line(
                    offset + color_line_offset.to_usize(),
                    foreground_color
                )?;

                let mut var_s = s.clone();
                var_s.advance(8);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<4>(var_index_base, coords);

                painter.paint(Paint::SweepGradient(SweepGradient {
                    center_x: s.read::<i16>()? as f32 + deltas[0],
                    center_y: s.read::<i16>()? as f32 + deltas[1],
                    start_angle: s.read::<F2DOT14>()?.apply_float_delta(deltas[2]),
                    end_angle: s.read::<F2DOT14>()?.apply_float_delta(deltas[3]),
                    extend: color_line.extend,
                    color_line: ColorLine::VarColorLine(color_line),
                }))
            }
            10 => {
                // PaintGlyph
                let paint_offset = s.read::<Offset24>()?;
                let glyph_id = s.read::<GlyphId>()?;
                painter.outline_glyph(glyph_id);
                painter.push_clip();

                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );

                painter.pop_clip();
            }
            11 => {
                // PaintColrGlyph
                let glyph_id = s.read::<GlyphId>()?;
                self.paint_impl(glyph_id, palette, painter, recursion_stack, coords, foreground_color);
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
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            13 => {
                // PaintVarTransform
                let paint_offset = s.read::<Offset24>()?;
                let ts_offset = s.read::<Offset24>()?;
                let mut s = Stream::new_at(self.data, offset + ts_offset.to_usize())?;

                let mut var_s = s.clone();
                var_s.advance(24);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<6>(var_index_base, coords);

                let ts = Transform {
                    a: s.read::<Fixed>()?.apply_float_delta(deltas[0]),
                    b: s.read::<Fixed>()?.apply_float_delta(deltas[1]),
                    c: s.read::<Fixed>()?.apply_float_delta(deltas[2]),
                    d: s.read::<Fixed>()?.apply_float_delta(deltas[3]),
                    e: s.read::<Fixed>()?.apply_float_delta(deltas[4]),
                    f: s.read::<Fixed>()?.apply_float_delta(deltas[5]),
                };

                println!("VarTransform: {:?}, {:?}", ts.a, ts.b);

                painter.transform(ts);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            14 => {
                // PaintTranslate
                let paint_offset = s.read::<Offset24>()?;
                let tx = f32::from(s.read::<i16>()?);
                let ty = f32::from(s.read::<i16>()?);

                painter.translate(tx, ty);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            15 => {
                // PaintVarTranslate
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(4);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<2>(var_index_base, coords);

                let tx = f32::from(s.read::<i16>()?) + deltas[0];
                let ty = f32::from(s.read::<i16>()?) + deltas[1];

                painter.translate(tx, ty);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            16 => {
                // PaintScale
                let paint_offset = s.read::<Offset24>()?;
                let sx = s.read::<F2DOT14>()?.to_f32();
                let sy = s.read::<F2DOT14>()?.to_f32();

                painter.scale(sx, sy);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            17 => {
                // PaintVarScale
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(4);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<2>(var_index_base, coords);

                let sx = s.read::<F2DOT14>()?.apply_float_delta(deltas[0]);
                let sy = s.read::<F2DOT14>()?.apply_float_delta(deltas[1]);

                painter.scale(sx, sy);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
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
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            19 => {
                // PaintVarScaleAroundCenter
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(8);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<4>(var_index_base, coords);

                let sx = s.read::<F2DOT14>()?.apply_float_delta(deltas[0]);
                let sy = s.read::<F2DOT14>()?.apply_float_delta(deltas[1]);
                let center_x = f32::from(s.read::<i16>()?) + deltas[2];
                let center_y = f32::from(s.read::<i16>()?) + deltas[3];

                painter.translate(center_x, center_y);
                painter.scale(sx, sy);
                painter.translate(-center_x, -center_y);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            20 => {
                // PaintScaleUniform
                let paint_offset = s.read::<Offset24>()?;
                let scale = s.read::<F2DOT14>()?.to_f32();

                painter.scale(scale, scale);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            21 => {
                // PaintVarScaleUniform
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(2);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<1>(var_index_base, coords);

                let scale = s.read::<F2DOT14>()?.apply_float_delta(deltas[0]);

                painter.scale(scale, scale);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
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
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            23 => {
                // PaintVarScaleUniformAroundCenter
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(6);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<3>(var_index_base, coords);

                let scale = s.read::<F2DOT14>()?.apply_float_delta(deltas[0]);
                let center_x = f32::from(s.read::<i16>()?) + deltas[1];
                let center_y = f32::from(s.read::<i16>()?) + deltas[2];

                painter.translate(center_x, center_y);
                painter.scale(scale, scale);
                painter.translate(-center_x, -center_y);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            24 => {
                // PaintRotate
                let paint_offset = s.read::<Offset24>()?;
                let angle = s.read::<F2DOT14>()?.to_f32();

                painter.rotate(angle);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            25 => {
                // PaintVarRotate
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(2);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<1>(var_index_base, coords);

                let angle = s.read::<F2DOT14>()?.apply_float_delta(deltas[0]);

                painter.rotate(angle);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            26 => {
                // PaintRotateAroundCenter
                let paint_offset = s.read::<Offset24>()?;
                let angle = s.read::<F2DOT14>()?.to_f32();
                let center_x = f32::from(s.read::<i16>()?);
                let center_y = f32::from(s.read::<i16>()?);

                painter.translate(center_x, center_y);
                painter.rotate(angle);
                painter.translate(-center_x, -center_y);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            27 => {
                // PaintVarRotateAroundCenter
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(6);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<3>(var_index_base, coords);

                let angle = s.read::<F2DOT14>()?.apply_float_delta(deltas[0]);
                let center_x = f32::from(s.read::<i16>()?) + deltas[1];
                let center_y = f32::from(s.read::<i16>()?) + deltas[2];

                painter.translate(center_x, center_y);
                painter.rotate(angle);
                painter.translate(-center_x, -center_y);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
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
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
            }
            29 => {
                // PaintVarSkew
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(4);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<2>(var_index_base, coords);

                let skew_x = s.read::<F2DOT14>()?.apply_float_delta(deltas[0]);
                let skew_y = s.read::<F2DOT14>()?.apply_float_delta(deltas[1]);

                painter.skew(skew_x, skew_y);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
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
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            31 => {
                // PaintVarSkewAroundCenter
                let paint_offset = s.read::<Offset24>()?;

                let mut var_s = s.clone();
                var_s.advance(8);
                let var_index_base = var_s.read::<u32>()?;

                let deltas = self
                    .variation_data()
                    .read_deltas::<4>(var_index_base, coords);

                let skew_x = s.read::<F2DOT14>()?.apply_float_delta(deltas[0]);
                let skew_y = s.read::<F2DOT14>()?.apply_float_delta(deltas[1]);
                let center_x = f32::from(s.read::<i16>()?) + deltas[2];
                let center_y = f32::from(s.read::<i16>()?) + deltas[3];

                painter.translate(center_x, center_y);
                painter.skew(skew_x, skew_y);
                painter.translate(-center_x, -center_y);
                self.parse_paint(
                    offset + paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_transform();
                painter.pop_transform();
                painter.pop_transform();
            }
            32 => {
                // PaintComposite
                let source_paint_offset = s.read::<Offset24>()?;
                let composite_mode = s.read::<CompositeMode>()?;
                let backdrop_paint_offset = s.read::<Offset24>()?;

                painter.push_layer(CompositeMode::SourceOver);
                self.parse_paint(
                    offset + backdrop_paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.push_layer(composite_mode);
                self.parse_paint(
                    offset + source_paint_offset.to_usize(),
                    palette,
                    painter,
                    recursion_stack,
                    coords,
                    foreground_color
                );
                painter.pop_layer();
                painter.pop_layer();
            }
            _ => {}
        }

        Some(())
    }

    fn parse_color_line(
        &self,
        offset: usize,
        foreground_color: RgbaColor,
    ) -> Option<NonVarColorLine<'a>> {
        let mut s = Stream::new_at(self.data, offset)?;
        let extend = s.read::<GradientExtend>()?;
        let count = s.read::<u16>()?;
        let colors = s.read_array16::<ColorStopRaw>(count)?;
        Some(NonVarColorLine {
            extend,
            colors,
            foreground_color,
            palettes: self.palettes,
        })
    }

    fn parse_var_color_line(
        &self,
        offset: usize,
        foreground_color: RgbaColor,
    ) -> Option<VarColorLine<'a>> {
        let mut s = Stream::new_at(self.data, offset)?;
        let extend = s.read::<GradientExtend>()?;
        let count = s.read::<u16>()?;
        let colors = s.read_array16::<VarColorStopRaw>(count)?;
        Some(VarColorLine {
            extend,
            colors,
            foreground_color,
            palettes: self.palettes,
        })
    }
}

struct RecursionStack {
    // The limit of 64 is chosen arbitrarily and not from the spec. But we have to stop somewhere...
    stack: [usize; 64],
    len: usize,
}

impl RecursionStack {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn push(&mut self, offset: usize) -> Result<(), ()> {
        if self.len == self.stack.len() {
            Err(())
        } else {
            self.stack[self.len] = offset;
            self.len += 1;
            Ok(())
        }
    }

    #[inline]
    pub fn contains(&self, offset: usize) -> bool {
        if let Some(offsets) = self.stack.get(..self.len) {
            return offsets.contains(&offset);
        }

        false
    }

    #[inline]
    pub fn pop(&mut self) {
        debug_assert!(!self.is_empty());
        self.len -= 1;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VariationData<'a> {
    variation_store: Option<ItemVariationStore<'a>>,
    delta_map: Option<DeltaSetIndexMap<'a>>,
}

impl VariationData<'_> {
    // Adapted from fontations
    fn read_deltas<const N: usize>(
        &self,
        var_index_base: u32,
        coordinates: &[NormalizedCoordinate],
    ) -> [f32; N] {
        const NO_VARIATION_DELTAS: u32 = 0xFFFFFFFF;
        let mut deltas = [0.0; N];

        if coordinates.is_empty()
            || self.variation_store.is_none()
            || var_index_base == NO_VARIATION_DELTAS
        {
            return deltas;
        }

        let variation_store = self.variation_store.as_ref().unwrap();

        for i in 0..N {
            deltas[i] = self
                .delta_map
                .and_then(|d| d.map(var_index_base + i as u32))
                .and_then(|d| variation_store.parse_delta(d.0, d.1, coordinates))
                .unwrap_or(0.0);
        }

        deltas
    }
}
