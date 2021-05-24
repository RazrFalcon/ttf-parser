//

/*!
A [character to glyph index mapping](https://docs.microsoft.com/en-us/typography/opentype/spec/cmap)
table implementation.

This module provides a low-level alternative to
[`Face::glyph_index`](../struct.Face.html#method.glyph_index) and
[`Face::glyph_variation_index`](../struct.Face.html#method.glyph_variation_index)
methods.
*/

use core::convert::TryFrom;

use crate::{GlyphId, PlatformId};
use crate::parser::{Stream, FromData, LazyArray16, NumFrom};

mod format0;
mod format2;
mod format4;
mod format6;
mod format10;
mod format12;
mod format13;
mod format14;

pub use format14::GlyphVariationResult;


/// An iterator over
/// [character encoding](https://docs.microsoft.com/en-us/typography/opentype/spec/cmap)
/// subtables.
#[derive(Clone, Copy, Default)]
#[allow(missing_debug_implementations)]
pub struct Subtables<'a> {
    data: &'a [u8],
    records: LazyArray16<'a, EncodingRecord>,
    index: u16,
}

impl<'a> Iterator for Subtables<'a> {
    type Item = Subtable<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.records.len() {
            let index = u16::try_from(self.index).ok()?;
            self.index += 1;

            let record = self.records.get(index)?;
            let subtable_data = self.data.get(usize::num_from(record.offset)..)?;
            let format: Format = Stream::read_at(subtable_data, 0)?;
            Some(Subtable {
                platform_id: record.platform_id,
                encoding_id: record.encoding_id,
                format,
                subtable_data,
            })
        } else {
            None
        }
    }

    #[inline]
    fn count(self) -> usize {
        usize::from(self.records.len())
    }
}


/// A character encoding subtable.
pub struct Subtable<'a> {
    platform_id: PlatformId,
    encoding_id: u16,
    format: Format,
    subtable_data: &'a [u8],
}

impl<'a> Subtable<'a> {
    /// Returns encoding's platform.
    #[inline]
    pub fn platform_id(&self) -> PlatformId {
        self.platform_id
    }

    /// Returns encoding ID.
    #[inline]
    pub fn encoding_id(&self) -> u16 {
        self.encoding_id
    }

    /// Returns encoding's format.
    #[inline]
    pub fn format(&self) -> Format {
        self.format
    }

    /// Checks that the current encoding is Unicode compatible.
    #[inline]
    pub fn is_unicode(&self) -> bool {
        // https://docs.microsoft.com/en-us/typography/opentype/spec/name#windows-encoding-ids
        const WINDOWS_UNICODE_BMP_ENCODING_ID: u16 = 1;
        const WINDOWS_UNICODE_FULL_REPERTOIRE_ENCODING_ID: u16 = 10;

        match self.platform_id {
            PlatformId::Unicode => true,
            PlatformId::Windows if self.encoding_id == WINDOWS_UNICODE_BMP_ENCODING_ID => true,
            PlatformId::Windows => {
                // "Note: Subtable format 13 has the same structure as format 12; it differs only
                // in the interpretation of the startGlyphID/glyphID fields".
                let is_format_12_compatible = self.format == Format::SegmentedCoverage ||
                                              self.format == Format::ManyToOneRangeMappings;

                // "Fonts that support Unicode supplementary-plane characters (U+10000 to U+10FFFF)
                // on the Windows platform must have a format 12 subtable for platform ID 3,
                // encoding ID 10."
                self.encoding_id == WINDOWS_UNICODE_FULL_REPERTOIRE_ENCODING_ID
                && is_format_12_compatible
            }
            _ => false,
        }
    }

    /// Maps a character to a glyph ID.
    ///
    /// This is a low-level method and unlike `Face::glyph_index` it doesn't
    /// check that the current encoding is Unicode.
    /// It simply maps a `u32` codepoint number to a glyph ID.
    ///
    /// Returns `None`:
    /// - when glyph ID is `0`.
    /// - when format is `MixedCoverage`, since it's not supported.
    /// - when format is `UnicodeVariationSequences`. Use `glyph_variation_index` instead.
    #[inline]
    pub fn glyph_index(&self, c: u32) -> Option<GlyphId> {
        let glyph = match self.format {
            Format::ByteEncodingTable => {
                format0::parse(self.subtable_data, c)
            }
            Format::HighByteMappingThroughTable => {
                format2::parse(self.subtable_data, c)
            }
            Format::SegmentMappingToDeltaValues => {
                format4::parse(self.subtable_data, c)
            }
            Format::TrimmedTableMapping => {
                format6::parse(self.subtable_data, c)
            }
            Format::MixedCoverage => {
                // Unsupported.
                None
            }
            Format::TrimmedArray => {
                format10::parse(self.subtable_data, c)
            }
            Format::SegmentedCoverage => {
                format12::parse(self.subtable_data, c)
            }
            Format::ManyToOneRangeMappings => {
                format13::parse(self.subtable_data, c)
            }
            Format::UnicodeVariationSequences => {
                // This subtable should be accessed via glyph_variation_index().
                None
            }
        };

        glyph.map(GlyphId)
    }

