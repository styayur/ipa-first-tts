// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Stya Yur Open Source Studio
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. A copy is in LICENSE or https://mozilla.org/MPL/2.0/.
#![forbid(unsafe_code)]
use ipa_core::{IpaSequence, Stress};
use std::{collections::HashSet, path::Path};
use tts_core::{ModelPhonemes, PhonemeAdapter};
#[cfg(feature = "sherpa")]
mod sherpa;
#[cfg(feature = "sherpa")]
pub use sherpa::Kokoro;

pub struct KokoroAdapter {
    vocabulary: HashSet<String>,
}
impl KokoroAdapter {
    pub fn from_tokens(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {e}", path.display()))?;
        Self::from_token_text(&text)
    }
    pub fn from_token_text(text: &str) -> Result<Self, String> {
        let mut vocabulary = HashSet::new();
        for (i, line) in text.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            let (symbol, id) = line
                .rsplit_once(' ')
                .ok_or_else(|| format!("Invalid tokens.txt line {}", i + 1))?;
            id.trim()
                .parse::<u32>()
                .map_err(|_| format!("Invalid token ID at line {}", i + 1))?;
            if symbol.is_empty() {
                return Err(format!("Empty token at line {}", i + 1));
            }
            vocabulary.insert(symbol.to_owned());
        }
        if !vocabulary.contains(" ") {
            return Err("Kokoro tokens.txt must contain a space token".into());
        }
        Ok(Self { vocabulary })
    }
}
impl PhonemeAdapter for KokoroAdapter {
    fn adapt(&self, ipa: &IpaSequence) -> Result<ModelPhonemes, String> {
        ipa.validate().map_err(|e| e.to_string())?;
        if ipa
            .language
            .as_deref()
            .is_some_and(|l| !matches!(l, "en" | "en-US" | "en-us" | "en-GB" | "en-gb"))
        {
            return Err("Kokoro adapter currently enables English only".into());
        }
        let mut symbols = Vec::new();
        let mut source_indices = Vec::new();
        for (i, p) in ipa.phones.iter().enumerate() {
            let mapped = match p.symbol.as_str() {
                "t͡ʃ" | "tʃ" => "ʧ".into(),
                "d͡ʒ" | "dʒ" => "ʤ".into(),
                "ɚ" => "əɹ".into(),
                "ɝ" => "ɜɹ".into(),
                "a͡ɪ" => "aɪ".into(),
                "a͡ʊ" => "aʊ".into(),
                "e͡ɪ" => "eɪ".into(),
                "o͡ʊ" => "oʊ".into(),
                "ɔ͡ɪ" => "ɔɪ".into(),
                "ə͡ʊ" => "əʊ".into(),
                "ɪ͡ə" => "ɪə".into(),
                "ɛ͡ə" => "ɛə".into(),
                "ʊ͡ə" => "ʊə".into(),
                _ if p.symbol.contains('͡') => {
                    return Err(format!(
                        "Kokoro cannot map tied phone '{}' at IR item {}",
                        p.symbol,
                        i + 1
                    ))
                }
                _ => p.symbol.clone(),
            };
            let prefix = match p.stress {
                Some(Stress::Primary) => "ˈ",
                Some(Stress::Secondary) => "ˌ",
                None => "",
            };
            for c in prefix.chars().chain(mapped.chars()) {
                let token = c.to_string();
                if !self.vocabulary.contains(&token) {
                    return Err(format!("Kokoro has no token for '{c}' (IR item {}); edit IPA or use another backend", i+1));
                }
                symbols.push(token);
                source_indices.push(i);
            }
        }
        if symbols.len() > 480 {
            return Err("Kokoro MVP supports at most 480 model tokens; split the input".into());
        }
        Ok(ModelPhonemes {
            backend: "kokoro".into(),
            symbols,
            source_indices,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn adapter() -> KokoroAdapter {
        KokoroAdapter::from_token_text("  16\nʧ 82\nʤ 83\nˈ 1\ni 2\nː 3\np 4\nə 5\nɹ 6\n").unwrap()
    }
    #[test]
    fn expands_affricates_stress_and_rhotic_vowels() {
        let result = adapter()
            .adapt(&IpaSequence::parse("ˈt͡ʃiːp ɚ", Some("en")).unwrap())
            .unwrap();
        assert_eq!(result.symbols.concat(), "ˈʧiːp əɹ");
        assert_eq!(result.source_indices, vec![0, 0, 1, 1, 2, 3, 4, 4]);
    }
    #[test]
    fn rejects_unknown_instead_of_dropping() {
        assert!(adapter()
            .adapt(&IpaSequence::parse("θ", None).unwrap())
            .unwrap_err()
            .contains("no token"));
    }
    #[test]
    fn maps_espeak_tied_diphthongs_without_losing_vowels() {
        let adapter = KokoroAdapter::from_token_text("  16\na 1\nʊ 2\no 3\ne 4\nɪ 5\n").unwrap();
        let result = adapter
            .adapt(&IpaSequence::parse("a͡ʊ o͡ʊ e͡ɪ", Some("en-US")).unwrap())
            .unwrap();
        assert_eq!(result.symbols.concat(), "aʊ oʊ eɪ");
        assert_eq!(result.source_indices, vec![0, 0, 1, 2, 2, 3, 4, 4]);
    }
    #[test]
    fn ipa_core_does_not_need_model_inventory() {
        assert!(IpaSequence::parse("ɬã", Some("fr")).is_ok());
    }
}
