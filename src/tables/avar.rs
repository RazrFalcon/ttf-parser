// https://docs.microsoft.com/en-us/typography/opentype/spec/avar

use core::convert::TryFrom;
use core::num::NonZeroU16;

use crate::NormalizedCoordinate;
use crate::parser::{Stream, FromData, LazyArray16};


#[derive(Clone, Copy)]
pub(crate) struct Table<'a> {
    axis_count: NonZeroU16,
    data: &'a [u8],
}

impl<'a> Table<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        let mut s = Stream::new(data);

        let version: u32 = s.read()?;
        if version != 0x00010000 {
            return None;
        }

        s.skip::<u16>(); // reserved
        // TODO: check that `axisCount` is the same as in `fvar`?
        let axis_count: u16 = s.read()?;
        let axis_count = NonZeroU16::new(axis_count)?;

        let data = s.tail()?;

        // Sanitize records.
        for _ in 0..axis_count.get() {
            let count: u16 = s.read()?;
            s.advance_checked(AxisValueMapRecord::SIZE * usize::from(count))?;
        }

        Some(Table {
            axis_count,
            data,
        })
    }

    pub fn map_coordinates(&self, coordinates: &mut [NormalizedCoordinate]) -> Option<()> {
        if usize::from(self.axis_count.get()) != coordinates.len() {
            return None;
        }

        let mut s = Stream::new(self.data);
        for coord in coordinates {
            let count: u16 = s.read()?;
            let map = s.read_array16::<AxisValueMapRecord>(count)?;
            *coord = NormalizedCoordinate::from(map_value(&map, coord.0)?);
        }

        Some(())
    }
}

fn map_value(map: &LazyArray16<AxisValueMapRecord>, value: i16) -> Option<i16> {
    // This code is based on harfbuzz implementation.

    if map.len() == 0 {
        return Some(value);
    } else if map.len() == 1 {
        let record = map.get(0)?;
        return Some(value - record.from_coordinate + record.to_coordinate);
    }

    let record_0 = map.get(0)?;
    if value <= record_0.from_coordinate {
        return Some(value - record_0.from_coordinate + record_0.to_coordinate);
    }

    let mut i = 1;
    while i < map.len() && value > map.get(i)?.from_coordinate {
        i += 1;
    }

    if i == map.len() {
        i -= 1;
    }

    let record_curr = map.get(i)?;
    let curr_from = record_curr.from_coordinate;
    let curr_to = record_curr.to_coordinate;
    if value >= curr_from {
        return Some(value - curr_from + curr_to);
    }

    let record_prev = map.get(i - 1)?;
    let prev_from = record_prev.from_coordinate;
    let prev_to = record_prev.to_coordinate;
    if prev_from == curr_from {
        return Some(prev_to);
    }

    let curr_from = i32::from(curr_from);
    let curr_to = i32::from(curr_to);
    let prev_from = i32::from(prev_from);
    let prev_to = i32::from(prev_to);

    let denom = curr_from - prev_from;
    let k = (curr_to - prev_to) * (i32::from(value) - prev_from) + denom / 2;
    let value = prev_to + k / denom;
    i16::try_from(value).ok()
}


#[derive(Clone, Copy)]
struct AxisValueMapRecord {
    from_coordinate: i16,
    to_coordinate: i16,
}

impl FromData for AxisValueMapRecord {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(AxisValueMapRecord {
            from_coordinate: s.read::<i16>()?,
            to_coordinate: s.read::<i16>()?,
        })
    }
}
