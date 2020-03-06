use std::fs;

use ttf_parser as ttf;
use ttf::{Font, GlyphId};

struct Builder(svgtypes::Path);

impl Builder {
    fn new() -> Self {
        Builder(svgtypes::Path::new())
    }
}

impl ttf::OutlineBuilder for Builder {
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

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.push(svgtypes::PathSegment::CurveTo {
            abs: true,
            x1: x1 as f64, y1: y1 as f64,
            x2: x2 as f64, y2: y2 as f64,
            x: x as f64, y: y as f64
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
    assert_eq!(font.number_of_glyphs(), 7);
}

#[test]
fn outline_glyph_single_contour() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let mut builder = Builder::new();
    font.outline_glyph(GlyphId(0), &mut builder).unwrap();
    assert_eq!(builder.0.to_string(),
               "M 50 0 L 50 750 L 450 750 L 450 0 L 50 0 Z");
}

#[test]
fn outline_glyph_two_contours() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let mut builder = Builder::new();
    font.outline_glyph(GlyphId(1), &mut builder).unwrap();
    assert_eq!(builder.0.to_string(),
               "M 56 416 L 56 487 L 514 487 L 514 416 L 56 416 Z \
                M 56 217 L 56 288 L 514 288 L 514 217 L 56 217 Z");
}

#[test]
fn outline_glyph_composite() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let mut builder = Builder::new();
    font.outline_glyph(GlyphId(4), &mut builder).unwrap();
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
fn outline_glyph_single_point() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let mut builder = Builder::new();
    font.outline_glyph(GlyphId(5), &mut builder).unwrap();
    assert_eq!(builder.0.to_string(), "");
}

#[test]
fn outline_glyph_single_point_2() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let mut builder = Builder::new();
    font.outline_glyph(GlyphId(6), &mut builder).unwrap();
    assert_eq!(builder.0.to_string(),
               "M 332 468 L 197 468 L 197 0 L 109 0 L 109 468 L 15 468 L 15 509 L 109 539 \
               L 109 570 Q 109 674 155 719.5 Q 201 765 283 765 Q 315 765 341.5 759.5 \
               Q 368 754 387 747 L 364 678 Q 348 683 327 688 Q 306 693 284 693 \
               Q 240 693 218.5 663.5 Q 197 634 197 571 L 197 536 L 332 536 L 332 468 Z");
}

#[test]
fn outline_glyph_cff_flex() {
    let data = fs::read("tests/fonts/cff1_flex.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let mut builder = Builder::new();
    font.outline_glyph(GlyphId(1), &mut builder).unwrap();
    assert_eq!(builder.0.to_string(),
               "M 0 0 C 100 0 150 -20 250 -20 C 350 -20 400 0 500 0 C 500 100 520 150 520 250 \
                C 520 350 500 400 500 500 C 400 500 350 520 250 520 C 150 520 100 500 0 500 \
                C 0 400 -20 350 -20 250 C -20 150 0 100 0 0 Z M 50 50 C 50 130 34 170 34 250 \
                C 34 330 50 370 50 450 C 130 450 170 466 250 466 C 330 466 370 450 450 450 \
                C 450 370 466 330 466 250 C 466 170 450 130 450 50 C 370 50 330 34 250 34 \
                C 170 34 130 50 50 50 Z");
}

#[test]
fn outline_glyph_cff_1() {
    let data = fs::read("tests/fonts/cff1_dotsect.nohints.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    let mut builder = Builder::new();
    font.outline_glyph(GlyphId(1), &mut builder).unwrap();
    assert_eq!(builder.0.to_string(),
               "M 82 0 L 164 0 L 164 486 L 82 486 Z M 124 586 C 156 586 181 608 181 639 \
                C 181 671 156 692 124 692 C 92 692 67 671 67 639 C 67 608 92 586 124 586 Z");
}

#[test]
fn glyf_bbox() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.outline_glyph(GlyphId(0), &mut Builder::new()).unwrap(),
               font.glyph_bounding_box(GlyphId(0)).unwrap());
}

#[test]
fn cff_bbox() {
    let data = fs::read("tests/fonts/cff1_dotsect.nohints.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.outline_glyph(GlyphId(1), &mut Builder::new()).unwrap(),
               font.glyph_bounding_box(GlyphId(1)).unwrap());
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
    assert_eq!(font.underline_metrics().unwrap(),
               ttf::LineMetrics { position: -75, thickness: 50 });
}

#[test]
fn weight() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.weight(), ttf::Weight::SemiBold);
}

#[test]
fn width() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.width(), ttf::Width::Expanded);
}

#[test]
fn invalid_width() {
    let data = fs::read("tests/fonts/os2-invalid-width.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.width(), ttf::Width::default());
}

#[test]
fn x_height() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.x_height().unwrap(), 536);
}

#[test]
fn no_x_height() {
    let data = fs::read("tests/fonts/os2-v0.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert!(font.x_height().is_none());
}

#[test]
fn strikeout_metrics() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.strikeout_metrics().unwrap(),
               ttf::LineMetrics { position: 322, thickness: 50 });
}

#[test]
fn is_regular() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.is_regular(), true);
}

#[test]
fn is_italic() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.is_italic(), false);
}

#[test]
fn os2_is_bold() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.is_bold(), false);
}

#[test]
fn is_oblique() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.is_oblique(), false);
}

#[test]
fn no_is_oblique() {
    let data = fs::read("tests/fonts/os2-v0.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.is_oblique(), false);
}

#[test]
fn subscript_metrics() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(
        font.subscript_metrics().unwrap(),
        ttf::ScriptMetrics { x_size: 650, y_size: 600, x_offset: 0, y_offset: 75 },
    );
}

