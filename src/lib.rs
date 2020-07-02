/*!
A high-level, safe, zero-allocation TrueType font parser.

## Features

- A high-level API, for people who doesn't know how TrueType works internally.
  Basically, no direct access to font tables.
- Zero heap allocations.
- Zero unsafe.
- Zero dependencies.
- `no_std`/WASM compatible.
- Fast.
- Stateless. All parsing methods are immutable methods.
- Simple and maintainable code (no magic numbers).

## Safety

- The library must not panic. Any panic considered as a critical bug and should be reported.
- The library forbids the unsafe code.
- No heap allocations, so crash due to OOM is not possible.
- All recursive methods have a depth limit.
- Technically, should use less than 64KiB of stack in worst case scenario.
- Most of arithmetic operations are checked.
- Most of numeric casts are checked.

## Error handling

`ttf-parser` is designed to parse well-formed fonts, so it does not have an `Error` enum.
It doesn't mean that it will crash or panic on malformed fonts, only that the
error handling will boil down to `Option::None`. So you will not get a detailed cause of an error.
By doing so we can simplify an API quite a lot since otherwise, we will have to use
`Result<Option<T>, Error>`.
*/

#![doc(html_root_url = "https://docs.rs/ttf-parser/0.6.2")]

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

#[cfg(feature = "std")]
use std::string::String;

use core::fmt;
use core::num::NonZeroU16;

macro_rules! try_opt_or {
    ($value:expr, $ret:expr) => {
        match $value {
            Some(v) => v,
            None => return $ret,
        }
    };
}

mod ggg;
mod parser;
mod tables;
mod var_store;

#[cfg(feature = "std")]
mod writer;

use tables::*;
use parser::{Stream, FromData, NumFrom, TryNumFrom, i16_bound, f32_bound};
use head::IndexToLocationFormat;
pub use fvar::{VariationAxes, VariationAxis};
pub use gdef::GlyphClass;
pub use ggg::*;
pub use name::*;
pub use os2::*;
pub use tables::kern;


/// A type-safe wrapper for glyph ID.
#[repr(transparent)]
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
pub struct GlyphId(pub u16);

impl FromData for GlyphId {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u16::parse(data).map(GlyphId)
    }
}


/// A variation coordinate in a normalized coordinate system.
///
/// Basically any number in a -1.0..1.0 range.
/// Where 0 is a default value.
///
/// The number is stored as f2.16
#[derive(Clone, Copy, PartialEq, Default, Debug)]
struct NormalizedCoord(i16);

impl From<i16> for NormalizedCoord {
    /// Creates a new coordinate.
    ///
    /// The provided number will be clamped to the -16384..16384 range.
    #[inline]
    fn from(n: i16) -> Self {
        NormalizedCoord(i16_bound(-16384, n, 16384))
    }
}

impl From<f32> for NormalizedCoord {
    /// Creates a new coordinate.
    ///
    /// The provided number will be clamped to the -1.0..1.0 range.
    #[inline]
    fn from(n: f32) -> Self {
        NormalizedCoord((f32_bound(-1.0, n, 1.0) * 16384.0) as i16)
    }
}

impl NormalizedCoord {
    /// Returns the coordinate value as f2.14.
    #[inline]
    pub fn get(self) -> i16 {
        self.0
    }
}


/// A font variation value.
///
/// # Example
///
/// ```
/// use ttf_parser::{Variation, Tag};
///
/// Variation { axis: Tag::from_bytes(b"wght"), value: 500.0 };
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Variation {
    /// An axis tag name.
    pub axis: Tag,
    /// An axis value.
    pub value: f32,
}


/// A 4-byte tag.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(pub u32);

impl Tag {
    /// Creates a `Tag` from bytes.
    #[inline]
    pub const fn from_bytes(bytes: &[u8; 4]) -> Self {
        Tag(((bytes[0] as u32) << 24) | ((bytes[1] as u32) << 16) |
            ((bytes[2] as u32) << 8) | (bytes[3] as u32))
    }

    /// Creates a `Tag` from bytes.
    ///
    /// In case of empty data will return `Tag` set to 0.
    ///
    /// When `bytes` are shorter than 4, will set missing bytes to ` `.
    ///
    /// Data after first 4 bytes is ignored.
    #[inline]
    pub fn from_bytes_lossy(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            return Tag::from_bytes(&[0, 0, 0, 0]);
        }

        let mut iter = bytes.iter().cloned().chain(core::iter::repeat(b' '));
        Tag::from_bytes(&[
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ])
    }

    /// Returns tag as 4-element byte array.
    #[inline]
    pub const fn to_bytes(self) -> [u8; 4] {
        [
            (self.0 >> 24 & 0xff) as u8,
            (self.0 >> 16 & 0xff) as u8,
            (self.0 >> 8 & 0xff) as u8,
            (self.0 >> 0 & 0xff) as u8,
        ]
    }

    /// Returns tag as 4-element byte array.
    #[inline]
    pub const fn to_chars(self) -> [char; 4] {
        [
            (self.0 >> 24 & 0xff) as u8 as char,
            (self.0 >> 16 & 0xff) as u8 as char,
            (self.0 >> 8 & 0xff) as u8 as char,
            (self.0 >> 0 & 0xff) as u8 as char,
        ]
    }

    /// Checks if tag is null / `[0, 0, 0, 0]`.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Returns tag value as `u32` number.
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    /// Converts tag to lowercase.
    #[inline]
    pub fn to_lowercase(&self) -> Self {
        let b = self.to_bytes();
        Tag::from_bytes(&[
            b[0].to_ascii_lowercase(),
            b[1].to_ascii_lowercase(),
            b[2].to_ascii_lowercase(),
            b[3].to_ascii_lowercase(),
        ])
    }

    /// Converts tag to uppercase.
    #[inline]
    pub fn to_uppercase(&self) -> Self {
        let b = self.to_bytes();
        Tag::from_bytes(&[
            b[0].to_ascii_uppercase(),
            b[1].to_ascii_uppercase(),
            b[2].to_ascii_uppercase(),
            b[3].to_ascii_uppercase(),
        ])
    }
}

