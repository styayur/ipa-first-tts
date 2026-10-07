// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Stya Yur Open Source Studio
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. A copy is in LICENSE or https://mozilla.org/MPL/2.0/.
#![forbid(unsafe_code)]
use ipa_core::{G2pBackend, IpaSequence, Stress};
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};
use tts_core::{Audio, ModelPhonemes, PhonemeAdapter, SynthesisOptions, TtsBackend};

#[derive(Clone)]
pub struct Espeak {
    pub executable: PathBuf,
}
impl Espeak {
    pub fn discover() -> Self {
        let executable = std::env::var_os("ESPEAK_NG")
            .map(PathBuf::from)
            .or_else(|| {
                let local = PathBuf::from("tools/espeak-ng/espeak-ng.exe");
                local.is_file().then_some(local)
            })
            .unwrap_or_else(|| PathBuf::from("espeak-ng"));
        Self { executable }
    }
    pub fn check(&self) -> Result<String, String> {
        self.generate("hello", "en-US")?;
        Ok(format!(
            "eSpeak-NG G2P ready: {}",
            self.executable.display()
        ))
    }
    fn launch_error(&self, e: std::io::Error) -> String {
        format!("Cannot run eSpeak-NG at {}: {e}. Install eSpeak-NG or set ESPEAK_NG to its executable.", self.executable.display())
    }
    fn command(&self) -> Command {
        let mut cmd = Command::new(&self.executable);
        if let Ok(path) = self.executable.canonicalize() {
            if let Some(parent) = path.parent().filter(|p| p.join("espeak-ng-data").is_dir()) {
                // Windows release embeds an installation path; portable builds need this.
                let parent = parent
                    .to_string_lossy()
                    .trim_start_matches("\\\\?\\")
                    .to_string();
                cmd.arg(format!("--path={parent}"));
            }
        }
        // No console flashes when used by the Windows UI.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        cmd
    }
    fn run_stdin(&self, args: &[&str], text: &str) -> Result<Vec<u8>, String> {
        if text.len() > 4096 {
            return Err("Input exceeds the MVP limit of 4096 UTF-8 bytes".into());
        }
        let mut child = self
            .command()
            .args(args)
            .arg("--stdin")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| self.launch_error(e))?;
        // Concurrent writer avoids pipe deadlocks even for large Unicode output.
        let mut stdin = child.stdin.take().ok_or("eSpeak stdin unavailable")?;
        let owned = text.as_bytes().to_vec();
        let writer = std::thread::spawn(move || stdin.write_all(&owned));
        let output = child
            .wait_with_output()
            .map_err(|e| format!("Waiting for eSpeak-NG failed: {e}"))?;
        writer
            .join()
            .map_err(|_| "eSpeak input writer panicked")?
            .map_err(|e| format!("Writing eSpeak input failed: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "eSpeak-NG exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(output.stdout)
    }
}
impl G2pBackend for Espeak {
    fn name(&self) -> &str {
        "espeak-ng"
    }
    fn generate(&self, text: &str, language: &str) -> Result<IpaSequence, String> {
        let voice = match language {
            "en" | "en-US" | "en-us" => "en-us",
            "en-GB" | "en-gb" => "en-gb",
            _ => {
                return Err(format!(
                    "Language '{language}' is reserved but not enabled; use en-US or en-GB"
                ))
            }
        };
        if text.trim().is_empty() || text.contains('\0') {
            return Err("Text must be nonempty and contain no NUL".into());
        }
        let bytes = self.run_stdin(&["-q", "--ipa=2", "-v", voice, "-b", "1"], text)?;
        let ipa =
            String::from_utf8(bytes).map_err(|e| format!("eSpeak returned invalid UTF-8: {e}"))?;
        IpaSequence::parse(&ipa, Some(language)).map_err(|e| format!("eSpeak IPA output: {e}"))
    }
}

/// Explicit English-only fallback. Converts IPA to eSpeak mnemonic input.
pub struct EspeakAdapter;
impl PhonemeAdapter for EspeakAdapter {
    fn adapt(&self, ipa: &IpaSequence) -> Result<ModelPhonemes, String> {
        ipa.validate().map_err(|e| e.to_string())?;
        let mut symbols = Vec::new();
        let mut source_indices = Vec::new();
        for (i, p) in ipa.phones.iter().enumerate() {
            let mut token = match p.stress {
                Some(Stress::Primary) => "'",
                Some(Stress::Secondary) => ",",
                None => "",
            }
            .to_owned();
            let symbol = p.symbol.replace('͡', "");
            let mut chars = symbol.chars().peekable();
            while let Some(c) = chars.next() {
                // eSpeak compound mnemonic sequences: keep diphthongs intact.
                let pair = chars.peek().map(|n| format!("{c}{n}"));
                let compound = match pair.as_deref() {
                    Some("tʃ") => Some("tS"),
                    Some("dʒ") => Some("dZ"),
                    Some("aɪ") => Some("aI"),
                    Some("aʊ") => Some("aU"),
                    Some("eɪ") => Some("eI"),
                    Some("oʊ") => Some("oU"),
                    Some("ɔɪ") => Some("OI"),
                    _ => None,
                };
                if let Some(s) = compound {
                    token.push_str(s);
                    chars.next();
                    continue;
                }
                token.push_str(match c {
                    'ɑ' => "A",
                    'ɐ' | 'ʌ' => "V",
                    'ɒ' => "0",
                    'æ' => "a",
                    'ɔ' => "O",
                    'ə' => "@",
                    'ɚ' => "@r",
                    'ɝ' => "3r",
                    'ɛ' => "E",
                    'ɜ' => "3",
                    'ɡ' => "g",
                    'ɪ' => "I",
                    'ʊ' => "U",
                    'ŋ' => "N",
                    'θ' => "T",
                    'ð' => "D",
                    'ɹ' | 'ɾ' => "r",
                    'ʃ' => "S",
                    'ʒ' => "Z",
                    'ʧ' => "tS",
                    'ʤ' => "dZ",
                    'ː' => ":",
                    ' ' => " ",
                    '.' => ".",
                    ',' | ';' | ':' => ",",
                    '!' => "!",
                    '?' => "?",
                    'a' => "a",
                    'b' => "b",
                    'd' => "d",
                    'e' => "e",
                    'f' => "f",
                    'h' => "h",
                    'i' => "i",
                    'j' => "j",
                    'k' => "k",
                    'l' => "l",
                    'm' => "m",
                    'n' => "n",
                    'o' => "o",
                    'p' => "p",
                    'r' => "r",
                    's' => "s",
                    't' => "t",
                    'u' => "u",
                    'v' => "v",
                    'w' => "w",
                    'z' => "z",
                    _ => {
                        return Err(format!(
                            "eSpeak fallback does not support '{c}' at IR item {}",
                            i + 1
                        ))
                    }
                });
            }
            symbols.push(token);
            source_indices.push(i);
        }
        Ok(ModelPhonemes {
            backend: "espeak-ng".into(),
            symbols,
            source_indices,
        })
    }
}
impl TtsBackend for Espeak {
    fn name(&self) -> &str {
        "espeak-ng fallback (not Kokoro)"
    }
    fn synthesize(&self, ipa: &IpaSequence, options: SynthesisOptions) -> Result<Audio, String> {
        options.validate()?;
        if options.speaker != 0 {
            return Err("eSpeak fallback only supports speaker 0".into());
        }
        let adapted = EspeakAdapter.adapt(ipa)?;
        let input = format!("[[{}]]", adapted.symbols.concat());
        let rate = (175.0 * options.speed) as u32;
        let bytes = self.run_stdin(
            &["--stdout", "-v", "en-us", "-s", &rate.to_string()],
            &input,
        )?;
        decode_wav(&bytes)
    }
}
fn decode_wav(bytes: &[u8]) -> Result<Audio, String> {
    if bytes.len() < 44 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("eSpeak did not return WAV audio".into());
    }
    let mut offset = 12;
    let mut sample_rate = None;
    while offset + 8 <= bytes.len() {
        let size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let start = offset + 8;
        let end = start.saturating_add(size).min(bytes.len());
        match &bytes[offset..offset + 4] {
            b"fmt " if end - start >= 16 => {
                let f = &bytes[start..end];
                if f[..2] != 1u16.to_le_bytes()
                    || f[2..4] != 1u16.to_le_bytes()
                    || f[14..16] != 16u16.to_le_bytes()
                {
                    return Err("eSpeak WAV must be mono PCM16".into());
                }
                sample_rate = Some(u32::from_le_bytes(f[4..8].try_into().unwrap()));
            }
            b"data" => {
                let samples = bytes[start..end]
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|v| i16::from_le_bytes([v[0], v[1]]) as f32 / 32768.0)
                    .collect();
                let audio = Audio {
                    sample_rate: sample_rate.ok_or("WAV has no fmt chunk")?,
                    samples,
                };
                audio.wav_bytes()?;
                return Ok(audio);
            }
            _ => {}
        }
        offset = end.saturating_add(size % 2);
    }
    Err("WAV has no data chunk".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ipa_edit_maps_to_mnemonics() {
        assert_eq!(
            EspeakAdapter
                .adapt(&IpaSequence::parse("ˈt͡ʃiːp", None).unwrap())
                .unwrap()
                .symbols
                .concat(),
            "'tSi:p"
        );
    }
}
