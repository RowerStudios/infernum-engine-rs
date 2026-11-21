use crate::modules::model;
use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{
    BlendMode, Canvas, ScaleMode, Texture, TextureCreator, VertexIndices,
};
use sdl3::surface::Surface;
use sdl3::video::{Window, WindowContext};
use std::error::Error;
use std::time::Instant;

const MODEL: &str = "src/models/m4a1/m4a1.glb";

pub struct Prologue<'a> {
    model: model::Model,
    texture: Option<Texture<'a>>,
    vertices: Vec<sdl3::render::Vertex>,
    size: (u32, u32),
}

impl<'a> Prologue<'a> {
    pub fn new(creator: &'a TextureCreator<WindowContext>) -> Result<Self, Box<dyn Error>> {
        let start = Instant::now();
        let model = model::load(MODEL)?;
        let texture = match &model.texture {
            Some(image) => {
                let mut pixels = image.rgba.clone();
                let surface = Surface::from_data(
                    &mut pixels,
                    image.width,
                    image.height,
                    image.width * 4,
                    PixelFormat::RGBA32,
                )?;
                let mut art = creator.create_texture_from_surface(&surface)?;
                art.set_blend_mode(BlendMode::Blend);
                art.set_scale_mode(ScaleMode::Linear);
                Some(art)
            }
            None => None,
        };
        println!(">> model ready in {}ms", start.elapsed().as_millis());
        Ok(Self {
            model,
            texture,
            vertices: Vec::new(),
            size: (0, 0),
        })
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), Box<dyn Error>> {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        let size = canvas.output_size()?;
        if size != self.size {
            self.vertices = self.model.vertices(size.0, size.1);
            self.size = size;
        }
        canvas.render_geometry(
            &self.vertices,
            self.texture.as_ref(),
            VertexIndices::Sequential,
        )?;
        Ok(())
    }
}
