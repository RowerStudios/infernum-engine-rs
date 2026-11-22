use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::Path};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "discord-rpc")]
    pub discord_rpc: DiscordRpc,
    pub game: Game,
    pub keybinds: Keybinds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordRpc {
    #[serde(default = "default_true")]
    pub enable: bool,
    pub app_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    #[serde(default = "default_true")]
    pub vsync: bool,
    pub window_mode: String,
    pub window_width: String,
    pub window_height: String,
    pub force_renderer: String,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinds {
    pub forward: String,
    pub backward: String,
    pub left: String,
    pub right: String,
    pub shoot: String,
    pub sights_down: String,
    pub i_menu: String,
    pub pause: String,
    pub debug_m: String,
    pub toggle_vsync: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowMode {
    Windowed,
    Fullscreen,
    BorderlessFullscreen,
    BorderlessWindowed,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            discord_rpc: DiscordRpc {
                enable: true,
                app_id: String::new(),
            },
            game: Game {
                vsync: true,
                window_mode: "1".to_string(),
                window_width: "800".to_string(),
                window_height: "600".to_string(),
                force_renderer: "1".to_string(),
                icon: "icon.png".to_string(),
            },
            keybinds: Keybinds {
                forward: "W".to_string(),
                backward: "S".to_string(),
                left: "A".to_string(),
                right: "D".to_string(),
                shoot: "LMB".to_string(),
                sights_down: "RMB".to_string(),
                i_menu: "MMB".to_string(),
                pause: "ESC".to_string(),
                debug_m: "F4".to_string(),
                toggle_vsync: "F3".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let text = fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
        let text = toml::to_string_pretty(self)?;
        fs::write(path, text)?;
        Ok(())
    }
}

impl Game {
    pub fn window_mode(&self) -> WindowMode {
        match self.window_mode.as_str() {
            "0" => WindowMode::Windowed,
            "1" => WindowMode::Fullscreen,
            "2" => WindowMode::BorderlessFullscreen,
            "3" => WindowMode::BorderlessWindowed,
            _ => WindowMode::Fullscreen,
        }
    }

    pub fn width(&self) -> u32 {
        self.window_width.parse().unwrap_or(800)
    }

    pub fn height(&self) -> u32 {
        self.window_height.parse().unwrap_or(600)
    }

    pub fn renderer(&self) -> &'static str {
        match self.force_renderer.as_str() {
            "0" => "software",
            "1" => "opengl",
            "2" => "vulkan",
            "3" => {
                if cfg!(windows) { "d3d12" } else { "opengl" }
            }
            _ => "opengl",
        }
    }
}
