use sdl3::mixer::{Audio, Mixer};
use std::{error::Error, fs, time::SystemTime};

const CLICK_PATH: &str = "src/sounds/UI/press.ogg";
const HOVER_PATH: &str = "src/sounds/UI/hover.ogg";
const MUSIC_PATH: &str = "src/sounds/main_menu_mus.mp3";
const VOICE_DIR: &str = "src/sounds/Voices/quit_voices";

// audios must drop before mixer and context, MIX_Quit frees everything
pub struct Sound {
    click: Audio,
    hover: Audio,
    voices: Vec<Audio>,
    mixer: Mixer,
    _context: sdl3::mixer::MixerContext,
}

impl Sound {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let context = sdl3::mixer::init()?;
        let mixer = Mixer::open_device(None)?;
        mixer.set_gain(0.5)?;
        let click = mixer.load_audio(CLICK_PATH, true)?;
        let hover = mixer.load_audio(HOVER_PATH, true)?;

        let mut voices = Vec::new();
        if let Ok(entries) = fs::read_dir(VOICE_DIR) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "mp3") {
                    match mixer.load_audio(&path, true) {
                        Ok(audio) => voices.push(audio),
                        Err(err) => println!(">> failed: {} ({err})", path.display()),
                    }
                }
            }
        }
        if voices.is_empty() {
            println!(">> no quit voices found in '{VOICE_DIR}'");
        }

        match mixer.load_audio(MUSIC_PATH, false) {
            Ok(music) => {
                let track = mixer.create_track()?;
                track.set_audio(&music)?;
                track.set_loops(-1)?;
                // tag lets us pause just the menu track
                track.tag("menu_music")?;
                track.play()?;
                // mixer destroys the track when dropped
                std::mem::forget(track);
                println!(">> music playing: {MUSIC_PATH}");
            }
            Err(err) => println!(">> music unavailable: {err}"),
        }

        Ok(Self {
            mixer,
            _context: context,
            click,
            hover,
            voices,
        })
    }

    pub fn play_click(&self) -> Result<(), Box<dyn Error>> {
        self.mixer.play_audio(&self.click)?;
        Ok(())
    }

    pub fn play_hover(&self) -> Result<(), Box<dyn Error>> {
        self.mixer.play_audio(&self.hover)?;
        Ok(())
    }

    // menu music only plays on the main menu
    pub fn pause_music(&self) -> Result<(), Box<dyn Error>> {
        self.mixer.pause_tag("menu_music")?;
        Ok(())
    }

    pub fn resume_music(&self) -> Result<(), Box<dyn Error>> {
        self.mixer.resume_tag("menu_music")?;
        Ok(())
    }

    // stop music, play a random quit voice, return its length in ms
    pub fn play_quit_voice(&self) -> Result<u64, Box<dyn Error>> {
        self.mixer.stop_all(0)?;
        if self.voices.is_empty() {
            return Err("no quit voices loaded".into());
        }
        let voice = &self.voices[pick() % self.voices.len()];
        self.mixer.play_audio(voice)?;
        let ms = voice.frames_to_ms(voice.duration()).max(0) as u64;
        Ok(ms + 100)
    }
}

// simple random index source without extra dependencies
fn pick() -> usize {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0)
}
