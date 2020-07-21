// https://docs.microsoft.com/en-us/typography/opentype/spec/cmap#format-14-unicode-variation-sequences

use crate::GlyphId;
use crate::parser::{Stream, FromData, Offset, Offset32, U24};

#[derive(Clone, Copy)]
struct VariationSelectorRecord {
    var_selector: u32,
    default_uvs_offset: Option<Offset32>,
    non_default_uvs_offset: Option<Offset32>,
}

impl FromData for VariationSelectorRecord {
    const SIZE: usize = 11;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(VariationSelectorRecord {
            var_selector: s.read::<U24>()?.0,
            default_uvs_offset: s.read::<Option<Offset32>>()?,
            non_default_uvs_offset: s.read::<Option<Offset32>>()?,
        })
    }
}


#[derive(Clone, Copy)]
struct UVSMappingRecord {
    unicode_value: u32,
    glyph_id: GlyphId,
}

impl FromData for UVSMappingRecord {
    const SIZE: usize = 5;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(UVSMappingRecord {
            unicode_value: s.read::<U24>()?.0,
            glyph_id: s.read::<GlyphId>()?,
        })
    }
}


#[derive(Clone, Copy)]
struct UnicodeRangeRecord {
    start_unicode_value: u32,
    additional_count: u8,
}

impl UnicodeRangeRecord {
    fn contains(&self, c: char) -> bool {
        let end = self.start_unicode_value + u32::from(self.additional_count);
        self.start_unicode_value >= u32::from(c) && u32::from(c) < end
    }
}

impl FromData for UnicodeRangeRecord {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(UnicodeRangeRecord {
            start_unicode_value: s.read::<U24>()?.0,
            additional_count: s.read::<u8>()?,
        })
    }
}


pub fn parse(
    table: &super::Table,
    data: &[u8],
    c: char,
    variation: u32,
) -> Option<GlyphId> {
    let cp = u32::from(c);

    let mut s = Stream::new(data);
    s.skip::<u16>(); // format
    s.skip::<u32>(); // length
    let count: u32 = s.read()?;
    let records = s.read_array32::<VariationSelectorRecord>(count)?;

    let (_, record) = match records.binary_search_by(|v| v.var_selector.cmp(&variation)) {
        Some(v) => v,
        None => return None,
    };

    if let Some(offset) = record.default_uvs_offset {
        let data = data.get(offset.to_usize()..)?;
        let mut s = Stream::new(data);
        let count: u32 = s.read()?;
        let ranges = s.read_array32::<UnicodeRangeRecord>(count)?;
        for range in ranges {
            if range.contains(c) {
                // This is a default glyph.
                return super::glyph_index(table, c);
            }
        }
    }

    if let Some(offset) = record.non_default_uvs_offset {
        let data = data.get(offset.to_usize()..)?;
        let mut s = Stream::new(data);
        let count: u32 = s.read()?;
        let uvs_mappings = s.read_array32::<UVSMappingRecord>(count)?;
        if let Some((_, mapping)) = uvs_mappings.binary_search_by(|v| v.unicode_value.cmp(&cp)) {
            return Some(mapping.glyph_id);
        }
    }

    None
}
