use crate::modules::config::{Config, WindowMode};
use crate::modules::scroll::ScrollPanel;
use crate::modules::ui::{ArrowAnim, ButtonArt, TextTexture};
use sdl3::keyboard::Keycode;
use sdl3::mouse::MouseButton;
use sdl3::pixels::Color;
use sdl3::render::{Canvas, FRect, TextureCreator};
use sdl3::ttf::Font;
use sdl3::video::{Window, WindowContext};
use std::error::Error;

// tabs
const GENERAL: usize = 0;
const KEYBINDS: usize = 1;
const TAB_NAMES: [&str; 2] = ["general", "keybinds"];

struct Opt {
    tab: usize,
    label: &'static str,
}

const OPTS: &[Opt] = &[
    Opt { tab: GENERAL, label: "vsync" },
    Opt { tab: GENERAL, label: "window mode" },
    Opt { tab: GENERAL, label: "window width" },
    Opt { tab: GENERAL, label: "window height" },
    Opt { tab: GENERAL, label: "renderer" },
    Opt { tab: KEYBINDS, label: "forward" },
    Opt { tab: KEYBINDS, label: "backward" },
    Opt { tab: KEYBINDS, label: "left" },
    Opt { tab: KEYBINDS, label: "right" },
    Opt { tab: KEYBINDS, label: "shoot" },
    Opt { tab: KEYBINDS, label: "sights down" },
    Opt { tab: KEYBINDS, label: "i menu" },
    Opt { tab: KEYBINDS, label: "pause" },
    Opt { tab: KEYBINDS, label: "debug menu" },
    Opt { tab: KEYBINDS, label: "vsync key" },
];

const WIDTHS: [&str; 5] = ["640", "800", "1024", "1280", "1920"];
const HEIGHTS: [&str; 5] = ["480", "600", "720", "900", "1080"];

// ids for hover and arrow state, tabs first, then back, then values
const BACK_ID: usize = TAB_NAMES.len();
const ANIM_COUNT: usize = TAB_NAMES.len() + 1 + OPTS.len();

fn value_id(i: usize) -> usize {
    TAB_NAMES.len() + 1 + i
}

pub enum SettingsAction {
    None,
    Clicked,
    Changed,
    Back,
}

enum Capture {
    None,
    Key(usize),
}

// computed positions for one frame
struct Layout {
    title_y: f32,
    view_y: f32,
    view_h: f32,
    content_h: f32,
    block_y: f32,
    tab_x: f32,
    tab_w: f32,
    tab_h: f32,
    tab_gap: f32,
    label_x: f32,
    value_right: f32,
    row_h: f32,
    row_gap: f32,
    back_y: f32,
    hint_y: f32,
}

// settings screen with category tabs and a scrolling option list
pub struct SettingsMenu<'a> {
    labels: Vec<TextTexture<'a>>,
    values: Vec<TextTexture<'a>>,
    title: TextTexture<'a>,
    tabs: Vec<TextTexture<'a>>,
    back_label: TextTexture<'a>,
    hint: TextTexture<'a>,
    art: ButtonArt<'a>,
    tab: usize,
    capture: Capture,
    scroll: ScrollPanel,
    hover: Option<usize>,
    anims: Vec<ArrowAnim>,
}

