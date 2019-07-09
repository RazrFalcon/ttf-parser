/*!
A high-level, safe, zero-allocation TrueType font parser.

## Features

- A high-level API.
- Zero allocations.
- Zero `unsafe`.
- Zero dependencies.
- Fast.
- Stateless.
- Simple and maintainable code (no magic numbers).

## Supported TrueType features

- (`cmap`) Character to glyph index mapping using [glyph_index()] method.
  <br/>All subtable formats except Mixed Coverage (8) are supported.
- (`cmap`) Character variation to glyph index mapping using [glyph_variation_index()] method.
- (`glyf`) Glyph outlining using [outline_glyph()] method.
- (`hmtx`) Retrieving a glyph's horizontal metrics using [glyph_hor_metrics()] method.
- (`vmtx`) Retrieving a glyph's vertical metrics using [glyph_ver_metrics()] method.
- (`kern`) Retrieving a glyphs pair kerning using [glyphs_kerning()] method.
- (`maxp`) Retrieving a total number of glyphs using [number_of_glyphs()] method.
- (`name`) Listing all name records using [names()] method.
- (`name`) Retrieving a font's family name using [family_name()] method.
- (`name`) Retrieving a font's PostScript name using [post_script_name()] method.
- (`post`) Retrieving a font's underline metrics name using [underline_metrics()] method.
- (`head`) Retrieving a font's units per EM value using [units_per_em()] method.
- (`hhea`) Retrieving a generic font info using: [ascender()], [descender()], [height()]
  and [line_gap()] methods.

[glyph_index()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_index
[glyph_variation_index()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_variation_index
[outline_glyph()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.outline_glyph
[glyph_hor_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_hor_metrics
[glyph_ver_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyph_ver_metrics
[glyphs_kerning()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.glyphs_kerning
[number_of_glyphs()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.number_of_glyphs
[names()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.names
[family_name()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.family_name
[post_script_name()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.post_script_name
[underline_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.underline_metrics
[units_per_em()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.units_per_em
[ascender()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.ascender
[descender()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.descender
[height()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.height
[line_gap()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.line_gap

## Supported OpenType features

- (`CFF `) Glyph outlining using [outline_glyph()] method.
- (`OS/2`) Retrieving a font kind using [is_regular()], [is_italic()],
  [is_bold()] and [is_oblique()] methods.
- (`OS/2`) Retrieving a font's weight using [weight()] method.
- (`OS/2`) Retrieving a font's width using [width()] method.
- (`OS/2`) Retrieving a font's X height using [x_height()] method.
- (`OS/2`) Retrieving a font's strikeout metrics using [strikeout_metrics()] method.
- (`OS/2`) Retrieving a font's subscript metrics using [subscript_metrics()] method.
- (`OS/2`) Retrieving a font's superscript metrics using [superscript_metrics()] method.

[is_regular()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.is_regular
[is_italic()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.is_italic
[is_bold()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.is_bold
[is_oblique()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.is_oblique
[weight()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.weight
[width()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.width
[x_height()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.x_height
[strikeout_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.strikeout_metrics
[subscript_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.subscript_metrics
[superscript_metrics()]: https://docs.rs/ttf-parser/0.1.0/ttf_parser/struct.Font.html#method.superscript_metrics

## Methods' computational complexity

TrueType fonts designed for fast querying, so most of the methods are very fast.
The main exception is glyph outlining. Glyphs can be stored using two different methods:
using [Glyph Data](https://docs.microsoft.com/en-us/typography/opentype/spec/glyf) format
and [Compact Font Format](http://wwwimages.adobe.com/content/dam/Adobe/en/devnet/font/pdfs/5176.CFF.pdf) (pdf).
The first one is fairly simple which makes it faster to process.
The second one is basically a tiny language with a stack-based VM, which makes it way harder to process.
Currently, it takes almost 2.5x times longer to outline all glyphs in
*SourceSansPro-Regular.otf* (which uses CFF) rather than in *SourceSansPro-Regular.ttf*.

```text
test outline_cff  ... bench:   2,694,360 ns/iter (+/- 3,437)
test outline_glyf ... bench:   1,155,789 ns/iter (+/- 1,518)
```

Here is some methods benchmarks:

```text
test outline_glyph_276_from_cff  ... bench:       1,823 ns/iter (+/- 20)
test outline_glyph_8_from_cff    ... bench:       1,085 ns/iter (+/- 10)
test outline_glyph_276_from_glyf ... bench:         981 ns/iter (+/- 20)
test family_name                 ... bench:         493 ns/iter (+/- 3)
test outline_glyph_8_from_glyf   ... bench:         422 ns/iter (+/- 32)
test glyph_index_u41             ... bench:          29 ns/iter (+/- 1)
test glyph_2_hor_metrics         ... bench:           8 ns/iter (+/- 0)
test width                       ... bench:           3 ns/iter (+/- 0)
test ascender                    ... bench:           2 ns/iter (+/- 0)
test units_per_em                ... bench:           2 ns/iter (+/- 0)
```

All other methods are essentially free. All they do is read a value at a specified offset.

`family_name` is expensive, because it allocates a `String` and the original data
is stored as UTF-16 BE.

## Safety

- The library must not panic. Any panic considered as a critical bug and should be reported.
- The library forbids `unsafe` code.
*/

