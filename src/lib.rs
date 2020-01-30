/*!
A high-level, safe, zero-allocation TrueType font parser.

## Features

- A high-level API.
- Zero allocations.
- Zero dependencies.
- `no_std` compatible.
- Fast.
- Stateless.
- Simple and maintainable code (no magic numbers).

## Supported TrueType features

- (`cmap`) Character to glyph index mapping using [glyph_index()] method.
  <br/>All subtable formats except Mixed Coverage (8) are supported.
- (`cmap`) Character variation to glyph index mapping using [glyph_variation_index()] method.
- (`glyf`) Glyph outlining using [outline_glyph()] method.
- (`hmtx`) Retrieving glyph's horizontal metrics using [glyph_hor_advance()] and [glyph_hor_side_bearing()] methods.
- (`vmtx`) Retrieving glyph's vertical metrics using [glyph_ver_advance()] and [glyph_ver_side_bearing()] methods.
- (`kern`) Retrieving glyphs pair kerning using [glyphs_kerning()] method.
- (`maxp`) Retrieving total number of glyphs using [number_of_glyphs()] method.
- (`name`) Listing all name records using [names()] method.
- (`name`) Retrieving font's family name using [family_name()] method.
- (`name`) Retrieving font's PostScript name using [post_script_name()] method.
- (`post`) Retrieving font's underline metrics name using [underline_metrics()] method.
- (`post`) Retrieving glyph's name using [glyph_name()] method.
- (`head`) Retrieving font's units per EM value using [units_per_em()] method.
- (`hhea`) Retrieving generic font info using: [ascender()], [descender()], [height()]
  and [line_gap()] methods.

[glyph_index()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_index
[glyph_variation_index()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_variation_index
[outline_glyph()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.outline_glyph
[glyph_hor_advance()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_hor_advance
[glyph_hor_side_bearing()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_hor_side_bearing
[glyph_ver_advance()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_ver_advance
[glyph_ver_side_bearing()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_ver_side_bearing
[glyphs_kerning()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyphs_kerning
[number_of_glyphs()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.number_of_glyphs
[names()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.names
[family_name()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.family_name
[post_script_name()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.post_script_name
[underline_metrics()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.underline_metrics
[glyph_name()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_name
[units_per_em()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.units_per_em
[ascender()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.ascender
[descender()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.descender
[height()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.height
[line_gap()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.line_gap

## Supported OpenType features

- (`CFF `) Glyph outlining using [outline_glyph()] method.
- (`CFF2`) Glyph outlining using [outline_glyph()] method.
- (`OS/2`) Retrieving font's kind using [is_regular()], [is_italic()],
  [is_bold()] and [is_oblique()] methods.
- (`OS/2`) Retrieving font's weight using [weight()] method.
- (`OS/2`) Retrieving font's width using [width()] method.
- (`OS/2`) Retrieving font's X height using [x_height()] method.
- (`OS/2`) Retrieving font's strikeout metrics using [strikeout_metrics()] method.
- (`OS/2`) Retrieving font's subscript metrics using [subscript_metrics()] method.
- (`OS/2`) Retrieving font's superscript metrics using [superscript_metrics()] method.
- (`GDEF`) Retrieving glyph's class using [glyph_class()] method.
- (`GDEF`) Retrieving glyph's mark attachment class using [glyph_mark_attachment_class()] method.
- (`GDEF`) Checking that glyph is a mark using [is_mark_glyph()] method.
- (`avar`) Variation coordinates normalization using [map_variation_coordinates()] method.
- (`fvar`) Variation axis parsing using [variation_axis()] method.
- (`VORG`) Retrieving glyph's vertical origin using [glyph_y_origin()] method.
- (`MVAR`) Retrieving font's metrics variation using [metrics_variation()] method.
- (`HVAR`) Retrieving glyph's variation offset for horizontal advance using [glyph_hor_advance_variation()] method.
- (`HVAR`) Retrieving glyph's variation offset for horizontal side bearing using [glyph_hor_side_bearing_variation()] method.
- (`VVAR`) Retrieving glyph's variation offset for vertical advance using [glyph_ver_advance_variation()] method.
- (`VVAR`) Retrieving glyph's variation offset for vertical side bearing using [glyph_ver_side_bearing_variation()] method.

[is_regular()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_regular
[is_italic()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_italic
[is_bold()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_bold
[is_oblique()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_oblique
[weight()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.weight
[width()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.width
[x_height()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.x_height
[strikeout_metrics()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.strikeout_metrics
[subscript_metrics()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.subscript_metrics
[superscript_metrics()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.superscript_metrics
[glyph_class()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_class
[glyph_mark_attachment_class()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_mark_attachment_class
[is_mark_glyph()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.is_mark_glyph
[map_variation_coordinates()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.map_variation_coordinates
[variation_axis()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.variation_axis
[glyph_y_origin()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_y_origin
[metrics_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.metrics_variation
[glyph_hor_advance_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_hor_advance_variation
[glyph_hor_side_bearing_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_hor_side_bearing_variation
[glyph_ver_advance_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_ver_advance_variation
[glyph_ver_side_bearing_variation()]: https://docs.rs/ttf-parser/0.3.0/ttf_parser/struct.Font.html#method.glyph_ver_side_bearing_variation

## Error handling

The library uses `Result<Option<T>, Error>` pattern, where `Error` indicates a parsing error
and `Ok(None)` a not set value.
This is a bit verbose, but allows us to separate malformed files and not set values.
For example, if a font doesn't have a glyph for a specified character - it's not an error.
And error will be emitted only in two cases: on a malformed file or bug in implementation.

## Methods' computational complexity

TrueType fonts designed for fast querying, so most of the methods are very fast.
The main exception is glyph outlining. Glyphs can be stored using two different methods:
using [Glyph Data](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) format
and [Compact Font Format](http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5176.CFF.pdf) (pdf).
The first one is fairly simple which makes it faster to process.
The second one is basically a tiny language with a stack-based VM, which makes it way harder to process.

```text
test outline_cff  ... bench:   1,010,120 ns/iter (+/- 11,517)
test outline_cff2 ... bench:   1,385,488 ns/iter (+/- 21,411)
test outline_glyf ... bench:     717,052 ns/iter (+/- 5,907)
```

Here is some methods benchmarks:

```text
test outline_glyph_276_from_cff  ... bench:   745.0 ns/iter (+/- 31)
test from_data_otf_cff2          ... bench:   673.0 ns/iter (+/- 9)
test outline_glyph_276_from_cff2 ... bench:   595.0 ns/iter (+/- 24)
test outline_glyph_276_from_glyf ... bench:   564.0 ns/iter (+/- 6)
test from_data_otf_cff           ... bench:   485.0 ns/iter (+/- 11)
test outline_glyph_8_from_cff2   ... bench:   371.0 ns/iter (+/- 54)
test outline_glyph_8_from_glyf   ... bench:   249.0 ns/iter (+/- 2)
test outline_glyph_8_from_cff    ... bench:   243.0 ns/iter (+/- 7)
test glyph_name_276              ... bench:   216.0 ns/iter (+/- 0)
test from_data_ttf               ... bench:   200.0 ns/iter (+/- 3)
test family_name                 ... bench:   161.0 ns/iter (+/- 5)
test glyph_index_u41             ... bench:    14.0 ns/iter (+/- 1)
test hor_advance                 ... bench:     3.0 ns/iter (+/- 0)
test hor_side_bearing            ... bench:     3.0 ns/iter (+/- 0)
test glyph_name_8                ... bench:     2.0 ns/iter (+/- 0)
test ascender                    ... bench:     0.6 ns/iter (+/- 0)
test x_height                    ... bench:     0.5 ns/iter (+/- 0)
test underline_metrics           ... bench:     0.5 ns/iter (+/- 0)
test strikeout_metrics           ... bench:     0.5 ns/iter (+/- 0)
test units_per_em                ... bench:     0.5 ns/iter (+/- 0)
test subscript_metrics           ... bench:     0.2 ns/iter (+/- 0)
test width                       ... bench:     0.2 ns/iter (+/- 0)
```

`family_name` is expensive, because it allocates a `String` and the original data
is stored as UTF-16 BE.

`glyph_name_8` is faster that `glyph_name_276`, because for glyph indexes lower than 258
we are using predefined names, so no parsing is involved.

## Safety

- The library must not panic. Any panic considered as a critical bug and should be reported.
- The library has a single `unsafe` block for array casting.
*/

