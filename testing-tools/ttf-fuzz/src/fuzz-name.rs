#[macro_use]
extern crate afl;

fn main() {
    afl::fuzz!(|data: &[u8]| {
        if let Some(font) = ttf_parser::Font::from_data(data, 0) {
            let _ = font.family_name();
            let _ = font.post_script_name();
            let _ = font.names().count();
        }
    });
}
