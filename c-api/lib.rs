#![allow(non_camel_case_types)]

use std::convert::TryFrom;
use std::os::raw::{c_void, c_char};

use ttf_parser::{GlyphId, Tag};

/// @brief An opaque pointer to the font structure.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ttfp_font {
    _unused: [u8; 0],
}

/// @brief An outline building interface.
#[repr(C)]
pub struct ttfp_outline_builder {
    pub move_to: unsafe extern "C" fn(x: f32, y: f32, data: *mut c_void),
    pub line_to: unsafe extern "C" fn(x: f32, y: f32, data: *mut c_void),
    pub quad_to: unsafe extern "C" fn(x1: f32, y1: f32, x: f32, y: f32, data: *mut c_void),
    pub curve_to: unsafe extern "C" fn(x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32, data: *mut c_void),
    pub close_path: unsafe extern "C" fn(data: *mut c_void),
}

struct Builder(ttfp_outline_builder, *mut c_void);

impl ttf_parser::OutlineBuilder for Builder {
    fn move_to(&mut self, x: f32, y: f32) {
        unsafe { (self.0.move_to)(x, y, self.1) }
    }

    fn line_to(&mut self, x: f32, y: f32) {
        unsafe { (self.0.line_to)(x, y, self.1) }
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        unsafe { (self.0.quad_to)(x1, y1, x, y, self.1) }
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        unsafe { (self.0.curve_to)(x1, y1, x2, y2, x, y, self.1) }
    }

    fn close(&mut self) {
        unsafe { (self.0.close_path)(self.1) }
    }
}

/// @brief A name record.
///
/// https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-records
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ttfp_name_record {
    pub platform_id: u16,
    pub encoding_id: u16,
    pub language_id: u16,
    pub name_id: u16,
    pub name_size: u16,
}

/// @brief A list of glyph classes.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ttfp_glyph_class {
    TTFP_GLYPH_CLASS_UNKNOWN   = 0,
    TTFP_GLYPH_CLASS_BASE      = 1,
    TTFP_GLYPH_CLASS_LIGATURE  = 2,
    TTFP_GLYPH_CLASS_MARK      = 3,
    TTFP_GLYPH_CLASS_COMPONENT = 4,
}

fn font_from_ptr(font: *const ttfp_font) -> &'static ttf_parser::Font<'static> {
    unsafe { &*(font as *const ttf_parser::Font) }
}

fn font_from_mut_ptr(font: *const ttfp_font) -> &'static mut ttf_parser::Font<'static> {
    unsafe { &mut *(font as *mut ttf_parser::Font) }
}

/// @brief Initializes the library log.
///
/// Use it if you want to see any warnings.
///
/// Will do nothing when library is built without the `logging` feature.
///
/// All warnings will be printed to the `stderr`.
#[cfg(feature = "logging")]
#[no_mangle]
pub extern "C" fn ttfp_init_log() {
    if let Ok(()) = log::set_logger(&logging::LOGGER) {
        log::set_max_level(log::LevelFilter::Warn);
    }
}

#[cfg(not(feature = "logging"))]
#[no_mangle]
pub extern "C" fn ttfp_init_log() {}

/// @brief Returns the number of fonts stored in a TrueType font collection.
///
/// @param data The font data.
/// @param len The size of the font data.
/// @return Number of fonts or -1 when provided data is not a TrueType font collection
///         or when number of fonts is larger than INT_MAX.
#[no_mangle]
pub extern "C" fn ttfp_fonts_in_collection(data: *const c_char, len: usize) -> i32 {
    let data = unsafe { std::slice::from_raw_parts(data as *const _, len) };
    match ttf_parser::fonts_in_collection(data) {
        Some(n) => i32::try_from(n).unwrap_or(-1),
        None => -1,
    }
}

