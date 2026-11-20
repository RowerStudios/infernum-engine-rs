use sdl3::pixels::{Color, PixelFormat};
use sdl3::render::{Canvas, FRect, Texture, TextureCreator};
use sdl3::surface::Surface;
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::error::Error;
use std::time::Instant;

const BUTTON_NORMAL: &str = "src/ui/button_normal.png";
const BUTTON_HOVER: &str = "src/ui/button_hover.png";
const ARROW: &str = "src/ui/arrow.png";

// transparent padding baked around the button art for the shadow
pub const BUTTON_PAD: f32 = 10.0;
const ARROW_W: f32 = 13.0;
const ARROW_H: f32 = 18.0;
const ARROW_GAP: f32 = 6.0;
const ARROW_TRAVEL: f32 = 18.0;
const ARROW_ANIM: f32 = 0.24;
const ARROW_BLINK: f32 = 9.0;

fn ease_out(p: f32) -> f32 {
    1.0 - (1.0 - p) * (1.0 - p)
}

fn ease_in(p: f32) -> f32 {
    p * p
}

#[derive(Clone, Copy)]
enum ArrowPhase {
    Hidden,
    In { start: Instant, from: f32 },
    Blinking,
    Out { start: Instant, from: f32 },
    Held,
}

// slide and fade state for the side arrows
pub struct ArrowAnim {
    phase: ArrowPhase,
    born: Instant,
}

impl ArrowAnim {
    pub fn new() -> Self {
        Self {
            phase: ArrowPhase::Hidden,
            born: Instant::now(),
        }
    }

    fn offset(&self) -> f32 {
        match self.phase {
            ArrowPhase::Hidden | ArrowPhase::Held => ARROW_TRAVEL,
            ArrowPhase::Blinking => 0.0,
            ArrowPhase::In { start, from } => {
                let p = (start.elapsed().as_secs_f32() / ARROW_ANIM).min(1.0);
                from * (1.0 - ease_out(p))
            }
            ArrowPhase::Out { start, from } => {
                let p = (start.elapsed().as_secs_f32() / ARROW_ANIM).min(1.0);
                from + (ARROW_TRAVEL - from) * ease_in(p)
            }
        }
    }

    pub fn hover(&mut self) {
        if matches!(self.phase, ArrowPhase::Hidden) {
            self.phase = ArrowPhase::In {
                start: Instant::now(),
                from: ARROW_TRAVEL,
            };
        }
    }

    pub fn leave(&mut self) {
        self.phase = ArrowPhase::Hidden;
    }

    // mouse down, arrows fly out and stay gone until the next hover
    pub fn press(&mut self) {
        if matches!(self.phase, ArrowPhase::In { .. } | ArrowPhase::Blinking) {
            let from = self.offset();
            self.phase = ArrowPhase::Out {
                start: Instant::now(),
                from,
            };
        }
    }

    // returns (outward offset, alpha)
    pub fn frame(&mut self, hovered: bool) -> Option<(f32, f32)> {
        match self.phase {
            ArrowPhase::Hidden | ArrowPhase::Held => None,
            ArrowPhase::In { start, .. } => {
                if start.elapsed().as_secs_f32() >= ARROW_ANIM {
                    self.phase = ArrowPhase::Blinking;
                    Some((0.0, 1.0))
                } else {
                    let o = self.offset();
                    Some((o, 1.0 - o / ARROW_TRAVEL))
                }
            }
            ArrowPhase::Blinking => {
                if !hovered {
                    self.phase = ArrowPhase::Hidden;
                    return None;
                }
                let t = self.born.elapsed().as_secs_f32();
                let pulse = (t * ARROW_BLINK).sin() * 0.5 + 0.5;
                Some((0.0, 0.35 + 0.65 * pulse))
            }
            ArrowPhase::Out { start, .. } => {
                if start.elapsed().as_secs_f32() >= ARROW_ANIM {
                    self.phase = ArrowPhase::Held;
                    None
                } else {
                    let o = self.offset();
                    Some((o, 1.0 - o / ARROW_TRAVEL))
                }
            }
        }
    }
}

fn load_png<'a>(
    creator: &'a TextureCreator<WindowContext>,
    path: &str,
) -> Result<Texture<'a>, Box<dyn Error>> {
    let img = image::open(path)?.to_rgba8();
    let (w, h) = img.dimensions();
    let mut data = img.into_raw();
    let surface = Surface::from_data(&mut data, w, h, w * 4, PixelFormat::RGBA32)?;
    Ok(creator.create_texture_from_surface(&surface)?)
}

// preloaded button art, normal and hover
pub struct ButtonArt<'a> {
    normal: Texture<'a>,
    hover: Texture<'a>,
    arrow: Texture<'a>,
}

impl<'a> ButtonArt<'a> {
    pub fn load(creator: &'a TextureCreator<WindowContext>) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            normal: load_png(creator, BUTTON_NORMAL)?,
            hover: load_png(creator, BUTTON_HOVER)?,
            arrow: load_png(creator, ARROW)?,
        })
    }
}

// png texture with its native size
pub struct ImageTexture<'a> {
    texture: Texture<'a>,
    width: u32,
    height: u32,
}

impl<'a> ImageTexture<'a> {
    pub fn load(
        creator: &'a TextureCreator<WindowContext>,
        path: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let texture = load_png(creator, path)?;
        let query = texture.query();
        Ok(Self {
            texture,
            width: query.width,
            height: query.height,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    // draw scaled into a rect
    pub fn draw_rect(
        &self,
        canvas: &mut Canvas<Window>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    ) -> Result<(), Box<dyn Error>> {
        let dst = FRect::new(x, y, w, h);
        canvas.copy(&self.texture, None, Some(dst))?;
        Ok(())
    }
}

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

// clickable button, art drawn from a texture
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
        art: &mut ButtonArt<'_>,
        arrows: Option<(f32, f32)>,
    ) -> Result<(), Box<dyn Error>> {
        let texture = if hovered { &art.hover } else { &art.normal };
        let dst = FRect::new(
            self.bounds.x - BUTTON_PAD,
            self.bounds.y - BUTTON_PAD,
            self.bounds.w + BUTTON_PAD * 2.0,
            self.bounds.h + BUTTON_PAD * 2.0,
        );
        canvas.copy(texture, None, Some(dst))?;

        if let Some((offset, alpha)) = arrows {
            art.arrow.set_alpha_mod((alpha.clamp(0.0, 1.0) * 255.0) as u8);
            let ay = self.bounds.y + self.bounds.h / 2.0 - ARROW_H / 2.0;

            let left = FRect::new(
                self.bounds.x - ARROW_W - ARROW_GAP - offset,
                ay,
                ARROW_W,
                ARROW_H,
            );
            canvas.copy(&art.arrow, None, Some(left))?;

            // right arrow is mirrored, points at the centre
            let right = FRect::new(
                self.bounds.x + self.bounds.w + ARROW_GAP + offset,
                ay,
                ARROW_W,
                ARROW_H,
            );
            canvas.copy_ex(
                &art.arrow,
                None,
                Some(right),
                0.0,
                None::<sdl3::render::FPoint>,
                true,
                false,
            )?;
        }

        let cx = self.bounds.x + self.bounds.w / 2.0;
        let cy = self.bounds.y + self.bounds.h / 2.0;
        label.draw_centered(canvas, cx, cy)
    }
}