#![doc(html_root_url = "https://docs.rs/ttf-parser/0.3.0")]

#![no_std]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

#[cfg(feature = "std")]
#[macro_use]
extern crate std;

use core::fmt;

macro_rules! try_ok {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return Ok(None),
        };
    };
}

macro_rules! bail {
    ($e:expr) => {
        match $e {
            Ok(Some(v)) => v,
            Ok(None) => return Ok(None),
            Err(e) => return Err(e),
        };
    };
}

mod avar;
mod cff;
mod cff2;
mod cmap;
mod fvar;
mod gdef;
mod glyf;
mod head;
mod hhea;
mod hmtx;
mod hvar;
mod kern;
mod loca;
mod mvar;
mod name;
mod os2;
mod parser;
mod post;
mod raw;
mod vhea;
mod vmtx;
mod vorg;
mod vvar;

#[cfg(feature = "std")]
mod writer;

use parser::{Stream, FromData, SafeStream, TrySlice, Offset};
pub use cff::CFFError;
pub use fvar::VariationAxis;
pub use gdef::{Class, GlyphClass};
pub use head::IndexToLocationFormat;
pub use name::*;
pub use os2::*;


/// A type-safe wrapper for glyph ID.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct GlyphId(pub u16);

impl FromData for GlyphId {
    #[inline]
    fn parse(data: &[u8]) -> Self {
        let mut s = SafeStream::new(data);
        GlyphId(s.read())
    }
}


