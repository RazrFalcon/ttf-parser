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

    if let Some(table) = font.name_table() {
        println!("Family name: {:?}", table.family_name());
        println!("PostScript name: {:?}", table.poststript_name());
    }

    println!("Units per EM: {:?}", font.units_per_em());
    println!("Ascender: {}", font.ascender());
    println!("Descender: {}", font.descender());
    println!("Line gap: {}", font.line_gap());
    println!("Number of glyphs: {:?}", font.number_of_glyphs());
    println!("Underline: {:?}", font.underline_metrics());

    if let Some(table) = font.os2_table() {
        println!("X height: {:?}", table.x_height());
        println!("Weight: {:?}", table.weight());
        println!("Width: {:?}", table.width());
        println!("Regular: {:?}", table.is_regular());
        println!("Italic: {:?}", table.is_italic());
        println!("Bold: {:?}", table.is_bold());
        println!("Oblique: {:?}", table.is_oblique());
        println!("Strikeout: {:?}", table.strikeout_metrics());
        println!("Subscript: {:?}", table.subscript_metrics());
        println!("Superscript: {:?}", table.superscript_metrics());
    }

    println!("Valid: {:?}", font.is_valid());

    let end = time::precise_time_ns();
    println!("Elapsed: {:.6}s", (end - start) as f64 / 1_000_000_000.0);

    Ok(())
}
