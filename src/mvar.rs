// https://docs.microsoft.com/en-us/typography/opentype/spec/mvar

use crate::{Font, Tag};
use crate::parser::{Stream, LazyArray, FromData, Offset, Offset16, Offset32};
use crate::raw::mvar as raw;


impl<'a> Font<'a> {
    /// Parses metrics variation offset using
    /// [Metrics Variations Table](https://docs.microsoft.com/en-us/typography/opentype/spec/mvar).
    ///
    /// Note: coordinates should be converted from fixed point 2.14 to i32
    /// by multiplying each coordinate by 16384.
    ///
    /// Number of `coordinates` should be the same as number of variation axes in the font.
    ///
    /// Returns `None` when `MVAR` table is not present or invalid.
    pub fn metrics_variation(&self, tag: Tag, coordinates: &[i32]) -> Option<f32> {
        let mut s = Stream::new(self.mvar?);

        let major_version: u16 = s.read().ok()?;
        let minor_version: u16 = s.read().ok()?;
        if major_version != 1 && minor_version != 0 {
            return None;
        }

        s.skip::<u16>(); // reserved
        s.skip::<u16>(); // valueRecordSize

        let count: u16 = s.read().ok()?;
        if count == 0 {
            return None;
        }

        let variation_store_offset: Option<Offset16> = s.read().ok()?;
        let variation_store_offset = variation_store_offset?; // Return when no offset.

        let value_records: LazyArray<raw::ValueRecord> = s.read_array(count).ok()?;
        let record = value_records.binary_search_by(|r| r.value_tag().cmp(&tag))?;

        let mut s2 = Stream::new_at(self.mvar?, variation_store_offset.to_usize());
        parse_item_variation_store(
            record.delta_set_outer_index(), record.delta_set_inner_index(), coordinates, &mut s2,
        )
    }
}

fn parse_item_variation_store(
    outer_index: u16,
    inner_index: u16,
    coordinates: &[i32],
    s: &mut Stream,
) -> Option<f32> {
    let orig = s.clone();

    let format: u16 = s.read().ok()?;
    if format != 1 {
        return None;
    }

    let variation_region_list_offset: Offset32 = s.read().ok()?;
    let item_variation_data_offsets: LazyArray<Offset32> = s.read_array16().ok()?;

    let var_data_offset = item_variation_data_offsets.get(outer_index)?;
    let mut s = orig.clone();
    s.advance(var_data_offset.0);

    let mut region_s = orig.clone();
    region_s.advance(variation_region_list_offset.0);

    parse_item_variation_data(inner_index, coordinates, &mut s, region_s)
}

fn parse_item_variation_data(
    inner_index: u16,
    coordinates: &[i32],
    s: &mut Stream,
    region_s: Stream,
) -> Option<f32> {
    let item_count: u16 = s.read().ok()?;
    if inner_index >= item_count {
        return None;
    }

    let short_delta_count = s.read::<u16>().ok()? as u32;
    let region_index_count = s.read::<u16>().ok()? as u32;
    let region_indexes: LazyArray<u16> = s.read_array(region_index_count as u16).ok()?;
    s.advance((i16::SIZE + i8::SIZE) as u32 * (short_delta_count + region_index_count));

    let mut delta = 0.0;
    let mut i = 0;
    while i < short_delta_count {
        let idx = region_indexes.get(i)?;
        delta += s.read::<i16>().ok()? as f32 * evaluate_region(idx, coordinates, region_s)?;
        i += 1;
    }

    while i < region_index_count {
        let idx = region_indexes.get(i)?;
        delta += s.read::<i8>().ok()? as f32 * evaluate_region(idx, coordinates, region_s)?;
        i += 1;
    }

    Some(delta)
}

fn evaluate_region(
    index: u16,
    coordinates: &[i32],
    mut s: Stream,
) -> Option<f32> {
    let axis_count: u16 = s.read().ok()?;
    s.skip::<u16>(); // region_count
    s.advance(index as u32 * axis_count as u32 * raw::RegionAxisCoordinatesRecord::SIZE as u32);
    let record: raw::RegionAxisCoordinatesRecord = s.read().ok()?;

    let mut v = 1.0;
    for i in 0..axis_count {
        let coord = coordinates.get(i as usize).cloned().unwrap_or(0);
        let factor = evaluate_axis(&record, coord);
        if factor == 0.0 {
            return None;
        }

        v *= factor;
    }

    Some(v)
}

fn evaluate_axis(axis: &raw::RegionAxisCoordinatesRecord, coord: i32) -> f32 {
    let start = axis.start_coord() as i32;
    let peak = axis.peak_coord() as i32;
    let end = axis.end_coord() as i32;

    if start > peak || peak > end {
        return 1.0;
    }

    if start < 0 && end > 0 && peak != 0 {
       return 1.0;
    }

    if peak == 0 || coord == peak {
        return 1.0;
    }

    if coord <= start || end <= coord {
        return 0.0;
    }

    if coord < peak {
        (coord - start) as f32 / (peak - start) as f32
    } else {
        (end - coord) as f32 / (end - peak) as f32
    }
}
