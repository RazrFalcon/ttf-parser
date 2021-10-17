use std::num::NonZeroU16;
use ttf_parser::{GlyphId, RasterImageFormat};
use ttf_parser::sbix::Table;

#[test]
fn single_glyph() {
    let data = &[
        0x00, 0x01, // version: 1
        0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x01, // number of strikes: 1
        0x00, 0x00, 0x00, 0x0C, // strike offset [0]: 12

        // Strike [0]
        0x00, 0x14, // pixels_per_em: 20
        0x00, 0x48, // ppi: 72
        0x00, 0x00, 0x00, 0x0C, // glyph data offset [0]: 12
        0x00, 0x00, 0x00, 0x2C, // glyph data offset [1]: 44

        // Glyph Data [0]
        0x00, 0x01, // x: 1
        0x00, 0x02, // y: 2
        0x70, 0x6E, 0x67, 0x20, // type tag: PNG
        // PNG data, just the part we need
        0x89, 0x50, 0x4E, 0x47,
        0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D,
        0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x14, // width: 20
        0x00, 0x00, 0x00, 0x1E, // height: 30
    ];

    let table = Table::parse(NonZeroU16::new(1).unwrap(), data).unwrap();
    assert_eq!(table.strikes.len(), 1);

    let strike = table.strikes.get(0).unwrap();
    assert_eq!(strike.pixels_per_em, 20);
    assert_eq!(strike.ppi, 72);
    assert_eq!(strike.len(), 1);

    let glyph_data = strike.get(GlyphId(0)).unwrap();
    assert_eq!(glyph_data.x, 1);
    assert_eq!(glyph_data.y, 2);
    assert_eq!(glyph_data.width, 20);
    assert_eq!(glyph_data.height, 30);
    assert_eq!(glyph_data.pixels_per_em, 20);
    assert_eq!(glyph_data.format, RasterImageFormat::PNG);
    assert_eq!(glyph_data.data.len(), 24);
}

#[test]
fn duplicate_glyph() {
    let data = &[
        0x00, 0x01, // version: 1
        0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x01, // number of strikes: 1
        0x00, 0x00, 0x00, 0x0C, // strike offset [0]: 12

        // Strike [0]
        0x00, 0x14, // pixels_per_em: 20
        0x00, 0x48, // ppi: 72
        0x00, 0x00, 0x00, 0x10, // glyph data offset [0]: 16
        0x00, 0x00, 0x00, 0x30, // glyph data offset [1]: 48
        0x00, 0x00, 0x00, 0x3A, // glyph data offset [2]: 58

        // Glyph Data [0]
        0x00, 0x01, // x: 1
        0x00, 0x02, // y: 2
        0x70, 0x6E, 0x67, 0x20, // type tag: png
        // PNG data, just the part we need
        0x89, 0x50, 0x4E, 0x47,
        0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D,
        0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x14, // width: 20
        0x00, 0x00, 0x00, 0x1E, // height: 30

        // Glyph Data [1]
        0x00, 0x01, // x: 3
        0x00, 0x02, // y: 4
        0x64, 0x75, 0x70, 0x65, // type tag: dupe
        0x00, 0x00, // glyph id: 0
    ];

    let table = Table::parse(NonZeroU16::new(2).unwrap(), data).unwrap();
    assert_eq!(table.strikes.len(), 1);

    let strike = table.strikes.get(0).unwrap();
    assert_eq!(strike.pixels_per_em, 20);
    assert_eq!(strike.ppi, 72);
    assert_eq!(strike.len(), 2);

    let glyph_data = strike.get(GlyphId(1)).unwrap();
    assert_eq!(glyph_data.x, 1);
    assert_eq!(glyph_data.y, 2);
    assert_eq!(glyph_data.width, 20);
    assert_eq!(glyph_data.height, 30);
    assert_eq!(glyph_data.pixels_per_em, 20);
    assert_eq!(glyph_data.format, RasterImageFormat::PNG);
    assert_eq!(glyph_data.data.len(), 24);
}

#[test]
fn recursive() {
    let data = &[
        0x00, 0x01, // version: 1
        0x00, 0x00, // flags
        0x00, 0x00, 0x00, 0x01, // number of strikes: 1
        0x00, 0x00, 0x00, 0x0C, // strike offset [0]: 12

        // Strike [0]
        0x00, 0x14, // pixels_per_em: 20
        0x00, 0x48, // ppi: 72
        0x00, 0x00, 0x00, 0x10, // glyph data offset [0]: 16
        0x00, 0x00, 0x00, 0x1A, // glyph data offset [1]: 26
        0x00, 0x00, 0x00, 0x24, // glyph data offset [2]: 36

        // Glyph Data [0]
        0x00, 0x01, // x: 0
        0x00, 0x02, // y: 0
        0x64, 0x75, 0x70, 0x65, // type tag: dupe
        0x00, 0x00, // glyph id: 1

        // Glyph Data [1]
        0x00, 0x01, // x: 0
        0x00, 0x02, // y: 0
        0x64, 0x75, 0x70, 0x65, // type tag: dupe
        0x00, 0x00, // glyph id: 0
    ];

    let table = Table::parse(NonZeroU16::new(2).unwrap(), data).unwrap();
    let strike = table.strikes.get(0).unwrap();
    assert!(strike.get(GlyphId(0)).is_none());
    assert!(strike.get(GlyphId(1)).is_none());
}
