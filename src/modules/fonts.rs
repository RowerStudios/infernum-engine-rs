use sdl3::ttf::{Font, Sdl3TtfContext};

// load one font file at the given point size
pub fn load_font(
    ttf: &Sdl3TtfContext,
    path: &str,
    point_size: f32,
) -> Result<Font<'static>, String> {
    ttf.load_font(path, point_size)
        .map_err(|err| format!("failed to load {path}: {err}"))
}
