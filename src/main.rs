use sdl3::event::{Event, WindowEvent};
use sdl3::mouse::MouseButton;
use sdl3::video::FullscreenType;
use std::ffi::CStr;

mod modules {
    pub mod audio;
    pub mod config;
    pub mod debug_m;
    pub mod fonts;
    pub mod functions {
        pub mod fade;
    }
    pub mod keybind;
    pub mod mainmenu;
    pub mod model;
    pub mod scene_manager;
    pub mod scenes {
        #[path = "built-in-scenes/loading.rs"]
        pub mod loading;
        pub mod prolouge;
    }
    pub mod scroll;
    pub mod settings;
    pub mod threading;
    pub mod thirdparty {
        pub mod discord;
    }
    pub mod ui;
}

use crate::modules::audio::Sound;
use crate::modules::config::{Config, WindowMode};
use crate::modules::debug_m::{DebugInfo, DebugMenu};
use crate::modules::fonts::load_font;
use crate::modules::functions::fade::{Fade, FadeDirection};
use crate::modules::keybind::{Action, Bindings};
use crate::modules::mainmenu::{MenuAction, MenuState, MenuTextures, display};
use crate::modules::scene_manager::{SceneId, SceneManager};
use crate::modules::settings::{SettingsAction, SettingsMenu};
use crate::modules::threading::ThreadPool;
use crate::modules::thirdparty::discord::Discord;

const WINDOW_TITLE: &str = "inferno";
const CONFIG_NAME: &str = "config.toml";
const FONT: &str = "src/fonts/audiowide.ttf";
const BUTTON_FONT_SIZE: f32 = 24.0;
const DEBUG_FONT_SIZE: f32 = 18.0;
const SETTINGS_FONT_SIZE: f32 = 27.0;
const SETTINGS_TITLE_SIZE: f32 = 48.0;

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

fn scene_name(id: SceneId) -> &'static str {
    match id {
        SceneId::Menu => "menu",
        SceneId::Loading => "loading",
        SceneId::Prologue => "prologue",
    }
}

// find the folder that holds src/, walking up from the executable
fn find_asset_root(start: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut dir = Some(start);
    while let Some(path) = dir {
        if path.join("src").join("fonts").join("audiowide.ttf").is_file() {
            return Some(path.to_path_buf());
        }
        dir = path.parent();
    }
    None
}

// apply fullscreen state after a settings change
fn apply_window_mode(canvas: &mut sdl3::render::Canvas<sdl3::video::Window>, mode: WindowMode) {
    let fullscreen = matches!(
        mode,
        WindowMode::Fullscreen | WindowMode::BorderlessFullscreen
    );
    let _ = canvas.window_mut().set_fullscreen(fullscreen);
}

