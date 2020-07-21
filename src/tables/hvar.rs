// https://docs.microsoft.com/en-us/typography/opentype/spec/hvar

use core::convert::TryFrom;

use crate::{GlyphId, NormalizedCoordinate};
use crate::parser::{Stream, Offset, Offset32};
use crate::var_store::ItemVariationStore;

#[derive(Clone, Copy)]
pub struct Table<'a> {
    data: &'a [u8],
    variation_store: ItemVariationStore<'a>,
    advance_width_mapping_offset: Option<Offset32>,
    lsb_mapping_offset: Option<Offset32>,
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version: u32 = s.read()?;
        if version != 0x00010000 {
            return None;
        }

        let variation_store_offset: Offset32 = s.read()?;
        let var_store_s = Stream::new_at(data, variation_store_offset.to_usize())?;
        let variation_store = ItemVariationStore::parse(var_store_s)?;

        Some(Table {
            data,
            variation_store,
            advance_width_mapping_offset: s.read::<Option<Offset32>>()?,
            lsb_mapping_offset: s.read::<Option<Offset32>>()?,
        })
    }
}


pub struct DeltaSetIndexMap<'a> {
    data: &'a [u8],
}

impl<'a> DeltaSetIndexMap<'a> {
    #[inline]
    pub fn new(data: &'a [u8]) -> Self {
        DeltaSetIndexMap { data }
    }

    #[inline]
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
        let inner_index_bit_count = u32::from((entry_format & 0xF) + 1);

        s.advance(usize::from(entry_size) * usize::from(idx));

        let mut n = 0u32;
        for b in s.read_bytes(usize::from(entry_size))? {
            n = (n << 8) + u32::from(*b);
        }

        let outer_index = n >> inner_index_bit_count;
        let inner_index = n & ((1 << inner_index_bit_count) - 1);
        Some((
            u16::try_from(outer_index).ok()?,
            u16::try_from(inner_index).ok()?
        ))
    }
}

#[inline]
pub(crate) fn glyph_advance_offset(
    table: Table,
    glyph_id: GlyphId,
    coordinates: &[NormalizedCoordinate],
) -> Option<f32> {
    let (outer_idx, inner_idx) = if let Some(offset) = table.advance_width_mapping_offset {
        DeltaSetIndexMap::new(table.data.get(offset.to_usize()..)?).map(glyph_id)?
    } else {
        // 'If there is no delta-set index mapping table for advance widths,
        // then glyph IDs implicitly provide the indices:
        // for a given glyph ID, the delta-set outer-level index is zero,
        // and the glyph ID is the delta-set inner-level index.'
        (0, glyph_id.0)
    };

    table.variation_store.parse_delta(outer_idx, inner_idx, coordinates)
}

#[inline]
pub(crate) fn glyph_side_bearing_offset(
    table: Table,
    glyph_id: GlyphId,
    coordinates: &[NormalizedCoordinate],
) -> Option<f32> {
    let set_data = table.data.get(table.lsb_mapping_offset?.to_usize()..)?;
    let (outer_idx, inner_idx) = DeltaSetIndexMap::new(set_data).map(glyph_id)?;
    table.variation_store.parse_delta(outer_idx, inner_idx, coordinates)
}
