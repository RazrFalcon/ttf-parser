use crate::{convert, Unit::*};
use ttf_parser::colr::{self, ClipBox, CompositeMode, GradientExtend, Paint, Painter};
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
        colr.paint(GlyphId(id), 0, &mut painter, &[], RgbaColor::new(0, 0, 0, 255)).map(|_| painter.0)
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

    let a = CustomPaint::Solid(a);
    let b = CustomPaint::Solid(b);
    let c = CustomPaint::Solid(c);

    assert_eq!(paint(1), None);

    assert_eq!(
        paint(2).unwrap(), vec![
        Command::OutlineGlyph(GlyphId(12)),
        Command::Paint(c.clone()),
        Command::OutlineGlyph(GlyphId(13)),
        Command::Paint(a.clone())]
    );

    assert_eq!(paint(3).unwrap(), vec![
        Command::OutlineGlyph(GlyphId(10)),
        Command::Paint(c.clone()),
        Command::OutlineGlyph(GlyphId(11)),
        Command::Paint(b.clone()),
        Command::OutlineGlyph(GlyphId(12)),
        Command::Paint(c.clone()),
    ]);

    assert_eq!(paint(7).unwrap(), vec![
        Command::OutlineGlyph(GlyphId(11)),
        Command::Paint(b.clone()),
    ]);
}

#[derive(Clone, Debug, PartialEq)]
struct CustomStop(f32, RgbaColor);

#[derive(Clone, Debug, PartialEq)]
enum CustomPaint {
    Solid(RgbaColor),
    LinearGradient(f32, f32, f32, f32, f32, f32, GradientExtend, Vec<CustomStop>),
    RadialGradient(f32, f32, f32, f32, f32, f32, GradientExtend, Vec<CustomStop>),
    SweepGradient(f32, f32, f32, f32, GradientExtend, Vec<CustomStop>),
}

#[derive(Clone, Debug, PartialEq)]
enum Command {
    OutlineGlyph(GlyphId),
    Paint(CustomPaint),
    PushLayer(CompositeMode),
    PopLayer,
    Translate(f32, f32),
    Scale(f32, f32),
    Rotate(f32),
    Skew(f32, f32),
    Transform(ttf_parser::Transform),
    PopTransform,
    PushClip,
    PushClipBox(ClipBox),
    PopClip
}

struct VecPainter(Vec<Command>);

impl<'a> Painter<'a> for VecPainter {
    fn outline_glyph(&mut self, glyph_id: GlyphId) {
        self.0.push(Command::OutlineGlyph(glyph_id));
    }

    fn paint(&mut self, paint: Paint<'a>) {
        let custom_paint = match paint {
            Paint::Solid(color) => CustomPaint::Solid(color),
            Paint::LinearGradient(lg) => CustomPaint::LinearGradient(lg.x0, lg.y0,
                                                                     lg.x1, lg.y1,
                                                                     lg.x2, lg.y2,
                                                                     lg.extend, lg.stops(0, &[]).map(|stop| CustomStop(stop.stop_offset, stop.color)).collect()),
            Paint::RadialGradient(rg) => CustomPaint::RadialGradient(rg.x0, rg.y0,
                                                                     rg.r0, rg.r1,
                                                                     rg.x1, rg.y1,
                                                                     // TODO: Make less ugly
                                                                     rg.extend, rg.stops(0, &[],).map(|stop| CustomStop(stop.stop_offset, stop.color)).collect()),
            Paint::SweepGradient(sg) => CustomPaint::SweepGradient(sg.center_x, sg.center_y,
                                                                     sg.start_angle, sg.end_angle,
                                                                     sg.extend, sg.stops(0, &[],).map(|stop| CustomStop(stop.stop_offset, stop.color)).collect()),
        };

        self.0.push(Command::Paint(custom_paint));
    }

    fn push_layer(&mut self, mode: colr::CompositeMode) {
        self.0.push(Command::PushLayer(mode));
    }

    fn pop_layer(&mut self) {
        self.0.push(Command::PopLayer)
    }

    fn translate(&mut self, tx: f32, ty: f32) {
        self.0.push(Command::Translate(tx, ty))
    }

    fn scale(&mut self, sx: f32, sy: f32) {
       self.0.push(Command::Scale(sx, sy))
    }

    fn rotate(&mut self, angle: f32) {
       self.0.push(Command::Rotate(angle))
    }

    fn skew(&mut self, skew_x: f32, skew_y: f32) {
       self.0.push(Command::Skew(skew_x, skew_y))
    }

    fn transform(&mut self, transform: ttf_parser::Transform) {
       self.0.push(Command::Transform(transform))
    }

    fn pop_transform(&mut self) {
       self.0.push(Command::PopTransform)
    }

    fn push_clip(&mut self) {
       self.0.push(Command::PushClip)
    }

    fn push_clip_box(&mut self, clipbox: ClipBox) {
       self.0.push(Command::PushClipBox(clipbox))
    }

    fn pop_clip(&mut self) {
        self.0.push(Command::PopClip)
    }

}
