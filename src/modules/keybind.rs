use crate::modules::config::Keybinds;
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;

// every action the game can react to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    Forward,
    Backward,
    Left,
    Right,
    Shoot,
    SightsDown,
    IMenu,
    Pause,
    Debug,
    ToggleVsync,
}

// a single bound input, either a key or a mouse button
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Key(Keycode),
    Mouse(MouseButton),
}

impl Input {
    // parse config strings like "W", "ESC", "F4", "LMB"
    pub fn parse(value: &str) -> Option<Self> {
        let upper = value.to_uppercase();
        match upper.as_str() {
            "LMB" => Some(Input::Mouse(MouseButton::Left)),
            "RMB" => Some(Input::Mouse(MouseButton::Right)),
            "MMB" => Some(Input::Mouse(MouseButton::Middle)),
            "ESC" | "ESCAPE" => Some(Input::Key(Keycode::Escape)),
            "SPACE" => Some(Input::Key(Keycode::Space)),
            "ENTER" | "RETURN" => Some(Input::Key(Keycode::Return)),
            "TAB" => Some(Input::Key(Keycode::Tab)),
            "SHIFT" => Some(Input::Key(Keycode::LShift)),
            "CTRL" | "CONTROL" => Some(Input::Key(Keycode::LCtrl)),
            "ALT" => Some(Input::Key(Keycode::LAlt)),
            "BACKSPACE" => Some(Input::Key(Keycode::Backspace)),
            "DELETE" => Some(Input::Key(Keycode::Delete)),
            "UP" => Some(Input::Key(Keycode::Up)),
            "DOWN" => Some(Input::Key(Keycode::Down)),
            "LEFT" => Some(Input::Key(Keycode::Left)),
            "RIGHT" => Some(Input::Key(Keycode::Right)),
            _ => {
                // single letters
                if upper.len() == 1 && upper.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
                    let key = match upper.chars().next().unwrap() {
                        'A' => Keycode::A, 'B' => Keycode::B, 'C' => Keycode::C,
                        'D' => Keycode::D, 'E' => Keycode::E, 'F' => Keycode::F,
                        'G' => Keycode::G, 'H' => Keycode::H, 'I' => Keycode::I,
                        'J' => Keycode::J, 'K' => Keycode::K, 'L' => Keycode::L,
                        'M' => Keycode::M, 'N' => Keycode::N, 'O' => Keycode::O,
                        'P' => Keycode::P, 'Q' => Keycode::Q, 'R' => Keycode::R,
                        'S' => Keycode::S, 'T' => Keycode::T, 'U' => Keycode::U,
                        'V' => Keycode::V, 'W' => Keycode::W, 'X' => Keycode::X,
                        'Y' => Keycode::Y, 'Z' => Keycode::Z,
                        _ => return None,
                    };
                    return Some(Input::Key(key));
                }
                // function keys f1-f12
                if let Some(num) = upper.strip_prefix('F') {
                    let key = match num {
                        "1" => Keycode::F1, "2" => Keycode::F2, "3" => Keycode::F3,
                        "4" => Keycode::F4, "5" => Keycode::F5, "6" => Keycode::F6,
                        "7" => Keycode::F7, "8" => Keycode::F8, "9" => Keycode::F9,
                        "10" => Keycode::F10, "11" => Keycode::F11, "12" => Keycode::F12,
                        _ => return None,
                    };
                    return Some(Input::Key(key));
                }
                None
            }
        }
    }
}

// maps raw inputs to game actions
pub struct Bindings {
    map: Vec<(Input, Action)>,
}

impl Bindings {
    pub fn from_config(config: &Keybinds) -> Self {
        let bind = |value: &str, fallback: Input, action: Action| {
            (Input::parse(value).unwrap_or(fallback), action)
        };
        Self {
            map: vec![
                bind(&config.forward, Input::Key(Keycode::W), Action::Forward),
                bind(&config.backward, Input::Key(Keycode::S), Action::Backward),
                bind(&config.left, Input::Key(Keycode::A), Action::Left),
                bind(&config.right, Input::Key(Keycode::D), Action::Right),
                bind(&config.shoot, Input::Mouse(MouseButton::Left), Action::Shoot),
                bind(&config.sights_down, Input::Mouse(MouseButton::Right), Action::SightsDown),
                bind(&config.i_menu, Input::Mouse(MouseButton::Middle), Action::IMenu),
                bind(&config.pause, Input::Key(Keycode::Escape), Action::Pause),
                bind(&config.debug_m, Input::Key(Keycode::F4), Action::Debug),
                bind(&config.toggle_vsync, Input::Key(Keycode::F3), Action::ToggleVsync),
            ],
        }
    }

    pub fn key(&self, keycode: Keycode) -> Action {
        self.action(Input::Key(keycode))
    }

    fn action(&self, input: Input) -> Action {
        for (bound, action) in &self.map {
            if *bound == input {
                return *action;
            }
        }
        Action::None
    }
}