/// @brief Creates a new font parser.
///
/// This is the only heap allocation in the library.
///
/// @param data The font data. Must outlive the #ttfp_font.
/// @param len The size of the font data.
/// @param index The font index in a collection (typically *.ttc). 0 should be used for basic fonts.
/// @return Font handle or NULL on error.
#[no_mangle]
pub extern "C" fn ttfp_create_font(data: *const c_char, len: usize, index: u32) -> *mut ttfp_font {
    // This method invokes a lot of parsing, so let's catch any panics just in case.
    std::panic::catch_unwind(|| {
        let data = unsafe { std::slice::from_raw_parts(data as *const _, len) };
        let font = ttf_parser::Font::from_data(data, index).unwrap();
        Box::into_raw(Box::new(font)) as *mut _
    }).unwrap_or(std::ptr::null_mut())
}

/// @brief Destroys the #ttfp_font.
#[no_mangle]
pub extern "C" fn ttfp_destroy_font(font: *mut ttfp_font) {
    unsafe { Box::from_raw(font) };
}

/// @brief Checks that font has a specified table.
///
/// @return `true` only for tables that were successfully parsed.
#[no_mangle]
pub extern "C" fn ttfp_has_table(font: *const ttfp_font, name: ttf_parser::TableName) -> bool {
    font_from_ptr(font).has_table(name)
}

/// @brief Returns the number of name records in the font.
#[no_mangle]
pub extern "C" fn ttfp_get_name_records_count(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).names().count() as u16
}

/// @brief Returns a name record.
///
/// @param Record's index. The total amount can be obtained via #ttfp_get_name_records_count.
/// @return `false` when `index` is out of range or `platform_id` is invalid.
#[no_mangle]
pub extern "C" fn ttfp_get_name_record(
    font: *const ttfp_font,
    index: u16,
    record: *mut ttfp_name_record,
) -> bool {
    match font_from_ptr(font).names().nth(index as usize) {
        Some(rec) => {
            unsafe {
                (*record).platform_id = match rec.platform_id() {
                    Some(ttf_parser::PlatformId::Unicode) => 0,
                    Some(ttf_parser::PlatformId::Macintosh) => 1,
                    Some(ttf_parser::PlatformId::Iso) => 2,
                    Some(ttf_parser::PlatformId::Windows) => 3,
                    Some(ttf_parser::PlatformId::Custom) => 4,
                    None => return false,
                };

                (*record).encoding_id = rec.encoding_id();
                (*record).language_id = rec.language_id();
                (*record).name_id = rec.name_id();
                (*record).name_size = rec.name().len() as u16;
            }

            true
        }
        None => false,
    }
}

/// @brief Returns a name record's string.
///
/// @param index Record's index.
/// @param name A string buffer that will be filled with the record's name.
///             Remember that a name will use encoding specified in `ttfp_name_record.encoding_id`
///             Because of that, the name will not be null-terminated.
/// @param len The size of a string buffer. Must be equal to `ttfp_name_record.name_size`.
/// @return `false` when `index` is out of range or string buffer is not equal
///         `ttfp_name_record.name_size`.
#[no_mangle]
pub extern "C" fn ttfp_get_name_record_string(
    font: *const ttfp_font,
    index: u16,
    name: *mut c_char,
    len: usize,
) -> bool {
    match font_from_ptr(font).names().nth(index as usize) {
        Some(r) => {
            let r_name = r.name();
            if r_name.len() != len {
                return false;
            }

            // TODO: memcpy?
            let name = unsafe { std::slice::from_raw_parts_mut(name, len) };
            for (i, c) in r_name.iter().enumerate() {
                name[i] = *c as c_char;
            }

            true
        }
        None => false,
    }
}

/// @brief Checks that font is marked as *Regular*.
///
/// @return `false` when OS/2 table is not present.
#[no_mangle]
pub extern "C" fn ttfp_is_regular(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_regular()
}

/// @brief Checks that font is marked as *Italic*.
///
/// @return `false` when OS/2 table is not present.
#[no_mangle]
pub extern "C" fn ttfp_is_italic(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_italic()
}

