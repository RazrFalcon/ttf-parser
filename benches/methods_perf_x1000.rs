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

bencher::benchmark_group!(perf,
    units_per_em,
    width,
    ascender,
    strikeout_metrics,
    subscript_metrics,
    x_height
);
bencher::benchmark_main!(perf);
