use crate::modules::scenes::prolouge::Prologue;
use sdl3::render::{Canvas, TextureCreator};
use sdl3::video::{Window, WindowContext};
use std::error::Error;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    Menu,
    Prologue,
}

// owns the active scene and builds scenes on first entry
pub struct SceneManager<'a> {
    current: SceneId,
    prologue: Option<Prologue<'a>>,
}

impl<'a> SceneManager<'a> {
    pub fn new() -> Self {
        Self {
            current: SceneId::Menu,
            prologue: None,
        }
    }

    pub fn current(&self) -> SceneId {
        self.current
    }

    pub fn enter(
        &mut self,
        id: SceneId,
        creator: &'a TextureCreator<WindowContext>,
    ) -> Result<(), Box<dyn Error>> {
        if id == SceneId::Prologue && self.prologue.is_none() {
            println!(">> loading prologue..");
            self.prologue = Some(Prologue::new(creator)?);
        }
        self.current = id;
        Ok(())
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), Box<dyn Error>> {
        if self.current == SceneId::Prologue
            && let Some(scene) = &mut self.prologue
        {
            scene.draw(canvas)?;
        }
        Ok(())
    }
}
