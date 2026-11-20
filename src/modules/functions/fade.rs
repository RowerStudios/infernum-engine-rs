use sdl3::pixels::Color;
use sdl3::render::{BlendMode, Canvas};
use sdl3::video::Window;
use std::error::Error;
use std::time::{Duration, Instant};

pub enum FadeDirection {
    In,
    #[allow(dead_code)]
    Out,
}

// draws a fading overlay, call after drawing the scene and before present
pub struct Fade {
    duration: Duration,
    direction: FadeDirection,
    color: Color,
    started: Option<Instant>,
}

impl Fade {
    pub fn new(duration_ms: u64, direction: FadeDirection, color: Option<Color>) -> Self {
        Self {
            duration: Duration::from_millis(duration_ms),
            direction,
            color: color.unwrap_or(Color::BLACK),
            started: None,
        }
    }

    pub fn start(&mut self) {
        self.started = Some(Instant::now());
    }

    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.started.is_some()
    }

    // returns true while the fade is still running
    pub fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<bool, Box<dyn Error>> {
        let Some(started) = self.started else {
            return Ok(false);
        };
        if self.duration.is_zero() || started.elapsed() >= self.duration {
            self.started = None;
            return Ok(false);
        }
        let progress = started.elapsed().as_secs_f32() / self.duration.as_secs_f32();
        let alpha = match self.direction {
            FadeDirection::In => 1.0 - progress,
            FadeDirection::Out => progress,
        };
        let old_blend = canvas.blend_mode();
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(
            self.color.r,
            self.color.g,
            self.color.b,
            (alpha * 255.0) as u8,
        ));
        let result = canvas.fill_rect(None);
        canvas.set_blend_mode(old_blend);
        result?;
        Ok(true)
    }
}
