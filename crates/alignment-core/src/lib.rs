// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Stya Yur Open Source Studio
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. A copy is in LICENSE or https://mozilla.org/MPL/2.0/.
#![forbid(unsafe_code)]
use ipa_core::{IpaSequence, PhoneKind};
use serde::Serialize;
use tts_core::Audio;
#[derive(Debug, Clone, Copy, Serialize)]
pub enum TimingQuality {
    Estimated,
    Model,
    Forced,
}
#[derive(Debug, Clone, Serialize)]
pub struct Alignment {
    pub sequence: IpaSequence,
    pub quality: TimingQuality,
}
pub trait Aligner {
    fn align(&self, ipa: &IpaSequence, audio: &Audio) -> Result<Alignment, String>;
}
/// UI-only approximation; deliberately never advertised as measured timing.
pub struct UniformAligner;
impl Aligner for UniformAligner {
    fn align(&self, ipa: &IpaSequence, audio: &Audio) -> Result<Alignment, String> {
        ipa.validate().map_err(|e| e.to_string())?;
        if audio.sample_rate == 0 || audio.samples.is_empty() {
            return Err("Cannot align empty audio".into());
        }
        let mut sequence = ipa.clone();
        let count = sequence
            .phones
            .iter()
            .filter(|p| p.kind == PhoneKind::Phone)
            .count();
        let width = audio.duration_ms() / count as f32;
        let mut n = 0;
        for p in &mut sequence.phones {
            p.start_ms = Some(n as f32 * width);
            if p.kind == PhoneKind::Phone {
                n += 1;
            }
            p.end_ms = Some(n as f32 * width);
        }
        Ok(Alignment {
            sequence,
            quality: TimingQuality::Estimated,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn estimates_are_explicit_and_valid() {
        let a = UniformAligner
            .align(
                &IpaSequence::parse("ab c", None).unwrap(),
                &Audio {
                    sample_rate: 1000,
                    samples: vec![0.0; 300],
                },
            )
            .unwrap();
        assert!(matches!(a.quality, TimingQuality::Estimated));
        a.sequence.validate().unwrap();
        assert_eq!(a.sequence.phones.last().unwrap().end_ms, Some(300.0));
    }
}
