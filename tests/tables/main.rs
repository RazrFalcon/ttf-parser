mod cff1;
mod cmap;
mod feat;
mod hmtx;
mod maxp;
mod sbix;

use ttf_parser::{Face, FaceParsingError, fonts_in_collection};

#[test]
fn empty_font() {
    assert_eq!(Face::from_slice(&[], 0).unwrap_err(),
               FaceParsingError::UnknownMagic);
}

#[test]
fn zero_tables() {
    let data = &[
        0x00, 0x01, 0x00, 0x00, // magic
        0x00, 0x00, // numTables: 0
        0x00, 0x00, // searchRange: 0
        0x00, 0x00, // entrySelector: 0
        0x00, 0x00, // rangeShift: 0
    ];

    assert_eq!(Face::from_slice(data, 0).unwrap_err(),
               FaceParsingError::NoHeadTable);
}

#[test]
fn tables_count_overflow() {
    let data = &[
        0x00, 0x01, 0x00, 0x00, // magic
        0xFF, 0xFF, // numTables: u16::MAX
        0x00, 0x00, // searchRange: 0
        0x00, 0x00, // entrySelector: 0
        0x00, 0x00, // rangeShift: 0
    ];

    assert_eq!(Face::from_slice(data, 0).unwrap_err(),
               FaceParsingError::MalformedFont);
}

#[test]
fn empty_font_collection() {
    let data = &[
        0x74, 0x74, 0x63, 0x66, // magic
        0x00, 0x00, // majorVersion: 0
        0x00, 0x00, // minorVersion: 0
        0x00, 0x00, 0x00, 0x00, // numFonts: 0
    ];

    assert_eq!(fonts_in_collection(data), Some(0));
    assert_eq!(Face::from_slice(data, 0).unwrap_err(),
               FaceParsingError::FaceIndexOutOfBounds);
}

#[test]
fn font_collection_num_fonts_overflow() {
    let data = &[
        0x74, 0x74, 0x63, 0x66, // magic
        0x00, 0x00, // majorVersion: 0
        0x00, 0x00, // minorVersion: 0
        0xFF, 0xFF, 0xFF, 0xFF, // numFonts: u32::MAX
    ];

    assert_eq!(fonts_in_collection(data), Some(std::u32::MAX));
    assert_eq!(Face::from_slice(data, 0).unwrap_err(),
               FaceParsingError::MalformedFont);
}

#[test]
fn font_index_overflow() {
    let data = &[
        0x74, 0x74, 0x63, 0x66, // magic
        0x00, 0x00, // majorVersion: 0
        0x00, 0x00, // minorVersion: 0
        0x00, 0x00, 0x00, 0x01, // numFonts: 1
        0x00, 0x00, 0x00, 0x0C, // offset [0]: 12
    ];

    assert_eq!(fonts_in_collection(data), Some(1));
    assert_eq!(Face::from_slice(data, std::u32::MAX).unwrap_err(),
               FaceParsingError::FaceIndexOutOfBounds);
}
