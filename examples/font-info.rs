fn main() {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage:\n\tfont-info font.ttf");
        std::process::exit(1);
    }

    let font_data = std::fs::read(&args[1]).unwrap();

    let now = std::time::Instant::now();

    let face = match ttf_parser::Face::parse(&font_data, 0) {
        Ok(f) => f,
        Err(e) => {
            eprint!("Error: {}.", e);
            std::process::exit(1);
        },
    };

    let family_name = face.names().into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::FULL_NAME && name.is_unicode())
        .and_then(|name| name.to_string());

    let post_script_name = face.names().into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && name.is_unicode())
        .and_then(|name| name.to_string());

    println!("Family name: {:?}", family_name);
    println!("PostScript name: {:?}", post_script_name);
    println!("Units per EM: {:?}", face.units_per_em());
    println!("Ascender: {}", face.ascender());
    println!("Descender: {}", face.descender());
    println!("Line gap: {}", face.line_gap());
    println!("Global bbox: {:?}", face.global_bounding_box());
    println!("Number of glyphs: {}", face.number_of_glyphs());
    println!("Underline: {:?}", face.underline_metrics());
    println!("X height: {:?}", face.x_height());
    println!("Weight: {:?}", face.weight());
    println!("Width: {:?}", face.width());
    println!("Regular: {}", face.is_regular());
    println!("Italic: {}", face.is_italic());
    println!("Bold: {}", face.is_bold());
    println!("Oblique: {}", face.is_oblique());
    println!("Strikeout: {:?}", face.strikeout_metrics());
    println!("Subscript: {:?}", face.subscript_metrics());
    println!("Superscript: {:?}", face.superscript_metrics());
    println!("Variable: {:?}", face.is_variable());

    #[cfg(feature = "opentype-layout")] {
        if let Some(ref table) = face.tables().gpos {
            print_opentype_layout("positioning", table);
        }

        if let Some(ref table) = face.tables().gsub {
            print_opentype_layout("substitution", table);
        }
    }

    #[cfg(feature = "variable-fonts")] {
        if face.is_variable() {
            println!("Variation axes:");
            for axis in face.variation_axes() {
                println!("  {} {}..{}, default {}",
                         axis.tag, axis.min_value, axis.max_value, axis.def_value);
            }
        }
    }

    println!("Elapsed: {}us", now.elapsed().as_micros());
}

fn print_opentype_layout(name: &str, table: &ttf_parser::opentype_layout::LayoutTable) {
    println!("OpenType {}:", name);
    println!("  Scripts:");
    for script in table.scripts {
        println!("    {}", script.tag);

        if script.languages.is_empty() {
            println!("      No languages");
            continue;
        }

        println!("      Languages:");
        for lang in script.languages {
            println!("        {}", lang.tag);
        }
    }

    let mut features: Vec<_> = table.features.into_iter().map(|f| f.tag).collect();
    features.dedup();
    println!("  Features:");
    for feature in features {
        println!("    {}", feature);
    }
}
