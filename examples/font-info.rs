fn main() {
    if let Err(e) = process() {
        eprintln!("Error: {}.", e);
        std::process::exit(1);
    }
}

fn process() -> Result<(), Box<std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage:\n\tfont-info font.ttf");
        std::process::exit(1);
    }

    let font_data = std::fs::read(&args[1])?;

    let start = time::precise_time_ns();

    let font = ttf_parser::Font::from_data(&font_data, 0)?;

    println!("Family name: {:?}", font.family_name());
    println!("PostScript name: {:?}", font.post_script_name());
    println!("Units per EM: {:?}", font.units_per_em());
    println!("Ascender: {:?}", font.ascender());
    println!("Descender: {:?}", font.descender());
    println!("Line gap: {:?}", font.line_gap());
    println!("Number of glyphs: {:?}", font.number_of_glyphs());
    println!("Underline: {:?}", font.underline_metrics());
    println!("X height: {:?}", font.x_height());
    println!("Weight: {:?}", font.weight());
    println!("Width: {:?}", font.width());
    println!("Regular: {:?}", font.is_regular());
    println!("Italic: {:?}", font.is_italic());
    println!("Bold: {:?}", font.is_bold());
    println!("Oblique: {:?}", font.is_oblique());
    println!("Strikeout: {:?}", font.strikeout_metrics());
    println!("Subscript: {:?}", font.subscript_metrics());
    println!("Superscript: {:?}", font.superscript_metrics());
    println!("Valid: {:?}", font.is_valid());

    let end = time::precise_time_ns();
    println!("Elapsed: {:.6}s", (end - start) as f64 / 1_000_000_000.0);

    Ok(())
}
