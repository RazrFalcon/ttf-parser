use ttf_parser as ttf;

fn outline_glyph_8_from_glyf(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        font.outline_glyph(ttf::GlyphId(8), &mut Builder(0))
    })
}

fn outline_glyph_276_from_glyf(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    let mut b = Builder(0);
    bencher.iter(|| {
        font.outline_glyph(ttf::GlyphId(276), &mut b)
    })
}

fn outline_glyph_8_from_cff(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.otf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        font.outline_glyph(ttf::GlyphId(8), &mut Builder(0))
    })
}

fn outline_glyph_276_from_cff(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.otf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        font.outline_glyph(ttf::GlyphId(276), &mut Builder(0))
    })
}

fn glyph_index_u41(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        bencher::black_box(font.glyph_index('A'))
    })
}

fn family_name(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        bencher::black_box(font.family_name())
    })
}

fn units_per_em(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        bencher::black_box(font.units_per_em())
    })
}

fn width(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        bencher::black_box(font.width())
    })
}

fn glyph_2_hor_metrics(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        bencher::black_box(font.glyph_hor_metrics(ttf::GlyphId(2)).unwrap());
    })
}

fn ascender(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        bencher::black_box(font.ascender());
    })
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

bencher::benchmark_group!(perf,
    outline_glyph_8_from_glyf,
    outline_glyph_276_from_glyf,
    outline_glyph_8_from_cff,
    outline_glyph_276_from_cff,
    glyph_index_u41,
    family_name,
    units_per_em,
    width,
    glyph_2_hor_metrics,
    ascender
);
bencher::benchmark_main!(perf);
