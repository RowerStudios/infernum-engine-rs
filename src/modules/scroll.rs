use sdl3::pixels::Color;
use sdl3::rect::Rect;
use sdl3::render::{BlendMode, Canvas, FRect};
use sdl3::video::Window;
use std::error::Error;

// vertical scrolling panel with a slim scrollbar
pub struct ScrollPanel {
    offset: f32,
    content_h: f32,
    view_h: f32,
    step: f32,
}

impl ScrollPanel {
    pub fn new(step: f32) -> Self {
        Self {
            offset: 0.0,
            content_h: 0.0,
            view_h: 0.0,
            step,
        }
    }

    pub fn reset(&mut self) {
        self.offset = 0.0;
    }

    pub fn set_content(&mut self, content_h: f32, view_h: f32) {
        self.content_h = content_h;
        self.view_h = view_h;
        self.offset = self.offset.clamp(0.0, self.max_offset());
    }

    pub fn max_offset(&self) -> f32 {
        (self.content_h - self.view_h).max(0.0)
    }

    pub fn offset(&self) -> f32 {
        self.offset
    }

    // wheel delta, positive scrolls up
    pub fn wheel(&mut self, delta: f32) -> bool {
        if self.max_offset() <= 0.0 {
            return false;
        }
        let before = self.offset;
        self.offset = (self.offset - delta * self.step).clamp(0.0, self.max_offset());
        self.offset != before
    }

    // clip drawing to the panel view
    pub fn clip(&self, canvas: &mut Canvas<Window>, view: FRect) {
        let rect = Rect::new(
            view.x as i32,
            view.y as i32,
            view.w.max(1.0) as u32,
            view.h.max(1.0) as u32,
        );
        canvas.set_clip_rect(rect);
    }

    pub fn unclip(canvas: &mut Canvas<Window>) {
        canvas.set_clip_rect(None::<Rect>);
    }

    // draw the scrollbar on the right edge of the view
    pub fn draw_bar(&self, canvas: &mut Canvas<Window>, view: FRect) -> Result<(), Box<dyn Error>> {
        if self.max_offset() <= 0.0 {
            return Ok(());
        }
        let bar_w = 6.0;
        let x = view.x + view.w - bar_w;
        let old = canvas.blend_mode();
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(255, 255, 255, 30));
        canvas.fill_rect(FRect::new(x, view.y, bar_w, view.h))?;
        let ratio = view.h / self.content_h;
        let thumb_h = (view.h * ratio).max(28.0);
        let t = self.offset / self.max_offset();
        let thumb_y = view.y + (view.h - thumb_h) * t;
        canvas.set_draw_color(Color::RGB(220, 70, 40));
        canvas.fill_rect(FRect::new(x, thumb_y, bar_w, thumb_h))?;
        canvas.set_blend_mode(old);
        Ok(())
    }
}