impl core::fmt::Debug for Tag {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Tag({})", self)
    }
}

impl core::fmt::Display for Tag {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let b = self.to_chars();
        write!(
            f,
            "{}{}{}{}",
            b.get(0).unwrap_or(&' '),
            b.get(1).unwrap_or(&' '),
            b.get(2).unwrap_or(&' '),
            b.get(3).unwrap_or(&' ')
        )
    }
}

impl FromData for Tag {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        u32::parse(data).map(Tag)
    }
}



/// A line metrics.
///
/// Used for underline and strikeout.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LineMetrics {
    /// Line position.
    pub position: i16,

    /// Line thickness.
    pub thickness: i16,
}


/// A rectangle.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct Rect {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}

impl Rect {
    /// Returns rect's width.
    #[inline]
    pub fn width(&self) -> i16 {
        self.x_max - self.x_min
    }

    /// Returns rect's height.
    #[inline]
    pub fn height(&self) -> i16 {
        self.y_max - self.y_min
    }
}


#[derive(Clone, Copy, Debug)]
pub(crate) struct BBox {
    x_min: f32,
    y_min: f32,
    x_max: f32,
    y_max: f32,
}

impl BBox {
    #[inline]
    fn new() -> Self {
        BBox {
            x_min: core::f32::MAX,
            y_min: core::f32::MAX,
            x_max: core::f32::MIN,
            y_max: core::f32::MIN,
        }
    }

    #[inline]
    fn is_default(&self) -> bool {
        self.x_min == core::f32::MAX &&
        self.y_min == core::f32::MAX &&
        self.x_max == core::f32::MIN &&
        self.y_max == core::f32::MIN
    }

    #[inline]
    fn extend_by(&mut self, x: f32, y: f32) {
        self.x_min = self.x_min.min(x);
        self.y_min = self.y_min.min(y);
        self.x_max = self.x_max.max(x);
        self.y_max = self.y_max.max(y);
    }

    #[inline]
    fn to_rect(&self) -> Option<Rect> {
        Some(Rect {
            x_min: i16::try_num_from(self.x_min)?,
            y_min: i16::try_num_from(self.y_min)?,
            x_max: i16::try_num_from(self.x_max)?,
            y_max: i16::try_num_from(self.y_max)?,
        })
    }
}


/// A trait for glyph outline construction.
pub trait OutlineBuilder {
    /// Appends a MoveTo segment.
    ///
    /// Start of a contour.
    fn move_to(&mut self, x: f32, y: f32);

    /// Appends a LineTo segment.
    fn line_to(&mut self, x: f32, y: f32);

    /// Appends a QuadTo segment.
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32);

    /// Appends a CurveTo segment.
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32);

    /// Appends a ClosePath segment.
    ///
    /// End of a contour.
    fn close(&mut self);
}


struct DummyOutline;
impl OutlineBuilder for DummyOutline {
    fn move_to(&mut self, _: f32, _: f32) {}
    fn line_to(&mut self, _: f32, _: f32) {}
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
    fn close(&mut self) {}
}


/// A glyph raster image format.
#[allow(missing_docs)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RasterImageFormat {
    PNG,
}


/// A glyph's raster image.
///
/// Note, that glyph metrics are in pixels and not in font units.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RasterGlyphImage<'a> {
    /// Horizontal offset.
    pub x: i16,

    /// Vertical offset.
    pub y: i16,

    /// Image width.
    ///
    /// It doesn't guarantee that this value is the same as set in the `data`.
    pub width: u16,

    /// Image height.
    ///
    /// It doesn't guarantee that this value is the same as set in the `data`.
    pub height: u16,

    /// A pixels per em of the selected strike.
    pub pixels_per_em: u16,

    /// An image format.
    pub format: RasterImageFormat,

    /// A raw image data. It's up to the caller to decode it.
    pub data: &'a [u8],
}


/// A table name.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum TableName {
    AxisVariations = 0,
    CharacterToGlyphIndexMapping,
    ColorBitmapData,
    ColorBitmapLocation,
    CompactFontFormat,
    CompactFontFormat2,
    FontVariations,
    GlyphData,
    GlyphDefinition,
    GlyphVariations,
    Header,
    HorizontalHeader,
    HorizontalMetrics,
    HorizontalMetricsVariations,
    IndexToLocation,
    Kerning,
    MaximumProfile,
    MetricsVariations,
    Naming,
    PostScript,
    ScalableVectorGraphics,
    StandardBitmapGraphics,
    VerticalHeader,
    VerticalMetrics,
    VerticalMetricsVariations,
    VerticalOrigin,
    WindowsMetrics,
}


#[derive(Clone, Copy)]
struct TableRecord {
    table_tag: Tag,
    #[allow(dead_code)]
    check_sum: u32,
    offset: u32,
    length: u32,
}

impl FromData for TableRecord {
    const SIZE: usize = 16;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(TableRecord {
            table_tag: s.read()?,
            check_sum: s.read()?,
            offset: s.read()?,
            length: s.read()?,
        })
    }
}


const MAX_VAR_COORDS: u8 = 32;

#[derive(Clone, Default)]
struct VarCoords {
    data: [NormalizedCoord; MAX_VAR_COORDS as usize],
    len: u8,
}

