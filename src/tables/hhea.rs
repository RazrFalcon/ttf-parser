// https://docs.microsoft.com/en-us/typography/opentype/spec/hhea

use core::num::NonZeroU16;

use crate::parser::Stream;
use crate::raw::hhea as raw;

#[inline]
pub fn parse(data: &[u8]) -> Option<&[u8]> {
    if data.len() == raw::TABLE_SIZE {
        Some(data)
    } else {
        None
    }
}

#[inline]
pub fn ascender(data: &[u8]) -> i16 {
    Stream::read_at(data, raw::ASCENDER_OFFSET).unwrap_or(0)
}

#[inline]
pub fn descender(data: &[u8]) -> i16 {
    Stream::read_at(data, raw::DESCENDER_OFFSET).unwrap_or(0)
}

#[inline]
pub fn line_gap(data: &[u8]) -> i16 {
    Stream::read_at(data, raw::LINE_GAP_OFFSET).unwrap_or(0)
}

#[inline]
pub fn number_of_h_metrics(data: &[u8]) -> Option<NonZeroU16> {
    Stream::read_at(data, raw::NUMBER_OF_H_METRICS_OFFSET).and_then(NonZeroU16::new)
}
