//! Sound resource bank with rate-limiting for high-frequency firing SFX.

use super::procedural_audio::synth_wav;
use super::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

pub struct SoundBank {
    pub shoot: Option<Sound>,
    pub hit: Option<Sound>,
    pub boom: Option<Sound>,
    pub dash: Option<Sound>,
    pub lvlup: Option<Sound>,
    pub shoot_cd: f32,
}

impl SoundBank {
    pub fn empty() -> Self {
        Self {
            shoot: None,
            hit: None,
            boom: None,
            dash: None,
            lvlup: None,
            shoot_cd: 0.0,
        }
    }

    pub async fn load_all(&mut self) {
        self.shoot = load_sound_from_bytes(&synth_wav(460.0, 110.0, 0.07, 0.22, true))
            .await
            .ok();
        self.hit = load_sound_from_bytes(&synth_wav(190.0, 55.0, 0.05, 0.25, false))
            .await
            .ok();
        self.boom = load_sound_from_bytes(&synth_wav(135.0, 24.0, 0.24, 0.42, true))
            .await
            .ok();
        self.dash = load_sound_from_bytes(&synth_wav(170.0, 540.0, 0.14, 0.30, false))
            .await
            .ok();
        self.lvlup = load_sound_from_bytes(&synth_wav(330.0, 880.0, 0.28, 0.32, false))
            .await
            .ok();
    }

    pub fn tick_cooldowns(&mut self, dt: f32) {
        self.shoot_cd = (self.shoot_cd - dt).max(0.0);
    }

    pub fn play(&self, snd: &Option<Sound>, enabled: bool, vol: f32) {
        if !enabled {
            return;
        }
        if let Some(s) = snd {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: vol.clamp(0.0, 1.0),
                },
            );
        }
    }

    pub fn play_shoot_throttled(&mut self, enabled: bool) {
        if self.shoot_cd <= 0.0 {
            self.play(&self.shoot, enabled, 0.5);
            self.shoot_cd = 0.055;
        }
    }
}
