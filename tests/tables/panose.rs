use ttf_parser::{os2::panose::Panose, FromData as _};

// Do we ignore everythign after a "any-fit" initial number
const PANOSE_ANY_FIT: [u8; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];

// A "Latten Text" number should be considered italic when the letterform is one of the oblique values
const PANOSE_ITALIC_TEXT: [u8; 10] = [2, 0, 0, 0, 0, 0, 0, 11, 0, 0];

// A "Latin Handwritten" number should be considered italic when the letterform is oblique or exaggerated
const PANOSE_ITALIC_HANDWRITTEN: [u8; 10] = [3, 0, 0, 0, 0, 0, 0, 7, 0, 0];

// Do we ignore everythign after a "no-fit" initial number
const PANOSE_NO_FIT: [u8; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

#[test]
fn panose_any_fit() {
    let classification = Panose::parse(&PANOSE_ANY_FIT).unwrap();
    assert_eq!(classification, Panose::AnyFit);
}

#[test]
fn panose_italic() {
    let classification = Panose::parse(&PANOSE_ITALIC_TEXT).unwrap();
    assert!(classification.is_italic());

    let classification = Panose::parse(&PANOSE_ITALIC_HANDWRITTEN).unwrap();
    assert!(classification.is_italic());
}

#[test]
fn panose_no_fit() {
    let classification = Panose::parse(&PANOSE_NO_FIT).unwrap();
    assert_eq!(classification, Panose::NoFit);
}
