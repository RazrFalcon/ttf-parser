use ttf_parser as ttf;

fn units_per_em(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.units_per_em());
        }
    })
}

fn width(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.width());
        }
    })
}

fn ascender(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.ascender());
        }
    })
}

fn underline_metrics(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.underline_metrics().unwrap());
        }
    })
}

fn strikeout_metrics(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.strikeout_metrics().unwrap());
        }
    })
}

fn subscript_metrics(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.subscript_metrics().unwrap());
        }
    })
}

fn x_height(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.x_height().unwrap());
        }
    })
}

fn glyph_hor_advance(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.glyph_hor_advance(ttf::GlyphId(2)).unwrap());
        }
    })
}

fn glyph_hor_side_bearing(bencher: &mut bencher::Bencher) {
    let font_data = std::fs::read("fonts/SourceSansPro-Regular.ttf").unwrap();
    let font = ttf::Font::from_data(&font_data, 0).unwrap();
    bencher.iter(|| {
        for _ in 0..1000 {
            bencher::black_box(font.glyph_hor_side_bearing(ttf::GlyphId(2)).unwrap());
        }
    })
}

bencher::benchmark_group!(perf,
    units_per_em,
    width,
    ascender,
    underline_metrics,
    strikeout_metrics,
    subscript_metrics,
    x_height,
    glyph_hor_advance,
    glyph_hor_side_bearing
);
bencher::benchmark_main!(perf);
