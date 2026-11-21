//! Procedural PCM WAV synthesizer for Neon Core's retro-cyberpunk sound effects.

/// Synthesizes a 16-bit mono 22.05 kHz WAV buffer with exponential frequency sweep.
pub fn synth_wav(
    freq_start: f32,
    freq_end: f32,
    duration: f32,
    volume: f32,
    square: bool,
) -> Vec<u8> {
    let sample_rate = 22_050_u32;
    let safe_dur = if duration.is_finite() {
        duration.clamp(0.005, 5.0)
    } else {
        0.05
    };
    let safe_start = if freq_start.is_finite() {
        freq_start.max(1.0)
    } else {
        220.0
    };
    let safe_end = if freq_end.is_finite() {
        freq_end.max(1.0)
    } else {
        110.0
    };
    let safe_vol = if volume.is_finite() {
        volume.clamp(0.0, 1.0)
    } else {
        0.25
    };

    let num_samples = (sample_rate as f32 * safe_dur) as usize;
    let data_size = (num_samples * 2) as u32;
    let mut buf = Vec::with_capacity(44 + data_size as usize);

    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(36 + data_size).to_le_bytes());
    buf.extend_from_slice(b"WAVEfmt ");
    buf.extend_from_slice(&16_u32.to_le_bytes());
    buf.extend_from_slice(&1_u16.to_le_bytes());
    buf.extend_from_slice(&1_u16.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    buf.extend_from_slice(&2_u16.to_le_bytes());
    buf.extend_from_slice(&16_u16.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());

    let inv_samples = 1.0_f32 / num_samples.max(1) as f32;
    let ratio = safe_end / safe_start;
    let mut phase = 0.0_f32;
    for i in 0..num_samples {
        let t = i as f32 * inv_samples;
        let freq = safe_start * ratio.powf(t);
        phase += freq / sample_rate as f32;
        let wave = if square {
            if phase.fract() < 0.5 {
                1.0_f32
            } else {
                -1.0_f32
            }
        } else {
            (phase * std::f32::consts::TAU).sin()
        };
        let env = (1.0_f32 - t).powf(1.6_f32) * safe_vol;
        let sample = (wave * env * 32_767.0_f32).clamp(-32_767.0_f32, 32_767.0_f32) as i16;
        buf.extend_from_slice(&sample.to_le_bytes());
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synth_wav_produces_valid_riff_header_and_length() {
        let wav = synth_wav(460.0, 110.0, 0.1, 0.25, true);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(&wav[36..40], b"data");
        let data_len = u32::from_le_bytes([wav[40], wav[41], wav[42], wav[43]]) as usize;
        assert_eq!(wav.len(), 44 + data_len);
        assert_eq!(data_len, 2205 * 2);
    }

    #[test]
    fn synth_wav_handles_non_finite_parameters_safely() {
        let wav = synth_wav(f32::NAN, -50.0, f32::INFINITY, 2.5, false);
        assert!(wav.len() > 44);
        assert_eq!(&wav[0..4], b"RIFF");
    }
}
