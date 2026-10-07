// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Stya Yur Open Source Studio
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. A copy is in LICENSE or https://mozilla.org/MPL/2.0/.
//! Model-independent, conservative IPA IR. Boundaries are explicit items.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::fmt;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stress {
    Primary,
    Secondary,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhoneKind {
    Phone,
    WordBoundary,
    Punctuation,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IpaPhone {
    pub symbol: String,
    pub stress: Option<Stress>,
    pub kind: PhoneKind,
    pub start_ms: Option<f32>,
    pub end_ms: Option<f32>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IpaSequence {
    pub language: Option<String>,
    pub phones: Vec<IpaPhone>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IpaError {
    pub char_index: usize,
    pub message: String,
}
impl fmt::Display for IpaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IPA character {}: {}", self.char_index + 1, self.message)
    }
}
impl std::error::Error for IpaError {}
fn error(i: usize, msg: impl Into<String>) -> IpaError {
    IpaError {
        char_index: i,
        message: msg.into(),
    }
}
fn item(symbol: String, kind: PhoneKind, stress: Option<Stress>) -> IpaPhone {
    IpaPhone {
        symbol,
        kind,
        stress,
        start_ms: None,
        end_ms: None,
    }
}
// Inventory deliberately excludes Latin spelling letters without IPA meaning.
const BASES: &str = "ab cdefhijklmnopqr stuvwxyzɑɐɒæɓʙβɔɕçɗɖðʤəɘɚɛɜɝɞɟʄɡɠɢʛɦɧħɥʜɨɪʝɭɬɫɮɯɰŋɳɲɴøɵɸθœɶʘɹɺɾɻʀʁɽʂʃʈʧʉʊʋⱱʌɣɤχʎʏʑʐʒʔʕʡʢǀǁǂǃ";
fn base(c: char) -> bool {
    c != ' ' && BASES.contains(c)
}
fn punctuation(c: char) -> bool {
    ".,!?;:—…".contains(c)
}
fn modifier(c: char) -> bool {
    is_combining_mark(c) || "ːˑʰʲʷˠˤⁿˡᵊᵝʼ".contains(c)
}

impl IpaSequence {
    /// Accept /.../ and [...] wrappers, normalize to NFD and ASCII whitespace.
    /// Stress attaches to the next phone, never to a boundary or punctuation.
    pub fn parse(input: &str, language: Option<&str>) -> Result<Self, IpaError> {
        let trimmed = input.trim();
        let body = if trimmed.len() >= 2
            && ((trimmed.starts_with('/') && trimmed.ends_with('/'))
                || (trimmed.starts_with('[') && trimmed.ends_with(']')))
        {
            &trimmed[1..trimmed.len().saturating_sub(1)]
        } else {
            trimmed
        };
        let normalized: String = body
            .nfd()
            .map(|c| match c {
                'g' => 'ɡ',
                '\u{035c}' => '\u{0361}',
                _ => c,
            })
            .collect();
        let chars: Vec<char> = normalized.chars().collect();
        let mut phones: Vec<IpaPhone> = Vec::new();
        let mut stress = None;
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            if c == 'ˈ' || c == 'ˌ' {
                if stress.is_some() {
                    return Err(error(i, "consecutive stress markers"));
                }
                stress = Some(if c == 'ˈ' {
                    Stress::Primary
                } else {
                    Stress::Secondary
                });
            } else if c.is_whitespace() || c == '|' {
                if stress.is_some() {
                    return Err(error(i, "stress must precede a phone"));
                }
                if phones
                    .last()
                    .is_some_and(|p| p.kind != PhoneKind::WordBoundary)
                {
                    phones.push(item(" ".into(), PhoneKind::WordBoundary, None));
                }
            } else if punctuation(c) {
                if stress.is_some() {
                    return Err(error(i, "stress cannot attach to punctuation"));
                }
                phones.push(item(c.to_string(), PhoneKind::Punctuation, None));
            } else if base(c) {
                let mut symbol = c.to_string();
                // Tied affricates remain one phone in IR.
                if chars.get(i + 1) == Some(&'͡') {
                    let next = chars
                        .get(i + 2)
                        .copied()
                        .filter(|c| base(*c))
                        .ok_or_else(|| error(i + 1, "tie bar requires two base phones"))?;
                    symbol.push('͡');
                    symbol.push(next);
                    i += 2;
                }
                phones.push(item(symbol, PhoneKind::Phone, stress.take()));
            } else if modifier(c) && c != '͡' {
                let prev = phones
                    .last_mut()
                    .filter(|p| p.kind == PhoneKind::Phone)
                    .ok_or_else(|| error(i, "diacritic/length mark requires a preceding phone"))?;
                if stress.is_some() {
                    return Err(error(i, "stress must precede a base phone"));
                }
                if (c == 'ː' || c == 'ˑ')
                    && (prev.symbol.contains('ː') || prev.symbol.contains('ˑ'))
                {
                    return Err(error(i, "duplicate length mark"));
                }
                prev.symbol.push(c);
            } else {
                return Err(error(
                    i,
                    format!("unsupported IPA symbol '{c}' (U+{:04X})", c as u32),
                ));
            }
            i += 1;
        }
        if stress.is_some() {
            return Err(error(
                chars.len().saturating_sub(1),
                "dangling stress marker",
            ));
        }
        if phones
            .last()
            .is_some_and(|p| p.kind == PhoneKind::WordBoundary)
        {
            phones.pop();
        }
        if !phones.iter().any(|p| p.kind == PhoneKind::Phone) {
            return Err(error(0, "at least one phone is required"));
        }
        Ok(Self {
            language: language.map(str::to_owned),
            phones,
        })
    }
    pub fn canonical(&self) -> String {
        let mut out = String::new();
        for p in &self.phones {
            match p.stress {
                Some(Stress::Primary) => out.push('ˈ'),
                Some(Stress::Secondary) => out.push('ˌ'),
                None => {}
            }
            out.push_str(&p.symbol);
        }
        out
    }
    /// Validate externally constructed / deserialized IR as well as timings.
    pub fn validate(&self) -> Result<(), IpaError> {
        let parsed = Self::parse(&self.canonical(), self.language.as_deref())?;
        if parsed.phones.len() != self.phones.len() {
            return Err(error(0, "IR is not canonical; normalize first"));
        }
        let mut previous_end = 0.0;
        for (i, (p, canonical)) in self.phones.iter().zip(&parsed.phones).enumerate() {
            if p.symbol != canonical.symbol
                || p.kind != canonical.kind
                || p.stress != canonical.stress
            {
                return Err(error(
                    i,
                    "invalid phone kind, stress or non-canonical symbol",
                ));
            }
            match (p.start_ms, p.end_ms) {
                (None, None) => {}
                (Some(s), Some(e))
                    if s.is_finite() && e.is_finite() && s >= previous_end && e >= s =>
                {
                    previous_end = e
                }
                _ => {
                    return Err(error(
                        i,
                        "timings must be paired, finite, nonnegative and monotonic",
                    ))
                }
            }
        }
        Ok(())
    }
    /// Text normalization intentionally clears stale timing information.
    pub fn normalized(&self) -> Result<Self, IpaError> {
        Self::parse(&self.canonical(), self.language.as_deref())
    }
}

