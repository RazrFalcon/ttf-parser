// https://docs.microsoft.com/en-us/typography/opentype/spec/svg

use crate::{GlyphId, GlyphImage, ImageFormat};
use crate::parser::{Stream, Offset, Offset32, NumFrom};
use crate::raw::svg as raw;

pub fn parse(
    data: &[u8],
    glyph_id: GlyphId,
) -> Option<GlyphImage> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // version
    let doc_list_offset = s.read::<Option<Offset32>>()??;

    let mut s = Stream::new_at(data, doc_list_offset.to_usize())?;
    let count: u16 = s.read()?;
    let records = s.read_array16::<raw::SvgDocumentRecord>(count)?;
    let record = records.into_iter()
        .find(|rec| (rec.start_glyph_id()..=rec.end_glyph_id()).contains(&glyph_id))?;

    let svg_offset = record.svg_doc_offset()?;
    let mut s = Stream::new_at(data, doc_list_offset.to_usize() + svg_offset.to_usize())?;
    let svg_data = s.read_bytes(usize::num_from(record.svg_doc_length()))?;

    Some(GlyphImage {
        x: None,
        y: None,
        width: None,
        height: None,
        pixels_per_em: 0,
        format: ImageFormat::SVG,
        data: svg_data,
    })
}