impl VarCoords {
    #[inline]
    fn as_slice(&self) -> &[NormalizedCoord] {
        &self.data[0..usize::from(self.len)]
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [NormalizedCoord] {
        let end = usize::from(self.len);
        &mut self.data[0..end]
    }
}


/// A font data handle.
#[derive(Clone)]
pub struct Font<'a> {
    avar: Option<avar::Table<'a>>,
    cbdt: Option<&'a [u8]>,
    cblc: Option<&'a [u8]>,
    cff_: Option<cff::Metadata<'a>>,
    cff2: Option<cff2::Metadata<'a>>,
    cmap: Option<cmap::Table<'a>>,
    fvar: Option<fvar::Table<'a>>,
    gdef: Option<gdef::Table<'a>>,
    glyf: Option<&'a [u8]>,
    gvar: Option<gvar::Table<'a>>,
    head: &'a [u8],
    hhea: &'a [u8],
    hmtx: Option<hmtx::Table<'a>>,
    hvar: Option<hvar::Table<'a>>,
    kern: Option<kern::Subtables<'a>>,
    loca: Option<loca::Table<'a>>,
    mvar: Option<mvar::Table<'a>>,
    name: Option<name::Names<'a>>,
    os_2: Option<os2::Table<'a>>,
    post: Option<post::Table<'a>>,
    vhea: Option<&'a [u8]>,
    vmtx: Option<hmtx::Table<'a>>,
    sbix: Option<&'a [u8]>,
    svg_: Option<&'a [u8]>,
    vorg: Option<vorg::Table<'a>>,
    vvar: Option<hvar::Table<'a>>,
    number_of_glyphs: NonZeroU16,
    coordinates: VarCoords,
}

impl<'a> Font<'a> {
    /// Creates a `Font` object from a raw data.
    ///
    /// You can set `index` for font collections.
    /// For simple `ttf` fonts set `index` to 0.
    ///
    /// This method will do some parsing and sanitization, so it's a bit expensive.
    ///
    /// Required tables: `head`, `hhea` and `maxp`.
    ///
    /// If an optional table has an invalid data it will be skipped.
    pub fn from_data(data: &'a [u8], index: u32) -> Option<Self> {
        const OFFSET_TABLE_SIZE: usize = 12;

        let table_data = if let Some(n) = fonts_in_collection(data) {
            if index < n {
                // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
                const OFFSET_32_SIZE: usize = 4;
                let offset = OFFSET_TABLE_SIZE + OFFSET_32_SIZE * usize::num_from(index);
                let font_offset: u32 = Stream::read_at(data, offset)?;
                data.get(usize::num_from(font_offset) .. data.len())?
            } else {
                return None;
            }
        } else {
            data
        };

        // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
        if data.len() < OFFSET_TABLE_SIZE {
            return None;
        }

        // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
        const SFNT_VERSION_TRUE_TYPE: u32 = 0x00010000;
        const SFNT_VERSION_OPEN_TYPE: u32 = 0x4F54544F;

        let mut s = Stream::new(table_data);

        let sfnt_version: u32 = s.read()?;
        if sfnt_version != SFNT_VERSION_TRUE_TYPE && sfnt_version != SFNT_VERSION_OPEN_TYPE {
            return None;
        }

        let num_tables: u16 = s.read()?;
        s.advance(6); // searchRange (u16) + entrySelector (u16) + rangeShift (u16)
        let tables = s.read_array16::<TableRecord>(num_tables)?;

        let mut font = Font {
            avar: None,
            cbdt: None,
            cblc: None,
            cff_: None,
            cff2: None,
            cmap: None,
            fvar: None,
            gdef: None,
            glyf: None,
            gvar: None,
            head: &[],
            hhea: &[],
            hmtx: None,
            hvar: None,
            kern: None,
            loca: None,
            mvar: None,
            name: None,
            os_2: None,
            post: None,
            vhea: None,
            vmtx: None,
            sbix: None,
            svg_: None,
            vorg: None,
            vvar: None,
            number_of_glyphs: NonZeroU16::new(1).unwrap(), // dummy
            coordinates: VarCoords::default(),
        };

        let mut number_of_glyphs = None;
        let mut hmtx = None;
        let mut vmtx = None;
        let mut loca = None;

        for table in tables {
            let offset = usize::num_from(table.offset);
            let length = usize::num_from(table.length);
            let range = offset..(offset + length);

            match &table.table_tag.to_bytes() {
                b"CBDT" => font.cbdt = data.get(range),
                b"CBLC" => font.cblc = data.get(range),
                b"CFF " => font.cff_ = data.get(range).and_then(|data| cff::parse_metadata(data)),
                b"CFF2" => font.cff2 = data.get(range).and_then(|data| cff2::parse_metadata(data)),
                b"GDEF" => font.gdef = data.get(range).and_then(|data| gdef::Table::parse(data)),
                b"HVAR" => font.hvar = data.get(range).and_then(|data| hvar::Table::parse(data)),
                b"MVAR" => font.mvar = data.get(range).and_then(|data| mvar::Table::parse(data)),
                b"OS/2" => font.os_2 = data.get(range).and_then(|data| os2::Table::parse(data)),
                b"SVG " => font.svg_ = data.get(range),
                b"VORG" => font.vorg = data.get(range).and_then(|data| vorg::Table::parse(data)),
                b"VVAR" => font.vvar = data.get(range).and_then(|data| hvar::Table::parse(data)),
                b"avar" => font.avar = data.get(range).and_then(|data| avar::Table::parse(data)),
                b"cmap" => font.cmap = data.get(range).and_then(|data| cmap::Table::parse(data)),
                b"fvar" => font.fvar = data.get(range).and_then(|data| fvar::Table::parse(data)),
                b"glyf" => font.glyf = data.get(range),
                b"gvar" => font.gvar = data.get(range).and_then(|data| gvar::Table::parse(data)),
                b"head" => font.head = data.get(range).and_then(|data| head::parse(data))?,
                b"hhea" => font.hhea = data.get(range).and_then(|data| hhea::parse(data))?,
                b"hmtx" => hmtx = data.get(range),
                b"kern" => font.kern = data.get(range).and_then(|data| kern::parse(data)),
                b"loca" => loca = data.get(range),
                b"maxp" => number_of_glyphs = data.get(range).and_then(|data| maxp::parse(data)),
                b"name" => font.name = data.get(range).and_then(|data| name::parse(data)),
                b"post" => font.post = data.get(range).and_then(|data| post::Table::parse(data)),
                b"sbix" => font.sbix = data.get(range),
                b"vhea" => font.vhea = data.get(range).and_then(|data| vhea::parse(data)),
                b"vmtx" => vmtx = data.get(range),
                _ => {}
            }
        }

        if font.head.is_empty() || font.hhea.is_empty() || number_of_glyphs.is_none() {
            return None;
        }

        font.number_of_glyphs = number_of_glyphs?;

        if let Some(ref fvar) = font.fvar {
            font.coordinates.len = fvar.axes().count().min(MAX_VAR_COORDS as usize) as u8;
        }

        if let Some(data) = hmtx {
            if let Some(number_of_h_metrics) = hhea::number_of_h_metrics(font.hhea) {
                font.hmtx = hmtx::Table::parse(data, number_of_h_metrics, font.number_of_glyphs);
            }
        }

        if let (Some(vhea), Some(data)) = (font.vhea, vmtx) {
            if let Some(number_of_v_metrics) = vhea::num_of_long_ver_metrics(vhea) {
                font.vmtx = hmtx::Table::parse(data, number_of_v_metrics, font.number_of_glyphs);
            }
        }

        if let Some(data) = loca {
            if let Some(format) = head::index_to_loc_format(font.head) {
                font.loca = loca::Table::parse(data, font.number_of_glyphs, format);
            }
        }

        Some(font)
    }

