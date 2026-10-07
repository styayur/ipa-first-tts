// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Stya Yur Open Source Studio
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. A copy is in LICENSE or https://mozilla.org/MPL/2.0/.
#![forbid(unsafe_code)]
use ipa_core::IpaSequence;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPhonemes {
    pub backend: String,
    pub symbols: Vec<String>,
    /// Maps model tokens back to IR items (one phone may expand to several tokens).
    pub source_indices: Vec<usize>,
}
pub trait PhonemeAdapter {
    fn adapt(&self, ipa: &IpaSequence) -> Result<ModelPhonemes, String>;
}
#[derive(Debug, Clone, Copy)]
pub struct SynthesisOptions {
    pub speaker: i32,
    pub speed: f32,
}
impl Default for SynthesisOptions {
    fn default() -> Self {
        Self {
            speaker: 0,
            speed: 1.0,
        }
    }
}
impl SynthesisOptions {
    pub fn validate(self) -> Result<(), String> {
        if self.speaker < 0 {
            return Err("Speaker must be nonnegative".into());
        }
        if !self.speed.is_finite() || !(0.25..=4.0).contains(&self.speed) {
            return Err("Speed must be finite and between 0.25 and 4".into());
        }
        Ok(())
    }
}
#[derive(Debug, Clone)]
pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}
impl Audio {
    pub fn duration_ms(&self) -> f32 {
        self.samples.len() as f32 * 1000.0 / self.sample_rate as f32
    }
    pub fn wav_bytes(&self) -> Result<Vec<u8>, String> {
        if self.sample_rate == 0
            || self.samples.is_empty()
            || self.samples.iter().any(|v| !v.is_finite())
        {
            return Err("Audio is empty or contains invalid samples/sample rate".into());
        }
        let size = self
            .samples
            .len()
            .checked_mul(2)
            .and_then(|n| u32::try_from(n).ok())
            .filter(|n| *n <= u32::MAX - 36)
            .ok_or("WAV exceeds RIFF size limit")?;
        let byte_rate = self
            .sample_rate
            .checked_mul(2)
            .ok_or("Invalid WAV sample rate")?;
        let mut out = Vec::with_capacity(size as usize + 44);
        out.extend(b"RIFF");
        out.extend((size + 36).to_le_bytes());
        out.extend(b"WAVEfmt ");
        out.extend(16u32.to_le_bytes());
        out.extend(1u16.to_le_bytes());
        out.extend(1u16.to_le_bytes());
        out.extend(self.sample_rate.to_le_bytes());
        out.extend(byte_rate.to_le_bytes());
        out.extend(2u16.to_le_bytes());
        out.extend(16u16.to_le_bytes());
        out.extend(b"data");
        out.extend(size.to_le_bytes());
        for &s in &self.samples {
            out.extend(((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
        Ok(out)
    }
    pub fn write_wav(&self, path: &Path) -> Result<(), String> {
        std::fs::write(path, self.wav_bytes()?)
            .map_err(|e| format!("Cannot write {}: {e}", path.display()))
    }
}
/// TTS receives IPA only; no original text is available to silently re-phonemize.
pub trait TtsBackend {
    fn name(&self) -> &str;
    fn synthesize(&self, ipa: &IpaSequence, options: SynthesisOptions) -> Result<Audio, String>;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_pcm_header_and_clipping() {
        let wav = Audio {
            sample_rate: 24000,
            samples: vec![0.0, 2.0, -2.0],
        }
        .wav_bytes()
        .unwrap();
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(wav.len(), 50);
        assert_eq!(i16::from_le_bytes([wav[46], wav[47]]), 32767);
    }
    #[test]
    fn reject_invalid_audio() {
        assert!(Audio {
            sample_rate: 0,
            samples: vec![0.0]
        }
        .wav_bytes()
        .is_err());
        assert!(SynthesisOptions {
            speaker: 0,
            speed: f32::NAN
        }
        .validate()
        .is_err());
    }
}
