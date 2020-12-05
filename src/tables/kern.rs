/*!
A kerning table implementation.

Supports both
[OpenType](https://docs.microsoft.com/en-us/typography/opentype/spec/kern)
and
[Apple Advanced Typography](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kern.html)
variants.

Since there is no single correct way to process a kerning data,
we have to provide an access to kerning subtables, so a caller can implement
a kerning algorithm manually.
But we still try to keep the API as high-level as possible.
*/

use crate::GlyphId;
use crate::parser::{Stream, FromData, NumFrom, Offset16, Offset};


#[derive(Clone, Copy, Debug)]
struct OTCoverage(u8);

impl OTCoverage {
    #[inline]
    fn is_horizontal(self) -> bool {
        self.0 & (1 << 0) != 0
    }

    #[inline]
    fn has_cross_stream(self) -> bool {
        self.0 & (1 << 2) != 0
    }
}

impl FromData for OTCoverage {
    const SIZE: usize = 1;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        data.get(0).copied().map(OTCoverage)
    }
}


#[derive(Clone, Copy, Debug)]
struct AATCoverage(u8);

impl AATCoverage {
    #[inline]
    fn is_horizontal(self) -> bool {
        self.0 & (1 << 7) == 0
    }

    #[inline]
    fn has_cross_stream(self) -> bool {
        self.0 & (1 << 6) != 0
    }

    #[inline]
    fn is_variable(self) -> bool {
        self.0 & (1 << 5) != 0
    }
}

impl FromData for AATCoverage {
    const SIZE: usize = 1;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        data.get(0).copied().map(AATCoverage)
    }
}


#[derive(Clone, Copy)]
struct KerningRecord {
    // In the kern table spec, a kerning pair is stored as two u16,
    // but we are using one u32, so we can binary search it directly.
    pair: u32,
    value: i16,
}

impl FromData for KerningRecord {
    const SIZE: usize = 6;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(KerningRecord {
            pair: s.read::<u32>()?,
            value: s.read::<i16>()?,
        })
    }
}


/// A kerning subtable.
#[derive(Clone, Copy, Default)]
pub struct Subtable<'a> {
    is_horizontal: bool,
    is_variable: bool,
    has_cross_stream: bool,
    format: u8,
    header_size: u8,
    data: &'a [u8],
}

impl<'a> Subtable<'a> {
    // Use getters so we can change flags storage type later.

    /// Checks that subtable is for horizontal text.
    #[inline]
    pub fn is_horizontal(&self) -> bool {
        self.is_horizontal
    }

    /// Checks that subtable is variable.
    #[inline]
    pub fn is_variable(&self) -> bool {
        self.is_variable
    }

    /// Checks that subtable has a cross-stream values.
    #[inline]
    pub fn has_cross_stream(&self) -> bool {
        self.has_cross_stream
    }

    /// Checks that subtable uses a state machine.
    ///
    /// In this case `glyphs_kerning()` will return `None`.
    #[inline]
    pub fn has_state_machine(&self) -> bool {
        self.format == 1
    }

    /// Returns kerning for a pair of glyphs.
    ///
    /// Returns `None` in case of state machine based subtable.
    #[inline]
    pub fn glyphs_kerning(&self, left: GlyphId, right: GlyphId) -> Option<i16> {
        match self.format {
            0 => parse_format0(self.data, left, right),
            2 => parse_format2(left, right, self.header_size, self.data),
            3 => parse_format3(self.data, left, right),
            _ => None,
        }
    }
}

impl core::fmt::Debug for Subtable<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        // TODO: finish_non_exhaustive
        f.debug_struct("Subtable")
            .field("is_horizontal", &self.is_horizontal())
            .field("has_state_machine", &self.has_state_machine())
            .field("has_cross_stream", &self.has_cross_stream())
            .field("format", &self.format)
            .finish()
    }
}


/// An iterator over kerning subtables.
#[allow(missing_debug_implementations)]
#[derive(Clone, Copy, Default)]
pub struct Subtables<'a> {
    /// Indicates an Apple Advanced Typography format.
    is_aat: bool,
    /// The current table index,
    table_index: u32,
    /// The total number of tables.
    number_of_tables: u32,
    /// Actual data. Starts right after `kern` header.
    stream: Stream<'a>,
}

impl<'a> Iterator for Subtables<'a> {
    type Item = Subtable<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.table_index == self.number_of_tables {
            return None;
        }

        if self.stream.at_end() {
            return None;
        }

        if self.is_aat {
            const HEADER_SIZE: u8 = 8;

            let table_len: u32 = self.stream.read()?;
            let coverage: AATCoverage = self.stream.read()?;
            let format: u8 = self.stream.read()?;
            self.stream.skip::<u16>(); // variation tuple index

            if format > 3 {
                // Unknown format.
                return None;
            }

            // Subtract the header size.
            let data_len = usize::num_from(table_len).checked_sub(usize::from(HEADER_SIZE))?;

            Some(Subtable {
                is_horizontal: coverage.is_horizontal(),
                is_variable: coverage.is_variable(),
                has_cross_stream: coverage.has_cross_stream(),
                format,
                header_size: HEADER_SIZE,
                data: self.stream.read_bytes(data_len)?,
            })
        } else {
            const HEADER_SIZE: u8 = 6;

            self.stream.skip::<u16>(); // version
            let table_len: u16 = self.stream.read()?;
            // In the OpenType variant, `format` comes first.
            let format: u8 = self.stream.read()?;
            let coverage: OTCoverage = self.stream.read()?;

            if format != 0 && format != 2 {
                // Unknown format.
                return None;
            }

            let data_len = if self.number_of_tables == 1 {
                // An OpenType `kern` table with just one subtable is a special case.
                // The `table_len` property is mainly required to jump to the next subtable,
                // but if there is only one subtable, this property can be ignored.
                // This is abused by some fonts, to get around the `u16` size limit.
                self.stream.tail()?.len()
            } else {
                // Subtract the header size.
                usize::from(table_len).checked_sub(usize::from(HEADER_SIZE))?
            };

            Some(Subtable {
                is_horizontal: coverage.is_horizontal(),
                is_variable: false, // Only AAT supports it.
                has_cross_stream: coverage.has_cross_stream(),
                format,
                header_size: HEADER_SIZE,
                data: self.stream.read_bytes(data_len)?,
            })
        }
    }
}