/// A font parsing error.
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Not a TrueType data.
    NotATrueType,

    /// The font index is out of bounds.
    FontIndexOutOfBounds,

    /// One of the required tables is missing.
    TableMissing(TableName),

    /// Table has an invalid size.
    InvalidTableSize(TableName),

    /// An unsupported table version.
    UnsupportedTableVersion,

    /// The number of provided variation coordinates must be the same
    /// as the axis number in the font.
    InvalidNumberOfVarCoordinates,

    /// Recursion detected in the `glyf` table.
    GlyphDataRecursion,

    /// An attempt to slice a raw data out of bounds.
    ///
    /// This may be caused by a bug in the library or by a malformed font.
    SliceOutOfBounds,

    /// A CFF/CFF2 table parsing error.
    CFFError(CFFError),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            Error::NotATrueType => {
                write!(f, "not a TrueType font")
            }
            Error::FontIndexOutOfBounds => {
                write!(f, "font index is out of bounds")
            }
            Error::TableMissing(name) => {
                write!(f, "font doesn't have a {:?} table", name)
            }
            Error::InvalidTableSize(name) => {
                write!(f, "table {:?} has an invalid size", name)
            }
            Error::UnsupportedTableVersion => {
                write!(f, "table has an unsupported version")
            }
            Error::InvalidNumberOfVarCoordinates => {
                write!(f, "invalid number of variation coordinates")
            }
            Error::GlyphDataRecursion => {
                write!(f, "recursion detected in the 'glyf' table")
            }
            Error::SliceOutOfBounds => {
                write!(f, "an attempt to slice out of bounds")
            }
            Error::CFFError(e) => {
                write!(f, "CFF table parsing failed cause {}", e)
            }
        }
    }
}

impl From<CFFError> for Error {
    #[inline]
    fn from(e: CFFError) -> Self {
        Error::CFFError(e)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

pub(crate) type Result<T> = core::result::Result<T, Error>;


/// A 4-byte tag.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tag(pub u32);

impl Tag {
    /// Creates a `Tag` from bytes.
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
    pub const fn to_bytes(self) -> [u8; 4] {
        [
            (self.0 >> 24 & 0xff) as u8,
            (self.0 >> 16 & 0xff) as u8,
            (self.0 >> 8 & 0xff) as u8,
            (self.0 >> 0 & 0xff) as u8,
        ]
    }

    /// Returns tag as 4-element byte array.
    pub const fn to_chars(self) -> [char; 4] {
        [
            (self.0 >> 24 & 0xff) as u8 as char,
            (self.0 >> 16 & 0xff) as u8 as char,
            (self.0 >> 8 & 0xff) as u8 as char,
            (self.0 >> 0 & 0xff) as u8 as char,
        ]
    }

    /// Returns tag for a default script.
    pub const fn default_script() -> Self {
        Tag::from_bytes(b"DFLT")
    }

    /// Returns tag for a default language.
    pub const fn default_language() -> Self {
        Tag::from_bytes(b"dflt")
    }

    /// Checks if tag is null / `[0, 0, 0, 0]`.
    pub const fn is_null(&self) -> bool {
        self.0 == 0
    }

    /// Returns tag value as `u32` number.
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    /// Converts tag to lowercase.
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Tag({})", self)
    }
}

impl core::fmt::Display for Tag {
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


/// A line metrics.
///
/// Used for underline and strikeout.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LineMetrics {
    /// Line position.
    pub position: i16,

