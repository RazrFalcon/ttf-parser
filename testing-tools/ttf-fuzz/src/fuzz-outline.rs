fn main() {
    afl::fuzz!(|data: &[u8]| {
        if let Some(font) = ttf_parser::Font::from_data(data, 0) {
            for id in 0..font.number_of_glyphs() {
                let _ = font.outline_glyph(ttf_parser::GlyphId(id), &mut Builder(0));
            }
        }
    });
}


struct Builder(usize);

impl ttf_parser::OutlineBuilder for Builder {
    #[inline]
    fn move_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }

    #[inline]
    fn line_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }

    #[inline]
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 2;
    }

    #[inline]
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 3;
    }

    #[inline]
    fn close(&mut self) {
        self.0 += 1;
    }
}