/// @brief Checks that font is marked as *Bold*.
///
/// @return `false` when OS/2 table is not present.
#[no_mangle]
pub extern "C" fn ttfp_is_bold(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_bold()
}

/// @brief Checks that font is marked as *Oblique*.
///
/// @return `false` when OS/2 table is not present.
#[no_mangle]
pub extern "C" fn ttfp_is_oblique(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_oblique()
}

/// @brief Checks that font is variable.
///
/// Simply checks the presence of a `fvar` table.
#[no_mangle]
pub extern "C" fn ttfp_is_variable(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_variable()
}

/// @brief Returns font's weight.
///
/// @return Font's weight or `400` when OS/2 table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_weight(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).weight().to_number()
}

/// @brief Returns font's width.
///
/// @return Font's width in a 1..9 range or `5` when OS/2 table is not present
///         or when value is invalid.
#[no_mangle]
pub extern "C" fn ttfp_get_width(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).width().to_number()
}

/// @brief Returns a horizontal font ascender.
///
/// This function is affected by variation axes.
#[no_mangle]
pub extern "C" fn ttfp_get_ascender(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).ascender()
}

/// @brief Returns a horizontal font descender.
///
/// This function is affected by variation axes.
#[no_mangle]
pub extern "C" fn ttfp_get_descender(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).descender()
}

/// @brief Returns a horizontal font height.
///
/// This function is affected by variation axes.
#[no_mangle]
pub extern "C" fn ttfp_get_height(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).height()
}

/// @brief Returns a horizontal font line gap.
///
/// This function is affected by variation axes.
#[no_mangle]
pub extern "C" fn ttfp_get_line_gap(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).line_gap()
}

/// @brief Returns a vertical font ascender.
///
/// This function is affected by variation axes.
///
/// @return `0` when `vhea` table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_vertical_ascender(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).vertical_ascender().unwrap_or(0)
}

/// @brief Returns a vertical font descender.
///
/// This function is affected by variation axes.
///
/// @return `0` when `vhea` table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_vertical_descender(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).vertical_descender().unwrap_or(0)
}

/// @brief Returns a vertical font height.
///
/// This function is affected by variation axes.
///
/// @return `0` when `vhea` table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_vertical_height(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).vertical_height().unwrap_or(0)
}

/// @brief Returns a vertical font line gap.
///
/// This function is affected by variation axes.
///
/// @return `0` when `vhea` table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_vertical_line_gap(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).vertical_line_gap().unwrap_or(0)
}

/// @brief Returns font's units per EM.
///
/// @return Units in a 16..16384 range or `0` otherwise.
#[no_mangle]
pub extern "C" fn ttfp_get_units_per_em(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).units_per_em().unwrap_or(0)
}

/// @brief Returns font's x height.
///
/// This function is affected by variation axes.
///
/// @return x height or 0 when OS/2 table is not present or when its version is < 2.
#[no_mangle]
pub extern "C" fn ttfp_get_x_height(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).x_height().unwrap_or(0)
}

/// @brief Returns font's underline metrics.
///
/// This function is affected by variation axes.
///
/// @return `false` when `post` table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_underline_metrics(
    font: *const ttfp_font,
    metrics: *mut ttf_parser::LineMetrics,
) -> bool {
    match font_from_ptr(font).underline_metrics() {
        Some(m) => {
            unsafe { *metrics = m; }
            true
        }
        None => false,
    }
}

/// @brief Returns font's strikeout metrics.
///
/// This function is affected by variation axes.
///
/// @return `false` when OS/2 table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_strikeout_metrics(
    font: *const ttfp_font,
    metrics: *mut ttf_parser::LineMetrics,
) -> bool {
    match font_from_ptr(font).strikeout_metrics() {
        Some(m) => {
            unsafe { *metrics = m; }
            true
        }
        None => false,
    }
}

