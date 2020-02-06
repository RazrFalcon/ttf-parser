// https://docs.microsoft.com/en-us/typography/opentype/spec/avar

use crate::Font;
use crate::parser::{Stream, SafeStream, LazyArray16, FromData};


impl<'a> Font<'a> {
    /// Performs normalization mapping to variation coordinates
    /// using [Axis Variations Table](https://docs.microsoft.com/en-us/typography/opentype/spec/avar).
    ///
    /// Note: coordinates should be converted from fixed point 2.14 to i32
    /// by multiplying each coordinate by 16384.
    ///
    /// Number of `coordinates` should be the same as number of variation axes in the font.
    pub fn map_variation_coordinates(&self, coordinates: &mut [i32]) -> Option<()> {
        let mut s = Stream::new(self.avar?);
        let major_version: u16 = s.read()?;
        let minor_version: u16 = s.read()?;

        if !(major_version == 1 && minor_version == 0) {
            return None;
        }

        s.skip::<u16>(); // reserved
        // TODO: check that `axisCount` is the same as in `fvar`?
        let axis_count = s.read::<u16>()? as usize;
        if axis_count != coordinates.len() {
            return None;
        }

        for i in 0..axis_count {
            let map = s.read_array16::<AxisValueMapRecord>()?;
            coordinates[i] = map_value(&map, coordinates[i]);
        }

        Some(())
    }
}

#[derive(Clone, Copy)]
struct AxisValueMapRecord {
    from_coordinate: i32,
    to_coordinate: i32,
}

impl FromData for AxisValueMapRecord {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Self {
        let mut s = SafeStream::new(data);
        AxisValueMapRecord {
            from_coordinate: s.read::<i16>() as i32,
            to_coordinate: s.read::<i16>() as i32,
        }
    }
}

fn map_value(map: &LazyArray16<AxisValueMapRecord>, value: i32) -> i32 {
    // This code is based on harfbuzz implementation.

    if map.len() == 0 {
        return value;
    } else if map.len() == 1 {
        let record = map.at(0);
        return value - record.from_coordinate + record.to_coordinate;
    }

    let record_0 = map.at(0);
    if value <= record_0.from_coordinate {
        return value - record_0.from_coordinate + record_0.to_coordinate;
    }

    let mut i = 1;
    while i < map.len() && value > map.at(i).from_coordinate {
        i += 1;
    }

    if i == map.len() {
        i -= 1;
    }

    let record_i = map.at(i);
    if value >= record_i.from_coordinate {
        return value - record_i.from_coordinate + record_i.to_coordinate;
    }

    let record_prev = map.at(i - 1);
    if record_prev.from_coordinate == record_i.from_coordinate {
        return record_prev.to_coordinate;
    }

    let denom = record_i.from_coordinate - record_prev.from_coordinate;
    record_prev.to_coordinate +
        ((record_i.to_coordinate - record_prev.to_coordinate) *
            (value - record_prev.from_coordinate) + denom / 2) / denom
}
