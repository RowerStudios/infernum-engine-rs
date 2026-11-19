use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::video::FullscreenType;

mod modules {
    pub mod audio;
    pub mod fonts;
    pub mod functions {
        pub mod fade;
    }
    pub mod mainmenu;
    pub mod ui;
}

use crate::modules::audio::Sound;
use crate::modules::fonts::load_fonts;
use crate::modules::mainmenu::{MenuAction, MenuState, MenuTextures, display};

const WINDOW_TITLE: &str = "inferno";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const FONT_DIR: &str = "src/fonts";
const TITLE_FONT_SIZE: f32 = 40.0;
const BUTTON_FONT_SIZE: f32 = 24.0;

type GameResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> GameResult<()> {
    println!("🔥 inferno engine 🔥");
    println!(">> starting engine..");

    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    let ttf = sdl3::ttf::init().map_err(|e| e.to_string())?;
    println!(">> sdl3 + sdl3_ttf initialized!");

    // audio is optional, the game still runs without a sound device
    let sound = match Sound::new() {
        Ok(sound) => {
            println!(">> sound ready!");
            Some(sound)
        }
        Err(err) => {
            println!(">> sound unavailable: {err}");
            None
        }
    };

    let window = video
        .window(WINDOW_TITLE, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .fullscreen()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas();
    if canvas.window().fullscreen_state() == FullscreenType::Off {
        canvas.window_mut().set_fullscreen(true)?;
    }
    let texture_creator = canvas.texture_creator();

    let title_fonts = load_fonts(&ttf, FONT_DIR, TITLE_FONT_SIZE);
    let button_fonts = load_fonts(&ttf, FONT_DIR, BUTTON_FONT_SIZE);
    if title_fonts.is_empty() || button_fonts.is_empty() {
        return Err(format!("no .ttf fonts found in '{FONT_DIR}'").into());
    }
    println!(">> loaded {} fonts!", title_fonts.len() + button_fonts.len());
    // render text once, not every frame
    let textures = MenuTextures::new(&texture_creator, &title_fonts[0], &button_fonts[0])?;
    println!(">> starting game loop..");
    let mut menu_state = MenuState::new();
    let mut event_pump = sdl.event_pump()?;
    let mut running = true;
    let mut quit_requested = false;
    let (mut mouse_x, mut mouse_y) = (0.0_f32, 0.0_f32);

    while running {
        let mut click = None;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    quit_requested = true;
                    running = false;
                }
                Event::Window {
                    win_event: WindowEvent::LeaveFullscreen,
                    ..
                } => {
                    canvas.window_mut().set_fullscreen(true)?;
                }
                Event::MouseMotion { x, y, .. } => {
                    mouse_x = x;
                    mouse_y = y;
                }
                Event::MouseButtonDown {
                    mouse_btn: MouseButton::Left,
                    x,
                    y,
                    ..
                } => click = Some((x, y)),
                _ => {}
            }
        }

        // keep fullscreen even if a shortcut or the desktop tries to leave it
        if canvas.window().fullscreen_state() == FullscreenType::Off {
            canvas.window_mut().set_fullscreen(true)?;
        }

        if !running {
            continue;
        }

        match display(&mut canvas, mouse_x, mouse_y, click, &textures, &mut menu_state)? {
            MenuAction::Play => {
                if let Some(sound) = &sound
                    && let Err(err) = sound.play_click()
                {
                    println!(">> audio error: {err}");
                }
            }
            MenuAction::HoverPlay | MenuAction::HoverQuit => {
                if let Some(sound) = &sound
                    && let Err(err) = sound.play_hover()
                {
                    println!(">> audio error: {err}");
                }
            }
            MenuAction::Quit => {
                quit_requested = true;
                running = false;
            }
            MenuAction::None => {}
        }
    }

    // close the window before waiting on the quit voice
    drop(textures);
    drop(texture_creator);
    drop(canvas);
    if quit_requested && let Some(sound) = &sound {
        match sound.play_quit_voice() {
            Ok(ms) => {
                println!(">> playing quit voice for {ms}ms");
                std::thread::sleep(std::time::Duration::from_millis(ms));
            }
            Err(err) => println!(">> audio error: {err}"),
        }
    }
    println!(">> shutting down..");
    Ok(())
}