    /// Line thickness.
    pub thickness: i16,
}


/// Rectangle.
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct Rect {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
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


/// A table name.
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum TableName {
    AxisVariations,
    CharacterToGlyphIndexMapping,
    CompactFontFormat,
    CompactFontFormat2,
    FontVariations,
    GlyphData,
    GlyphDefinition,
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
    VerticalHeader,
    VerticalMetrics,
    VerticalMetricsVariations,
    VerticalOrigin,
    WindowsMetrics,
}


/// A font data handle.
#[derive(Clone)]
pub struct Font<'a> {
    avar: Result<&'a [u8]>,
    head: raw::head::Table<'a>,
    hhea: raw::hhea::Table<'a>,
    cff_: Result<cff::Metadata<'a>>,
    cff2: Result<cff2::Metadata<'a>>,
    cmap: Result<&'a [u8]>,
    fvar: Result<raw::fvar::Table<'a>>,
    gdef: Result<raw::gdef::Table<'a>>,
    glyf: Result<&'a [u8]>,
    hmtx: Result<&'a [u8]>,
    hvar: Result<&'a [u8]>,
    kern: Result<&'a [u8]>,
    loca: Result<&'a [u8]>,
    mvar: Result<&'a [u8]>,
    name: Result<&'a [u8]>,
    os_2: Result<raw::os_2::Table<'a>>,
    post: Result<&'a [u8]>,
    vhea: Result<raw::vhea::Table<'a>>,
    vmtx: Result<&'a [u8]>,
    vorg: Result<&'a [u8]>,
    vvar: Result<&'a [u8]>,
    number_of_glyphs: GlyphId,
}

impl<'a> Font<'a> {
    /// Creates a `Font` object from a raw data.
    ///
    /// You can set `index` in case of font collections.
    /// For simple `ttf` fonts set `index` to 0.
    ///
    /// This function only parses font tables, so it's relatively light.
    ///
    /// Required tables: `head` and `hhea`.
    pub fn from_data(data: &'a [u8], index: u32) -> Result<Self> {
        let table_data = if let Some(n) = fonts_in_collection(data) {
            if index < n {
                // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
                const OFFSET_32_SIZE: usize = 4;
                let offset = raw::TTCHeader::SIZE + OFFSET_32_SIZE * index as usize;
                let font_offset: u32 = Stream::read_at(data, offset)?;
                data.try_slice(font_offset as usize .. data.len())?
            } else {
                return Err(Error::FontIndexOutOfBounds);
            }
        } else {
            data
        };

        // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
        const OFFSET_TABLE_SIZE: usize = 12;
        if data.len() < OFFSET_TABLE_SIZE {
            return Err(Error::NotATrueType);
        }

        // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
        const SFNT_VERSION_TRUE_TYPE: u32 = 0x00010000;
        const SFNT_VERSION_OPEN_TYPE: u32 = 0x4F54544F;

        let mut s = Stream::new(table_data);

        let sfnt_version: u32 = s.read()?;
        if sfnt_version != SFNT_VERSION_TRUE_TYPE && sfnt_version != SFNT_VERSION_OPEN_TYPE {
            return Err(Error::NotATrueType);
        }

        let num_tables: u16 = s.read()?;
        s.advance(6u32); // searchRange (u16) + entrySelector (u16) + rangeShift (u16)
        let tables = s.read_array::<raw::TableRecord, u16>(num_tables)?;

        let mut font = Font {
            avar: Err(Error::TableMissing(TableName::AxisVariations)),
            cff2: Err(Error::TableMissing(TableName::CompactFontFormat2)),
            cff_: Err(Error::TableMissing(TableName::CompactFontFormat)),
            cmap: Err(Error::TableMissing(TableName::CharacterToGlyphIndexMapping)),
            fvar: Err(Error::TableMissing(TableName::FontVariations)),
            gdef: Err(Error::TableMissing(TableName::GlyphDefinition)),
            glyf: Err(Error::TableMissing(TableName::GlyphData)),
            head: raw::head::Table::new(&[0; raw::head::Table::SIZE]),
            hhea: raw::hhea::Table::new(&[0; raw::hhea::Table::SIZE]),
            hmtx: Err(Error::TableMissing(TableName::HorizontalMetrics)),
            hvar: Err(Error::TableMissing(TableName::HorizontalMetricsVariations)),
            kern: Err(Error::TableMissing(TableName::Kerning)),
            loca: Err(Error::TableMissing(TableName::IndexToLocation)),
            mvar: Err(Error::TableMissing(TableName::MetricsVariations)),
            name: Err(Error::TableMissing(TableName::Naming)),
            os_2: Err(Error::TableMissing(TableName::WindowsMetrics)),
            post: Err(Error::TableMissing(TableName::PostScript)),
            vhea: Err(Error::TableMissing(TableName::VerticalHeader)),
            vmtx: Err(Error::TableMissing(TableName::VerticalMetrics)),
            vorg: Err(Error::TableMissing(TableName::VerticalOrigin)),
            vvar: Err(Error::TableMissing(TableName::VerticalMetricsVariations)),
            number_of_glyphs: GlyphId(0),
        };

        let mut has_head = false;
        let mut has_hhea = false;
        for table in tables {
            let offset = table.offset().to_usize();
            let length = table.length() as usize;
            let range = offset..(offset + length);

            // It's way faster to compare `[u8; 4]` with `&[u8]`
            // rather than `&[u8]` with `&[u8]`.
            match &table.table_tag().to_bytes() {
                b"head" => {
                    if length != raw::head::Table::SIZE {
                        return Err(Error::InvalidTableSize(TableName::Header));
                    }

                    font.head = raw::head::Table::new(data.try_slice(range)?);
                    has_head = true;
                }
                b"hhea" => {
                    if length != raw::hhea::Table::SIZE {
                        return Err(Error::InvalidTableSize(TableName::HorizontalHeader));
                    }

                    font.hhea = raw::hhea::Table::new(data.try_slice(range)?);
                    has_hhea = true;
                }
                b"maxp" => {
                    if length < raw::maxp::Table::SIZE {
                        return Err(Error::InvalidTableSize(TableName::MaximumProfile));
                    }

                    let data = data.try_slice(offset..(offset + raw::maxp::Table::SIZE))?;
                    let table = raw::maxp::Table::new(data);
                    font.number_of_glyphs = GlyphId(table.num_glyphs());
                }
                b"OS/2" => {
                    if length < raw::os_2::Table::MIN_SIZE {
                        return Err(Error::InvalidTableSize(TableName::WindowsMetrics));
                    }

                    if let Some(table) = data.get(range) {
                        font.os_2 = Ok(raw::os_2::Table::new(table));
                    }
                }
                b"vhea" => {
                    if length != raw::vhea::Table::SIZE {
                        return Err(Error::InvalidTableSize(TableName::VerticalHeader));
                    }

                    if let Some(data) = data.get(range) {
                        font.vhea = Ok(raw::vhea::Table::new(data));
                    }
                }
                b"GDEF" => {
                    if length < raw::gdef::Table::MIN_SIZE {
                        return Err(Error::InvalidTableSize(TableName::GlyphDefinition));
                    }

                    if let Some(data) = data.get(range) {
                        font.gdef = Ok(raw::gdef::Table::new(data));
                    }
                }
                b"fvar" => {
                    if length < raw::fvar::Table::MIN_SIZE {
                        return Err(Error::InvalidTableSize(TableName::FontVariations));
                    }

                    if let Some(table) = data.get(range) {
                        font.fvar = Ok(raw::fvar::Table::new(table));
                    }
                }
                b"CFF " => {
                    if let Some(data) = data.get(range) {
                        font.cff_ = cff::parse_metadata(data);
                    }
                }
                b"CFF2" => {
                    if let Some(data) = data.get(range) {
                        font.cff2 = cff2::parse_metadata(data);
                    }
                }
                b"avar" => if let Some(data) = data.get(range) { font.avar = Ok(data); }
                b"cmap" => if let Some(data) = data.get(range) { font.cmap = Ok(data); }
                b"glyf" => if let Some(data) = data.get(range) { font.glyf = Ok(data); }
                b"hmtx" => if let Some(data) = data.get(range) { font.hmtx = Ok(data); }
                b"kern" => if let Some(data) = data.get(range) { font.kern = Ok(data); }
                b"loca" => if let Some(data) = data.get(range) { font.loca = Ok(data); }
                b"name" => if let Some(data) = data.get(range) { font.name = Ok(data); }
                b"post" => if let Some(data) = data.get(range) { font.post = Ok(data); }
                b"vmtx" => if let Some(data) = data.get(range) { font.vmtx = Ok(data); }
                b"VORG" => if let Some(data) = data.get(range) { font.vorg = Ok(data); }
                b"MVAR" => if let Some(data) = data.get(range) { font.mvar = Ok(data); }
                b"HVAR" => if let Some(data) = data.get(range) { font.hvar = Ok(data); }
                b"VVAR" => if let Some(data) = data.get(range) { font.vvar = Ok(data); }
                _ => {}
            }
        }

        // Check for mandatory tables.
        if !has_head {
            return Err(Error::TableMissing(TableName::Header));
        }

        if !has_hhea {
            return Err(Error::TableMissing(TableName::HorizontalHeader));
        }

        Ok(font)
    }