    /// Resolves a variation of a glyph ID from two code points.
    ///
    /// Returns `None`:
    /// - when glyph ID is `0`.
    /// - when format is not `UnicodeVariationSequences`.
    #[inline]
    pub fn glyph_variation_index(&self, c: char, variation: char) -> Option<GlyphVariationResult> {
        if self.format == Format::UnicodeVariationSequences {
            format14::parse(self.subtable_data, u32::from(c), u32::from(variation))
        } else {
            None
        }
    }

    /// Calls `f` for all codepoints contained in this subtable.
    ///
    /// This is a low-level method and it doesn't check that the current
    /// encoding is Unicode. It simply calls the function `f` for all `u32`
    /// codepoints that are present in this subtable.
    ///
    /// Note that this may list codepoints for which `glyph_index` still returns
    /// `None` because this method finds all codepoints which were _defined_ in
    /// this subtable. The subtable may still map them to glyph ID `0`.
    ///
    /// Returns without doing anything:
    /// - when format is `MixedCoverage`, since it's not supported.
    /// - when format is `UnicodeVariationSequences`, since it's not supported.
    pub fn codepoints<F: FnMut(u32)>(&self, f: F) {
        let _ = match self.format {
            Format::ByteEncodingTable => {
                format0::codepoints(self.subtable_data, f)
            }
            Format::HighByteMappingThroughTable => {
                format2::codepoints(self.subtable_data, f)
            },
            Format::SegmentMappingToDeltaValues => {
                format4::codepoints(self.subtable_data, f)
            },
            Format::TrimmedTableMapping => {
                format6::codepoints(self.subtable_data, f)
            },
            Format::MixedCoverage => {
                // Unsupported
                None
            },
            Format::TrimmedArray => {
                format10::codepoints(self.subtable_data, f)
            },
            Format::SegmentedCoverage => {
                format12::codepoints(self.subtable_data, f)
            }
            Format::ManyToOneRangeMappings => {
                format13::codepoints(self.subtable_data, f)
            },
            Format::UnicodeVariationSequences => {
                // Unsupported
                None
            },
        };
    }
}

impl<'a> core::fmt::Debug for Subtable<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Encoding")
            .field("platform_id", &self.platform_id)
            .field("encoding_id", &self.encoding_id)
            .field("format", &self.format)
            .finish()
    }
}


#[derive(Clone, Copy)]
struct EncodingRecord {
    platform_id: PlatformId,
    encoding_id: u16,
    offset: u32,
}

impl FromData for EncodingRecord {
    const SIZE: usize = 8;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(EncodingRecord {
            platform_id: s.read::<PlatformId>()?,
            encoding_id: s.read::<u16>()?,
            offset: s.read::<u32>()?,
        })
    }
}


/// A character map encoding format.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash)]
#[allow(missing_docs)]
pub enum Format {
    ByteEncodingTable = 0,
    HighByteMappingThroughTable = 2,
    SegmentMappingToDeltaValues = 4,
    TrimmedTableMapping = 6,
    MixedCoverage = 8,
    TrimmedArray = 10,
    SegmentedCoverage = 12,
    ManyToOneRangeMappings = 13,
    UnicodeVariationSequences = 14,
}

impl FromData for Format {
    const SIZE: usize = 2;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        match u16::parse(data)? {
             0 => Some(Format::ByteEncodingTable),
             2 => Some(Format::HighByteMappingThroughTable),
             4 => Some(Format::SegmentMappingToDeltaValues),
             6 => Some(Format::TrimmedTableMapping),
             8 => Some(Format::MixedCoverage),
            10 => Some(Format::TrimmedArray),
            12 => Some(Format::SegmentedCoverage),
            13 => Some(Format::ManyToOneRangeMappings),
            14 => Some(Format::UnicodeVariationSequences),
            _ => None,
        }
    }
}

pub(crate) fn parse(data: &[u8]) -> Option<Subtables> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // version
    let count: u16 = s.read()?;
    let records = s.read_array16::<EncodingRecord>(count)?;

    Some(Subtables {
        data,
        records,
        index: 0,
    })
}
