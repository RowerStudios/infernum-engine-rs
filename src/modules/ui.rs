use sdl3::pixels::Color;
use sdl3::render::{Canvas, FRect, Texture, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::error::Error;

// text rendered once into a texture
pub struct TextTexture<'a> {
    texture: Texture<'a>,
    width: u32,
    height: u32,
}

impl<'a> TextTexture<'a> {
    pub fn new(
        creator: &'a TextureCreator<WindowContext>,
        font: &Font,
        text: &str,
        color: Color,
    ) -> Result<Self, Box<dyn Error>> {
        let surface = font.render(text).blended(color)?;
        let (width, height) = surface.size();
        let texture = creator.create_texture_from_surface(&surface)?;
        Ok(Self {
            texture,
            width,
            height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    // draw with top-left corner at (x, y)
    pub fn draw_at(
        &self,
        canvas: &mut Canvas<Window>,
        x: f32,
        y: f32,
    ) -> Result<(), Box<dyn Error>> {
        let dst = FRect::new(x, y, self.width as f32, self.height as f32);
        canvas.copy(&self.texture, None, Some(dst))?;
        Ok(())
    }

    // draw centered on (cx, cy)
    pub fn draw_centered(
        &self,
        canvas: &mut Canvas<Window>,
        cx: f32,
        cy: f32,
    ) -> Result<(), Box<dyn Error>> {
        self.draw_at(
            canvas,
            cx - self.width as f32 / 2.0,
            cy - self.height as f32 / 2.0,
        )
    }
}

// clickable button, label drawn from a texture
pub struct Button {
    pub bounds: FRect,
}

impl Button {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            bounds: FRect::new(x, y, width, height),
        }
    }

    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.bounds.x
            && x < self.bounds.x + self.bounds.w
            && y >= self.bounds.y
            && y < self.bounds.y + self.bounds.h
    }

    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        hovered: bool,
        label: &TextTexture<'_>,
    ) -> Result<(), Box<dyn Error>> {
        canvas.set_draw_color(if hovered { Color::GRAY } else { Color::BLACK });
        canvas.fill_rect(self.bounds)?;

        let cx = self.bounds.x + self.bounds.w / 2.0;
        let cy = self.bounds.y + self.bounds.h / 2.0;
        label.draw_centered(canvas, cx, cy)
    }
}
