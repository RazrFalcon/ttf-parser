use ttf_parser::stat::{AxisValue, AxisValueTable};

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
        }
    };

    let mut family_names = Vec::new();
    for name in face.names() {
        if name.name_id == ttf_parser::name_id::FULL_NAME && name.is_unicode() {
            if let Some(family_name) = name.to_string() {
                let language = name.language();
                family_names.push(format!(
                    "{} ({}, {})",
                    family_name,
                    language.primary_language(),
                    language.region()
                ));
            }
        }
    }

    let post_script_name = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && name.is_unicode())
        .and_then(|name| name.to_string());

    println!("Family names: {:?}", family_names);
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
    println!("Permissions: {:?}", face.permissions());
    println!("Variable: {:?}", face.is_variable());

    #[cfg(feature = "opentype-layout")]
    {
        if let Some(ref table) = face.tables().gpos {
            print_opentype_layout("positioning", table);
        }

        if let Some(ref table) = face.tables().gsub {
            print_opentype_layout("substitution", table);
        }
    }

    #[cfg(feature = "variable-fonts")]
    {
        if face.is_variable() {
            println!("Variation axes:");
            for axis in face.variation_axes() {
                println!(
                    "  {} {}..{}, default {}",
                    axis.tag, axis.min_value, axis.max_value, axis.def_value
                );
            }
        }
    }

    if let Some(stat) = face.tables().stat {
        let axis_names = stat
            .axes
            .into_iter()
            .map(|axis| axis.tag)
            .collect::<Vec<_>>();

        println!("Style attributes:");

        println!("  Axes:");
        for axis in axis_names.iter() {
            println!("    {}", axis);
        }

        println!("  Axis Values:");
        for table in stat.subtables() {
            match table {
                AxisValueTable::Format1(table) => {
                    let value_name = face
                        .names()
                        .into_iter()
                        .filter(|name| name.name_id == table.value_name_id)
                        .map(|name| name.to_string().unwrap())
                        .collect::<Vec<_>>()
                        .join(", ");

                    let axis_name = &axis_names[table.axis_index as usize];
                    let value = table.value;
                    let flags = table.flags;

                    println!("    {axis_name} {value:?}={value_name:?} flags={flags:?}");
                }
                AxisValueTable::Format2(table) => {
                    let value_name = face
                        .names()
                        .into_iter()
                        .filter(|name| name.name_id == table.value_name_id)
                        .map(|name| name.to_string().unwrap())
                        .collect::<Vec<_>>()
                        .join(", ");

                    let axis_name = &axis_names[table.axis_index as usize];
                    let nominal_value = table.nominal_value;
                    let min_value = table.range_min_value;
                    let max_value = table.range_max_value;
                    let flags = table.flags;

                    println!("    {axis_name} {min_value:?}..{max_value:?}={value_name:?} nominal={nominal_value:?} flags={flags:?}");
                }
                AxisValueTable::Format3(table) => {
                    let value_name = face
                        .names()
                        .into_iter()
                        .filter(|name| name.name_id == table.value_name_id)
                        .map(|name| name.to_string().unwrap())
                        .collect::<Vec<_>>()
                        .join(", ");

                    let axis_name = &axis_names[table.axis_index as usize];
                    let value = table.value;
                    let linked_value = table.linked_value;
                    let flags = table.flags;

                    println!(
                        "    {axis_name} {value:?}<=>{linked_value:?} = {value_name:?} flags={flags:?}",
                    );
                }
                AxisValueTable::Format4(table) => {
                    let value_name = face
                        .names()
                        .into_iter()
                        .filter(|name| name.name_id == table.value_name_id)
                        .map(|name| name.to_string().unwrap())
                        .collect::<Vec<_>>()
                        .join(", ");

                    let flags = table.flags;

                    println!("    {value_name:?} flags={flags:?}");
                    for pair in table.values {
                        let AxisValue { axis_index, value } = pair;
                        println!("      {axis_index} = {value:?}")
                    }
                }
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
