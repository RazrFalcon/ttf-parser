use std::num::NonZeroU16;
use ttf_parser::maxp::Table;

#[test]
fn version_05() {
    let table = Table::parse(&[
        0x00, 0x00, 0x50, 0x00, // version: 0.3125
        0x00, 0x01, // number of glyphs: 1
    ]).unwrap();
    assert_eq!(table.number_of_glyphs, NonZeroU16::new(1).unwrap());
}

#[test]
fn version_1_full() {
    let table = Table::parse(&[
        0x00, 0x01, 0x00, 0x00, // version: 1
        0x00, 0x01, // number of glyphs: 1
        0x00, 0x00, // maximum points in a non-composite glyph: 0
        0x00, 0x00, // maximum contours in a non-composite glyph: 0
        0x00, 0x00, // maximum points in a composite glyph: 0
        0x00, 0x00, // maximum contours in a composite glyph: 0
        0x00, 0x00, // maximum zones: 0
        0x00, 0x00, // maximum twilight points: 0
        0x00, 0x00, // number of Storage Area locations: 0
        0x00, 0x00, // number of FDEFs: 0
        0x00, 0x00, // number of IDEFs: 0
        0x00, 0x00, // maximum stack depth: 0
        0x00, 0x00, // maximum byte count for glyph instructions: 0
        0x00, 0x00, // maximum number of components: 0
        0x00, 0x00, // maximum levels of recursion: 0
    ]).unwrap();
    assert_eq!(table.number_of_glyphs, NonZeroU16::new(1).unwrap());
}

#[test]
fn version_1_trimmed() {
    // We don't really care about the data after the number of glyphs.
    let table = Table::parse(&[
        0x00, 0x01, 0x00, 0x00, // version: 1
        0x00, 0x01, // number of glyphs: 1
    ]).unwrap();
    assert_eq!(table.number_of_glyphs, NonZeroU16::new(1).unwrap());
}

#[test]
fn unknown_version() {
    let table = Table::parse(&[
        0x00, 0x00, 0x00, 0x00, // version: 0
        0x00, 0x01, // number of glyphs: 1
    ]);
    assert!(table.is_none());
}

#[test]
fn zero_glyphs() {
    let table = Table::parse(&[
        0x00, 0x00, 0x50, 0x00, // version: 0.3125
        0x00, 0x00, // number of glyphs: 0
    ]);
    assert!(table.is_none());
}

// TODO: what to do when the number of glyphs is 0xFFFF?
//       we're actually checking this in loca