#![doc(html_root_url = "https://docs.rs/ttf-parser/0.1.0")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]


use std::ops::Range;

mod cff;
mod cmap;
mod glyf;
mod head;
mod hhea;
mod hmtx;
mod kern;
mod loca;
mod name;
mod os2;
mod parser;
mod post;
mod vhea;
mod vmtx;

use parser::{Stream, FromData, SafeStream, LazyArray, TrySlice};
pub use cff::CFFError;
pub use name::*;
pub use os2::*;


/// A type-safe wrapper for glyph ID.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct GlyphId(pub u16);

impl FromData for GlyphId {
    #[inline]
    fn parse(s: &mut SafeStream) -> Self {
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

    /// Font doesn't have such glyph ID.
    NoGlyph,

    /// Glyph doesn't have an outline.
    NoOutline,

    /// An invalid glyph class.
    InvalidGlyphClass(u16),

    /// An invalid font width.
    InvalidFontWidth(u16),

    /// No horizontal metrics for this glyph.
    NoHorizontalMetrics,

    /// No vertical metrics for this glyph.
    NoVerticalMetrics,

    /// No kerning for this glyph.
    NoKerning,

    /// An unsupported table version.
    UnsupportedTableVersion(TableName, u16),

    /// A CFF table parsing error.
    CFFError(CFFError),

    /// An attempt to slice a raw data out of bounds.
    ///
    /// This may be caused by a bug in the library or by a malformed font.
    #[allow(missing_docs)]
    SliceOutOfBounds {
        // u32 is enough, since fonts are usually times smaller.
        start: u32,
        end: u32,
        data_len: u32,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
            Error::SliceOutOfBounds { start, end, data_len } => {
                write!(f, "an attempt to slice {}..{} on 0..{}", start, end, data_len)
            }
            Error::NoGlyph => {
                write!(f, "font doesn't have such glyph ID")
            }
            Error::NoOutline => {
                write!(f, "glyph has no outline")
            }
            Error::InvalidGlyphClass(n) => {
                write!(f, "{} is not a valid glyph class", n)
            }
            Error::InvalidFontWidth(n) => {
                write!(f, "{} is not a valid font width", n)
            }
            Error::NoHorizontalMetrics => {
                write!(f, "glyph has no horizontal metrics")
            }
            Error::NoVerticalMetrics => {
                write!(f, "glyph has no vertical metrics")
            }
            Error::NoKerning => {
                write!(f, "glyph has no kerning")
            }
            Error::UnsupportedTableVersion(name, version) => {
                write!(f, "table {:?} with version {} is not supported", name, version)
            }
            Error::CFFError(e) => {
                write!(f, "{:?} table parsing failed cause {}", TableName::CompactFontFormat, e)
            }
        }
    }
}

