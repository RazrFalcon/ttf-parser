use std::path::PathBuf;
use std::io::Write;

use ttf_parser as ttf;
use svgtypes::WriteBuffer;

const FONT_SIZE: f64 = 128.0;
const COLUMNS: u32 = 100;

const HELP: &str = "\
Usage:
    font2svg font.ttf out.svg
    font2svg --variations 'wght:500;wdth:200' font.ttf out.svg
";

struct Args {
    #[allow(dead_code)] variations: Vec<ttf::Variation>,
    ttf_path: PathBuf,
    svg_path: PathBuf,
}

fn main() {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}.", e);
            print!("{}", HELP);
            std::process::exit(1);
        }
    };

    if let Err(e) = process(args) {
        eprintln!("Error: {}.", e);
        std::process::exit(1);
    }
}

fn parse_args() -> Result<Args, Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }

    let variations = args.opt_value_from_fn("--variations", parse_variations)?;
    let free = args.free()?;
    if free.len() != 2 {
        return Err("invalid number of arguments".into());
    }

    Ok(Args {
        variations: variations.unwrap_or_default(),
        ttf_path: PathBuf::from(&free[0]),
        svg_path: PathBuf::from(&free[1]),
    })
}

fn parse_variations(s: &str) -> Result<Vec<ttf::Variation>, &'static str> {
    let mut variations = Vec::new();
    for part in s.split(';') {
        let mut iter = part.split(':');

        let axis = iter.next().ok_or("failed to parse a variation")?;
        let axis = ttf::Tag::from_bytes_lossy(axis.as_bytes());

        let value = iter.next().ok_or("failed to parse a variation")?;
        let value: f32 = value.parse().map_err(|_| "failed to parse a variation")?;

        variations.push(ttf::Variation { axis, value });
    }

    Ok(variations)
}

fn process(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let font_data = std::fs::read(&args.ttf_path)?;

    // Exclude IO operations.
    let now = std::time::Instant::now();

    #[allow(unused_mut)]
    let mut face = ttf::Face::from_slice(&font_data, 0)?;
    if face.is_variable() {
        #[cfg(feature = "variable-fonts")] {
            for variation in args.variations {
                face.set_variation(variation.axis, variation.value)
                    .ok_or("failed to create variation coordinates")?;
            }
        }
    }

    let units_per_em = face.units_per_em().ok_or("invalid units per em")?;
    let scale = FONT_SIZE / units_per_em as f64;

    let cell_size = face.height() as f64 * FONT_SIZE / units_per_em as f64;
    let rows = (face.number_of_glyphs() as f64 / COLUMNS as f64).ceil() as u32;

    let mut svg = xmlwriter::XmlWriter::with_capacity(
        face.number_of_glyphs() as usize * 512,
        xmlwriter::Options::default(),
    );
    svg.start_element("svg");
    svg.write_attribute("xmlns", "http://www.w3.org/2000/svg");
    svg.write_attribute("xmlns:xlink", "http://www.w3.org/1999/xlink");
    svg.write_attribute_fmt(
        "viewBox",
        format_args!("{} {} {} {}", 0, 0, cell_size * COLUMNS as f64, cell_size * rows as f64),
    );

    draw_grid(face.number_of_glyphs(), cell_size, &mut svg);

    let mut path_buf = svgtypes::Path::with_capacity(64);
    let mut row = 0;
    let mut column = 0;
    for id in 0..face.number_of_glyphs() {
        let x = column as f64 * cell_size;
        let y = row as f64 * cell_size;

        svg.start_element("text");
        svg.write_attribute("x", &(x + 2.0));
        svg.write_attribute("y", &(y + cell_size - 4.0));
        svg.write_attribute("font-size", "36");
        svg.write_attribute("fill", "gray");
        svg.write_text_fmt(format_args!("{}", &id));
        svg.end_element();

        if let Some(img) = face.glyph_raster_image(ttf::GlyphId(id), std::u16::MAX) {
            svg.start_element("image");
            svg.write_attribute("x", &(x + 2.0 + img.x as f64));
            svg.write_attribute("y", &(y - img.y as f64));
            svg.write_attribute("width", &img.width);
            svg.write_attribute("height", &img.height);
            svg.write_attribute_raw("xlink:href", |buf| {
                buf.extend_from_slice(b"data:image/png;base64, ");

                let mut enc = base64::write::EncoderWriter::new(buf, base64::STANDARD);
                enc.write_all(img.data).unwrap();
                enc.finish().unwrap();
            });
            svg.end_element();
        } else if let Some(img) = face.glyph_svg_image(ttf::GlyphId(id)) {
            svg.start_element("image");
            svg.write_attribute("x", &(x + 2.0));
            svg.write_attribute("y", &(y + cell_size));
            svg.write_attribute("width", &cell_size);
            svg.write_attribute("height", &cell_size);
            svg.write_attribute_raw("xlink:href", |buf| {
                buf.extend_from_slice(b"data:image/svg+xml;base64, ");

                let mut enc = base64::write::EncoderWriter::new(buf, base64::STANDARD);
                enc.write_all(img).unwrap();
                enc.finish().unwrap();
            });
            svg.end_element();
        } else {
            glyph_to_path(
                x,
                y,
                &face,
                ttf::GlyphId(id),
                cell_size,
                scale,
                &mut svg,
                &mut path_buf,
            );
        }

        column += 1;
        if column == COLUMNS {
            column = 0;
            row += 1;
        }
    }

    println!("Elapsed: {}ms", now.elapsed().as_micros() as f64 / 1000.0);

    std::fs::write(&args.svg_path, &svg.end_document())?;

    Ok(())
}