    /// Checks that font has a specified table.
    #[inline]
    pub fn has_table(&self, name: TableName) -> bool {
        match name {
            TableName::Header                       => true,
            TableName::HorizontalHeader             => true,
            TableName::MaximumProfile               => true,
            TableName::AxisVariations               => self.avar.is_ok(),
            TableName::CharacterToGlyphIndexMapping => self.cmap.is_ok(),
            TableName::CompactFontFormat            => self.cff_.is_ok(),
            TableName::CompactFontFormat2           => self.cff2.is_ok(),
            TableName::FontVariations               => self.fvar.is_ok(),
            TableName::GlyphData                    => self.glyf.is_ok(),
            TableName::GlyphDefinition              => self.gdef.is_ok(),
            TableName::HorizontalMetrics            => self.hmtx.is_ok(),
            TableName::HorizontalMetricsVariations  => self.hvar.is_ok(),
            TableName::IndexToLocation              => self.loca.is_ok(),
            TableName::Kerning                      => self.kern.is_ok(),
            TableName::MetricsVariations            => self.mvar.is_ok(),
            TableName::Naming                       => self.name.is_ok(),
            TableName::PostScript                   => self.post.is_ok(),
            TableName::VerticalHeader               => self.vhea.is_ok(),
            TableName::VerticalMetrics              => self.vmtx.is_ok(),
            TableName::VerticalMetricsVariations    => self.vvar.is_ok(),
            TableName::VerticalOrigin               => self.vorg.is_ok(),
            TableName::WindowsMetrics               => self.os_2.is_ok(),
        }
    }

