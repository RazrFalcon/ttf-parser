fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage:\n\tfont-info font.ttf");
        std::process::exit(1);
    }

    let font_data = std::fs::read(&args[1]).unwrap();

    let now = std::time::Instant::now();

    let font = match ttf_parser::Font::from_data(&font_data, 0) {
        Some(f) => f,
        None => {
            eprint!("Error: failed to open a font.");
            std::process::exit(1);
        },
    };

    println!("Family name: {:?}", font.family_name());
    println!("PostScript name: {:?}", font.post_script_name());
    println!("Units per EM: {:?}", font.units_per_em());
    println!("Ascender: {}", font.ascender());
    println!("Descender: {}", font.descender());
    println!("Line gap: {}", font.line_gap());
    println!("Number of glyphs: {}", font.number_of_glyphs());
    println!("Underline: {:?}", font.underline_metrics());
    println!("X height: {:?}", font.x_height());
    println!("Weight: {:?}", font.weight());
    println!("Width: {:?}", font.width());
    println!("Regular: {}", font.is_regular());
    println!("Italic: {}", font.is_italic());
    println!("Bold: {}", font.is_bold());
    println!("Oblique: {}", font.is_oblique());
    println!("Strikeout: {:?}", font.strikeout_metrics());
    println!("Subscript: {:?}", font.subscript_metrics());
    println!("Superscript: {:?}", font.superscript_metrics());
    println!("Variable: {:?}", font.is_variable());

    if font.is_variable() {
        println!("Variation axes:");
        for axis in font.variation_axes() {
            println!("  {} {}..{}, default {}",
                     axis.tag, axis.min_value, axis.max_value, axis.def_value);
        }
    }

    println!("Elapsed: {}us", now.elapsed().as_micros());
}
