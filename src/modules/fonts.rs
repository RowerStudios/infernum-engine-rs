use sdl3::ttf::{Font, Sdl3TtfContext};
use std::{fs, path::Path};

// load all .ttf files in a directory at the given point size
pub fn load_fonts(
    ttf: &Sdl3TtfContext,
    dir: impl AsRef<Path>,
    point_size: f32,
) -> Vec<Font<'static>> {
    let mut fonts = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            println!(">> failed to read fonts directory: {err}");
            return fonts;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "ttf") {
            match ttf.load_font(&path, point_size) {
                Ok(font) => {
                    println!(">> loaded: {}", path.display());
                    fonts.push(font);
                }
                Err(err) => println!(">> failed: {} ({err})", path.display()),
            }
        }
    }
    fonts
}