#[test]
fn superscript_metrics() {
    let data = fs::read("tests/fonts/glyphs.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(
        font.superscript_metrics().unwrap(),
        ttf::ScriptMetrics { x_size: 550, y_size: 800, x_offset: 100, y_offset: 350 },
    );
}

#[test]
fn glyph_class_1() {
    let data = fs::read("tests/fonts/TestGPOSThree.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_class(GlyphId(0)), None);
}

#[test]
fn glyph_class_2() {
    let data = fs::read("tests/fonts/TestGPOSThree.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_class(GlyphId(2)), Some(ttf::GlyphClass::Base));
}

#[test]
fn glyph_class_3() {
    let data = fs::read("tests/fonts/TestGPOSThree.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_class(GlyphId(4)), Some(ttf::GlyphClass::Mark));
}

#[test]
fn glyph_mark_attachment_class_1() {
    let data = fs::read("tests/fonts/TestGPOSThree.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_mark_attachment_class(GlyphId(0)), ttf::Class(0));
}

#[test]
fn glyph_mark_attachment_class_2() {
    let data = fs::read("tests/fonts/TestGPOSThree.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_mark_attachment_class(GlyphId(4)), ttf::Class(1));
}

#[test]
fn glyph_index_f00_01() {
    let data = fs::read("tests/fonts/cmap0_font1.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_index('4'), Some(GlyphId(17)));
    assert_eq!(font.glyph_index('5'), Some(GlyphId(56)));
    assert_eq!(font.glyph_index('6'), Some(GlyphId(12)));
}

#[test]
fn glyph_index_f02_01() {
    let data = fs::read("tests/fonts/cmap2_font1.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_index('4'), Some(GlyphId(17)));
    assert_eq!(font.glyph_index('5'), Some(GlyphId(56)));
    assert_eq!(font.glyph_index('6'), Some(GlyphId(12)));
    assert_eq!(font.glyph_index('\u{8432}'), Some(GlyphId(20)));
    assert_eq!(font.glyph_index('\u{8433}'), Some(GlyphId(21)));
    assert_eq!(font.glyph_index('\u{8434}'), Some(GlyphId(22)));
    assert_eq!(font.glyph_index('\u{9232}'), Some(GlyphId(23)));
    assert_eq!(font.glyph_index('\u{9233}'), Some(GlyphId(24)));
    assert_eq!(font.glyph_index('\u{9234}'), Some(GlyphId(25)));
}

#[test]
fn glyph_index_f04_01() {
    let data = fs::read("tests/fonts/TestCMAP14.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_index('芦'), Some(GlyphId(1)));
}

#[test]
fn glyph_index_f06_01() {
    let data = fs::read("tests/fonts/cmap-6.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();

    assert_eq!(font.glyph_index('"'), Some(GlyphId(6)));
    assert_eq!(font.glyph_index('#'), Some(GlyphId(7)));
    assert_eq!(font.glyph_index('$'), Some(GlyphId(5)));

    // Char before character map.
    // Should not overflow.
    assert_eq!(font.glyph_index('!'), None);

    // Char after character map.
    // Should not read out of bounds.
    assert_eq!(font.glyph_index('A'), None);
}

#[test]
fn glyph_index_f10_01() {
    let data = fs::read("tests/fonts/cmap-10.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();

    assert_eq!(font.glyph_index('\u{109423}'), Some(GlyphId(26)));
    assert_eq!(font.glyph_index('\u{109424}'), Some(GlyphId(27)));
    assert_eq!(font.glyph_index('\u{109425}'), Some(GlyphId(32)));

    // Char before character map.
    // Should not overflow.
    assert_eq!(font.glyph_index('!'), None);

    // Char after character map.
    // Should not read out of bounds.
    assert_eq!(font.glyph_index('\u{109426}'), None);
}

#[test]
fn glyph_index_f12_01() {
    let data = fs::read("tests/fonts/vmtx.ttf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_index('明'), Some(GlyphId(1)));
}

#[test]
fn glyph_variation_index_01() {
    let data = fs::read("tests/fonts/TestCMAP14.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();
    assert_eq!(font.glyph_variation_index('芦', '\u{E0101}'), Some(GlyphId(2)));
}

#[test]
fn glyphs_kerning_01() {
    let data = fs::read("tests/fonts/TestKERNOne.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();

    let t_id = font.glyph_index('T').unwrap();
    let u_id = font.glyph_index('u').unwrap();
    let dotless_i_id = font.glyph_index('\u{131}').unwrap();

    assert_eq!(font.glyphs_kerning(t_id, dotless_i_id), Some(-200));
    assert_eq!(font.glyphs_kerning(t_id, u_id), Some(-200));
    assert_eq!(font.glyphs_kerning(dotless_i_id, t_id), Some(-200));
    assert_eq!(font.glyphs_kerning(dotless_i_id, dotless_i_id), Some(500));
    assert_eq!(font.glyphs_kerning(u_id, t_id), Some(-200));
}

#[test]
fn glyphs_kerning_02() {
    let data = fs::read("tests/fonts/TestKERNOne.otf").unwrap();
    let font = Font::from_data(&data, 0).unwrap();

    // Random GID's.
    assert_eq!(font.glyphs_kerning(GlyphId(0), GlyphId(0)), None);
    assert_eq!(font.glyphs_kerning(GlyphId(0), GlyphId(100)), None);
}
