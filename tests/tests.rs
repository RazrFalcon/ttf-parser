use std::fs;

use ttf_parser::Font;

struct Builder(svgtypes::Path);

impl Builder {
    fn new() -> Self {
        Builder(svgtypes::Path::new())
    }
}

impl ttf_parser::glyf::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(svgtypes::PathSegment::MoveTo { abs: true, x: x as f64, y: y as f64 });
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push(svgtypes::PathSegment::LineTo { abs: true, x: x as f64, y: y as f64 });
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.push(svgtypes::PathSegment::Quadratic {
            abs: true, x1: x1 as f64, y1: y1 as f64, x: x as f64, y: y as f64
        });
    }

    fn close(&mut self) {
        self.0.push(svgtypes::PathSegment::ClosePath { abs: true });
    }
}


#[test]
fn number_of_glyphs() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.number_of_glyphs(), 5);
}

#[test]
fn glyph_outline_single_contour() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let glyph = font.glyph(0).unwrap();

    let mut builder = Builder::new();
    glyph.outline(&mut builder);
    assert_eq!(builder.0.to_string(),
               "M 50 0 L 50 750 L 450 750 L 450 0 L 50 0 Z");
}

#[test]
fn glyph_outline_two_contours() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let glyph = font.glyph(1).unwrap();

    let mut builder = Builder::new();
    glyph.outline(&mut builder);
    assert_eq!(builder.0.to_string(),
               "M 56 416 L 56 487 L 514 487 L 514 416 L 56 416 Z \
                M 56 217 L 56 288 L 514 288 L 514 217 L 56 217 Z");
}

#[test]
fn glyph_outline_composite() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let glyph = font.glyph(4).unwrap();

    let mut builder = Builder::new();
    glyph.outline(&mut builder);
    assert_eq!(builder.0.to_string(),
               "M 332 468 L 197 468 L 197 0 L 109 0 L 109 468 L 15 468 L 15 509 L 109 539 \
               L 109 570 Q 109 674 155 719.5 Q 201 765 283 765 Q 315 765 341.5 759.5 \
               Q 368 754 387 747 L 364 678 Q 348 683 327 688 Q 306 693 284 693 \
               Q 240 693 218.5 663.5 Q 197 634 197 571 L 197 536 L 332 536 L 332 468 Z \
               M 474 737 Q 494 737 509.5 723.5 Q 525 710 525 681 Q 525 653 509.5 639 \
               Q 494 625 474 625 Q 452 625 437 639 Q 422 653 422 681 Q 422 710 437 723.5 \
               Q 452 737 474 737 Z M 517 536 L 517 0 L 429 0 L 429 536 L 517 536 Z");
}

#[test]
fn checksum() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert!(font.is_valid().is_ok());
}

#[test]
fn units_per_em() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.units_per_em(), Some(1000));
}

#[test]
fn units_per_em_invalid() {
    let data = fs::read("tests/fonts/invalid-em.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.units_per_em(), None);
}

#[test]
fn ascender() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.ascender(), 900);
}

#[test]
fn descender() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.descender(), -300);
}

#[test]
fn line_gap() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.line_gap(), 200);
}

#[test]
fn underline_metrics() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.underline_metrics(),
               ttf_parser::LineMetrics { position: -75, thickness: 50 });
}

#[test]
fn os2_weight() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.weight(), ttf_parser::os2::Weight::SemiBold);
}

#[test]
fn os2_width() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.width().unwrap(), ttf_parser::os2::Width::Expanded);
}

#[test]
fn os2_invalid_width() {
    let data = fs::read("tests/fonts/os2-invalid-width.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.width(), None);
}

#[test]
fn os2_x_height() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.x_height().unwrap(), 536);
}

#[test]
fn os2_no_x_height() {
    let data = fs::read("tests/fonts/os2-v0.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.x_height(), None);
}

#[test]
fn os2_strikeout_metrics() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.strikeout_metrics(),
               ttf_parser::LineMetrics { position: 322, thickness: 50 });
}

#[test]
fn os2_is_regular() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.is_regular(), true);
}

#[test]
fn os2_is_italic() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.is_italic(), false);
}

#[test]
fn os2_is_bold() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.is_bold(), false);
}

#[test]
fn os2_is_oblique() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.is_oblique(), Some(false));
}

#[test]
fn os2_no_is_oblique() {
    let data = fs::read("tests/fonts/os2-v0.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(table.is_oblique(), None);
}

#[test]
fn os2_subscript_metrics() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(
        table.subscript_metrics(),
        ttf_parser::os2::ScriptMetrics { x_size: 650, y_size: 600, x_offset: 0, y_offset: 75 },
    );
}

#[test]
fn os2_superscript_metrics() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let table = font.os2_table().unwrap();
    assert_eq!(
        table.superscript_metrics(),
        ttf_parser::os2::ScriptMetrics { x_size: 550, y_size: 800, x_offset: 100, y_offset: 350 },
    );
}