fn main() -> GameResult<()> {
    println!("🔥 inferno engine 🔥");
    println!(">> starting engine..");

    // config lives next to the executable
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    let config_path = exe_dir.join(CONFIG_NAME);
    // assets are referenced relative to the project root, so work from there
    if let Some(root) = find_asset_root(&exe_dir)
        .or_else(|| find_asset_root(&std::env::current_dir().unwrap_or_default()))
    {
        let _ = std::env::set_current_dir(&root);
        println!(">> asset root: {}", root.display());
    }
    let mut config = match Config::load(&config_path) {
        Ok(config) => {
            println!(">> config loaded: {}", config_path.display());
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
    let mut discord = if config.discord_rpc.enable {
        match Discord::new(&config.discord_rpc.app_id) {
            Ok(discord) => Some(discord),
            Err(err) => {
                println!(">> discord rpc unavailable: {err}");
                None
            }
        }
    } else {
        println!(">> discord rpc disabled");
        None
    };

    let mode = config.game.window_mode();
    let mut enforce_fullscreen = matches!(
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
    let mut window = builder.build().map_err(|e| e.to_string())?;
    // set window icon from config, try plain path then src/ prefix
    let icon_name = &config.game.icon;
    let icon_path = if std::path::Path::new(icon_name).exists() {
        std::path::PathBuf::from(icon_name)
    } else {
        std::path::Path::new("src").join(icon_name)
    };
    if let Ok(img) = image::open(&icon_path) {
        // scale down, most wms want small icons
        let rgba = img.to_rgba8();
        let resized = image::imageops::resize(
            &rgba,
            64,
            64,
            image::imageops::FilterType::Lanczos3,
        );
        let (iw, ih) = resized.dimensions();
        let mut pixels = resized.into_raw();
        if let Ok(surface) = sdl3::surface::Surface::from_data(
            &mut pixels,
            iw,
            ih,
            iw * 4,
            sdl3::pixels::PixelFormat::RGBA32,
        ) {
            let ok = window.set_icon(&surface);
            println!(">> icon loaded: {} ({ok})", icon_path.display());
        } else {
            println!(">> icon surface failed");
        }
    } else {
        println!(">> icon not found: {}", icon_path.display());
    }
    let mut canvas = window.into_canvas();
    let (win_w, win_h) = canvas.output_size()?;
    println!(">> window: {win_w}x{win_h}");
    println!(">> renderer: {}", canvas.renderer_name);
    if canvas.renderer_name == "software" {
        println!(">> warning: software renderer in use");
    }
    // pace frames to the display refresh rate instead of a hardcoded value
    let vsync_wanted = if config.game.vsync { 1 } else { 0 };
    let vsync_enabled = unsafe { sdl3::sys::render::SDL_SetRenderVSync(canvas.raw(), vsync_wanted) };
    let mut vsync_interval: i32 = 0;
    unsafe {
        sdl3::sys::render::SDL_GetRenderVSync(canvas.raw(), &mut vsync_interval);
    }
    // query the desktop refresh rate for the debug overlay
    let refresh = video
        .get_primary_display()
        .and_then(|display| display.get_mode())
        .map(|mode| mode.refresh_rate)
        .unwrap_or(0.0);
    if vsync_enabled {
        println!(">> vsync enabled ({vsync_interval})");
    } else {
        println!(">> vsync unavailable");
    }
    if enforce_fullscreen && canvas.window().fullscreen_state() == FullscreenType::Off {
        canvas.window_mut().set_fullscreen(true)?;
    }
    let texture_creator = canvas.texture_creator();
    let button_font = load_font(&ttf, FONT, BUTTON_FONT_SIZE)?;
    let debug_font = load_font(&ttf, FONT, DEBUG_FONT_SIZE)?;
    let settings_font = load_font(&ttf, FONT, SETTINGS_FONT_SIZE)?;
    let settings_title_font = load_font(&ttf, FONT, SETTINGS_TITLE_SIZE)?;
    println!(">> loaded font: {FONT}");
    // render text once, not every frame
    let mut textures = MenuTextures::new(&texture_creator, &button_font)?;
    let mut settings =
        SettingsMenu::new(&texture_creator, &settings_font, &settings_title_font)?;
    settings.refresh(&texture_creator, &settings_font, &config)?;
    let mut bindings = Bindings::from_config(&config.keybinds);
    let pool = ThreadPool::new(4);
    println!(">> thread pool: {} workers", pool.size());
    let debug_info = DebugInfo::query(&canvas.renderer_name, vsync_interval, refresh, &pool);
    let mut debug = DebugMenu::new(&texture_creator, debug_font, debug_info);
    let mut scenes = SceneManager::new();
    let mut fade = Fade::new(800, FadeDirection::In, None);
    fade.start();
    println!(">> starting game loop..");
    let mut menu_state = MenuState::new();
    let mut event_pump = sdl.event_pump()?;
    let mut running = true;
    let mut quit_requested = false;
    let mut show_settings = false;
    let (mut mouse_x, mut mouse_y) = (0.0_f32, 0.0_f32);

    while running {
        let mut click = None;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    quit_requested = true;
                    running = false;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    // settings capture takes priority over game actions
                    if show_settings && settings.is_capturing_input() {
                        if settings.on_key(keycode, &mut config) {
                            settings.refresh(&texture_creator, &settings_font, &config)?;
                            config.save(&config_path)?;
                            bindings = Bindings::from_config(&config.keybinds);
                            println!(">> config saved");
                        }
                    } else {
                        match bindings.key(keycode) {
                            Action::Debug => debug.toggle(),
                            Action::ToggleVsync => {
                                config.game.vsync = !config.game.vsync;
                                let current = if config.game.vsync { 1 } else { 0 };
                                unsafe {
                                    sdl3::sys::render::SDL_SetRenderVSync(canvas.raw(), current);
                                }
                                let mut actual: i32 = 0;
                                unsafe {
                                    sdl3::sys::render::SDL_GetRenderVSync(canvas.raw(), &mut actual);
                                }
                                debug.set_vsync(actual);
                                config.save(&config_path)?;
                                println!(">> vsync set to {actual}");
                            }
                            Action::Pause => {
                                if show_settings {
                                    if let Some(sound) = &sound
                                        && let Err(err) = sound.play_click()
                                    {
                                        println!(">> audio error: {err}");
                                    }
                                    show_settings = false;
                                } else {
                                    quit_requested = true;
                                    running = false;
                                }
                            }
                            _ => {}
                        }
                    }
                },
                Event::MouseWheel { y, .. } => {
                    if show_settings {
                        settings.wheel(y);
                    }
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
                    mouse_btn,
                    x,
                    y,
                    ..
                } => {
                    // capture mouse button when rebinding in settings
                    if show_settings && settings.is_capturing_input() {
                        if settings.on_mouse(mouse_btn, &mut config) {
                            settings.refresh(&texture_creator, &settings_font, &config)?;
                            config.save(&config_path)?;
                            bindings = Bindings::from_config(&config.keybinds);
                            println!(">> config saved");
                        }
                    } else if mouse_btn == MouseButton::Left {
                        click = Some((x, y));
                    }
                }
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

        if show_settings {
            let (cw, ch) = canvas.output_size()?;
            if settings.poll_hover(mouse_x, mouse_y, cw as f32, ch as f32)
                && let Some(sound) = &sound
                && let Err(err) = sound.play_hover()
            {
                println!(">> audio error: {err}");
            }
            if let Some((x, y)) = click {
                let action = settings.handle_click(x, y, cw as f32, ch as f32, &mut config);
                if !matches!(action, SettingsAction::None)
                    && let Some(sound) = &sound
                    && let Err(err) = sound.play_click()
                {
                    println!(">> audio error: {err}");
                }
                match action {
                    SettingsAction::Back => {
                        config.save(&config_path)?;
                        println!(">> config saved");
                        show_settings = false;
                    }
                    SettingsAction::Changed => {
                        config.save(&config_path)?;
                        let mut actual: i32 = 0;
                        unsafe {
                            sdl3::sys::render::SDL_SetRenderVSync(
                                canvas.raw(),
                                if config.game.vsync { 1 } else { 0 },
                            );
                            sdl3::sys::render::SDL_GetRenderVSync(canvas.raw(), &mut actual);
                        }
                        debug.set_vsync(actual);
                        apply_window_mode(&mut canvas, config.game.window_mode());
                        enforce_fullscreen = matches!(
                            config.game.window_mode(),
                            WindowMode::Fullscreen | WindowMode::BorderlessFullscreen
                        );
                        println!(">> config saved");
                    }
                    SettingsAction::Clicked | SettingsAction::None => {}
                }
                settings.refresh(&texture_creator, &settings_font, &config)?;
            }
            settings.draw(&mut canvas)?;
        } else {
            match scenes.current() {
                SceneId::Menu => {
                    match display(&mut canvas, mouse_x, mouse_y, click, &mut textures, &mut menu_state)? {
                        MenuAction::Play => {
                            if let Some(sound) = &sound {
                                if let Err(err) = sound.play_click() {
                                    println!(">> audio error: {err}");
                                }
                                if let Err(err) = sound.pause_music() {
                                    println!(">> audio error: {err}");
                                }
                            }
                            scenes.enter(SceneId::Loading, &texture_creator, &button_font, &pool)?;
                        }
                        MenuAction::Settings => {
                            if let Some(sound) = &sound
                                && let Err(err) = sound.play_click()
                            {
                                println!(">> audio error: {err}");
                            }
                            settings.refresh(&texture_creator, &settings_font, &config)?;
                            settings.reset_hover();
                            show_settings = true;
                        }
                        MenuAction::HoverPlay | MenuAction::HoverSettings | MenuAction::HoverQuit => {
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
                SceneId::Loading | SceneId::Prologue => scenes.draw(&mut canvas)?,
            }
        }

        debug.tick();
        debug.draw(&mut canvas, scene_name(scenes.current()), (mouse_x, mouse_y))?;
        fade.draw(&mut canvas)?;
        canvas.present();
    }

    // close the window before waiting on the quit voice
    drop(scenes);
    drop(settings);
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