/// @brief Returns font's subscript metrics.
///
/// This function is affected by variation axes.
///
/// @return `false` when OS/2 table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_subscript_metrics(
    font: *const ttfp_font,
    metrics: *mut ttf_parser::ScriptMetrics,
) -> bool {
    match font_from_ptr(font).subscript_metrics() {
        Some(m) => {
            unsafe { *metrics = m; }
            true
        }
        None => false,
    }
}

/// @brief Returns font's superscript metrics.
///
/// This function is affected by variation axes.
///
/// @return `false` when OS/2 table is not present.
#[no_mangle]
pub extern "C" fn ttfp_get_superscript_metrics(
    font: *const ttfp_font,
    metrics: *mut ttf_parser::ScriptMetrics,
) -> bool {
    match font_from_ptr(font).superscript_metrics() {
        Some(m) => {
            unsafe { *metrics = m; }
            true
        }
        None => false,
    }
}

/// @brief Returns a total number of glyphs in the font.
///
/// @return The number of glyphs which is never zero.
#[no_mangle]
pub extern "C" fn ttfp_get_number_of_glyphs(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).number_of_glyphs()
}

/// @brief Resolves a Glyph ID for a code point.
///
/// All subtable formats except Mixed Coverage (8) are supported.
///
/// @param codepoint A valid Unicode codepoint. Otherwise 0 will be returned.
/// @return Returns 0 when glyph is not present or parsing is failed.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_index(font: *const ttfp_font, codepoint: u32) -> u16 {
    // This method invokes a lot of parsing, so let's catch any panics just in case.
    std::panic::catch_unwind(|| {
        let get = || {
            let c = char::try_from(codepoint).ok()?;
            font_from_ptr(font).glyph_index(c).map(|gid| gid.0)
        };

        get().unwrap_or(0)
    }).unwrap_or(0)
}

/// @brief Resolves a variation of a Glyph ID from two code points.
///
/// @param codepoint A valid Unicode codepoint. Otherwise 0 will be returned.
/// @param variation A valid Unicode codepoint. Otherwise 0 will be returned.
/// @return Returns 0 when glyph is not present or parsing is failed.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_var_index(
    font: *const ttfp_font,
    codepoint: u32,
    variation: u32,
) -> u16 {
    // This method invokes a lot of parsing, so let's catch any panics just in case.
    std::panic::catch_unwind(|| {
        let get = || {
            let c = char::try_from(codepoint).ok()?;
            let v = char::try_from(variation).ok()?;
            font_from_ptr(font).glyph_variation_index(c, v).map(|gid| gid.0)
        };

        get().unwrap_or(0)
    }).unwrap_or(0)
}

/// @brief Returns glyph's horizontal advance.
///
/// @return Glyph's advance or 0 when not set.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_hor_advance(font: *const ttfp_font, glyph_id: GlyphId) -> u16 {
    font_from_ptr(font).glyph_hor_advance(glyph_id).unwrap_or(0)
}

/// @brief Returns glyph's vertical advance.
///
/// This function is affected by variation axes.
///
/// @return Glyph's advance or 0 when not set.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_ver_advance(font: *const ttfp_font, glyph_id: GlyphId) -> u16 {
    font_from_ptr(font).glyph_ver_advance(glyph_id).unwrap_or(0)
}

/// @brief Returns glyph's horizontal side bearing.
///
/// @return Glyph's side bearing or 0 when not set.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_hor_side_bearing(font: *const ttfp_font, glyph_id: GlyphId) -> i16 {
    font_from_ptr(font).glyph_hor_side_bearing(glyph_id).unwrap_or(0)
}

/// @brief Returns glyph's vertical side bearing.
///
/// This function is affected by variation axes.
///
/// @return Glyph's side bearing or 0 when not set.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_ver_side_bearing(font: *const ttfp_font, glyph_id: GlyphId) -> i16 {
    font_from_ptr(font).glyph_ver_side_bearing(glyph_id).unwrap_or(0)
}

/// @brief Returns glyph's vertical origin.
///
/// @return Glyph's vertical origin or 0 when not set.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_y_origin(font: *const ttfp_font, glyph_id: GlyphId) -> i16 {
    font_from_ptr(font).glyph_y_origin(glyph_id).unwrap_or(0)
}