impl From<CFFError> for Error {
    fn from(e: CFFError) -> Self {
        Error::CFFError(e)
    }
}

impl std::error::Error for Error {}

pub(crate) type Result<T> = std::result::Result<T, Error>;


#[derive(Clone, Copy)]
struct Tag {
    tag: [u8; Tag::LENGTH],
}

impl Tag {
    const LENGTH: usize = 4;
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let d = self.tag;
        write!(f, "Tag({}{}{}{})", d[0] as char, d[1] as char, d[2] as char, d[3] as char)
    }
}

impl FromData for Tag {
    #[inline]
    fn parse(s: &mut SafeStream) -> Self {
        let tag = [s.read(), s.read(), s.read(), s.read()];
        Tag { tag }
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

// We cannot implement FromData for LineMetrics, because order of values in
// `post` and OS/2 tables are inverted.


/// A horizontal metrics of a glyph.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct HorizontalMetrics {
    /// A horizontal advance.
    pub advance: u16,

    /// Left side bearing.
    pub left_side_bearing: i16,
}

impl FromData for HorizontalMetrics {
    fn parse(s: &mut SafeStream) -> Self {
        HorizontalMetrics {
            advance: s.read(),
            left_side_bearing: s.read(),
        }
    }
}


/// A vertical metrics of a glyph.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VerticalMetrics {
    /// A vertical advance.
    pub advance: u16,

    /// Top side bearing.
    pub top_side_bearing: i16,
}

