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
test outline_cff  ... bench:   2,528,662 ns/iter (+/- 2,362)
test outline_glyf ... bench:   1,171,966 ns/iter (+/- 1,642)
```

Here is some methods benchmarks:

```text
test outline_glyph_276_from_cff  ... bench:       1,649 ns/iter (+/- 3)
test outline_glyph_8_from_cff    ... bench:         965 ns/iter (+/- 1)
test outline_glyph_276_from_glyf ... bench:         950 ns/iter (+/- 7)
test family_name                 ... bench:         429 ns/iter (+/- 7)
test outline_glyph_8_from_glyf   ... bench:         394 ns/iter (+/- 11)
test glyph_index_u41             ... bench:          33 ns/iter (+/- 1)
test width                       ... bench:          24 ns/iter (+/- 1)
test glyph_2_hor_metrics         ... bench:          18 ns/iter (+/- 0)
test units_per_em                ... bench:           6 ns/iter (+/- 0)
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


use std::convert::TryFrom;
use std::ops::Range;

mod cff;
mod cmap;
mod glyf;
mod head;
mod hhea;
mod hmtx;
mod loca;
mod name;
mod os2;
mod parser;
mod post;
mod vhea;
mod vmtx;

use parser::{Stream, FromData, SafeStream, LazyArray};
pub use cff::CFFError;
pub use glyf::*;
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

    /// An invalid table checksum.
    InvalidTableChecksum(TableName),

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

    /// An unsupported table version.
    UnsupportedTableVersion(TableName, u8),

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
        origin_len: u32,
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
            Error::InvalidTableChecksum(name) => {
                write!(f, "table {:?} has an invalid checksum", name)
            }
            Error::SliceOutOfBounds { start, end, origin_len } => {
                write!(f, "an attempt to slice {}..{} on 0..{}", start, end, origin_len)
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


/// A TrueType's `Tag` data type.
#[derive(Clone, Copy, PartialEq)]
pub struct Tag {
    tag: [u8; Tag::LENGTH],
}

impl Tag {
    const LENGTH: usize = 4;

    /// Creates a `Tag` object from bytes.
    #[inline]
    pub const fn new(c1: u8, c2: u8, c3: u8, c4: u8) -> Self {
        Tag { tag: [c1, c2, c3, c4] }
    }

    /// Creates a `Tag` object from a slice.
    ///
    /// Will panic if data length != 4.
    pub fn from_slice(data: &[u8]) -> Self {
        assert_eq!(data.len(), Tag::LENGTH);
        Tag { tag: [data[0], data[1], data[2], data[3]] }
    }

    const fn make_u32(data: &[u8]) -> u32 {
        (data[0] as u32) << 24 | (data[1] as u32) << 16 | (data[2] as u32) << 8 | data[3] as u32
    }

    fn to_ascii(self) -> [char; Tag::LENGTH] {
        let mut tag2 = [' '; Tag::LENGTH];
        for i in 0..Tag::LENGTH {
            if self.tag[i].is_ascii() {
                tag2[i] = self.tag[i] as char;
            }
        }

        tag2
    }
}

impl std::ops::Deref for Tag {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.tag
    }
}

impl std::fmt::Debug for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let tag = self.to_ascii();
        write!(f, "Tag({}{}{}{})", tag[0], tag[1], tag[2], tag[3])
    }
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let tag = self.to_ascii();
        write!(f, "{}{}{}{}", tag[0], tag[1], tag[2], tag[3])
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
#[repr(u32)]
pub enum TableName {
    CharacterToGlyphIndexMapping    = Tag::make_u32(b"cmap"),
    CompactFontFormat               = Tag::make_u32(b"CFF "),
    GlyphData                       = Tag::make_u32(b"glyf"),
    Header                          = Tag::make_u32(b"head"),
    HorizontalHeader                = Tag::make_u32(b"hhea"),
    HorizontalMetrics               = Tag::make_u32(b"hmtx"),
    IndexToLocation                 = Tag::make_u32(b"loca"),
    MaximumProfile                  = Tag::make_u32(b"maxp"),
    Naming                          = Tag::make_u32(b"name"),
    PostScript                      = Tag::make_u32(b"post"),
    VerticalHeader                  = Tag::make_u32(b"vhea"),
    VerticalMetrics                 = Tag::make_u32(b"vmtx"),
    WindowsMetrics                  = Tag::make_u32(b"OS/2"),
}

impl TryFrom<Tag> for TableName {
    type Error = ();