impl<'a> SettingsMenu<'a> {
    pub fn new(
        creator: &'a TextureCreator<WindowContext>,
        font: &Font,
        title_font: &Font,
    ) -> Result<Self, Box<dyn Error>> {
        let labels = OPTS
            .iter()
            .map(|opt| TextTexture::new(creator, font, opt.label, Color::WHITE))
            .collect::<Result<Vec<_>, _>>()?;
        let values = OPTS
            .iter()
            .map(|_| TextTexture::new(creator, font, " ", Color::WHITE))
            .collect::<Result<Vec<_>, _>>()?;
        let tabs = TAB_NAMES
            .iter()
            .map(|name| TextTexture::new(creator, font, name, Color::WHITE))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            labels,
            values,
            title: TextTexture::new(creator, title_font, "settings", Color::WHITE)?,
            tabs,
            back_label: TextTexture::new(creator, font, "back", Color::WHITE)?,
            hint: TextTexture::new(creator, font, "esc : back | scroll : wheel", Color::RGB(140, 140, 140))?,
            art: ButtonArt::load(creator)?,
            tab: GENERAL,
            capture: Capture::None,
            scroll: ScrollPanel::new(64.0),
            hover: None,
            anims: (0..ANIM_COUNT).map(|_| ArrowAnim::new()).collect(),
        })
    }

    // rebuild value textures from the current config
    pub fn refresh(
        &mut self,
        creator: &'a TextureCreator<WindowContext>,
        font: &Font,
        config: &Config,
    ) -> Result<(), Box<dyn Error>> {
        let capturing = self.capture_index();
        for (i, texture) in self.values.iter_mut().enumerate() {
            let mut text = opt_value(i, config);
            if capturing == Some(i) {
                text.push_str(" <");
            }
            *texture = TextTexture::new(creator, font, &text, Color::WHITE)?;
        }
        Ok(())
    }

    pub fn wheel(&mut self, delta: f32) {
        self.scroll.wheel(delta);
    }

    fn capture_index(&self) -> Option<usize> {
        match self.capture {
            Capture::Key(i) => Some(i),
            Capture::None => None,
        }
    }

    pub fn is_capturing_input(&self) -> bool {
        !matches!(self.capture, Capture::None)
    }

    // element under the mouse, in the same zones clicks use
    fn target_at(&self, x: f32, y: f32, w: f32, h: f32) -> Option<usize> {
        let layout = self.layout(w, h);
        for tab in 0..TAB_NAMES.len() {
            if contains(self.tab_rect(&layout, tab), x, y) {
                return Some(tab);
            }
        }
        let back = FRect::new(layout.tab_x, layout.back_y, layout.tab_w, layout.tab_h);
        if contains(back, x, y) {
            return Some(BACK_ID);
        }
        if y < layout.view_y || y >= layout.view_y + layout.view_h {
            return None;
        }
        for (slot, i) in visible_opts(self.tab).enumerate() {
            let rect = self.value_rect(&layout, slot, self.values[i].width());
            if contains(rect, x, y) {
                return Some(value_id(i));
            }
        }
        None
    }

    // returns true when the mouse enters a new element, for hover sound
    pub fn poll_hover(&mut self, x: f32, y: f32, w: f32, h: f32) -> bool {
        let target = self.target_at(x, y, w, h);
        if target == self.hover {
            return false;
        }
        if let Some(prev) = self.hover {
            self.anims[prev].leave();
        }
        if let Some(cur) = target {
            self.anims[cur].hover();
        }
        self.hover = target;
        target.is_some()
    }

    // clear hover without playing a sound when the menu opens
    pub fn reset_hover(&mut self) {
        if let Some(prev) = self.hover {
            self.anims[prev].leave();
        }
        self.hover = None;
    }

    fn layout(&self, w: f32, h: f32) -> Layout {
        let rows = visible_opts(self.tab).count().max(1) as f32;
        let view_y = h * 0.16;
        let view_h = h * 0.72;
        let row_gap = (h * 0.025).clamp(10.0, 18.0);
        let row_h = (h * 0.115).clamp(56.0, 96.0);
        let content_h = rows * row_h + (rows - 1.0).max(0.0) * row_gap;
        Layout {
            title_y: h * 0.025,
            view_y,
            view_h,
            content_h,
            // tabs and rows stay pinned to the top so they never move between tabs
            block_y: view_y,
            tab_x: w * 0.05,
            tab_w: w * 0.21,
            tab_h: row_h,
            tab_gap: row_gap * 1.6,
            label_x: w * 0.30,
            value_right: w * 0.94,
            row_h,
            row_gap,
            back_y: view_y + view_h - row_h,
            hint_y: h * 0.965,
        }
    }

    fn view_rect(layout: &Layout, w: f32) -> FRect {
        FRect::new(
            layout.label_x,
            layout.view_y,
            w * 0.975 - layout.label_x,
            layout.view_h,
        )
    }

    fn tab_rect(&self, layout: &Layout, tab: usize) -> FRect {
        FRect::new(
            layout.tab_x,
            layout.block_y + tab as f32 * (layout.tab_h + layout.tab_gap),
            layout.tab_w,
            layout.tab_h,
        )
    }

    fn row_y(&self, layout: &Layout, slot: usize) -> f32 {
        layout.block_y + slot as f32 * (layout.row_h + layout.row_gap)
            - self.scroll.offset()
    }

    fn value_rect(&self, layout: &Layout, slot: usize, text_w: u32) -> FRect {
        let w = (layout.tab_w * 1.1).max(text_w as f32 + 32.0);
        FRect::new(
            layout.value_right - w,
            self.row_y(layout, slot),
            w,
            layout.row_h,
        )
    }

    // click a tab, value, or back
    pub fn handle_click(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        config: &mut Config,
    ) -> SettingsAction {
        let layout = self.layout(w, h);
        for tab in 0..TAB_NAMES.len() {
            let rect = self.tab_rect(&layout, tab);
            if contains(rect, x, y) {
                self.anims[tab].press();
                self.tab = tab;
                self.capture = Capture::None;
                self.scroll.reset();
                return SettingsAction::Clicked;
            }
        }
        let back = FRect::new(layout.tab_x, layout.back_y, layout.tab_w, layout.tab_h);
        if contains(back, x, y) {
            self.anims[BACK_ID].press();
            self.capture = Capture::None;
            return SettingsAction::Back;
        }
        if y < layout.view_y || y >= layout.view_y + layout.view_h {
            return SettingsAction::None;
        }
        let indices = visible_opts(self.tab).collect::<Vec<_>>();
        for (slot, &i) in indices.iter().enumerate() {
            let rect = self.value_rect(&layout, slot, self.values[i].width());
            if contains(rect, x, y) {
                self.anims[value_id(i)].press();
                if !activate(i, config) {
                    self.capture = Capture::Key(i);
                }
                return SettingsAction::Changed;
            }
        }
        SettingsAction::None
    }

    // key press while capturing, returns true if the config changed
    pub fn on_key(&mut self, key: Keycode, config: &mut Config) -> bool {
        match self.capture {
            Capture::Key(i) => {
                if key == Keycode::Escape {
                    self.capture = Capture::None;
                    return false;
                }
                let name = key_name(key);
                let changed = set_key(i, config, name);
                if changed {
                    self.capture = Capture::None;
                }
                changed
            }
            Capture::None => false,
        }
    }

    // mouse button while capturing a mouse bind
    pub fn on_mouse(&mut self, button: MouseButton, config: &mut Config) -> bool {
        if let Capture::Key(i) = self.capture {
            let name = match button {
                MouseButton::Left => "LMB",
                MouseButton::Right => "RMB",
                MouseButton::Middle => "MMB",
                _ => return false,
            };
            let changed = set_key(i, config, name.to_string());
            if changed {
                self.capture = Capture::None;
            }
            changed
        } else {
            false
        }
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>) -> Result<(), Box<dyn Error>> {
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        let (w, h) = canvas.output_size()?;
        let (wf, hf) = (w as f32, h as f32);
        let layout = self.layout(wf, hf);
        self.scroll.set_content(layout.content_h, layout.view_h);

        self.title
            .draw_centered(canvas, wf / 2.0, layout.title_y + self.title.height() as f32 / 2.0)?;
        self.hint.draw_centered(canvas, wf / 2.0, layout.hint_y)?;

        // category tabs, selected tab uses the hover art
        for tab in 0..TAB_NAMES.len() {
            let rect = self.tab_rect(&layout, tab);
            let hovered = self.hover == Some(tab);
            self.art.draw_fit(canvas, self.tab == tab || hovered, rect)?;
            let arrows = self.anims[tab].frame(hovered);
            self.art.draw_arrows(canvas, rect, arrows)?;
            self.tabs[tab].draw_centered(canvas, rect.x + rect.w / 2.0, rect.y + rect.h / 2.0)?;
        }

        let back = FRect::new(layout.tab_x, layout.back_y, layout.tab_w, layout.tab_h);
        let back_hover = self.hover == Some(BACK_ID);
        self.art.draw_fit(canvas, back_hover, back)?;
        let arrows = self.anims[BACK_ID].frame(back_hover);
        self.art.draw_arrows(canvas, back, arrows)?;
        self.back_label
            .draw_centered(canvas, back.x + back.w / 2.0, back.y + back.h / 2.0)?;

        // clipped scrolling rows
        let view = Self::view_rect(&layout, wf);
        self.scroll.clip(canvas, view);
        let indices = visible_opts(self.tab).collect::<Vec<_>>();
        for (slot, &i) in indices.iter().enumerate() {
            let row_y = self.row_y(&layout, slot);
            if row_y + layout.row_h < view.y - 4.0 || row_y > view.y + view.h + 4.0 {
                continue;
            }
            let label = &self.labels[i];
            label.draw_at(
                canvas,
                layout.label_x,
                row_y + (layout.row_h - label.height() as f32) / 2.0,
            )?;
            let rect = self.value_rect(&layout, slot, self.values[i].width());
            let hovered = self.hover == Some(value_id(i));
            let active = hovered || self.capture_index() == Some(i);
            self.art.draw_fit(canvas, active, rect)?;
            let arrows = self.anims[value_id(i)].frame(hovered);
            self.art.draw_arrows(canvas, rect, arrows)?;
            let value = &self.values[i];
            value.draw_centered(canvas, rect.x + rect.w / 2.0, rect.y + rect.h / 2.0)?;
        }
        ScrollPanel::unclip(canvas);
        self.scroll.draw_bar(canvas, view)?;
        Ok(())
    }
}

