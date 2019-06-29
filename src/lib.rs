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

## Limitations

- Non [ARGS_ARE_XY_VALUES] transform is not supported yet.
- `cmap` table formats 6, 8 and 10 are not supported.

[ARGS_ARE_XY_VALUES]: https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#composite-glyph-description

## Safety

- The library heavily relies on Rust's bounds checking and assumes that font is well-formed.
  You can invoke a checksums checking manually.
- The library uses per table slices, so it can't read data outside the specified TrueType table.
- The library forbids `unsafe` code.
*/

#![doc(html_root_url = "https://docs.rs/ttf-parser/0.1.0")]

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]


macro_rules! impl_bit_ops {
    ($name:ty) => {
        impl std::ops::BitOr for $name {
            type Output = Self;

            #[inline]
            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl std::ops::BitAnd for $name {
            type Output = Self;

            #[inline]
            fn bitand(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }
        }
    }
}

use std::convert::TryFrom;

mod cff;
mod cmap;
mod gdef;
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

use parser::{Stream, FromData, LazyArray};
pub use cff::CFFError;
pub use gdef::*;
pub use glyf::*;
pub use name::*;
pub use os2::*;


/// A type-safe wrapper for glyph ID.
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct GlyphId(pub u16);

impl FromData for GlyphId {
    #[inline]
    fn parse(s: &mut Stream) -> Self {
        GlyphId(s.read())
    }
}


/// A font parsing error.
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Not a TrueType data.
    NotATrueType,

    /// The font index is out of bounds.
    IndexOutOfBounds,

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

    /// An unsupported table version.
    UnsupportedTableVersion(TableName, u8),

    /// A CFF table parsing error.
    CFFError(CFFError)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::NotATrueType => {
                write!(f, "not a TrueType font")
            }
            Error::IndexOutOfBounds => {
                write!(f, "font index is out of bounds")
            }
            Error::TableMissing(name) => {
                write!(f, "font doesn't have a {:?} table", name)
            }
            Error::InvalidTableChecksum(name) => {
                write!(f, "table {:?} has an invalid checksum", name)
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
    fn parse(s: &mut Stream) -> Self {
        let data = s.tail();
        let tag = [data[0], data[1], data[2], data[3]];
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


/// A vertical metrics of a glyph.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct VerticalMetrics {
    /// A vertical advance.
    pub advance: u16,

    /// Top side bearing.
    pub top_side_bearing: i16,
}

impl FromData for VerticalMetrics {
    fn parse(s: &mut Stream) -> Self {
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
    GlyphDefinition                 = Tag::make_u32(b"GDEF"),
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
            b"GDEF" => Ok(TableName::GlyphDefinition),
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


struct RawTable {
    tag: Tag,
    checksum: u32,
    offset: u32,
    length: u32,
}

impl RawTable {
    fn range(&self) -> std::ops::Range<usize> {
        // 'The length of a table must be a multiple of four bytes.'
        // But Table Record stores an actual table length.
        // So we have to expand it.
        //
        // This is mainly for checksum code.
        let length = (self.length + 3) & !3;

        self.offset as usize .. (self.offset as usize + length as usize)
    }
}

impl FromData for RawTable {
    fn parse(s: &mut Stream) -> Self {
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


/// A font data handle.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Font<'a> {
    tables: [TableInfo<'a>; 12],
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
                let font_offset: u32 = Stream::read_at(data, offset);
                &data[font_offset as usize ..]
            } else {
                return Err(Error::IndexOutOfBounds);
            }
        } else {
            data
        };

        // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#organization-of-an-opentype-font
        const SFNT_VERSION_TRUE_TYPE: u32 = 0x00010000;
        const SFNT_VERSION_OPEN_TYPE: u32 = 0x4F54544F;

        let mut s = Stream::new(table_data);

        let sfnt_version: u32 = s.read();
        if let SFNT_VERSION_TRUE_TYPE | SFNT_VERSION_OPEN_TYPE = sfnt_version {} else {
            return Err(Error::NotATrueType);
        }

        let num_tables: u16 = s.read();
        s.skip::<u16>(); // searchRange
        s.skip::<u16>(); // entrySelector
        s.skip::<u16>(); // rangeShift

        let mut tables = [TableInfo {
            name: TableName::MaximumProfile, // dummy
            checksum: 0,
            data: b"",
        }; 12];

        let mut number_of_glyphs = GlyphId(0);

        let raw_tables: LazyArray<RawTable> = s.read_array(num_tables);

        let mut i = 0;
        for table in raw_tables {
            let name = match TableName::try_from(table.tag) {
                Ok(v) => v,
                Err(_) => continue,
            };

            tables[i] = TableInfo {
                name,
                checksum: table.checksum,
                data: &data[table.range()],
            };

            if name == TableName::MaximumProfile {
                number_of_glyphs = Self::parse_number_of_glyphs(&data[table.range()]);
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

    /// Returns a total number of glyphs in the font.
    ///
    /// The value was already parsed, so this function doesn't involve any parsing.
    #[inline]
    pub fn number_of_glyphs(&self) -> u16 {
        self.number_of_glyphs.0
    }

    /// Parses a total number of glyphs in the font.
    fn parse_number_of_glyphs(data: &[u8]) -> GlyphId {
        const NUM_GLYPHS_OFFSET: usize = 4;
        GlyphId(Stream::read_at(data, NUM_GLYPHS_OFFSET))
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

            let sum = calc_checksum(table.data);
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

fn calc_checksum(data: &[u8]) -> u32 {
    // TODO: speed up

    // 'Table checksums are the unsigned sum of the uint32 units of a given table.'
    let mut sum: u32 = 0;
    let numbers: LazyArray<u32> = Stream::new(data).read_array(data.len() as u32 / 4);
    for n in numbers {
        sum = sum.wrapping_add(n);
    }

    sum
}

/// Checks that provided data is a TrueType font collection.
fn is_collection(data: &[u8]) -> bool {
    data.len() >= Tag::LENGTH && &data[0..Tag::LENGTH] == b"ttcf"
}

/// Returns a number of fonts stored in a TrueType font collection.
///
/// Returns `None` if a provided data is not a TrueType font collection.
pub fn fonts_in_collection(data: &[u8]) -> Option<u32> {
    // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
    const NUM_FONTS_OFFSET: usize = 8;

    if !is_collection(data) {
        return None;
    }

    Some(Stream::read_at(data, NUM_FONTS_OFFSET))
}
