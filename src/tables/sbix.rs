// https://docs.microsoft.com/en-us/typography/opentype/spec/sbix

use core::convert::TryFrom;
use core::num::NonZeroU16;

use crate::{GlyphId, RasterGlyphImage, RasterImageFormat, Tag};
use crate::parser::{Stream, FromData, Offset, Offset32};

pub fn parse(
    data: &[u8],
    number_of_glyphs: NonZeroU16,
    glyph_id: GlyphId,
    pixels_per_em: u16,
    depth: u8,
) -> Option<RasterGlyphImage> {
    if depth == 10 {
        return None;
    }

    let total_glyphs = u32::from(number_of_glyphs.get().checked_add(1)?);

    let mut s = Stream::new(data);
    let version: u16 = s.read()?;
    if version != 1 {
        return None;
    }

    s.skip::<u16>(); // flags
    let count: u32 = s.read()?;
    if count == 0 {
        return None;
    }

    let strikes = s.read_array32::<Offset32>(count)?;

    // Select a best matching strike based on `pixels_per_em`.
    let mut idx = 0;
    let mut max_ppem = 0;
    {
        for (i, offset) in strikes.into_iter().enumerate() {
            let mut s = Stream::new_at(data, offset.to_usize())?;
            let ppem: u16 = s.read()?;
            s.skip::<u16>(); // ppi

            if (pixels_per_em <= ppem && ppem < max_ppem) ||
                (pixels_per_em > max_ppem && ppem > max_ppem)
            {
                idx = i as u32;
                max_ppem = ppem;
            }
        }
    }

    let offset = strikes.get(idx)?;
    let mut s = Stream::new_at(data, offset.to_usize())?;
    s.skip::<u16>(); // ppem
    s.skip::<u16>(); // ppi

    let glyph_offsets = s.read_array32::<Offset32>(total_glyphs)?;
    let start = glyph_offsets.get(u32::from(glyph_id.0))?.to_usize();
    let end = glyph_offsets.get(u32::from(glyph_id.0.checked_add(1)?))?.to_usize();

    if start == end {
        // No bitmap data for that glyph.
        return None;
    }

    let data_len = end.checked_sub(start)?.checked_sub(8)?; // 8 is a Glyph data header size.

    let mut s = Stream::new_at(data, offset.to_usize() + start)?;
    let x: i16 = s.read()?;
    let y: i16 = s.read()?;
    let image_type: Tag = s.read()?;
    let image_data = s.read_bytes(data_len)?;

    // We do ignore `pdf` and `mask` intentionally, because Apple docs state that:
    // 'Support for the 'pdf ' and 'mask' data types and sbixDrawOutlines flag
    // are planned for future releases of iOS and OS X.'
    let format = match &image_type.to_bytes() {
        b"png " => RasterImageFormat::PNG,
        b"dupe" => {
            // 'The special graphicType of 'dupe' indicates that
            // the data field contains a glyph ID. The bitmap data for
            // the indicated glyph should be used for the current glyph.'
            let glyph_id = GlyphId::parse(image_data)?;
            return parse(data, number_of_glyphs, glyph_id, pixels_per_em, depth + 1);
        }
        _ => {
            // TODO: support JPEG and TIFF
            return None;
        }
    };

    let (width, height) = png_size(image_data)?;

    Some(RasterGlyphImage {
        x,
        y,
        width,
        height,
        pixels_per_em: max_ppem,
        format,
        data: image_data,
    })
}

// The `sbix` table doesn't store the image size, so we have to parse it manually.
// Which is quite simple in case of PNG, but way more complex for JPEG.
// Therefore we are omitting it for now.
fn png_size(data: &[u8]) -> Option<(u16, u16)> {
    // PNG stores its size as u32 BE at a fixed offset.
    let mut s = Stream::new_at(data, 16)?;
    let width: u32 = s.read()?;
    let height: u32 = s.read()?;

    // PNG size larger than u16::MAX is an error.
    Some((
        u16::try_from(width).ok()?,
        u16::try_from(height).ok()?,
    ))
}