/// @brief Returns glyph's name.
///
/// Uses the `post` table as a source.
///
/// A glyph name cannot be larger than 255 bytes + 1 byte for '\0'.
///
/// @param name A char buffer longer than 256 bytes.
/// @return `true` on success.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_name(
    font: *const ttfp_font,
    glyph_id: GlyphId,
    name: *mut c_char,
) -> bool {
    match font_from_ptr(font).glyph_name(glyph_id) {
        Some(n) => {
            // TODO: memcpy?
            let name = unsafe { std::slice::from_raw_parts_mut(name as *mut _, 256) };
            for (i, c) in n.bytes().enumerate() {
                name[i] = c;
            }

            name[n.len()] = 0;

            true
        }
        None => false,
    }
}

/// @brief Returns glyph's class according to Glyph Class Definition Table.
///
/// @return A glyph class or TTFP_GLYPH_CLASS_UNKNOWN otherwise.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_class(font: *const ttfp_font, glyph_id: GlyphId) -> ttfp_glyph_class {
    match font_from_ptr(font).glyph_class(glyph_id) {
        None => ttfp_glyph_class::TTFP_GLYPH_CLASS_UNKNOWN,
        Some(ttf_parser::GlyphClass::Base) => ttfp_glyph_class::TTFP_GLYPH_CLASS_BASE,
        Some(ttf_parser::GlyphClass::Ligature) => ttfp_glyph_class::TTFP_GLYPH_CLASS_LIGATURE,
        Some(ttf_parser::GlyphClass::Mark) => ttfp_glyph_class::TTFP_GLYPH_CLASS_MARK,
        Some(ttf_parser::GlyphClass::Component) => ttfp_glyph_class::TTFP_GLYPH_CLASS_COMPONENT,
    }
}

/// @brief Returns glyph's mark attachment class according to Mark Attachment Class Definition Table.
///
/// @return All glyphs not assigned to a class fall into Class 0.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_mark_attachment_class(font: *const ttfp_font, glyph_id: GlyphId) -> u16 {
    font_from_ptr(font).glyph_mark_attachment_class(glyph_id).0
}

/// @brief Checks that glyph is a mark according to Mark Glyph Sets Table.
#[no_mangle]
pub extern "C" fn ttfp_is_mark_glyph(font: *const ttfp_font, glyph_id: GlyphId) -> bool {
    font_from_ptr(font).is_mark_glyph(glyph_id, None)
}

/// @brief Returns a glyphs pair kerning.
///
/// Only a horizontal kerning is supported.
///
/// @param glyph_id1 First glyph ID.
/// @param glyph_id1 Second glyph ID.
/// @return A kerning offset or 0 otherwise.
#[no_mangle]
pub extern "C" fn ttfp_get_glyphs_kerning(
    font: *const ttfp_font,
    glyph_id1: GlyphId,
    glyph_id2: GlyphId,
) -> i16 {
    font_from_ptr(font).glyphs_kerning(glyph_id1, glyph_id2).unwrap_or(0)
}

/// @brief Outlines a glyph and returns its tight bounding box.
///
/// **Warning**: since `ttf-parser` is a pull parser,
/// `OutlineBuilder` will emit segments even when outline is partially malformed.
/// You must check #ttfp_outline_glyph() result before using
/// #ttfp_outline_builder 's output.
///
/// `glyf`, `gvar`, `CFF` and `CFF2` tables are supported.
///
/// This function is affected by variation axes.
///
/// @return `false` when glyph has no outline or on error.
#[no_mangle]
pub extern "C" fn ttfp_outline_glyph(
    font: *const ttfp_font,
    builder: ttfp_outline_builder,
    user_data: *mut c_void,
    glyph_id: GlyphId,
    bbox: *mut ttf_parser::Rect,
) -> bool {
    // This method invokes a lot of parsing, so let's catch any panics just in case.
    std::panic::catch_unwind(|| {
        let mut b = Builder(builder, user_data);
        match font_from_ptr(font).outline_glyph(glyph_id, &mut b) {
            Some(bb) => {
                unsafe { *bbox = bb }
                true
            }
            None => false,
        }
    }).unwrap_or(false)
}