/// G2P implementations never depend on a TTS model.
pub trait G2pBackend {
    fn name(&self) -> &str;
    fn generate(&self, text: &str, language: &str) -> Result<IpaSequence, String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stress_and_length() {
        let s = IpaSequence::parse("ˈhiː ˌðɛə", Some("en")).unwrap();
        assert_eq!(s.phones[0].stress, Some(Stress::Primary));
        assert_eq!(s.phones[1].symbol, "iː");
        assert_eq!(s.phones[2].kind, PhoneKind::WordBoundary);
        assert_eq!(s.canonical(), "ˈhiː ˌðɛə");
        s.validate().unwrap();
    }
    #[test]
    fn unicode_normalization_and_ties() {
        let s = IpaSequence::parse(" /  gã\t t͜ʃ  / ", None).unwrap();
        assert_eq!(s.canonical(), "ɡã t͡ʃ");
        assert_eq!(s.phones.last().unwrap().symbol, "t͡ʃ");
        assert_eq!(s.normalized().unwrap(), s);
    }
    #[test]
    fn rejects_malformed() {
        for text in [
            "", "ˈ", "ːa", "aːː", "ˈˌa", "ˈ a", "a😀", "123", "t͡", "/", "[", ".",
        ] {
            assert!(IpaSequence::parse(text, None).is_err(), "accepted {text}");
        }
    }
    #[test]
    fn punctuation_and_boundaries() {
        let s = IpaSequence::parse("[hɛloʊ | wɜːld!]", Some("en-US")).unwrap();
        assert_eq!(s.canonical(), "hɛloʊ wɜːld!");
        assert_eq!(s.phones.last().unwrap().kind, PhoneKind::Punctuation);
    }
    #[test]
    fn timing_validation() {
        let mut s = IpaSequence::parse("ab", None).unwrap();
        s.phones[0].start_ms = Some(0.0);
        s.phones[0].end_ms = Some(20.0);
        s.phones[1].start_ms = Some(10.0);
        s.phones[1].end_ms = Some(30.0);
        assert!(s.validate().is_err());
        s.phones[1].start_ms = Some(20.0);
        s.validate().unwrap();
        s.phones[1].end_ms = Some(f32::NAN);
        assert!(s.validate().is_err());
    }
    #[test]
    fn corrupt_ir_is_rejected() {
        let mut s = IpaSequence::parse("a", None).unwrap();
        s.phones[0].kind = PhoneKind::WordBoundary;
        assert!(s.validate().is_err());
    }
}