    /// Checks that font has a specified table.
    ///
    /// Will return `true` only for tables that were successfully parsed.
    #[inline]
    pub fn has_table(&self, name: TableName) -> bool {
        match name {
            TableName::Header                       => true,
            TableName::HorizontalHeader             => true,
            TableName::MaximumProfile               => true,
            TableName::AxisVariations               => self.avar.is_some(),
            TableName::CharacterToGlyphIndexMapping => self.cmap.is_some(),
            TableName::ColorBitmapData              => self.cbdt.is_some(),
            TableName::ColorBitmapLocation          => self.cblc.is_some(),
            TableName::CompactFontFormat            => self.cff_.is_some(),
            TableName::CompactFontFormat2           => self.cff2.is_some(),
            TableName::FontVariations               => self.fvar.is_some(),
            TableName::GlyphData                    => self.glyf.is_some(),
            TableName::GlyphDefinition              => self.gdef.is_some(),
            TableName::GlyphVariations              => self.gvar.is_some(),
            TableName::HorizontalMetrics            => self.hmtx.is_some(),
            TableName::HorizontalMetricsVariations  => self.hvar.is_some(),
            TableName::IndexToLocation              => self.loca.is_some(),
            TableName::Kerning                      => self.kern.is_some(),
            TableName::MetricsVariations            => self.mvar.is_some(),
            TableName::Naming                       => self.name.is_some(),
            TableName::PostScript                   => self.post.is_some(),
            TableName::ScalableVectorGraphics       => self.svg_.is_some(),
            TableName::StandardBitmapGraphics       => self.sbix.is_some(),
            TableName::VerticalHeader               => self.vhea.is_some(),
            TableName::VerticalMetrics              => self.vmtx.is_some(),
            TableName::VerticalMetricsVariations    => self.vvar.is_some(),
            TableName::VerticalOrigin               => self.vorg.is_some(),
            TableName::WindowsMetrics               => self.os_2.is_some(),
        }
    }

    /// Returns an iterator over [Name Records].
    ///
    /// An iterator can be empty.
    ///
    /// [Name Records]: https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records
    #[inline]
    pub fn names(&self) -> Names {
        self.name.unwrap_or_default()
    }

    /// Returns font's family name.
    ///
    /// *Typographic Family* is preferred over *Family*.
    ///
    /// Note that font can have multiple names. You can use [`names()`] to list them all.
    ///
    /// [`names()`]: #method.names
    #[cfg(feature = "std")]
    #[inline]
    pub fn family_name(&self) -> Option<String> {
        let mut idx = None;
        let mut iter = self.names();
        for (i, name) in iter.enumerate() {
            if name.name_id() == name_id::TYPOGRAPHIC_FAMILY && name.is_unicode() {
                // Break the loop as soon as we reached 'Typographic Family'.
                idx = Some(i);
                break;
            } else if name.name_id() == name_id::FAMILY && name.is_unicode() {
                idx = Some(i);
                // Do not break the loop since 'Typographic Family' can be set later
                // and it has a higher priority.
            }
        }

        iter.nth(idx?).and_then(|name| name.name_from_utf16_be())
    }

    /// Returns font's PostScript name.
    ///
    /// Note that font can have multiple names. You can use [`names()`] to list them all.
    ///
    /// [`names()`]: #method.names
    #[cfg(feature = "std")]
    #[inline]
    pub fn post_script_name(&self) -> Option<String> {
        self.names()
            .find(|name| name.name_id() == name_id::POST_SCRIPT_NAME && name.is_unicode())
            .and_then(|name| name.name_from_utf16_be())
    }

    /// Checks that font is marked as *Regular*.
    ///
    /// Returns `false` when OS/2 table is not present.
    #[inline]
    pub fn is_regular(&self) -> bool {
        try_opt_or!(self.os_2, false).is_regular()
    }

    /// Checks that font is marked as *Italic*.
    ///
    /// Returns `false` when OS/2 table is not present.
    #[inline]
    pub fn is_italic(&self) -> bool {
        try_opt_or!(self.os_2, false).is_italic()
    }

    /// Checks that font is marked as *Bold*.
    ///
    /// Returns `false` when OS/2 table is not present.
    #[inline]
    pub fn is_bold(&self) -> bool {
        try_opt_or!(self.os_2, false).is_bold()
    }

    /// Checks that font is marked as *Oblique*.
    ///
    /// Returns `false` when OS/2 table is not present or when its version is < 4.
    #[inline]
    pub fn is_oblique(&self) -> bool {
        try_opt_or!(self.os_2, false).is_oblique()
    }

    /// Checks that font is variable.
    ///
    /// Simply checks the presence of a `fvar` table.
    #[inline]
    pub fn is_variable(&self) -> bool {
        // `fvar::Table::parse` already checked that `axisCount` is non-zero.
        self.fvar.is_some()
    }

