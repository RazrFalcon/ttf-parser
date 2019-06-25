use std::fmt::Write;

const FONT_SIZE: f64 = 128.0;
const COLUMNS: u32 = 50;

fn main() {
    if let Err(e) = process() {
        eprintln!("Error: {}.", e);
        std::process::exit(1);
    }
}

fn process() -> Result<(), Box<std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage:\n\tfont2svg font.ttf out.svg");
        std::process::exit(1);
    }

    let font_data = std::fs::read(&args[1])?;
    let font = ttf_parser::Font::from_data(&font_data, 0)?;
    let units_per_em = font.units_per_em().ok_or("invalid units per em")?;
    let scale = FONT_SIZE / units_per_em as f64;

    let cell_size = font.height() as f64 * FONT_SIZE / units_per_em as f64;
    let rows = (font.number_of_glyphs() as f64 / COLUMNS as f64).ceil() as u32;

    let mut output = String::new();
    output.write_fmt(format_args!(
        "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {} {}'>\n",
        cell_size * COLUMNS as f64, cell_size * rows as f64
    )).unwrap();

    draw_grid(font.number_of_glyphs(), cell_size, &mut output);

    let mut row = 0;
    let mut column = 0;
    for id in 0..font.number_of_glyphs() {
        glyph_to_path(
            column as f64 * cell_size,
            row as f64 * cell_size,
            &font,
            ttf_parser::GlyphId(id),
            cell_size,
            scale,
            &mut output,
        );

        column += 1;
        if column == COLUMNS {
            column = 0;
            row += 1;
        }
    }

    output.write_fmt(format_args!("</svg>\n")).unwrap();

    std::fs::write(&args[2], output)?;

    Ok(())
}

fn draw_grid(
    n_glyphs: u16,
    cell_size: f64,
    output: &mut String,
) {
    let columns = COLUMNS;
    let rows = (n_glyphs as f64 / columns as f64).ceil() as u32;

    let width = columns as f64 * cell_size;
    let height = rows as f64 * cell_size;

    let mut path_builder = svgtypes::PathBuilder::new();

    let mut x = 0.0;
    for _ in 0..=columns {
        path_builder = path_builder
            .move_to(x, 0.0)
            .line_to(x, height);
        x += cell_size;
    }

    let mut y = 0.0;
    for _ in 0..=rows {
        path_builder = path_builder
            .move_to(0.0, y)
            .line_to(width, y);
        y += cell_size;
    }

    output.write_fmt(format_args!(
        "<path fill='none' stroke='black' stroke-width='5' d='{}'/>\n",
        path_builder.finalize()
    )).unwrap();
}

fn glyph_to_path(
    x: f64,
    y: f64,
    font: &ttf_parser::Font,
    glyph_id: ttf_parser::GlyphId,
    cell_size: f64,
    scale: f64,
    output: &mut String,
) {
    let glyph = match font.glyph(glyph_id) {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut builder = Builder(svgtypes::Path::new());
    glyph.outline(&mut builder);

    let path = builder.0;
    if path.is_empty() {
        return;
    }

    let dx = (cell_size - font.glyph_hor_metrics(glyph_id).unwrap().advance as f64 * scale) / 2.0;

    let mut ts = svgtypes::Transform::default();
    ts.translate(x + dx, y + font.height() as f64 * scale + font.descender() as f64 * scale);
    ts.scale(1.0, -1.0);
    ts.scale(scale, scale);

    output.write_fmt(format_args!("<path d='{}' transform='{}'/>\n", path, ts)).unwrap();
}

struct Builder(svgtypes::Path);

impl ttf_parser::OutlineBuilder for Builder {
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