impl FromData for VerticalMetrics {
    fn parse(s: &mut SafeStream) -> Self {
        VerticalMetrics {
            advance: s.read(),
            top_side_bearing: s.read(),
        }
    }
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

impl Rect {
    pub(crate) fn zero() -> Self {
        Rect {
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
        }
    }
}

impl FromData for Rect {
    fn parse(s: &mut SafeStream) -> Self {
        Rect {
            x_min: s.read(),
            y_min: s.read(),
            x_max: s.read(),
            y_max: s.read(),
        }
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


/// A table name.
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub enum TableName {
    CharacterToGlyphIndexMapping,
    CompactFontFormat,
    GlyphData,
    Header,
    HorizontalHeader,
    HorizontalMetrics,
    IndexToLocation,
    Kerning,
    MaximumProfile,
    Naming,
    PostScript,
    VerticalHeader,
    VerticalMetrics,
    WindowsMetrics,
}


struct RawTable {
    tag: Tag,
    #[allow(dead_code)]
    checksum: u32,
    offset: u32,
    length: u32,
}

impl RawTable {
    fn range(&self) -> Option<Range<usize>> {
        // 'The length of a table must be a multiple of four bytes.'
        // But Table Record stores an actual table length.
        // So we have to expand it.
        //
        // This is mainly for checksum code.

        // Check for overflow.
        let length = self.length.checked_add(3)?;
        let length = length & !3;

        let start = self.offset as usize;
        let range = start..(start + length as usize);
        Some(range)
    }
}

impl FromData for RawTable {
    fn parse(s: &mut SafeStream) -> Self {
        RawTable {
            tag: s.read(),
            checksum: s.read(),
            offset: s.read(),
            length: s.read(),
        }
    }
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
const OFFSET_TABLE_SIZE: usize = 12;

// https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
const MIN_TTC_SIZE: usize = 12 + OFFSET_TABLE_SIZE;


/// A font data handle.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Font<'a> {
    head: &'a [u8],
    hhea: &'a [u8],
    cff_: Option<&'a [u8]>,
    cmap: Option<&'a [u8]>,
    glyf: Option<&'a [u8]>,
    hmtx: Option<&'a [u8]>,
    kern: Option<&'a [u8]>,
    loca: Option<&'a [u8]>,
    name: Option<&'a [u8]>,
    os_2: Option<&'a [u8]>,
    post: Option<&'a [u8]>,
    vhea: Option<&'a [u8]>,
    vmtx: Option<&'a [u8]>,
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
    /// Required tables: `head`, `hhea` and `maxp`.
    pub fn from_data(data: &'a [u8], index: u32) -> Result<Self> {
        let table_data = if let Some(n) = fonts_in_collection(data) {
            if index < n {
                // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
                const OFFSETS_TABLE_OFFSET: usize = 12;
                const OFFSET_32_SIZE: usize = 4;

                let offset = OFFSETS_TABLE_OFFSET + OFFSET_32_SIZE * index as usize;
                let font_offset: u32 = Stream::read_at(data, offset)?;
                &data[font_offset as usize ..]
            } else {
                return Err(Error::FontIndexOutOfBounds);
            }
        } else {
            data
        };

        if data.len() < OFFSET_TABLE_SIZE {
            return Err(Error::NotATrueType);
        }

        // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
        const SFNT_VERSION_TRUE_TYPE: u32 = 0x00010000;
        const SFNT_VERSION_OPEN_TYPE: u32 = 0x4F54544F;

        let mut s = Stream::new(table_data);

        let sfnt_version: u32 = s.read()?;
        if let SFNT_VERSION_TRUE_TYPE | SFNT_VERSION_OPEN_TYPE = sfnt_version {} else {
            return Err(Error::NotATrueType);
        }

        let num_tables: u16 = s.read()?;
        s.skip::<u16>(); // searchRange
        s.skip::<u16>(); // entrySelector
        s.skip::<u16>(); // rangeShift
        let raw_tables: LazyArray<RawTable> = s.read_array(num_tables)?;

        let mut font = Font {
            head: &[],
            hhea: &[],
            cff_: None,
            cmap: None,
            glyf: None,
            hmtx: None,
            kern: None,
            loca: None,
            name: None,
            os_2: None,
            post: None,
            vhea: None,
            vmtx: None,
            number_of_glyphs: GlyphId(0),
        };

        for table in raw_tables {
            let range = match table.range() {
                Some(range) => range,
                None => continue,
            };

            match &table.tag.tag {
                b"head" => font.head = data.try_slice(range)?,
                b"hhea" => font.hhea = data.try_slice(range)?,
                b"maxp" => {
                    const NUM_GLYPHS_OFFSET: usize = 4;
                    const MAXP_TABLE_MIN_SIZE: usize = 6;

                    let data = data.try_slice(range)?;

                    if data.len() < MAXP_TABLE_MIN_SIZE {
                        return Err(Error::InvalidTableSize(TableName::MaximumProfile));
                    }

                    font.number_of_glyphs = SafeStream::read_at(data, NUM_GLYPHS_OFFSET);
                }
                b"CFF " => font.cff_ = data.get(range),
                b"cmap" => font.cmap = data.get(range),
                b"glyf" => font.glyf = data.get(range),
                b"hmtx" => font.hmtx = data.get(range),
                b"kern" => font.kern = data.get(range),
                b"loca" => font.loca = data.get(range),
                b"name" => font.name = data.get(range),
                b"OS/2" => font.os_2 = data.get(range),
                b"post" => font.post = data.get(range),
                b"vhea" => font.vhea = data.get(range),
                b"vmtx" => font.vmtx = data.get(range),
                _ => {},
            }
        }

        // Check for mandatory tables.

        // All sizes are multiple of four bytes.
        const HEAD_TABLE_SIZE: usize = 56;
        const HHEA_TABLE_SIZE: usize = 36;

        if font.head.len() != HEAD_TABLE_SIZE {
            return Err(Error::InvalidTableSize(TableName::Header));
        }

        if font.hhea.len() != HHEA_TABLE_SIZE {
            return Err(Error::InvalidTableSize(TableName::HorizontalHeader));
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
            TableName::CharacterToGlyphIndexMapping => self.cmap.is_some(),
            TableName::CompactFontFormat            => self.cff_.is_some(),
            TableName::GlyphData                    => self.glyf.is_some(),
            TableName::HorizontalMetrics            => self.hmtx.is_some(),
            TableName::IndexToLocation              => self.loca.is_some(),
            TableName::Kerning                      => self.kern.is_some(),
            TableName::Naming                       => self.name.is_some(),
            TableName::PostScript                   => self.post.is_some(),
            TableName::VerticalHeader               => self.vhea.is_some(),
            TableName::VerticalMetrics              => self.vmtx.is_some(),
            TableName::WindowsMetrics               => self.os_2.is_some(),
        }
    }

    #[inline]
    pub(crate) fn table_data(&self, name: TableName) -> Result<&[u8]> {
        match name {
            TableName::Header                       => Ok(self.head),
            TableName::HorizontalHeader             => Ok(self.hhea),
            TableName::MaximumProfile               => unreachable!(), // We do not store this table.
            TableName::CharacterToGlyphIndexMapping => self.cmap.ok_or_else(|| Error::TableMissing(name)),
            TableName::CompactFontFormat            => self.cff_.ok_or_else(|| Error::TableMissing(name)),
            TableName::GlyphData                    => self.glyf.ok_or_else(|| Error::TableMissing(name)),
            TableName::HorizontalMetrics            => self.hmtx.ok_or_else(|| Error::TableMissing(name)),
            TableName::IndexToLocation              => self.loca.ok_or_else(|| Error::TableMissing(name)),
            TableName::Kerning                      => self.kern.ok_or_else(|| Error::TableMissing(name)),
            TableName::Naming                       => self.name.ok_or_else(|| Error::TableMissing(name)),
            TableName::PostScript                   => self.post.ok_or_else(|| Error::TableMissing(name)),
            TableName::VerticalHeader               => self.vhea.ok_or_else(|| Error::TableMissing(name)),
            TableName::VerticalMetrics              => self.vmtx.ok_or_else(|| Error::TableMissing(name)),
            TableName::WindowsMetrics               => self.os_2.ok_or_else(|| Error::TableMissing(name)),
        }
    }

    #[inline]
    pub(crate) fn table_stream(&self, name: TableName) -> Result<Stream> {
        Ok(Stream::new(self.table_data(name)?))
    }

    /// Returns a total number of glyphs in the font.
    ///
    /// The value was already parsed, so this function doesn't involve any parsing.
    #[inline]
    pub fn number_of_glyphs(&self) -> u16 {
        self.number_of_glyphs.0
    }

    pub(crate) fn check_glyph_id(&self, glyph_id: GlyphId) -> Result<()> {
        if glyph_id < self.number_of_glyphs {
            Ok(())
        } else {
            Err(Error::NoGlyph)
        }
    }

    /// Outlines a glyph. Returns a tight glyph bounding box.
    ///
    /// This method support both `glyf` and `CFF` tables.
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
    /// let glyph = font.outline_glyph(ttf_parser::GlyphId(0), &mut builder).unwrap();
    /// assert_eq!(builder.0, "M 50 0 L 50 750 L 450 750 L 450 0 L 50 0 Z ");
    /// ```
    pub fn outline_glyph(
        &self,
        glyph_id: GlyphId,
        builder: &mut impl OutlineBuilder,
    ) -> Result<Rect> {
        if self.has_table(TableName::GlyphData) {
            self.glyf_glyph_outline(glyph_id, builder)
        } else if self.has_table(TableName::CompactFontFormat) {
            self.cff_glyph_outline(glyph_id, builder)
        } else {
            Err(Error::NoGlyph)
        }
    }
}

/// Checks that provided data is a TrueType font collection.
fn is_collection(data: &[u8]) -> bool {
    data.len() >= Tag::LENGTH && &data[0..Tag::LENGTH] == b"ttcf"
}

/// Returns a number of fonts stored in a TrueType font collection.
///
/// Returns `None` if a provided data is not a TrueType font collection.
pub fn fonts_in_collection(data: &[u8]) -> Option<u32> {
    if data.len() < MIN_TTC_SIZE {
        return None;
    }

    // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
    const NUM_FONTS_OFFSET: usize = 8;

    if !is_collection(data) {
        return None;
    }

    Stream::read_at(data, NUM_FONTS_OFFSET).ok()
}
