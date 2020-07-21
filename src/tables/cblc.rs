// https://docs.microsoft.com/en-us/typography/opentype/spec/cblc

use crate::GlyphId;
use crate::parser::{Stream, FromData, Offset, Offset16, Offset32, NumFrom};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BitmapFormat {
    Format17,
    Format18,
    Format19,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Metrics {
    pub x: i8,
    pub y: i8,
    pub width: u8,
    pub height: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Location {
    pub format: BitmapFormat,
    pub offset: usize,
    pub metrics: Metrics,
    pub ppem: u16,
}

pub fn find_location(
    data: &[u8],
    glyph_id: GlyphId,
    pixels_per_em: u16,
) -> Option<Location> {
    let mut s = Stream::new(data);

    // The CBLC table version is a bit tricky, so we are ignoring it for now.
    // The CBLC table is based on EBLC table, which was based on the `bloc` table.
    // And before the CBLC table specification was finished, some fonts,
    // notably Noto Emoji, have used version 2.0, but the final spec allows only 3.0.
    // So there are perfectly valid fonts in the wild, which have an invalid version.
    s.skip::<u32>(); // version

    let size_table = select_bitmap_size_table(glyph_id, pixels_per_em, s)?;
    let info = select_index_subtable(data, size_table, glyph_id)?;

    let mut s = Stream::new_at(data, info.offset)?;
    let index_format: u16 = s.read()?;
    let image_format: u16 = s.read()?;
    let mut image_offset = s.read::<Offset32>()?.to_usize();

    let image_format = match image_format {
        17 => BitmapFormat::Format17,
        18 => BitmapFormat::Format18,
        19 => BitmapFormat::Format19,
        _ => return None, // Invalid format.
    };

    // TODO: I wasn't able to find fonts with index 4 and 5, so they are untested.

    let glyph_diff = glyph_id.0.checked_sub(info.start_glyph_id.0)?;
    let metrics = Metrics::default();
    match index_format {
        1 => {
            s.advance(usize::from(glyph_diff) * Offset32::SIZE);
            let offset: Offset32 = s.read()?;
            image_offset += offset.to_usize();
        }
        2 => {
            let image_size: u32 = s.read()?;
            image_offset += usize::from(glyph_diff).checked_mul(usize::num_from(image_size))?;
        }
        3 => {
            s.advance(usize::from(glyph_diff) * Offset16::SIZE);
            let offset: Offset16 = s.read()?;
            image_offset += offset.to_usize();
        }
        4 => {
            let num_glyphs: u32 = s.read()?;
            let num_glyphs = num_glyphs.checked_add(1)?;
            let pairs = s.read_array32::<GlyphIdOffsetPair>(num_glyphs)?;
            let pair = pairs.into_iter().find(|pair| pair.glyph_id == glyph_id)?;
            image_offset += pair.offset.to_usize();
        }
        5 => {
            let image_size: u32 = s.read()?;
            s.advance(8); // big metrics
            let num_glyphs: u32 = s.read()?;
            let glyphs = s.read_array32::<GlyphId>(num_glyphs)?;
            let (index, _) = glyphs.binary_search(&glyph_id)?;
            image_offset = image_offset
                .checked_add(usize::num_from(index).checked_mul(usize::num_from(image_size))?)?;
        }
        _ => return None, // Invalid format.
    }

    Some(Location {
        format: image_format,
        offset: image_offset,
        metrics,
        ppem: size_table.ppem,
    })
}


#[derive(Clone, Copy)]
struct BitmapSizeTable {
    subtable_array_offset: Offset32,
    number_of_subtables: u32,
    ppem: u16,
    // Many fields are omitted.
}

fn select_bitmap_size_table(
    glyph_id: GlyphId,
    pixels_per_em: u16,
    mut s: Stream,
) -> Option<BitmapSizeTable> {
    let subtable_count: u32 = s.read()?;
    let orig_s = s.clone();

    let mut idx = None;
    let mut max_ppem = 0;
    for i in 0..subtable_count {
        // The BitmapSize Table is larger than 32 bytes, so we cannot use scripts/gen-tables.py

        // Check that the current subtable contains a provided glyph id.
        s.advance(40); // Jump to `start_glyph_index`.
        let start_glyph_id: GlyphId = s.read()?;
        let end_glyph_id: GlyphId = s.read()?;
        let ppem = u16::from(s.read::<u8>()?);

        if !(start_glyph_id..=end_glyph_id).contains(&glyph_id) {
            s.advance(4); // Jump to the end of the subtable.
            continue;
        }

        // Select a best matching subtable based on `pixels_per_em`.
        if (pixels_per_em <= ppem && ppem < max_ppem) || (pixels_per_em > max_ppem && ppem > max_ppem) {
            idx = Some(usize::num_from(i));
            max_ppem = ppem;
        }
    }

    let mut s = orig_s;
    s.advance(idx? * 48); // 48 is BitmapSize Table size

    let subtable_array_offset: Offset32 = s.read()?;
    s.skip::<u32>(); // index_tables_size
    let number_of_subtables: u32 = s.read()?;

    Some(BitmapSizeTable {
        subtable_array_offset,
        number_of_subtables,
        ppem: max_ppem,
    })
}


#[derive(Clone, Copy)]
struct IndexSubtableInfo {
    start_glyph_id: GlyphId,
    offset: usize, // absolute offset
}

fn select_index_subtable(
    data: &[u8],
    size_table: BitmapSizeTable,
    glyph_id: GlyphId,
) -> Option<IndexSubtableInfo> {
    let mut s = Stream::new_at(data, size_table.subtable_array_offset.to_usize())?;
    for _ in 0..size_table.number_of_subtables {
        let start_glyph_id: GlyphId = s.read()?;
        let end_glyph_id: GlyphId = s.read()?;
        let offset: Offset32 = s.read()?;

        if (start_glyph_id..=end_glyph_id).contains(&glyph_id) {
            let offset = size_table.subtable_array_offset.to_usize() + offset.to_usize();
            return Some(IndexSubtableInfo {
                start_glyph_id,
                offset,
            })
        }
    }

    None
}


#[derive(Clone, Copy)]
pub struct GlyphIdOffsetPair {
    glyph_id: GlyphId,
    offset: Offset16,
}

impl FromData for GlyphIdOffsetPair {
    const SIZE: usize = 4;

    #[inline]
    fn parse(data: &[u8]) -> Option<Self> {
        let mut s = Stream::new(data);
        Some(GlyphIdOffsetPair {
            glyph_id: s.read::<GlyphId>()?,
            offset: s.read::<Offset16>()?,
        })
    }
}
