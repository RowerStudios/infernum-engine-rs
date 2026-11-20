use crate::modules::ui::{ArrowAnim, Button, ButtonArt, TextTexture};
use sdl3::pixels::Color;
use sdl3::render::{Canvas, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::error::Error;

const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 60.0;
const BUTTON_GAP: f32 = 80.0;
const TITLE_MARGIN: f32 = 100.0;

pub enum MenuAction {
    None,
    Play,
    Quit,
    HoverPlay,
    HoverQuit,
}

// remembers last hover to catch enter events
pub struct MenuState {
    play_hovered: bool,
    quit_hovered: bool,
    play_anim: ArrowAnim,
    quit_anim: ArrowAnim,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            play_hovered: false,
            quit_hovered: false,
            play_anim: ArrowAnim::new(),
            quit_anim: ArrowAnim::new(),
        }
    }
}

// pre-rendered text and button art for the menu
pub struct MenuTextures<'a> {
    pub title: TextTexture<'a>,
    pub play: TextTexture<'a>,
    pub quit: TextTexture<'a>,
    pub buttons: ButtonArt<'a>,
}

impl<'a> MenuTextures<'a> {
    pub fn new(
        creator: &'a TextureCreator<WindowContext>,
        title_font: &Font,
        button_font: &Font,
    ) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            title: TextTexture::new(creator, title_font, "inferno", Color::WHITE)?,
            play: TextTexture::new(creator, button_font, "play", Color::WHITE)?,
            quit: TextTexture::new(creator, button_font, "quit", Color::WHITE)?,
            buttons: ButtonArt::load(creator)?,
        })
    }
}

// draw menu, handle input, return the clicked action
pub fn display(
    canvas: &mut Canvas<Window>,
    mouse_x: f32,
    mouse_y: f32,
    click: Option<(f32, f32)>,
    textures: &mut MenuTextures<'_>,
    state: &mut MenuState,
) -> Result<MenuAction, Box<dyn Error>> {
    let (screen_w, screen_h) = canvas.output_size()?;
    let center_x = screen_w as f32 / 2.0;
    let center_y = screen_h as f32 / 2.0;
    let button_x = center_x - BUTTON_WIDTH / 2.0;
    let play = Button::new(
        button_x,
        center_y - BUTTON_GAP / 2.0 - BUTTON_HEIGHT,
        BUTTON_WIDTH,
        BUTTON_HEIGHT,
    );
    let quit = Button::new(
        button_x,
        center_y + BUTTON_GAP / 2.0,
        BUTTON_WIDTH,
        BUTTON_HEIGHT,
    );
    let play_hovered = play.contains(mouse_x, mouse_y);
    let quit_hovered = quit.contains(mouse_x, mouse_y);
    let mut action = MenuAction::None;
    if play_hovered && !state.play_hovered {
        action = MenuAction::HoverPlay;
        state.play_anim.hover();
    } else if quit_hovered && !state.quit_hovered {
        action = MenuAction::HoverQuit;
        state.quit_anim.hover();
    }
    if !play_hovered && state.play_hovered {
        state.play_anim.leave();
    }
    if !quit_hovered && state.quit_hovered {
        state.quit_anim.leave();
    }

    if let Some((x, y)) = click {
        if play.contains(x, y) {
            println!(">> play clicked!");
            action = MenuAction::Play;
            state.play_anim.press();
        } else if quit.contains(x, y) {
            action = MenuAction::Quit;
            state.quit_anim.press();
        }
    }

    state.play_hovered = play_hovered;
    state.quit_hovered = quit_hovered;

    let play_arrows = state.play_anim.frame(play_hovered);
    let quit_arrows = state.quit_anim.frame(quit_hovered);

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    textures.title.draw_at(
        canvas,
        center_x - textures.title.width() as f32 / 2.0,
        play.bounds.y - TITLE_MARGIN,
    )?;
    play.draw(
        canvas,
        play_hovered,
        &textures.play,
        &mut textures.buttons,
        play_arrows,
    )?;
    quit.draw(
        canvas,
        quit_hovered,
        &textures.quit,
        &mut textures.buttons,
        quit_arrows,
    )?;
   Ok(action)
}