/// @brief Returns a tight glyph bounding box.
///
/// Unless the current font has a `glyf` table, this is just a shorthand for `outline_glyph()`
/// since only the `glyf` table stores a bounding box. In case of CFF and variable fonts
/// we have to actually outline a glyph to find it's bounding box.
///
/// This function is affected by variation axes.
#[no_mangle]
pub extern "C" fn ttfp_get_glyph_bbox(
    font: *const ttfp_font,
    glyph_id: GlyphId,
    bbox: *mut ttf_parser::Rect,
) -> bool {
    // This method invokes a lot of parsing, so let's catch any panics just in case.
    std::panic::catch_unwind(|| {
        match font_from_ptr(font).glyph_bounding_box(glyph_id) {
            Some(bb) => {
                unsafe { *bbox = bb }
                true
            }
            None => false,
        }
    }).unwrap_or(false)
}

/// @brief Returns the amount of variation axes.
#[no_mangle]
pub extern "C" fn ttfp_get_variation_axes_count(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).variation_axes().count() as u16
}

/// @brief Returns a variation axis by index.
#[no_mangle]
pub extern "C" fn ttfp_get_variation_axis(
    font: *const ttfp_font,
    index: u16,
    axis: *mut ttf_parser::VariationAxis,
) -> bool {
    match font_from_ptr(font).variation_axes().nth(index as usize) {
        Some(a) => {
            unsafe { *axis = a };
            true
        }
        None => false,
    }
}

/// @brief Returns a variation axis by tag.
#[no_mangle]
pub extern "C" fn ttfp_get_variation_axis_by_tag(
    font: *const ttfp_font,
    tag: ttf_parser::Tag,
    axis: *mut ttf_parser::VariationAxis,
) -> bool {
    match font_from_ptr(font).variation_axes().find(|axis| axis.tag == tag) {
        Some(a) => {
            unsafe { *axis = a };
            true
        }
        None => false,
    }
}

/// @brief Sets a variation axis coordinate.
///
/// This is the only mutable function in the library.
/// We can simplify the API a lot by storing the variable coordinates
/// in the font object itself.
///
/// This function is reentrant.
///
/// Since coordinates are stored on the stack, we allow only 32 of them.
///
/// @return `false` when font is not variable or doesn't have such axis.
#[no_mangle]
pub extern "C" fn ttfp_set_variation(font: *mut ttfp_font, axis: Tag, value: f32) -> bool {
    font_from_mut_ptr(font).set_variation(axis, value).is_some()
}

#[cfg(feature = "logging")]
mod logging {
    pub static LOGGER: SimpleLogger = SimpleLogger;

    pub struct SimpleLogger;

    impl log::Log for SimpleLogger {
        fn enabled(&self, metadata: &log::Metadata) -> bool {
            metadata.level() <= log::LevelFilter::Warn
        }

        fn log(&self, record: &log::Record) {
            if self.enabled(record.metadata()) {
                let target = if record.target().len() > 0 {
                    record.target()
                } else {
                    record.module_path().unwrap_or_default()
                };

                // ttf-parser will emit only warnings.
                eprintln!("Warning: [{}] {}", target, record.args());
            }
        }

        fn flush(&self) {}
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn sizes() {
        assert_eq!(std::mem::size_of::<ttf_parser::TableName>(),
                   std::mem::size_of::<i32>());

        assert_eq!(std::mem::size_of::<ttf_parser::Rect>(), 8);

        assert_eq!(std::mem::size_of::<ttf_parser::LineMetrics>(), 4);

        assert_eq!(std::mem::size_of::<ttf_parser::ScriptMetrics>(), 8);
    }
}
