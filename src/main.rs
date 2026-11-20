use sdl3::event::{Event, WindowEvent};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::video::FullscreenType;
use std::ffi::CStr;

mod modules {
    pub mod audio;
    pub mod config;
    pub mod fonts;
    pub mod functions {
        pub mod fade;
    }
    pub mod mainmenu;
    pub mod thirdparty {
        pub mod discord;
    }
    pub mod ui;
}

use crate::modules::audio::Sound;
use crate::modules::config::{Config, WindowMode};
use crate::modules::fonts::load_font;
use crate::modules::functions::fade::{Fade, FadeDirection};
use crate::modules::mainmenu::{MenuAction, MenuState, MenuTextures, display};
use crate::modules::thirdparty::discord::Discord;

const WINDOW_TITLE: &str = "inferno";
const CONFIG_PATH: &str = "src/config.toml";
const FONT: &str = "src/fonts/audiowide.ttf";
const BUTTON_FONT_SIZE: f32 = 24.0;

type GameResult<T> = Result<T, Box<dyn std::error::Error>>;

// show available renderers so it is clear which one sdl picked
fn print_render_drivers() {
    let count = unsafe { sdl3::sys::render::SDL_GetNumRenderDrivers() };
    let names: Vec<String> = (0..count)
        .filter_map(|i| {
            let ptr = unsafe { sdl3::sys::render::SDL_GetRenderDriver(i) };
            (!ptr.is_null())
                .then(|| unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned())
        })
        .collect();
    println!(">> render drivers: {}", names.join(", "));
}

fn main() -> GameResult<()> {
    println!("🔥 inferno engine 🔥");
    println!(">> starting engine..");

    let config = match Config::load(CONFIG_PATH) {
        Ok(config) => {
            println!(">> config loaded: {CONFIG_PATH}");
            config
        }
        Err(err) => {
            println!(">> config unavailable: {err}, using defaults");
            Config::default()
        }
    };

    let sdl = sdl3::init()?;
    let video = sdl.video()?;
    print_render_drivers();
    // forced renderer from config, software stays as a fallback
    let driver = format!("{},software", config.game.renderer());
    sdl3::hint::set("SDL_RENDER_DRIVER", &driver);
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

    // discord rpc runs in the background, never blocks the window
    let mut discord = match Discord::new(&config.discord_rpc.app_id) {
        Ok(discord) => Some(discord),
        Err(err) => {
            println!(">> discord rpc unavailable: {err}");
            None
        }
    };

    let mode = config.game.window_mode();
    let enforce_fullscreen = matches!(
        mode,
        WindowMode::Fullscreen | WindowMode::BorderlessFullscreen
    );
    let (width, height) = (config.game.width(), config.game.height());

    let mut builder = video.window(WINDOW_TITLE, width, height);
    match mode {
        WindowMode::Windowed => {
            builder.position_centered();
        }
        WindowMode::Fullscreen => {
            builder.fullscreen();
        }
        WindowMode::BorderlessFullscreen => {
            builder.fullscreen().borderless();
        }
        WindowMode::BorderlessWindowed => {
            builder.borderless().position_centered();
        }
    }
    let window = builder.build().map_err(|e| e.to_string())?;
    let mut canvas = window.into_canvas();
    println!(">> renderer: {}", canvas.renderer_name);
    if canvas.renderer_name == "software" {
        println!(">> warning: software renderer in use");
    }
    if enforce_fullscreen && canvas.window().fullscreen_state() == FullscreenType::Off {
        canvas.window_mut().set_fullscreen(true)?;
    }
    let texture_creator = canvas.texture_creator();
    let button_font = load_font(&ttf, FONT, BUTTON_FONT_SIZE)?;
    println!(">> loaded font: {FONT}");
    // render text once, not every frame
    let mut textures = MenuTextures::new(&texture_creator, &button_font)?;
    let mut fade = Fade::new(800, FadeDirection::In, None);
    fade.start();
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
                    if enforce_fullscreen {
                        canvas.window_mut().set_fullscreen(true)?;
                    }
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
        if enforce_fullscreen && canvas.window().fullscreen_state() == FullscreenType::Off {
            canvas.window_mut().set_fullscreen(true)?;
        }

        if !running {
            continue;
        }

        match display(&mut canvas, mouse_x, mouse_y, click, &mut textures, &mut menu_state)? {
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

        fade.draw(&mut canvas)?;
        canvas.present();
    }

    // close the window before waiting on the quit voice
    drop(textures);
    drop(texture_creator);
    drop(canvas);

    if let Some(discord) = &mut discord
        && let Err(err) = discord.clear()
    {
        println!(">> discord error: {err}");
    }

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