    /// Returns a total number of glyphs in the font.
    ///
    /// The value was already parsed, so this function doesn't involve any parsing.
    #[inline]
    pub fn number_of_glyphs(&self) -> u16 {
        self.number_of_glyphs.0
    }

    #[inline]
    pub(crate) fn check_glyph_id(&self, glyph_id: GlyphId) -> Result<Option<()>> {
        if glyph_id < self.number_of_glyphs {
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    /// Outlines a glyph and returns its tight bounding box.
    ///
    /// **Warning**: since `ttf-parser` is a pull parser,
    /// `OutlineBuilder` will emit segments even when outline is partially malformed.
    /// You must check `outline_glyph()` result for error before using
    /// `OutlineBuilder`'s output.
    ///
    /// This method supports `glyf`, `CFF` and `CFF2` tables.
    ///
    /// Returns `Ok(None)` when glyph has no outline.
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
    /// let data = std::fs::read("tests/fonts/glyphs.ttf").unwrap();
    /// let font = ttf_parser::Font::from_data(&data, 0).unwrap();
    /// let mut builder = Builder(String::new());
    /// let bbox = font.outline_glyph(ttf_parser::GlyphId(0), &mut builder).unwrap().unwrap();
    /// assert_eq!(builder.0, "M 50 0 L 50 750 L 450 750 L 450 0 L 50 0 Z ");
    /// assert_eq!(bbox, ttf_parser::Rect { x_min: 50, y_min: 0, x_max: 450, y_max: 750 });
    /// ```
    #[inline]
    pub fn outline_glyph(
        &self,
        glyph_id: GlyphId,
        builder: &mut dyn OutlineBuilder,
    ) -> Result<Option<Rect>> {
        if self.glyf.is_ok() {
            return self.glyf_glyph_outline(glyph_id, builder);
        }

        if let Ok(ref metadata) = self.cff_ {
            return self.cff_glyph_outline(metadata, glyph_id, builder);
        }

        if let Ok(ref metadata) = self.cff2 {
            return self.cff2_glyph_outline(metadata, glyph_id, builder);
        }

        Ok(None)
    }

    /// Returns a tight glyph bounding box.
    ///
    /// Note that this method's performance depends on a table type the current font is using.
    /// In case of a `glyf` table, it's basically free, since this table stores
    /// bounding box separately. In case of `CFF` and `CFF2`, we should actually outline
    /// a glyph and then calculate its bounding box. So if you need an outline and
    /// a bounding box and you have an OpenType font (which uses CFF/CFF2)
    /// then prefer `outline_glyph()` method.
    #[inline]
    pub fn glyph_bounding_box(&self, glyph_id: GlyphId) -> Result<Option<Rect>> {
        struct DummyOutline;
        impl OutlineBuilder for DummyOutline {
            fn move_to(&mut self, _: f32, _: f32) {}
            fn line_to(&mut self, _: f32, _: f32) {}
            fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {}
            fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {}
            fn close(&mut self) {}
        }

        if self.glyf.is_ok() {
            return self.glyf_glyph_bbox(glyph_id);
        }

        if let Ok(ref metadata) = self.cff_ {
            return self.cff_glyph_outline(metadata, glyph_id, &mut DummyOutline);
        }

        if let Ok(ref metadata) = self.cff2 {
            return self.cff2_glyph_outline(metadata, glyph_id, &mut DummyOutline);
        }

        Ok(None)
    }
}

impl fmt::Debug for Font<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Font()")
    }
}

/// Returns a number of fonts stored in a TrueType font collection.
///
/// Returns `None` if a provided data is not a TrueType font collection.
#[inline]
pub fn fonts_in_collection(data: &[u8]) -> Option<u32> {
    let table = raw::TTCHeader::new(data.get(0..raw::TTCHeader::SIZE)?);

    if &table.ttc_tag().to_bytes() != b"ttcf" {
        return None;
    }

    Some(table.num_fonts())
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;
    use crate::writer;
    use writer::TtfType::*;

    #[test]
    fn empty_font() {
        assert_eq!(Font::from_data(&[], 0).unwrap_err().to_string(),
                   "not a TrueType font");
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
            assert_eq!(Font::from_data(&data[0..i], 0).unwrap_err().to_string(),
                       "not a TrueType font");
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

        assert_eq!(Font::from_data(&data, 0).unwrap_err().to_string(),
                   "font doesn't have a Header table");
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

        assert_eq!(Font::from_data(&data, 0).unwrap_err().to_string(),
                   "an attempt to slice out of bounds");
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

        assert_eq!(Font::from_data(&data, 0).unwrap_err().to_string(),
                   "font doesn't have a Header table");
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

        assert_eq!(Font::from_data(&data, 0).unwrap_err().to_string(),
                   "not a TrueType font");
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
        assert_eq!(Font::from_data(&data, 0).unwrap_err().to_string(),
                   "font index is out of bounds");
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
        assert_eq!(Font::from_data(&data, 0).unwrap_err().to_string(),
                   "an attempt to slice out of bounds");
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
        assert_eq!(Font::from_data(&data, std::u32::MAX).unwrap_err().to_string(),
                   "font index is out of bounds");
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
        assert_eq!(Font::from_data(&data, std::u32::MAX - 1).unwrap_err().to_string(),
                   "an attempt to slice out of bounds");
    }
}