fn visible_opts(tab: usize) -> impl Iterator<Item = usize> {
    OPTS.iter()
        .enumerate()
        .filter(move |(_, opt)| opt.tab == tab)
        .map(|(i, _)| i)
}

fn contains(rect: FRect, x: f32, y: f32) -> bool {
    x >= rect.x && x < rect.x + rect.w && y >= rect.y && y < rect.y + rect.h
}

// value shown in the button for each option
fn opt_value(i: usize, config: &Config) -> String {
    match i {
        0 => on_off(config.game.vsync),
        1 => window_mode_name(config.game.window_mode()),
        2 => config.game.window_width.clone(),
        3 => config.game.window_height.clone(),
        4 => renderer_name(&config.game.force_renderer),
        5 => config.keybinds.forward.clone(),
        6 => config.keybinds.backward.clone(),
        7 => config.keybinds.left.clone(),
        8 => config.keybinds.right.clone(),
        9 => config.keybinds.shoot.clone(),
        10 => config.keybinds.sights_down.clone(),
        11 => config.keybinds.i_menu.clone(),
        12 => config.keybinds.pause.clone(),
        13 => config.keybinds.debug_m.clone(),
        14 => config.keybinds.toggle_vsync.clone(),
        _ => String::new(),
    }
}

// toggle or cycle an option in place, returns true when the config changed
fn activate(i: usize, config: &mut Config) -> bool {
    match i {
        0 => {
            config.game.vsync = !config.game.vsync;
            true
        }
        1 => {
            let next = match config.game.window_mode() {
                WindowMode::Windowed => WindowMode::Fullscreen,
                WindowMode::Fullscreen => WindowMode::BorderlessFullscreen,
                WindowMode::BorderlessFullscreen => WindowMode::BorderlessWindowed,
                WindowMode::BorderlessWindowed => WindowMode::Windowed,
            };
            config.game.window_mode = window_mode_value(next).to_string();
            true
        }
        2 => {
            config.game.window_width = cycle(&WIDTHS, &config.game.window_width);
            true
        }
        3 => {
            config.game.window_height = cycle(&HEIGHTS, &config.game.window_height);
            true
        }
        4 => {
            config.game.force_renderer = cycle_num(&config.game.force_renderer, 4);
            true
        }
        _ => false,
    }
}

