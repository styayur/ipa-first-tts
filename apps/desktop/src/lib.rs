// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use g2p_espeak::Espeak;
use ipa_core::IpaSequence;
use std::path::PathBuf;
use tts_core::{Audio, SynthesisOptions, TtsBackend};
pub const EXAMPLE: &str = "The quick brown fox jumps over the lazy dog.";
pub struct Engine {
    pub g2p: Espeak,
    pub model_dir: PathBuf,
    #[cfg(feature = "kokoro")]
    kokoro: Option<tts_kokoro::Kokoro>,
}
impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
impl Engine {
    pub fn new() -> Self {
        Self {
            g2p: Espeak::discover(),
            model_dir: std::env::var_os("KOKORO_MODEL_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("models/kokoro-multi-lang-v1_0")),
            #[cfg(feature = "kokoro")]
            kokoro: None,
        }
    }
    pub fn synthesize(
        &mut self,
        ipa: &IpaSequence,
        backend: &str,
        options: SynthesisOptions,
    ) -> Result<Audio, String> {
        match backend {
            "espeak" => self.g2p.synthesize(ipa, options),
            "kokoro" => {
                #[cfg(feature = "kokoro")]
                {
                    if self.kokoro.is_none() {
                        self.kokoro = Some(tts_kokoro::Kokoro::new(&self.model_dir)?);
                    }
                    self.kokoro
                        .as_ref()
                        .ok_or("Kokoro unavailable")?
                        .synthesize(ipa, options)
                }
                #[cfg(not(feature = "kokoro"))]
                {
                    Err("This build has no Kokoro runtime. Build with default features or select eSpeak fallback.".into())
                }
            }
            _ => Err(format!("Unknown TTS backend '{backend}'")),
        }
    }
}
