use crate::modules::model::{self, Model};
use crate::modules::threading::ThreadPool;
use crate::modules::ui::TextTexture;
use sdl3::pixels::Color;
use sdl3::render::{Canvas, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::error::Error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const MODEL: &str = "src/models/m4a1/m4a1.glb";
const MIN_DISPLAY_MS: u64 = 350;

type LoadResult = Result<Model, String>;

// loading scene, shows text while the model loads on the thread pool
pub struct Loading<'a> {
    text: TextTexture<'a>,
    started: Instant,
    done: Arc<AtomicBool>,
    result: Arc<Mutex<Option<LoadResult>>>,
}

impl<'a> Loading<'a> {
    pub fn new(
        creator: &'a TextureCreator<WindowContext>,
        font: &Font,
        pool: &ThreadPool,
    ) -> Result<Self, Box<dyn Error>> {
        let text = TextTexture::new(creator, font, "loading...", Color::WHITE)?;
        let done = Arc::new(AtomicBool::new(false));
        let result = Arc::new(Mutex::new(None));

        let task_done = Arc::clone(&done);
        let task_result = Arc::clone(&result);
        pool.spawn(move || {
            let start = Instant::now();
            let loaded = model::load(MODEL).map_err(|err| err.to_string());
            println!(">> model loaded in {}ms", start.elapsed().as_millis());
            *task_result.lock().unwrap() = Some(loaded);
            task_done.store(true, Ordering::Release);
        });

        Ok(Self {
            text,
            started: Instant::now(),
            done,
            result,
        })
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), Box<dyn Error>> {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        let (w, h) = canvas.output_size()?;
        // pulse the text brightness so it feels alive
        let t = self.started.elapsed().as_secs_f32();
        let alpha = (155.0 + 100.0 * (t * 3.0).sin().abs()) as u8;
        self.text.set_alpha(alpha);
        self.text.draw_centered(canvas, w as f32 / 2.0, h as f32 / 2.0)?;
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.done.load(Ordering::Acquire)
            && self.started.elapsed() >= Duration::from_millis(MIN_DISPLAY_MS)
    }

    pub fn started(&self) -> Instant {
        self.started
    }

    pub fn take_model(&mut self) -> Result<Model, Box<dyn Error>> {
        let mut guard = self.result.lock().unwrap();
        match guard.take() {
            Some(Ok(model)) => Ok(model),
            Some(Err(err)) => Err(err.into()),
            None => Err("no model load result".into()),
        }
    }
}
