// https://docs.microsoft.com/en-us/typography/opentype/spec/hhea

use core::num::NonZeroU16;

use crate::parser::Stream;


const TABLE_SIZE: usize = 36;
const ASCENDER_OFFSET: usize = 4;
const DESCENDER_OFFSET: usize = 6;
const LINE_GAP_OFFSET: usize = 8;
const NUMBER_OF_H_METRICS_OFFSET: usize = 34;


#[inline]
pub fn parse(data: &[u8]) -> Option<&[u8]> {
    if data.len() == TABLE_SIZE {
        Some(data)
    } else {
        None
    }
}

#[inline]
pub fn ascender(data: &[u8]) -> i16 {
    Stream::read_at::<i16>(data, ASCENDER_OFFSET).unwrap_or(0)
}

#[inline]
pub fn descender(data: &[u8]) -> i16 {
    Stream::read_at::<i16>(data, DESCENDER_OFFSET).unwrap_or(0)
}

#[inline]
pub fn line_gap(data: &[u8]) -> i16 {
    Stream::read_at::<i16>(data, LINE_GAP_OFFSET).unwrap_or(0)
}

#[inline]
pub fn number_of_h_metrics(data: &[u8]) -> Option<NonZeroU16> {
    Stream::read_at::<u16>(data, NUMBER_OF_H_METRICS_OFFSET).and_then(NonZeroU16::new)
}
