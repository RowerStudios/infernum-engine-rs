use serde::Deserialize;
use std::{error::Error, fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(rename = "discord-rpc")]
    pub discord_rpc: DiscordRpc,
    pub game: Game,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscordRpc {
    pub app_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Game {
    pub window_mode: String,
    pub window_width: String,
    pub window_height: String,
    pub force_renderer: String,
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
                app_id: String::new(),
            },
            game: Game {
                window_mode: "1".to_string(),
                window_width: "800".to_string(),
                window_height: "600".to_string(),
                force_renderer: "1".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let text = fs::read_to_string(path)?;
        Ok(toml::from_str(&text)?)
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