fn set_key(i: usize, config: &mut Config, name: String) -> bool {
    let field = match i {
        5 => &mut config.keybinds.forward,
        6 => &mut config.keybinds.backward,
        7 => &mut config.keybinds.left,
        8 => &mut config.keybinds.right,
        9 => &mut config.keybinds.shoot,
        10 => &mut config.keybinds.sights_down,
        11 => &mut config.keybinds.i_menu,
        12 => &mut config.keybinds.pause,
        13 => &mut config.keybinds.debug_m,
        14 => &mut config.keybinds.toggle_vsync,
        _ => return false,
    };
    if *field != name {
        *field = name;
        true
    } else {
        false
    }
}

fn on_off(value: bool) -> String {
    if value { "on".to_string() } else { "off".to_string() }
}

fn window_mode_name(mode: WindowMode) -> String {
    match mode {
        WindowMode::Windowed => "windowed".to_string(),
        WindowMode::Fullscreen => "full screen".to_string(),
        WindowMode::BorderlessFullscreen => "borderless full".to_string(),
        WindowMode::BorderlessWindowed => "borderless win".to_string(),
    }
}

fn window_mode_value(mode: WindowMode) -> &'static str {
    match mode {
        WindowMode::Windowed => "0",
        WindowMode::Fullscreen => "1",
        WindowMode::BorderlessFullscreen => "2",
        WindowMode::BorderlessWindowed => "3",
    }
}

fn renderer_name(value: &str) -> String {
    match value {
        "0" => "software".to_string(),
        "1" => "opengl".to_string(),
        "2" => "vulkan".to_string(),
        "3" => "directx".to_string(),
        _ => "opengl".to_string(),
    }
}

// cycle through a list of string values
fn cycle(values: &[&str], current: &str) -> String {
    let i = values.iter().position(|v| *v == current).unwrap_or(0);
    values[(i + 1) % values.len()].to_string()
}

// cycle a numeric string within [0, max)
fn cycle_num(value: &str, max: u32) -> String {
    let n = value.parse::<u32>().unwrap_or(0);
    ((n + 1) % max).to_string()
}

// short key name for config storage
fn key_name(key: Keycode) -> String {
    match key {
        Keycode::Escape => "ESC".to_string(),
        Keycode::Space => "SPACE".to_string(),
        Keycode::LShift => "SHIFT".to_string(),
        Keycode::LCtrl => "CTRL".to_string(),
        Keycode::LAlt => "ALT".to_string(),
        _ => key.name(),
    }
}