    fn try_from(value: Tag) -> std::result::Result<Self, Self::Error> {
        // TODO: Rust doesn't support `const fn` in patterns yet
        match &*value {
            b"CFF " => Ok(TableName::CompactFontFormat),
            b"cmap" => Ok(TableName::CharacterToGlyphIndexMapping),
            b"glyf" => Ok(TableName::GlyphData),
            b"head" => Ok(TableName::Header),
            b"head" => Ok(TableName::HorizontalMetrics),
            b"hhea" => Ok(TableName::HorizontalHeader),
            b"hmtx" => Ok(TableName::HorizontalMetrics),
            b"loca" => Ok(TableName::IndexToLocation),
            b"maxp" => Ok(TableName::MaximumProfile),
            b"name" => Ok(TableName::Naming),
            b"OS/2" => Ok(TableName::WindowsMetrics),
            b"post" => Ok(TableName::PostScript),
            b"vhea" => Ok(TableName::VerticalHeader),
            b"vmtx" => Ok(TableName::VerticalMetrics),
            _ => Err(()),
        }
    }
}

const MAX_NUMBER_OF_TABLES: usize = 12;


struct RawTable {
    tag: Tag,
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


#[derive(Clone, Copy)]
struct TableInfo<'a> {
    name: TableName,
    checksum: u32,
    data: &'a [u8],
}


// https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
const OFFSET_TABLE_SIZE: usize = 12;

// https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
const MIN_TTC_SIZE: usize = 12 + OFFSET_TABLE_SIZE;


/// A font data handle.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Font<'a> {
    tables: [TableInfo<'a>; MAX_NUMBER_OF_TABLES],
    number_of_glyphs: GlyphId,
}

impl<'a> Font<'a> {
    /// Creates a `Font` object from raw data.
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

        let mut tables = [TableInfo {
            name: TableName::MaximumProfile, // dummy
            checksum: 0,
            data: b"",
        }; MAX_NUMBER_OF_TABLES];

        let mut number_of_glyphs = GlyphId(0);

        let raw_tables: LazyArray<RawTable> = s.read_array(num_tables)?;

        let mut i = 0;
        for table in raw_tables {
            let name = match TableName::try_from(table.tag) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Check for duplicates.
            if tables.iter().take(i).any(|t| t.name == name) {
                continue;
            }

            let range = match table.range() {
                Some(range) => range,
                None => continue,
            };

            let data = match data.get(range) {
                Some(data) => data,
                None => continue,
            };

            tables[i] = TableInfo {
                name,
                checksum: table.checksum,
                data,
            };

            if name == TableName::MaximumProfile {
                number_of_glyphs = Self::parse_number_of_glyphs(data)?;
            }

            i += 1;
        }

        let font = Font {
            tables,
            number_of_glyphs,
        };

        // Check for mandatory tables.
        font.table_data(TableName::Header)?;
        font.table_data(TableName::HorizontalHeader)?;
        font.table_data(TableName::MaximumProfile)?;

        Ok(font)
    }

    /// Checks that font has a specified table.
    pub fn has_table(&self, name: TableName) -> bool {
        self.tables.iter().any(|t| t.name == name)
    }

    #[inline(never)]
    pub(crate) fn table_data(&self, name: TableName) -> Result<&[u8]> {
        let info = self.tables
            .iter()
            .find(|t| t.name == name)
            .ok_or_else(|| Error::TableMissing(name))?;

        Ok(info.data)
    }

    #[inline(never)]
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

    /// Parses a total number of glyphs in the font.
    fn parse_number_of_glyphs(data: &[u8]) -> Result<GlyphId> {
        const NUM_GLYPHS_OFFSET: usize = 4;
        Ok(GlyphId(Stream::read_at(data, NUM_GLYPHS_OFFSET)?))
    }

    pub(crate) fn check_glyph_id(&self, glyph_id: GlyphId) -> Result<()> {
        if glyph_id < self.number_of_glyphs {
            Ok(())
        } else {
            Err(Error::NoGlyph)
        }
    }

    /// Checks tables [checksum](https://docs.microsoft.com/en-us/typography/opentype/spec/otff#calculating-checksums).
    ///
    /// Checks only used tables.
    pub fn is_valid(&self) -> Result<()> {
        for table in &self.tables {
            if table.name == TableName::Header {
                // We are ignoring the `head` table, because to calculate it's checksum
                // we have to modify an original data. And we can't, since it's read-only.
                // TODO: write a custom `calc_checksum` for `head`.
                continue;
            }

            let sum = calc_checksum(table.data)?;
            if sum != table.checksum {
                return Err(Error::InvalidTableChecksum(table.name));
            }
        }

        Ok(())
    }

    /// Outlines a glyph.
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
    ) -> Result<()> {
        if self.has_table(TableName::GlyphData) {
            self.glyf_glyph_outline(glyph_id, builder)
        } else if self.has_table(TableName::CompactFontFormat) {
            self.cff_glyph_outline(glyph_id, builder)
        } else {
            Err(Error::NoGlyph)
        }
    }
}

fn calc_checksum(data: &[u8]) -> Result<u32> {
    // TODO: speed up

    // 'Table checksums are the unsigned sum of the uint32 units of a given table.'
    let mut sum: u32 = 0;
    let numbers: LazyArray<u32> = Stream::new(data).read_array(data.len() as u32 / 4)?;
    for n in numbers {
        sum = sum.wrapping_add(n);
    }

    Ok(sum)
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
