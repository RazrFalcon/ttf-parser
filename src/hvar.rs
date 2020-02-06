// https://docs.microsoft.com/en-us/typography/opentype/spec/hvar

use crate::{Font, GlyphId};
use crate::parser::{Stream, Offset, Offset32};


impl<'a> Font<'a> {
    /// Parses glyph's variation offset for horizontal advance using
    /// [Horizontal Metrics Variations Table](https://docs.microsoft.com/en-us/typography/opentype/spec/hvar).
    ///
    /// Note: coordinates should be converted from fixed point 2.14 to i32
    /// by multiplying each coordinate by 16384.
    ///
    /// Number of `coordinates` should be the same as number of variation axes in the font.
    ///
    /// Returns `None` when `HVAR` table is not present or invalid.
    pub fn glyph_hor_advance_variation(
        &self,
        glyph_id: GlyphId,
        coordinates: &[i32],
    ) -> Option<f32> {
        glyph_advance_variation(self.hvar?, glyph_id, coordinates)
    }

    /// Parses glyph's variation offset for horizontal side bearing using
    /// [Horizontal Metrics Variations Table](https://docs.microsoft.com/en-us/typography/opentype/spec/hvar).
    ///
    /// Note: coordinates should be converted from fixed point 2.14 to i32
    /// by multiplying each coordinate by 16384.
    ///
    /// Number of `coordinates` should be the same as number of variation axes in the font.
    ///
    /// Returns `None` when `HVAR` table is not present or invalid.
    pub fn glyph_hor_side_bearing_variation(
        &self,
        glyph_id: GlyphId,
        coordinates: &[i32],
    ) -> Option<f32> {
        glyph_side_bearing_variation(self.hvar?, glyph_id, coordinates)
    }
}

pub struct DeltaSetIndexMap<'a> {
    data: &'a [u8],
}

impl<'a> DeltaSetIndexMap<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        DeltaSetIndexMap { data }
    }

    pub fn map(&self, glyph_id: GlyphId) -> Option<(u16, u16)> {
        let mut idx = glyph_id.0;

        let mut s = Stream::new(self.data);
        let entry_format: u16 = s.read()?;
        let map_count: u16 = s.read()?;

        if map_count == 0 {
            return None;
        }

        // 'If a given glyph ID is greater than mapCount-1, then the last entry is used.'
        if idx >= map_count {
            idx = map_count - 1;
        }

        let entry_size = ((entry_format >> 4) & 3) + 1;
        let inner_index_bit_count = ((entry_format & 0xF) + 1) as u32;

        s.advance(entry_size as u32 * idx as u32);

        let mut n = 0u32;
        for b in s.read_bytes(entry_size)? {
            n = (n << 8) + *b as u32;
        }

        let outer_index = n >> inner_index_bit_count;
        let inner_index = n & ((1 << inner_index_bit_count) - 1);
        Some((outer_index as u16, inner_index as u16))
    }
}

pub fn glyph_advance_variation(
    data: &[u8], // HVAR or VVAR
    glyph_id: GlyphId,
    coordinates: &[i32],
) -> Option<f32> {
    let mut s = Stream::new(data);

    let major_version: u16 = s.read()?;
    let minor_version: u16 = s.read()?;
    if !(major_version == 1 && minor_version == 0) {
        return None;
    }

    let variation_store_offset: Offset32 = s.read()?;
    let advance_width_mapping_offset: Option<Offset32> = s.read()?;

    let (outer_idx, inner_idx) = if let Some(offset) = advance_width_mapping_offset {
        DeltaSetIndexMap::new(data.get(offset.to_usize()..)?).map(glyph_id)?
    } else {
        let outer_index = glyph_id.0 as u32 >> 16;
        let inner_index = glyph_id.0 as u32 & 0xFFFF;
        (outer_index as u16, inner_index as u16)
    };

    let mut s2 = Stream::new_at(data, variation_store_offset.to_usize());
    crate::mvar::parse_item_variation_store(outer_idx, inner_idx, coordinates, &mut s2)
}

pub fn glyph_side_bearing_variation(
    data: &[u8], // HVAR or VVAR
    glyph_id: GlyphId,
    coordinates: &[i32],
) -> Option<f32> {
    let mut s = Stream::new(data);

    let major_version: u16 = s.read()?;
    let minor_version: u16 = s.read()?;
    if !(major_version == 1 && minor_version == 0) {
        return None;
    }

    let variation_store_offset: Offset32 = s.read()?;
    s.skip::<Offset32>();
    let lsb_mapping_offset = s.read::<Option<Offset32>>()??;
    let set_data = data.get(lsb_mapping_offset.to_usize()..)?;
    let (outer_idx, inner_idx) = DeltaSetIndexMap::new(set_data).map(glyph_id)?;

    let mut s2 = Stream::new_at(data, variation_store_offset.to_usize());
    crate::mvar::parse_item_variation_store(outer_idx, inner_idx, coordinates, &mut s2)
}
