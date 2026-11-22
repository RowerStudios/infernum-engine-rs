use crate::modules::threading::ThreadPool;
use crate::modules::ui::TextTexture;
use sdl3::pixels::Color;
use sdl3::render::{BlendMode, Canvas, FRect, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::error::Error;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const PANEL_W: f32 = 360.0;
const LINE_H: f32 = 22.0;

// gl info filled in by a background worker
type GlInfo = Arc<Mutex<(String, String, String)>>;

// system info shown in the debug overlay
pub struct DebugInfo {
    renderer: String,
    gl: GlInfo,
    vsync: i32,
    refresh: f32,
}

impl DebugInfo {
    pub fn query(renderer: &str, vsync: i32, refresh: f32, pool: &ThreadPool) -> Self {
        let gl: GlInfo = Arc::new(Mutex::new(("n/a".into(), "n/a".into(), "n/a".into())));
        let shared = Arc::clone(&gl);
        pool.spawn(move || {
            let info = query_gl();
            *shared.lock().unwrap() = info;
        });
        Self {
            renderer: renderer.to_string(),
            gl,
            vsync,
            refresh,
        }
    }
}

// read glxinfo once, fallback to "n/a" when unavailable
fn query_gl() -> (String, String, String) {
    let unknown = "n/a".to_string();
    let Ok(output) = Command::new("glxinfo").arg("-B").output() else {
        return (unknown.clone(), unknown.clone(), unknown);
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut vendor = unknown.clone();
    let mut renderer = unknown.clone();
    let mut version = unknown.clone();
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("OpenGL vendor string:") {
            vendor = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("OpenGL renderer string:") {
            renderer = v.trim().to_string();
        } else if let Some(v) = line.strip_prefix("OpenGL core profile version string:") {
            version = v.trim().to_string();
        }
    }
    (vendor, renderer, version)
}

// overlay shown on top of every scene, toggled with the debug keybind
pub struct DebugMenu<'a> {
    creator: &'a TextureCreator<WindowContext>,
    font: Font<'static>,
    info: DebugInfo,
    visible: bool,
    frames: u32,
    last_update: Instant,
    fps: f32,
    last_frame: Instant,
    delta_ms: f32,
}

impl<'a> DebugMenu<'a> {
    pub fn new(
        creator: &'a TextureCreator<WindowContext>,
        font: Font<'static>,
        info: DebugInfo,
    ) -> Self {
        Self {
            creator,
            font,
            info,
            visible: false,
            frames: 0,
            last_update: Instant::now(),
            fps: 0.0,
            last_frame: Instant::now(),
            delta_ms: 0.0,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn set_vsync(&mut self, interval: i32) {
        self.info.vsync = interval;
    }

    // call once per frame to track fps and delta time
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.delta_ms = now.duration_since(self.last_frame).as_secs_f32() * 1000.0;
        self.last_frame = now;
        self.frames += 1;
        let elapsed = now.duration_since(self.last_update);
        if elapsed >= Duration::from_millis(500) {
            self.fps = self.frames as f32 / elapsed.as_secs_f32();
            self.frames = 0;
            self.last_update = now;
        }
    }

    pub fn draw(
        &self,
        canvas: &mut Canvas<Window>,
        scene: &str,
        mouse: (f32, f32),
    ) -> Result<(), Box<dyn Error>> {
        if !self.visible {
            return Ok(());
        }
        let (sw, sh) = canvas.output_size()?;
        let vsync_tag = if self.info.vsync > 0 { " (vsync)" } else { "" };
        let (vendor, gpu, version) = {
            let guard = self.info.gl.lock().unwrap();
            guard.clone()
        };
        let lines = [
            format!("fps: {:.1}{}", self.fps, vsync_tag),
            format!("delta: {:.2} ms", self.delta_ms),
            format!("scene: {}", scene),
            format!("mouse: {:.0}, {:.0}", mouse.0, mouse.1),
            format!("size: {}x{}", sw, sh),
            format!("renderer: {}", self.info.renderer),
            format!("refresh: {:.0} hz", self.info.refresh),
            format!("vsync: {}", self.info.vsync),
            format!("gpu: {}", gpu),
            format!("vendor: {}", vendor),
            format!("opengl: {}", version),
            format!("f4: hide debug"),
        ];
        let panel_h = 16.0 + lines.len() as f32 * LINE_H;

        let old_blend = canvas.blend_mode();
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 190));
        canvas.fill_rect(FRect::new(8.0, 8.0, PANEL_W, panel_h))?;
        canvas.set_blend_mode(old_blend);

        let mut y = 14.0;
        for line in &lines {
            let text = TextTexture::new(self.creator, &self.font, line, Color::WHITE)?;
            text.draw_at(canvas, 16.0, y)?;
            y += LINE_H;
        }
        Ok(())
    }
}
