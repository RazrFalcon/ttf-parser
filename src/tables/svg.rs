// https://docs.microsoft.com/en-us/typography/opentype/spec/svg

use crate::GlyphId;
use crate::parser::{Stream, FromData, Offset, Offset32, NumFrom};


#[derive(Clone, Copy)]
struct SvgDocumentRecord {
    start_glyph_id: GlyphId,
    end_glyph_id: GlyphId,
    svg_doc_offset: Option<Offset32>,
    svg_doc_length: u32,
}

impl FromData for SvgDocumentRecord {
    const SIZE: usize = 12;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(SvgDocumentRecord {
            start_glyph_id: s.read::<GlyphId>()?,
            end_glyph_id: s.read::<GlyphId>()?,
            svg_doc_offset: s.read::<Option<Offset32>>()?,
            svg_doc_length: s.read::<u32>()?,
        })
    }
}


pub fn parse(
    data: &[u8],
    glyph_id: GlyphId,
) -> Option<&[u8]> {
    let mut s = Stream::new(data);
    s.skip::<u16>(); // version
    let doc_list_offset = s.read::<Option<Offset32>>()??;

    let mut s = Stream::new_at(data, doc_list_offset.to_usize())?;
    let count: u16 = s.read()?;
    let records = s.read_array16::<SvgDocumentRecord>(count)?;
    let record = records.into_iter()
        .find(|rec| (rec.start_glyph_id..=rec.end_glyph_id).contains(&glyph_id))?;

    let svg_offset = record.svg_doc_offset?;
    let mut s = Stream::new_at(data, doc_list_offset.to_usize() + svg_offset.to_usize())?;
    let svg_data = s.read_bytes(usize::num_from(record.svg_doc_length))?;

    Some(svg_data)
}
