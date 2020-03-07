#![allow(non_camel_case_types)]

use std::convert::TryFrom;
use std::os::raw::{c_void, c_char};

use ttf_parser::GlyphId;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ttfp_font {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct ttfp_outline_builder {
    move_to: unsafe extern "C" fn(x: f32, y: f32, data: *mut c_void),
    line_to: unsafe extern "C" fn(x: f32, y: f32, data: *mut c_void),
    quad_to: unsafe extern "C" fn(x1: f32, y1: f32, x: f32, y: f32, data: *mut c_void),
    curve_to: unsafe extern "C" fn(x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32, data: *mut c_void),
    close_path: unsafe extern "C" fn(data: *mut c_void),
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ttfp_name_record {
    platform_id: u16,
    encoding_id: u16,
    language_id: u16,
    name_id: u16,
    name_size: u16,
}

fn font_from_ptr(font: *const ttfp_font) -> &'static ttf_parser::Font<'static> {
    unsafe { &*(font as *const ttf_parser::Font) }
}

#[no_mangle]
pub extern "C" fn ttfp_fonts_in_collection(data: *const c_char, len: usize) -> i32 {
    let data = unsafe { std::slice::from_raw_parts(data as *const _, len) };
    match ttf_parser::fonts_in_collection(data) {
        Some(n) => i32::try_from(n).unwrap_or(-1),
        None => -1,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_create_font(data: *const c_char, len: usize, index: u32) -> *mut ttfp_font {
    // This method invokes a lot of parsing, so let's catch any panics just in case.
    std::panic::catch_unwind(|| {
        let data = unsafe { std::slice::from_raw_parts(data as *const _, len) };
        let font = ttf_parser::Font::from_data(data, index).unwrap();
        Box::into_raw(Box::new(font)) as *mut _
    }).unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn ttfp_destroy_font(font: *mut ttfp_font) {
    unsafe { Box::from_raw(font) };
}

#[no_mangle]
pub extern "C" fn ttfp_has_table(font: *const ttfp_font, name: ttf_parser::TableName) -> bool {
    font_from_ptr(font).has_table(name)
}

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

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_hor_advance(font: *const ttfp_font, glyph_id: GlyphId) -> u16 {
    font_from_ptr(font).glyph_hor_advance(glyph_id).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_hor_side_bearing(font: *const ttfp_font, glyph_id: GlyphId) -> i16 {
    font_from_ptr(font).glyph_hor_side_bearing(glyph_id).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_ver_advance(font: *const ttfp_font, glyph_id: GlyphId) -> u16 {
    font_from_ptr(font).glyph_ver_advance(glyph_id).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_ver_side_bearing(font: *const ttfp_font, glyph_id: GlyphId) -> i16 {
    font_from_ptr(font).glyph_ver_side_bearing(glyph_id).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_y_origin(font: *const ttfp_font, glyph_id: GlyphId) -> i16 {
    font_from_ptr(font).glyph_y_origin(glyph_id).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyphs_kerning(
    font: *const ttfp_font,
    glyph_id1: GlyphId,
    glyph_id2: GlyphId,
) -> i16 {
    font_from_ptr(font).glyphs_kerning(glyph_id1, glyph_id2).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_name(
    font: *const ttfp_font,
    glyph_id: GlyphId,
    raw_name: *mut c_char,
) -> bool {
    match font_from_ptr(font).glyph_name(glyph_id) {
        Some(name) => {
            let raw_name = unsafe { std::slice::from_raw_parts_mut(raw_name as *mut _, 256) };
            for (i, c) in name.bytes().enumerate() {
                raw_name[i] = c;
            }

            raw_name[name.len()] = 0;

            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_class(font: *const ttfp_font, glyph_id: GlyphId) -> i32 {
    match font_from_ptr(font).glyph_class(glyph_id) {
        None => 0,
        Some(ttf_parser::GlyphClass::Base) => 1,
        Some(ttf_parser::GlyphClass::Ligature) => 2,
        Some(ttf_parser::GlyphClass::Mark) => 3,
        Some(ttf_parser::GlyphClass::Component) => 4,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_mark_attachment_class(font: *const ttfp_font, glyph_id: GlyphId) -> u16 {
    font_from_ptr(font).glyph_mark_attachment_class(glyph_id).0
}

#[no_mangle]
pub extern "C" fn ttfp_is_mark_glyph(font: *const ttfp_font, glyph_id: GlyphId) -> bool {
    font_from_ptr(font).is_mark_glyph(glyph_id, None)
}

#[no_mangle]
pub extern "C" fn ttfp_get_name_records_count(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).names().count() as u16
}

#[no_mangle]
pub extern "C" fn ttfp_get_name_record(
    font: *const ttfp_font,
    index: u16,
    raw_record: *mut ttfp_name_record,
) -> bool {
    match font_from_ptr(font).names().nth(index as usize) {
        Some(record) => {
            unsafe {
                (*raw_record).platform_id = match record.platform_id() {
                    Some(ttf_parser::PlatformId::Unicode) => 0,
                    Some(ttf_parser::PlatformId::Macintosh) => 1,
                    Some(ttf_parser::PlatformId::Iso) => 2,
                    Some(ttf_parser::PlatformId::Windows) => 3,
                    Some(ttf_parser::PlatformId::Custom) => 4,
                    None => return false,
                };

                (*raw_record).encoding_id = record.encoding_id();
                (*raw_record).language_id = record.language_id();
                (*raw_record).name_id = record.name_id();
                (*raw_record).name_size = record.name().len() as u16;
            }

            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_get_name_record_string(
    font: *const ttfp_font,
    index: u16,
    raw_name: *mut c_char,
    raw_name_size: usize,
) -> bool {
    match font_from_ptr(font).names().nth(index as usize) {
        Some(record) => {
            let name = record.name();
            if name.len() != raw_name_size {
                return false;
            }

            let raw_name = unsafe { std::slice::from_raw_parts_mut(raw_name, raw_name_size) };
            for (i, c) in name.iter().enumerate() {
                raw_name[i] = *c as c_char;
            }

            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_get_units_per_em(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).units_per_em().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ttfp_get_ascender(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).ascender()
}

#[no_mangle]
pub extern "C" fn ttfp_get_descender(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).descender()
}

#[no_mangle]
pub extern "C" fn ttfp_get_height(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).height()
}

#[no_mangle]
pub extern "C" fn ttfp_get_line_gap(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).line_gap()
}

#[no_mangle]
pub extern "C" fn ttfp_is_regular(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_regular()
}

#[no_mangle]
pub extern "C" fn ttfp_is_italic(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_italic()
}

#[no_mangle]
pub extern "C" fn ttfp_is_bold(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_bold()
}

#[no_mangle]
pub extern "C" fn ttfp_is_oblique(font: *const ttfp_font) -> bool {
    font_from_ptr(font).is_oblique()
}

#[no_mangle]
pub extern "C" fn ttfp_get_weight(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).weight().to_number()
}

#[no_mangle]
pub extern "C" fn ttfp_get_width(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).width().to_number()
}

#[no_mangle]
pub extern "C" fn ttfp_get_x_height(font: *const ttfp_font) -> i16 {
    font_from_ptr(font).x_height().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn ttfp_get_underline_metrics(
    font: *const ttfp_font,
    raw_metrics: *mut ttf_parser::LineMetrics,
) -> bool {
    match font_from_ptr(font).underline_metrics() {
        Some(metrics) => {
            unsafe { *raw_metrics = metrics; }
            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_get_strikeout_metrics(
    font: *const ttfp_font,
    raw_metrics: *mut ttf_parser::LineMetrics,
) -> bool {
    match font_from_ptr(font).strikeout_metrics() {
        Some(metrics) => {
            unsafe { *raw_metrics = metrics; }
            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_get_subscript_metrics(
    font: *const ttfp_font,
    raw_metrics: *mut ttf_parser::ScriptMetrics,
) -> bool {
    match font_from_ptr(font).subscript_metrics() {
        Some(metrics) => {
            unsafe { *raw_metrics = metrics; }
            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_get_superscript_metrics(
    font: *const ttfp_font,
    raw_metrics: *mut ttf_parser::ScriptMetrics,
) -> bool {
    match font_from_ptr(font).superscript_metrics() {
        Some(metrics) => {
            unsafe { *raw_metrics = metrics; }
            true
        }
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn ttfp_get_number_of_glyphs(font: *const ttfp_font) -> u16 {
    font_from_ptr(font).number_of_glyphs()
}

#[no_mangle]
pub extern "C" fn ttfp_outline_glyph(
    font: *const ttfp_font,
    raw_builder: ttfp_outline_builder,
    user_data: *mut c_void,
    glyph_id: GlyphId,
    raw_bbox: *mut ttf_parser::Rect,
) -> bool {
    // This method invokes a lot of parsing, so let's catch any panics just in case.
    std::panic::catch_unwind(|| {
        let mut builder = Builder(raw_builder, user_data);
        match font_from_ptr(font).outline_glyph(glyph_id, &mut builder) {
            Some(bbox) => {
                unsafe { *raw_bbox = bbox }
                true
            }
            None => false,
        }
    }).unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn ttfp_get_glyph_bbox(
    font: *const ttfp_font,
    glyph_id: GlyphId,
    raw_bbox: *mut ttf_parser::Rect,
) -> bool {
    // This method invokes a lot of parsing, so let's catch any panics just in case.
    std::panic::catch_unwind(|| {
        match font_from_ptr(font).glyph_bounding_box(glyph_id) {
            Some(bbox) => {
                unsafe { *raw_bbox = bbox }
                true
            }
            None => false,
        }
    }).unwrap_or(false)
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

#[cfg(feature = "logging")]
#[no_mangle]
pub extern "C" fn ttfp_init_log() {
    if let Ok(()) = log::set_logger(&logging::LOGGER) {
        log::set_max_level(log::LevelFilter::Warn);
    }
}

#[cfg(not(feature = "logging"))]
#[no_mangle]
pub extern "C" fn ttfp_init_log() {
    // Do nothing.
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
