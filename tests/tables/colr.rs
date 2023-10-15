use crate::{convert, Unit::*};
use ttf_parser::colr::{self, Painter};
use ttf_parser::{cpal, GlyphId, RgbaColor};

#[test]
fn basic() {
    let cpal_data = convert(&[
        UInt16(0),  // version
        UInt16(3),  // number of palette entries
        UInt16(1),  // number of palettes
        UInt16(3),  // number of colors
        UInt32(14), // offset to colors
        UInt16(0),  // index of palette 0's first color
        UInt8(10), UInt8(15), UInt8(20), UInt8(25), // color 0
        UInt8(30), UInt8(35), UInt8(40), UInt8(45), // color 1
        UInt8(50), UInt8(55), UInt8(60), UInt8(65), // color 2
    ]);

    let colr_data = convert(&[
        UInt16(0),  // version
        UInt16(3),  // number of base glyphs
        UInt32(14), // offset to base glyphs
        UInt32(32), // offset to layers
        UInt16(4),  // number of layers
        UInt16(2), UInt16(2), UInt16(2), // base glyph 0 (id 2)
        UInt16(3), UInt16(0), UInt16(3), // base glyph 1 (id 3)
        UInt16(7), UInt16(1), UInt16(1), // base glyph 2 (id 7)
        UInt16(10), UInt16(2), // layer 0
        UInt16(11), UInt16(1), // layer 1
        UInt16(12), UInt16(2), // layer 2
        UInt16(13), UInt16(0), // layer 3
    ]);

    let cpal = cpal::Table::parse(&cpal_data).unwrap();
    let colr = colr::Table::parse(cpal, &colr_data).unwrap();
    let paint = |id| {
        let mut painter = VecPainter(vec![]);
        colr.paint(GlyphId(id), 0, &mut painter).map(|_| painter.0)
    };

    let a = RgbaColor::new(20, 15, 10, 25);
    let b = RgbaColor::new(40, 35, 30, 45);
    let c = RgbaColor::new(60, 55, 50, 65);

    assert_eq!(cpal.get(0, 0), Some(a));
    assert_eq!(cpal.get(0, 1), Some(b));
    assert_eq!(cpal.get(0, 2), Some(c));
    assert_eq!(cpal.get(0, 3), None);
    assert_eq!(cpal.get(1, 0), None);

    assert!(!colr.contains(GlyphId(1)));
    assert!(colr.contains(GlyphId(2)));
    assert!(colr.contains(GlyphId(3)));
    assert!(!colr.contains(GlyphId(4)));
    assert!(!colr.contains(GlyphId(5)));
    assert!(!colr.contains(GlyphId(6)));
    assert!(colr.contains(GlyphId(7)));

    assert_eq!(paint(1), None);

    assert_eq!(paint(2).unwrap(), vec![
        Command::Outline(12),
        Command::PaintColor(c),
        Command::Outline(13),
        Command::PaintColor(a),
    ]);

    assert_eq!(paint(3).unwrap(), vec![
        Command::Outline(10),
        Command::PaintColor(c),
        Command::Outline(11),
        Command::PaintColor(b),
        Command::Outline(12),
        Command::PaintColor(c),
    ]);

    assert_eq!(paint(7).unwrap(), vec![
        Command::Outline(11),
        Command::PaintColor(b),
    ]);
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Command {
    Outline(u16),
    Foreground,
    PaintColor(RgbaColor),
}

struct VecPainter(Vec<Command>);

impl Painter for VecPainter {
    fn outline(&mut self, glyph_id: GlyphId) {
        self.0.push(Command::Outline(glyph_id.0));
    }

    fn paint_foreground(&mut self) {
        self.0.push(Command::Foreground);
    }

    fn paint_color(&mut self, color: RgbaColor) {
        self.0.push(Command::PaintColor(color));
    }
}
