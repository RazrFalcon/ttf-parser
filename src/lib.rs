/*!
A high-level, safe, zero-allocation TrueType font parser.

## Goals

- A high-level API.
- Zero allocations.
- Zero `unsafe`.
- Fast.
- Simple and maintainable code (no magic numbers).
- Minimal dependencies (currently, depends only on `bitflags`).

## Limitations

- Non [ARGS_ARE_XY_VALUES] transform is not supported yet.
- Only 0, 4, 12 and 13 formats of `cmap` table are supported.

[ARGS_ARE_XY_VALUES]: https://docs.microsoft.com/en-us/typography/opentype/spec/glyf#composite-glyph-description

## Safety

- The library heavily relies on Rust's bounds checking and assumes that font is well-formed.
  You can invoke a checksums checking manually.
- The library uses per table slices, so it can't read data outside the specified TrueType table.
- The library forbids `unsafe` code.
*/

#![doc(html_root_url = "https://docs.rs/ttf-parser/0.1.0")]

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

pub mod cmap;
pub mod gdef;
pub mod glyf;
pub mod hmtx;
pub mod loca;
pub mod name;
pub mod os2;
pub mod post;
mod head;
mod hhea;
mod stream;

type Range32 = std::ops::Range<u32>;

use stream::Stream;

/// Rectangle.
#[derive(Clone, Copy, PartialEq, Debug)]
#[allow(missing_docs)]
pub struct Rect {
    pub x_min: i16,
    pub y_min: i16,
    pub x_max: i16,
    pub y_max: i16,
}


/// A font parsing error.
#[derive(Clone, Copy, Debug)]
pub enum Error {
    /// Not a TrueType data.
    NotATrueType,

    /// The font index is out of bounds.
    IndexOutOfBounds,

    /// One of the required tables is missing.
    TableMissing(Tag),

    /// An invalid table checksum.
    InvalidTableChecksum(Tag),
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
            Error::TableMissing(tag) => {
                write!(f, "font doesn't have a {} table", tag)
            }
            Error::InvalidTableChecksum(tag) => {
                write!(f, "table {} has an invalid checksum", tag)
            }
        }
    }
}

impl std::error::Error for Error {}


/// A TrueType's `Tag` data type.
#[derive(Clone, Copy, PartialEq)]
pub struct Tag {
    tag: [u8; 4],
}

impl Tag {
    /// Creates a `Tag` object from bytes.
    pub fn new(c1: u8, c2: u8, c3: u8, c4: u8) -> Self {
        Tag { tag: [c1, c2, c3, c4] }
    }

    /// Creates a `Tag` object from a slice.
    ///
    /// Will panic if data length != 4.
    pub fn from_slice(data: &[u8]) -> Self {
        assert_eq!(data.len(), 4);
        Tag { tag: [data[0], data[1], data[2], data[3]] }
    }

    fn to_ascii(&self) -> [char; 4] {
        let mut tag2 = [' '; 4];
        for i in 0..4 {
            if self.tag[i].is_ascii() {
                tag2[i] = self.tag[i] as char;
            }
        }

        tag2
    }

    fn zero() -> Self {
        Tag { tag: [0; 4] }
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


/// A line metrics.
///
/// Used for underline and strikeout.
#[derive(Clone, Copy, Debug)]
pub struct LineMetrics {
    /// Line position.
    pub position: i16,

    /// Line thickness.
    pub thickness: i16,
}


#[derive(Clone, Copy, Debug)]
struct TableInfo {
    tag: Tag,
    checksum: u32,
    offset: u32,
    length: u32,
}

impl TableInfo {
    fn range(&self) -> std::ops::Range<usize> {
        self.offset as usize .. (self.offset as usize + self.length as usize)
    }
}


/// A font data handle.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Font<'a> {
    data: &'a [u8],
    cmap: TableInfo,
    gdef: Option<TableInfo>,
    glyf: TableInfo,
    head: TableInfo,
    hhea: TableInfo,
    hmtx: TableInfo,
    loca: TableInfo,
    name: Option<TableInfo>,
    os_2: Option<TableInfo>,
    post: TableInfo,
    number_of_glyphs: u16,
}

impl<'a> Font<'a> {
    /// Checks that provided data is a TrueType font collection.
    pub fn is_collection(data: &'a [u8]) -> bool {
        data.len() >= 4 && &data[0..4] == b"ttcf"
    }

    /// Returns number of fonts stored in a TrueType font collection.
    ///
    /// Returns `Note` if a provided data is not a TrueType font collection.
    pub fn fonts_number(data: &'a [u8]) -> Option<u32> {
        // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
        const NUM_FONTS_OFFSET: usize = 8;

        if !Self::is_collection(data) {
            return None;
        }

        Some(Stream::read_at(data, NUM_FONTS_OFFSET))
    }

