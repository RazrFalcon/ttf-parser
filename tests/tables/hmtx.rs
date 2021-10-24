use std::num::NonZeroU16;
use ttf_parser::GlyphId;
use ttf_parser::hmtx::Table;

macro_rules! nzu16 {
    ($n:expr) => { NonZeroU16::new($n).unwrap() };
}

#[test]
fn simple_case() {
    let data = &[
        0x00, 0x01, // advance width [0]: 1
        0x00, 0x02, // side bearing [0]: 2
    ];

    let table = Table::parse(data, 1, nzu16!(1)).unwrap();
    assert_eq!(table.advance(GlyphId(0)), Some(1));
    assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
}

#[test]
fn empty() {
    assert!(Table::parse(&[], 1, nzu16!(1)).is_none());
}

#[test]
fn zero_metrics() {
    let data = &[
        0x00, 0x01, // advance width [0]: 1
        0x00, 0x02, // side bearing [0]: 2
    ];

    assert!(Table::parse(data, 0, nzu16!(1)).is_none());
}

#[test]
fn smaller_than_glyphs_count() {
    let data = &[
        0x00, 0x01, // advance width [0]: 1
        0x00, 0x02, // side bearing [0]: 2

        0x00, 0x03, // side bearing [1]: 3
    ];

    let table = Table::parse(data, 1, nzu16!(2)).unwrap();
    assert_eq!(table.advance(GlyphId(0)), Some(1));
    assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
    assert_eq!(table.advance(GlyphId(1)), Some(1));
    assert_eq!(table.side_bearing(GlyphId(1)), Some(3));
}

#[test]
fn less_metrics_than_glyphs() {
    let data = &[
        0x00, 0x01, // advance width [0]: 1
        0x00, 0x02, // side bearing [0]: 2

        0x00, 0x03, // advance width [1]: 3
        0x00, 0x04, // side bearing [1]: 4

        0x00, 0x05, // side bearing [2]: 5
    ];

    let table = Table::parse(data, 2, nzu16!(1)).unwrap();
    assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
    assert_eq!(table.side_bearing(GlyphId(1)), Some(4));
    assert_eq!(table.side_bearing(GlyphId(2)), None);
}

#[test]
fn glyph_out_of_bounds_0() {
    let data = &[
        0x00, 0x01, // advance width [0]: 1
        0x00, 0x02, // side bearing [0]: 2
    ];

    let table = Table::parse(data, 1, nzu16!(1)).unwrap();
    assert_eq!(table.advance(GlyphId(0)), Some(1));
    assert_eq!(table.side_bearing(GlyphId(0)), Some(2));
    assert_eq!(table.advance(GlyphId(1)), None);
    assert_eq!(table.side_bearing(GlyphId(1)), None);
}

#[test]
fn glyph_out_of_bounds_1() {
    let data = &[
        0x00, 0x01, // advance width [0]: 1
        0x00, 0x02, // side bearing [0]: 2

        0x00, 0x03, // side bearing [1]: 3
    ];

    let table = Table::parse(data, 1, nzu16!(2)).unwrap();
    assert_eq!(table.advance(GlyphId(1)), Some(1));
    assert_eq!(table.side_bearing(GlyphId(1)), Some(3));
    assert_eq!(table.advance(GlyphId(2)), None);
    assert_eq!(table.side_bearing(GlyphId(2)), None);
}