fn draw_grid(
    n_glyphs: u16,
    cell_size: f64,
    svg: &mut xmlwriter::XmlWriter,
) {
    let columns = COLUMNS;
    let rows = (n_glyphs as f64 / columns as f64).ceil() as u32;

    let width = columns as f64 * cell_size;
    let height = rows as f64 * cell_size;

    svg.start_element("path");
    svg.write_attribute("fill", "none");
    svg.write_attribute("stroke", "black");
    svg.write_attribute("stroke-width", "5");

    let mut path = svgtypes::Path::with_capacity(256);

    let mut x = 0.0;
    for _ in 0..=columns {
        path.push_move_to(x, 0.0);
        path.push_line_to(x, height);
        x += cell_size;
    }

    let mut y = 0.0;
    for _ in 0..=rows {
        path.push_move_to(0.0, y);
        path.push_line_to(width, y);
        y += cell_size;
    }

    svg.write_attribute_raw("d", |buf| path.write_buf(buf));
    svg.end_element();
}

fn glyph_to_path(
    x: f64,
    y: f64,
    face: &ttf::Face,
    glyph_id: ttf::GlyphId,
    cell_size: f64,
    scale: f64,
    svg: &mut xmlwriter::XmlWriter,
    path_buf: &mut svgtypes::Path,
) {
    path_buf.clear();
    let mut builder = Builder(path_buf);
    let bbox = match face.outline_glyph(glyph_id, &mut builder) {
        Some(v) => v,
        None => return,
    };

    let bbox_w = (bbox.x_max as f64 - bbox.x_min as f64) * scale;
    let dx = (cell_size - bbox_w) / 2.0;
    let y = y + cell_size + face.descender() as f64 * scale;

    let mut ts = svgtypes::Transform::default();
    ts.translate(x + dx, y);
    ts.scale(1.0, -1.0);
    ts.scale(scale, scale);

    svg.start_element("path");
    svg.write_attribute_raw("d", |buf| path_buf.write_buf(buf));
    svg.write_attribute_raw("transform", |buf| ts.write_buf(buf));
    svg.end_element();

    {
        let bbox_h = (bbox.y_max as f64 - bbox.y_min as f64) * scale;
        let bbox_x = x + dx + bbox.x_min as f64 * scale;
        let bbox_y = y - bbox.y_max as f64 * scale;

        svg.start_element("rect");
        svg.write_attribute("x", &bbox_x);
        svg.write_attribute("y", &bbox_y);
        svg.write_attribute("width", &bbox_w);
        svg.write_attribute("height", &bbox_h);
        svg.write_attribute("fill", "none");
        svg.write_attribute("stroke", "green");
        svg.end_element();
    }
}

struct Builder<'a>(&'a mut svgtypes::Path);

impl ttf::OutlineBuilder for Builder<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push_move_to(x as f64, y as f64);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push_line_to(x as f64, y as f64);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.0.push_quad_to(x1 as f64, y1 as f64, x as f64, y as f64);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.0.push_curve_to(x1 as f64, y1 as f64, x2 as f64, y2 as f64, x as f64, y as f64);
    }

    fn close(&mut self) {
        self.0.push_close_path();
    }
}
