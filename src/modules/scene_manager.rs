use crate::modules::scenes::loading::Loading;
use crate::modules::scenes::prolouge::Prologue;
use crate::modules::threading::ThreadPool;
use sdl3::render::{Canvas, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::error::Error;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SceneId {
    Menu,
    Loading,
    Prologue,
}

// owns the active scene and builds scenes on first entry
pub struct SceneManager<'a> {
    current: SceneId,
    creator: Option<&'a TextureCreator<WindowContext>>,
    loading: Option<Loading<'a>>,
    prologue: Option<Prologue<'a>>,
}

impl<'a> SceneManager<'a> {
    pub fn new() -> Self {
        Self {
            current: SceneId::Menu,
            creator: None,
            loading: None,
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
        font: &Font,
        pool: &ThreadPool,
    ) -> Result<(), Box<dyn Error>> {
        self.creator = Some(creator);
        match id {
            SceneId::Loading => {
                println!(">> entering loading scene..");
                self.loading = Some(Loading::new(creator, font, pool)?);
                self.current = SceneId::Loading;
            }
            SceneId::Prologue => {
                println!(">> loading prologue..");
                self.prologue = Some(Prologue::new(creator)?);
                self.current = SceneId::Prologue;
            }
            SceneId::Menu => {
                self.current = SceneId::Menu;
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), Box<dyn Error>> {
        match self.current {
            SceneId::Menu => {}
            SceneId::Loading => {
                if let Some(loading) = &mut self.loading {
                    loading.draw(canvas)?;
                    if loading.is_ready() {
                        let start = loading.started();
                        let model = loading.take_model()?;
                        let creator = self.creator.ok_or("no texture creator")?;
                        self.prologue = Some(Prologue::from_model(creator, model, start)?);
                        self.current = SceneId::Prologue;
                        println!(">> loading complete, entering prologue..");
                    }
                }
            }
            SceneId::Prologue => {
                if let Some(prologue) = &mut self.prologue {
                    prologue.draw(canvas)?;
                }
            }
        }
        Ok(())
    }
}