pub(crate) fn parse(data: &[u8]) -> Option<Subtables> {
    // The `kern` table has two variants: OpenType one and Apple one.
    // And they both have different headers.
    // There are no robust way to distinguish them, so we have to guess.
    //
    // The OpenType one has the first two bytes (UInt16) as a version set to 0.
    // While Apple one has the first four bytes (Fixed) set to 1.0
    // So the first two bytes in case of an OpenType format will be 0x0000
    // and 0x0001 in case of an Apple format.
    let mut s = Stream::new(data);
    let version: u16 = s.read()?;
    if version == 0 {
        let number_of_tables: u16 = s.read()?;
        Some(Subtables {
            is_aat: false,
            table_index: 0,
            number_of_tables: u32::from(number_of_tables),
            stream: s,
        })
    } else {
        s.skip::<u16>(); // Skip the second part of u32 version.
        // Note that AAT stores the number of tables as u32 and not as u16.
        let number_of_tables: u32 = s.read()?;
        Some(Subtables {
            is_aat: true,
            table_index: 0,
            number_of_tables: u32::from(number_of_tables),
            stream: s,
        })
    }
}

/// A *Format 0 Kerning Subtable (Ordered List of Kerning Pairs)* implementation
/// from https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kern.html
fn parse_format0(data: &[u8], left: GlyphId, right: GlyphId) -> Option<i16> {
    let mut s = Stream::new(data);
    let number_of_pairs: u16 = s.read()?;
    s.advance(6); // search_range (u16) + entry_selector (u16) + range_shift (u16)
    let pairs = s.read_array16::<KerningRecord>(number_of_pairs)?;

    let needle = u32::from(left.0) << 16 | u32::from(right.0);
    pairs.binary_search_by(|v| v.pair.cmp(&needle)).map(|(_, v)| v.value)
}

/// A *Format 2 Kerning Table (Simple n x m Array of Kerning Values)* implementation
/// from https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kern.html
fn parse_format2(left: GlyphId, right: GlyphId, header_len: u8, data: &[u8]) -> Option<i16> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // row_width

    // Offsets are from beginning of the subtable and not from the `data` start,
    // so we have to subtract the header.
    let header_len = usize::from(header_len);
    let left_hand_table_offset = s.read::<Offset16>()?.to_usize().checked_sub(header_len)?;
    let right_hand_table_offset = s.read::<Offset16>()?.to_usize().checked_sub(header_len)?;
    let array_offset = s.read::<Offset16>()?.to_usize().checked_sub(header_len)?;

    // 'The array can be indexed by completing the left-hand and right-hand class mappings,
    // adding the class values to the address of the subtable,
    // and fetching the kerning value to which the new address points.'

    let left_class = get_format2_class(left.0, left_hand_table_offset, data).unwrap_or(0);
    let right_class = get_format2_class(right.0, right_hand_table_offset, data).unwrap_or(0);

    // 'Values within the left-hand offset table should not be less than the kerning array offset.'
    if usize::from(left_class) < array_offset {
        return None;
    }

    // Classes are already premultiplied, so we only need to sum them.
    let index = usize::from(left_class) + usize::from(right_class);
    let value_offset = index.checked_sub(header_len)?;
    Stream::read_at::<i16>(data, value_offset)
}

fn get_format2_class(glyph_id: u16, offset: usize, data: &[u8]) -> Option<u16> {
    let mut s = Stream::new_at(data, offset)?;
    let first_glyph: u16 = s.read()?;
    let index = glyph_id.checked_sub(first_glyph)?;

    let number_of_classes: u16 = s.read()?;
    let classes = s.read_array16::<u16>(number_of_classes)?;
    classes.get(index)
}

/// A *Format 3 Kerning Table (Simple n x m Array of Kerning Indices)* implementation
/// from https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kern.html
fn parse_format3(data: &[u8], left: GlyphId, right: GlyphId) -> Option<i16> {
    let mut s = Stream::new(data);
    let glyph_count: u16 = s.read()?;
    let kerning_values_count: u8 = s.read()?;
    let left_hand_classes_count: u8 = s.read()?;
    let right_hand_classes_count: u8 = s.read()?;
    s.skip::<u8>(); // reserved
    let indices_count = u16::from(left_hand_classes_count) * u16::from(right_hand_classes_count);

    let kerning_values = s.read_array16::<i16>(u16::from(kerning_values_count))?;
    let left_hand_classes = s.read_array16::<u8>(glyph_count)?;
    let right_hand_classes = s.read_array16::<u8>(glyph_count)?;
    let indices = s.read_array16::<u8>(indices_count)?;

    let left_class = left_hand_classes.get(left.0)?;
    let right_class = right_hand_classes.get(right.0)?;

    if left_class > left_hand_classes_count || right_class > right_hand_classes_count {
        return None;
    }

    let index = u16::from(left_class) * u16::from(right_hand_classes_count) + u16::from(right_class);
    let index = indices.get(index)?;
    kerning_values.get(u16::from(index))
}