    /// Returns font's weight.
    ///
    /// Returns `Weight::Normal` when OS/2 table is not present.
    #[inline]
    pub fn weight(&self) -> Weight {
        try_opt_or!(self.os_2, Weight::default()).weight()
    }

    /// Returns font's width.
    ///
    /// Returns `Width::Normal` when OS/2 table is not present or when value is invalid.
    #[inline]
    pub fn width(&self) -> Width {
        try_opt_or!(self.os_2, Width::default()).width()
    }

    #[inline]
    fn use_typo_metrics(&self) -> Option<os2::Table> {
        self.os_2.filter(|table| table.is_use_typo_metrics())
    }

    /// Returns a horizontal font ascender.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn ascender(&self) -> i16 {
        if let Some(os_2) = self.use_typo_metrics() {
            let v = os_2.typo_ascender();
            self.apply_metrics_variation(Tag::from_bytes(b"hasc"), v)
        } else {
            hhea::ascender(self.hhea)
        }
    }

    /// Returns a horizontal font descender.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn descender(&self) -> i16 {
        if let Some(os_2) = self.use_typo_metrics() {
            let v = os_2.typo_descender();
            self.apply_metrics_variation(Tag::from_bytes(b"hdsc"), v)
        } else {
            hhea::descender(self.hhea)
        }
    }

    /// Returns font's height.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn height(&self) -> i16 {
        self.ascender() - self.descender()
    }

    /// Returns a horizontal font line gap.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn line_gap(&self) -> i16 {
        if let Some(os_2) = self.use_typo_metrics() {
            let v = os_2.typo_line_gap();
            self.apply_metrics_variation(Tag::from_bytes(b"hlgp"), v)
        } else {
            hhea::line_gap(self.hhea)
        }
    }

    // TODO: does this affected by USE_TYPO_METRICS?

    /// Returns a vertical font ascender.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn vertical_ascender(&self) -> Option<i16> {
        self.vhea.map(vhea::ascender)
            .map(|v| self.apply_metrics_variation(Tag::from_bytes(b"vasc"), v))
    }

    /// Returns a vertical font descender.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn vertical_descender(&self) -> Option<i16> {
        self.vhea.map(vhea::descender)
            .map(|v| self.apply_metrics_variation(Tag::from_bytes(b"vdsc"), v))
    }

    /// Returns a vertical font height.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn vertical_height(&self) -> Option<i16> {
        Some(self.vertical_ascender()? - self.vertical_descender()?)
    }

    /// Returns a vertical font line gap.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn vertical_line_gap(&self) -> Option<i16> {
        self.vhea.map(vhea::line_gap)
            .map(|v| self.apply_metrics_variation(Tag::from_bytes(b"vlgp"), v))
    }

    /// Returns font's units per EM.
    ///
    /// Returns `None` when value is not in a 16..=16384 range.
    #[inline]
    pub fn units_per_em(&self) -> Option<u16> {
        head::units_per_em(self.head)
    }

    /// Returns font's x height.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns `None` when OS/2 table is not present or when its version is < 2.
    #[inline]
    pub fn x_height(&self) -> Option<i16> {
        self.os_2.and_then(|os_2| os_2.x_height())
            .map(|v| self.apply_metrics_variation(Tag::from_bytes(b"xhgt"), v))
    }

    /// Returns font's underline metrics.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns `None` when `post` table is not present.
    #[inline]
    pub fn underline_metrics(&self) -> Option<LineMetrics> {
        let mut metrics = self.post?.underline_metrics();
        if self.is_variable() {
            self.apply_metrics_variation_to(Tag::from_bytes(b"undo"), &mut metrics.position);
            self.apply_metrics_variation_to(Tag::from_bytes(b"unds"), &mut metrics.thickness);
        }
        Some(metrics)
    }

    /// Returns font's strikeout metrics.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns `None` when OS/2 table is not present.
    #[inline]
    pub fn strikeout_metrics(&self) -> Option<LineMetrics> {
        let mut metrics = self.os_2?.strikeout_metrics();
        if self.is_variable() {
            self.apply_metrics_variation_to(Tag::from_bytes(b"stro"), &mut metrics.position);
            self.apply_metrics_variation_to(Tag::from_bytes(b"strs"), &mut metrics.thickness);
        }
        Some(metrics)
    }

    /// Returns font's subscript metrics.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns `None` when OS/2 table is not present.
    #[inline]
    pub fn subscript_metrics(&self) -> Option<ScriptMetrics> {
        let mut metrics = self.os_2?.subscript_metrics();
        if self.is_variable() {
            self.apply_metrics_variation_to(Tag::from_bytes(b"sbxs"), &mut metrics.x_size);
            self.apply_metrics_variation_to(Tag::from_bytes(b"sbys"), &mut metrics.y_size);
            self.apply_metrics_variation_to(Tag::from_bytes(b"sbxo"), &mut metrics.x_offset);
            self.apply_metrics_variation_to(Tag::from_bytes(b"sbyo"), &mut metrics.y_offset);
        }
        Some(metrics)
    }

    /// Returns font's superscript metrics.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns `None` when OS/2 table is not present.
    #[inline]
    pub fn superscript_metrics(&self) -> Option<ScriptMetrics> {
        let mut metrics = self.os_2?.superscript_metrics();
        if self.is_variable() {
            self.apply_metrics_variation_to(Tag::from_bytes(b"spxs"), &mut metrics.x_size);
            self.apply_metrics_variation_to(Tag::from_bytes(b"spys"), &mut metrics.y_size);
            self.apply_metrics_variation_to(Tag::from_bytes(b"spxo"), &mut metrics.x_offset);
            self.apply_metrics_variation_to(Tag::from_bytes(b"spyo"), &mut metrics.y_offset);
        }
        Some(metrics)
    }

    /// Returns a total number of glyphs in the font.
    ///
    /// Never zero.
    ///
    /// The value was already parsed, so this function doesn't involve any parsing.
    #[inline]
    pub fn number_of_glyphs(&self) -> u16 {
        self.number_of_glyphs.get()
    }

    /// Resolves a Glyph ID for a code point.
    ///
    /// Returns `None` instead of `0` when glyph is not found.
    ///
    /// All subtable formats except Mixed Coverage (8) are supported.
    #[inline]
    pub fn glyph_index(&self, c: char) -> Option<GlyphId> {
        cmap::glyph_index(self.cmap.as_ref()?, c)
    }

    /// Resolves a variation of a Glyph ID from two code points.
    ///
    /// Implemented according to
    /// [Unicode Variation Sequences](
    /// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences).
    ///
    /// Returns `None` instead of `0` when glyph is not found.
    #[inline]
    pub fn glyph_variation_index(&self, c: char, variation: char) -> Option<GlyphId> {
        cmap::glyph_variation_index(self.cmap.as_ref()?, c, variation)
    }

    /// Returns glyph's horizontal advance.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn glyph_hor_advance(&self, glyph_id: GlyphId) -> Option<u16> {
        let mut advance = self.hmtx?.advance(glyph_id)? as f32;

        if self.is_variable() {
            // Ignore variation offset when `hvar` is not set.
            if let Some(hvar_data) = self.hvar {
                // We can't use `round()` in `no_std`, so this is the next best thing.
                advance += hvar::glyph_advance_offset(hvar_data, glyph_id, self.coords())? + 0.5;
            }
        }

        u16::try_num_from(advance)
    }

    /// Returns glyph's vertical advance.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn glyph_ver_advance(&self, glyph_id: GlyphId) -> Option<u16> {
        let mut advance = self.vmtx?.advance(glyph_id)? as f32;

        if self.is_variable() {
            // Ignore variation offset when `vvar` is not set.
            if let Some(vvar_data) = self.vvar {
                // We can't use `round()` in `no_std`, so this is the next best thing.
                advance += hvar::glyph_advance_offset(vvar_data, glyph_id, self.coords())? + 0.5;
            }
        }

        u16::try_num_from(advance)
    }

    /// Returns glyph's horizontal side bearing.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn glyph_hor_side_bearing(&self, glyph_id: GlyphId) -> Option<i16> {
        let mut bearing = self.hmtx?.side_bearing(glyph_id)? as f32;

        if self.is_variable() {
            // Ignore variation offset when `hvar` is not set.
            if let Some(hvar_data) = self.hvar {
                // We can't use `round()` in `no_std`, so this is the next best thing.
                bearing += hvar::glyph_side_bearing_offset(hvar_data, glyph_id, self.coords())? + 0.5;
            }
        }

        i16::try_num_from(bearing)
    }

    /// Returns glyph's vertical side bearing.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn glyph_ver_side_bearing(&self, glyph_id: GlyphId) -> Option<i16> {
        let mut bearing = self.vmtx?.side_bearing(glyph_id)? as f32;

        if self.is_variable() {
            // Ignore variation offset when `vvar` is not set.
            if let Some(vvar_data) = self.vvar {
                // We can't use `round()` in `no_std`, so this is the next best thing.
                bearing += hvar::glyph_side_bearing_offset(vvar_data, glyph_id, self.coords())? + 0.5;
            }
        }

        i16::try_num_from(bearing)
    }

    /// Returns glyph's vertical origin according to
    /// [Vertical Origin Table](https://docs.microsoft.com/en-us/typography/opentype/spec/vorg).
    pub fn glyph_y_origin(&self, glyph_id: GlyphId) -> Option<i16> {
        self.vorg.map(|vorg| vorg.glyph_y_origin(glyph_id))
    }

    /// Returns glyph's name.
    ///
    /// Uses the `post` table as a source.
    ///
    /// Returns `None` when no name is associated with a `glyph`.
    #[inline]
    pub fn glyph_name(&self, glyph_id: GlyphId) -> Option<&str> {
        self.post.and_then(|post| post.glyph_name(glyph_id))
    }

    /// Checks that font has
    /// [Glyph Class Definition Table](
    /// https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#glyph-class-definition-table).
    pub fn has_glyph_classes(&self) -> bool {
        try_opt_or!(self.gdef, false).has_glyph_classes()
    }

    /// Returns glyph's class according to
    /// [Glyph Class Definition Table](
    /// https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#glyph-class-definition-table).
    ///
    /// Returns `None` when *Glyph Class Definition Table* is not set
    /// or glyph class is not set or invalid.
    pub fn glyph_class(&self, glyph_id: GlyphId) -> Option<GlyphClass> {
        self.gdef.and_then(|gdef| gdef.glyph_class(glyph_id))
    }

    /// Returns glyph's mark attachment class according to
    /// [Mark Attachment Class Definition Table](
    /// https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#mark-attachment-class-definition-table).
    ///
    /// All glyphs not assigned to a class fall into Class 0.
    pub fn glyph_mark_attachment_class(&self, glyph_id: GlyphId) -> Class {
        try_opt_or!(self.gdef, Class(0)).glyph_mark_attachment_class(glyph_id)
    }

    /// Checks that glyph is a mark according to
    /// [Mark Glyph Sets Table](
    /// https://docs.microsoft.com/en-us/typography/opentype/spec/gdef#mark-glyph-sets-table).
    ///
    /// `set_index` allows checking a specific glyph coverage set.
    /// Otherwise all sets will be checked.
    ///
    /// Returns `Ok(false)` when *Mark Glyph Sets Table* is not set.
    #[inline]
    pub fn is_mark_glyph(&self, glyph_id: GlyphId, set_index: Option<u16>) -> bool {
        try_opt_or!(self.gdef, false).is_mark_glyph(glyph_id, set_index)
    }

    /// Returns a iterator over kerning subtables.
    ///
    /// Supports both
    /// [OpenType](https://docs.microsoft.com/en-us/typography/opentype/spec/kern)
    /// and
    /// [Apple Advanced Typography](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kern.html)
    /// variants.
    pub fn kerning_subtables(&self) -> kern::Subtables {
        self.kern.unwrap_or_default()
    }

    /// Outlines a glyph and returns its tight bounding box.
    ///
    /// **Warning**: since `ttf-parser` is a pull parser,
    /// `OutlineBuilder` will emit segments even when outline is partially malformed.
    /// You must check `outline_glyph()` result before using
    /// `OutlineBuilder`'s output.
    ///
    /// `glyf`, `gvar`, `CFF` and `CFF2` tables are supported.
    ///
    /// This method is affected by variation axes.
    ///
    /// Returns `None` when glyph has no outline or on error.
    ///
    /// # Example
    ///
    /// ```
    /// use std::fmt::Write;
    /// use ttf_parser;
    ///
    /// struct Builder(String);
    ///
    /// impl ttf_parser::OutlineBuilder for Builder {
    ///     fn move_to(&mut self, x: f32, y: f32) {
    ///         write!(&mut self.0, "M {} {} ", x, y).unwrap();
    ///     }
    ///
    ///     fn line_to(&mut self, x: f32, y: f32) {
    ///         write!(&mut self.0, "L {} {} ", x, y).unwrap();
    ///     }
    ///
    ///     fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
    ///         write!(&mut self.0, "Q {} {} {} {} ", x1, y1, x, y).unwrap();
    ///     }
    ///
    ///     fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
    ///         write!(&mut self.0, "C {} {} {} {} {} {} ", x1, y1, x2, y2, x, y).unwrap();
    ///     }
    ///
    ///     fn close(&mut self) {
    ///         write!(&mut self.0, "Z ").unwrap();
    ///     }
    /// }
    ///
    /// let data = std::fs::read("fonts/SourceSansPro-Regular-Tiny.ttf").unwrap();
    /// let font = ttf_parser::Font::from_data(&data, 0).unwrap();
    /// let mut builder = Builder(String::new());
    /// let bbox = font.outline_glyph(ttf_parser::GlyphId(13), &mut builder).unwrap();
    /// assert_eq!(builder.0, "M 90 0 L 90 656 L 173 656 L 173 71 L 460 71 L 460 0 L 90 0 Z ");
    /// assert_eq!(bbox, ttf_parser::Rect { x_min: 90, y_min: 0, x_max: 460, y_max: 656 });
    /// ```
    #[inline]
    pub fn outline_glyph(
        &self,
        glyph_id: GlyphId,
        builder: &mut dyn OutlineBuilder,
    ) -> Option<Rect> {
        if let Some(ref gvar_table) = self.gvar {
            return gvar::outline(self.loca?, self.glyf?, gvar_table, self.coords(), glyph_id, builder);
        }

        if let Some(glyf_table) = self.glyf {
            return glyf::outline(self.loca?, glyf_table, glyph_id, builder);
        }

        if let Some(ref metadata) = self.cff_ {
            return cff::outline(metadata, glyph_id, builder);
        }

        if let Some(ref metadata) = self.cff2 {
            return cff2::outline(metadata, self.coords(), glyph_id, builder);
        }

        None
    }

    /// Returns a tight glyph bounding box.
    ///
    /// Unless the current font has a `glyf` table, this is just a shorthand for `outline_glyph()`
    /// since only the `glyf` table stores a bounding box. In case of CFF and variable fonts
    /// we have to actually outline a glyph to find it's bounding box.
    ///
    /// When a glyph is defined by a raster or a vector image,
    /// that can be obtained via `glyph_image()`,
    /// the bounding box must be calculated manually and this method will return `None`.
    ///
    /// This method is affected by variation axes.
    #[inline]
    pub fn glyph_bounding_box(&self, glyph_id: GlyphId) -> Option<Rect> {
        if !self.is_variable() {
            if let Some(glyf_table) = self.glyf {
                return glyf::glyph_bbox(self.loca?, glyf_table, glyph_id);
            }
        }

        self.outline_glyph(glyph_id, &mut DummyOutline)
    }

    /// Returns a reference to a glyph's raster image.
    ///
    /// A font can define a glyph using a raster or a vector image instead of a simple outline.
    /// Which is primarily used for emojis. This method should be used to access raster images.
    ///
    /// `pixels_per_em` allows selecting a preferred image size. The chosen size will
    /// be closer to an upper one. So when font has 64px and 96px images and `pixels_per_em`
    /// is set to 72, 96px image will be returned.
    /// To get the largest image simply use `std::u16::MAX`.
    ///
    /// Note that this method will return an encoded image. It should be decoded
    /// by the caller. We don't validate or preprocess it in any way.
    ///
    /// Currently, only PNG images are supported.
    ///
    /// Also, a font can contain both: images and outlines. So when this method returns `None`
    /// you should also try `outline_glyph()` afterwards.
    ///
    /// There are multiple ways an image can be stored in a TrueType font
    /// and this method supports only `sbix`, `CBLC`+`CBDT`.
    #[inline]
    pub fn glyph_raster_image(&self, glyph_id: GlyphId, pixels_per_em: u16) -> Option<RasterGlyphImage> {
        if let Some(sbix_data) = self.sbix {
            return sbix::parse(sbix_data, self.number_of_glyphs, glyph_id, pixels_per_em, 0);
        }

        if let (Some(cblc_data), Some(cbdt_data)) = (self.cblc, self.cbdt) {
            let location = cblc::find_location(cblc_data, glyph_id, pixels_per_em)?;
            return cbdt::parse(cbdt_data, location);
        }

        None
    }

    /// Returns a reference to a glyph's SVG image.
    ///
    /// A font can define a glyph using a raster or a vector image instead of a simple outline.
    /// Which is primarily used for emojis. This method should be used to access SVG images.
    ///
    /// Note that this method will return just an SVG data. It should be rendered
    /// or even decompressed (in case of SVGZ) by the caller.
    /// We don't validate or preprocess it in any way.
    ///
    /// Also, a font can contain both: images and outlines. So when this method returns `None`
    /// you should also try `outline_glyph()` afterwards.
    #[inline]
    pub fn glyph_svg_image(&self, glyph_id: GlyphId) -> Option<&'a [u8]> {
        self.svg_.and_then(|svg_data| svg::parse(svg_data, glyph_id))
    }

    /// Returns an iterator over variation axes.
    #[inline]
    pub fn variation_axes(&self) -> VariationAxes {
        self.fvar.map(|fvar| fvar.axes()).unwrap_or_default()
    }

    /// Sets a variation axis coordinate.
    ///
    /// This is the only mutable method in the library.
    /// We can simplify the API a lot by storing the variable coordinates
    /// in the font object itself.
    ///
    /// Since coordinates are stored on the stack, we allow only 32 of them.
    ///
    /// Returns `None` when font is not variable or doesn't have such axis.
    pub fn set_variation(&mut self, axis: Tag, value: f32) -> Option<()> {
        if !self.is_variable() {
            return None;
        }

        let v = self.variation_axes().enumerate().find(|(_, a)| a.tag == axis);
        if let Some((idx, a)) = v {
            if idx >= usize::from(MAX_VAR_COORDS) {
                return None;
            }

            self.coordinates.data[idx] = a.normalized_value(value);
        } else {
            return None;
        }

        // TODO: optimize
        if let Some(avar) = self.avar {
            // Ignore error.
            let _ = avar.map_coordinates(self.coordinates.as_mut_slice());
        }

        Some(())
    }

    #[inline]
    fn metrics_var_offset(&self, tag: Tag) -> f32 {
        self.mvar.and_then(|table| table.metrics_offset(tag, self.coords())).unwrap_or(0.0)
    }

    #[inline]
    fn apply_metrics_variation(&self, tag: Tag, mut value: i16) -> i16 {
        self.apply_metrics_variation_to(tag, &mut value);
        value
    }

    #[inline]
    fn apply_metrics_variation_to(&self, tag: Tag, value: &mut i16) {
        if self.is_variable() {
            let v = f32::from(*value) + self.metrics_var_offset(tag);
            // TODO: Should probably round it, but f32::round is not available in core.
            if let Some(v) = i16::try_num_from(v) {
                *value = v;
            }
        }
    }

    #[inline]
    fn coords(&self) -> &[NormalizedCoord] {
        self.coordinates.as_slice()
    }
}