    /// Creates a `Font` object from raw data.
    ///
    /// You can set `index` in case of font collections.
    /// For simple `ttf` fonts set `index` to 0.
    ///
    /// This function only parses font tables, so it's relatively light.
    ///
    /// Required tables: `cmap`, `glyp`, `head`, `hhea`, `hmtx`, `loca`, `post`.
    ///
    /// Optional tables: `GDEF`, `name`, `OS/2`
    pub fn from_data(data: &'a [u8], index: u32) -> Result<Self, Error> {
        let table_data = if let Some(n) = Self::fonts_number(data) {
            if index < n {
                // // https://docs.microsoft.com/en-us/typography/opentype/spec/otff#ttc-header
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

        let mut s = stream::Stream::new(table_data);

        let sfnt_version = s.read_u32();
        if sfnt_version != SFNT_VERSION_TRUE_TYPE {
            return Err(Error::NotATrueType);
        }

        let num_tables = s.read_u16();
        s.skip_u16(); // searchRange
        s.skip_u16(); // entrySelector
        s.skip_u16(); // rangeShift

        let maxp = Self::find_table(s.tail(), num_tables, b"maxp")?;
        let number_of_glyphs = Self::parse_number_of_glyphs(&data[maxp.range()]);

        Ok(Font {
            data,
            cmap: Self::find_table(s.tail(), num_tables, b"cmap")?,
            gdef: Self::find_table(s.tail(), num_tables, b"GDEF").ok(),
            glyf: Self::find_table(s.tail(), num_tables, b"glyf")?,
            head: Self::find_table(s.tail(), num_tables, b"head")?,
            hhea: Self::find_table(s.tail(), num_tables, b"hhea")?,
            hmtx: Self::find_table(s.tail(), num_tables, b"hmtx")?,
            loca: Self::find_table(s.tail(), num_tables, b"loca")?,
            name: Self::find_table(s.tail(), num_tables, b"name").ok(),
            os_2: Self::find_table(s.tail(), num_tables, b"OS/2").ok(),
            post: Self::find_table(s.tail(), num_tables, b"post")?,
            number_of_glyphs,
        })
    }

    fn find_table(data: &[u8], num_tables: u16, name: &[u8]) -> Result<TableInfo, Error> {
        // TODO: do not parse each time

        let mut s = Stream::new(data);
        for _ in 0..num_tables {
            let tag = s.read_tag();
            let table = TableInfo {
                tag,
                checksum: s.read_u32(),
                offset: s.read_u32(),
                length: s.read_u32(),
            };

            if &*tag == name {
                return Ok(table);
            }
        }

        Err(Error::TableMissing(Tag::from_slice(name)))
    }

    /// Returns a total number of glyphs in the font.
    ///
    /// The value was already parsed, so this function doesn't involve any parsing.
    #[inline]
    pub fn number_of_glyphs(&self) -> u16 {
        self.number_of_glyphs
    }

    /// Parses a total number of glyphs in the font.
    fn parse_number_of_glyphs(data: &[u8]) -> u16 {
        const NUM_GLYPHS_OFFSET: usize = 4;
        Stream::read_at(data, NUM_GLYPHS_OFFSET)
    }

    /// Checks tables [checksum](https://docs.microsoft.com/en-us/typography/opentype/spec/otff#calculating-checksums).
    ///
    /// Checks only used tables.
    pub fn is_valid(&self) -> Result<(), Error> {
        // We are ignoring the `head` table, because to calculate it's checksum
        // we have to modify it. And we can't, since input data is read-only.
        // TODO: write a custom `calc_checksum` for `head`.

        self.calc_table_checksum(&self.cmap)?;
        self.calc_table_checksum(&self.glyf)?;
        self.calc_table_checksum(&self.hhea)?;
        self.calc_table_checksum(&self.hmtx)?;
        self.calc_table_checksum(&self.loca)?;
        self.calc_table_checksum(&self.post)?;

        if let Some(ref gdef) = self.gdef { self.calc_table_checksum(gdef)?; }
        if let Some(ref name) = self.name { self.calc_table_checksum(name)?; }
        if let Some(ref os_2) = self.os_2 { self.calc_table_checksum(os_2)?; }

        Ok(())
    }

    fn calc_table_checksum(&self, table: &TableInfo) -> Result<(), Error> {
        let sum = calc_checksum(&self.data[table.offset as usize..], table.length);
        if sum == table.checksum {
            Ok(())
        } else {
            Err(Error::InvalidTableChecksum(table.tag))
        }
    }

    /// Returns a handle to a OS/2 table.
    pub fn os2_table(&self) -> Option<os2::Table> {
        Some(os2::Table { data: &self.data[self.os_2?.range()] })
    }

    /// Returns a handle to a `name` table.
    pub fn name_table(&self) -> Option<name::Table> {
        Some(name::Table { data: &self.data[self.name?.range()] })
    }

    /// Returns a handle to a `GDEF` table.
    pub fn gdef_table(&self) -> Option<gdef::Table> {
        Some(gdef::Table {
            data: &self.data[self.name?.range()],
            number_of_glyphs: self.number_of_glyphs,
        })
    }
}

fn calc_checksum(data: &[u8], length: u32) -> u32 {
    // TODO: speed up

    // 'This function implies that the length of a table must be a multiple of four bytes.'
    let length = (length + 3) & !3;

    // 'Table checksums are the unsigned sum of the uint32 units of a given table.'
    let mut sum: u32 = 0;
    for n in Stream::new(data).read_array::<u32>(length as usize / 4) {
        sum = sum.wrapping_add(n);
    }

    sum
}
