// https://docs.microsoft.com/en-us/typography/opentype/spec/cbdt

use crate::{RasterGlyphImage, RasterImageFormat};
use crate::parser::{Stream, NumFrom};
use super::cblc::{BitmapFormat, Location};

pub fn parse(
    data: &[u8],
    location: Location,
) -> Option<RasterGlyphImage> {
    let mut s = Stream::new_at(data, location.offset)?;
    match location.format {
        BitmapFormat::Format17 => {
            let height: u8 = s.read()?;
            let width: u8 = s.read()?;
            let bearing_x: i8 = s.read()?;
            let bearing_y: i8 = s.read()?;
            s.skip::<u8>(); // advance
            let data_len: u32 = s.read()?;
            let data = s.read_bytes(usize::num_from(data_len))?;
            Some(RasterGlyphImage {
                x: i16::from(bearing_x),
                // `y` in CBDT is a bottom bound, not top one.
                y: i16::from(bearing_y) - i16::from(height),
                width: u16::from(width),
                height: u16::from(height),
                pixels_per_em: location.ppem,
                format: RasterImageFormat::PNG,
                data,
            })
        }
        BitmapFormat::Format18 => {
            let height: u8 = s.read()?;
            let width: u8 = s.read()?;
            let hor_bearing_x: i8 = s.read()?;
            let hor_bearing_y: i8 = s.read()?;
            s.skip::<u8>(); // hor_advance
            s.skip::<i8>(); // ver_bearing_x
            s.skip::<i8>(); // ver_bearing_y
            s.skip::<u8>(); // ver_advance
            let data_len: u32 = s.read()?;
            let data = s.read_bytes(usize::num_from(data_len))?;
            Some(RasterGlyphImage {
                x: i16::from(hor_bearing_x),
                // `y` in CBDT is a bottom bound, not top one.
                y: i16::from(hor_bearing_y) - i16::from(height),
                width: u16::from(width),
                height: u16::from(height),
                pixels_per_em: location.ppem,
                format: RasterImageFormat::PNG,
                data,
            })
        }
        BitmapFormat::Format19 => {
            let data_len: u32 = s.read()?;
            let data = s.read_bytes(usize::num_from(data_len))?;
            Some(RasterGlyphImage {
                x: i16::from(location.metrics.x),
                // `y` in CBDT is a bottom bound, not top one.
                y: i16::from(location.metrics.y) - i16::from(location.metrics.height),
                width: u16::from(location.metrics.width),
                height: u16::from(location.metrics.height),
                pixels_per_em: location.ppem,
                format: RasterImageFormat::PNG,
                data,
            })
        }
    }
}