impl fmt::Debug for Font<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Font()")
    }
}

/// Returns the number of fonts stored in a TrueType font collection.
///
/// Returns `None` if a provided data is not a TrueType font collection.
#[inline]
pub fn fonts_in_collection(data: &[u8]) -> Option<u32> {
    let mut s = Stream::new(data);
    if &s.read::<Tag>()?.to_bytes() != b"ttcf" {
        return None;
    }

    s.skip::<u32>(); // version
    s.read()
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::writer;
    use writer::TtfType::*;

    #[test]
    fn empty_font() {
        assert!(Font::from_data(&[], 0).is_none());
    }

    #[test]
    fn incomplete_header() {
        let data = writer::convert(&[
            TrueTypeMagic,
            UInt16(0), // numTables
            UInt16(0), // searchRange
            UInt16(0), // entrySelector
            UInt16(0), // rangeShift
        ]);

        for i in 0..data.len() {
            assert!(Font::from_data(&data[0..i], 0).is_none());
        }
    }

    #[test]
    fn zero_tables() {
        let data = writer::convert(&[
            TrueTypeMagic,
            UInt16(0), // numTables
            UInt16(0), // searchRange
            UInt16(0), // entrySelector
            UInt16(0), // rangeShift
        ]);

        assert!(Font::from_data(&data, 0).is_none());
    }

    #[test]
    fn tables_count_overflow() {
        let data = writer::convert(&[
            TrueTypeMagic,
            UInt16(std::u16::MAX), // numTables
            UInt16(0), // searchRange
            UInt16(0), // entrySelector
            UInt16(0), // rangeShift
        ]);

        assert!(Font::from_data(&data, 0).is_none());
    }

    #[test]
    fn open_type_magic() {
        let data = writer::convert(&[
            OpenTypeMagic,
            UInt16(0), // numTables
            UInt16(0), // searchRange
            UInt16(0), // entrySelector
            UInt16(0), // rangeShift
        ]);

        assert!(Font::from_data(&data, 0).is_none());
    }

    #[test]
    fn unknown_magic() {
        let data = writer::convert(&[
            Raw(&[0xFF, 0xFF, 0xFF, 0xFF]),
            UInt16(0), // numTables
            UInt16(0), // searchRange
            UInt16(0), // entrySelector
            UInt16(0), // rangeShift
        ]);

        assert!(Font::from_data(&data, 0).is_none());
    }

    #[test]
    fn empty_font_collection() {
        let data = writer::convert(&[
            FontCollectionMagic,
            UInt16(1), // majorVersion
            UInt16(0), // minorVersion
            UInt32(0), // numFonts
        ]);

        assert_eq!(fonts_in_collection(&data), Some(0));
        assert!(Font::from_data(&data, 0).is_none());
    }

    #[test]
    fn font_collection_num_fonts_overflow() {
        let data = writer::convert(&[
            FontCollectionMagic,
            UInt16(1), // majorVersion
            UInt16(0), // minorVersion
            UInt32(std::u32::MAX), // numFonts
        ]);

        assert_eq!(fonts_in_collection(&data), Some(std::u32::MAX));
        assert!(Font::from_data(&data, 0).is_none());
    }

    #[test]
    fn font_index_overflow_1() {
        let data = writer::convert(&[
            FontCollectionMagic,
            UInt16(1), // majorVersion
            UInt16(0), // minorVersion
            UInt32(1), // numFonts
        ]);

        assert_eq!(fonts_in_collection(&data), Some(1));
        assert!(Font::from_data(&data, std::u32::MAX).is_none());
    }

    #[test]
    fn font_index_overflow_2() {
        let data = writer::convert(&[
            FontCollectionMagic,
            UInt16(1), // majorVersion
            UInt16(0), // minorVersion
            UInt32(std::u32::MAX), // numFonts
        ]);

        assert_eq!(fonts_in_collection(&data), Some(std::u32::MAX));
        assert!(Font::from_data(&data, std::u32::MAX - 1).is_none());
    }
}
