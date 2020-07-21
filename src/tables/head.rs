// https://docs.microsoft.com/en-us/typography/opentype/spec/head

use crate::Rect;
use crate::parser::Stream;


const TABLE_SIZE: usize = 54;
const UNITS_PER_EM_OFFSET: usize = 18;
const BBOX_OFFSET: usize = 36;
const INDEX_TO_LOC_FORMAT_OFFSET: usize = 50;


#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum IndexToLocationFormat {
    Short,
    Long,
}

#[inline]
pub fn parse(data: &[u8]) -> Option<&[u8]> {
    if data.len() == TABLE_SIZE {
        Some(data)
    } else {
        None
    }
}

#[inline]
pub fn units_per_em(data: &[u8]) -> Option<u16> {
    let num: u16 = Stream::read_at(data, UNITS_PER_EM_OFFSET)?;
    if num >= 16 && num <= 16384 {
        Some(num)
    } else {
        None
    }
}

#[inline]
pub fn global_bbox(data: &[u8]) -> Option<Rect> {
    let mut s = Stream::new_at(data, BBOX_OFFSET)?;
    Some(Rect {
        x_min: s.read::<i16>()?,
        y_min: s.read::<i16>()?,
        x_max: s.read::<i16>()?,
        y_max: s.read::<i16>()?,
    })
}

#[inline]
pub(crate) fn index_to_loc_format(data: &[u8]) -> Option<IndexToLocationFormat> {
    let format: i16 = Stream::read_at(data, INDEX_TO_LOC_FORMAT_OFFSET)?;
    match format {
        0 => Some(IndexToLocationFormat::Short),
        1 => Some(IndexToLocationFormat::Long),
        _ => None,
    }
}
