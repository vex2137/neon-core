//! Platform-gated audio backend and procedural `SoundBank`.

pub mod procedural_audio;
pub mod sound_bank;

#[cfg(not(target_arch = "wasm32"))]
pub use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
mod wasm_audio {
    #[derive(Clone, Copy)]
    pub struct Sound;

    #[derive(Clone, Copy)]
    pub struct PlaySoundParams {
        pub looped: bool,
        pub volume: f32,
    }

    pub async fn load_sound_from_bytes(_data: &[u8]) -> Result<Sound, ()> {
        Ok(Sound)
    }

    pub fn play_sound(_sound: &Sound, _params: PlaySoundParams) {}
}

#[cfg(target_arch = "wasm32")]
pub use wasm_audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

pub use sound_bank::SoundBank;
