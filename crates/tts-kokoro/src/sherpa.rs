// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Stya Yur Open Source Studio
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. A copy is in LICENSE or https://mozilla.org/MPL/2.0/.
use super::KokoroAdapter;
use ipa_core::IpaSequence;
use sherpa_onnx::{
    GenerationConfig, OfflineTts, OfflineTtsConfig, OfflineTtsKokoroModelConfig,
    OfflineTtsModelConfig,
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use tts_core::{Audio, PhonemeAdapter, SynthesisOptions, TtsBackend};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Result<Self, String> {
        for _ in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "ipa-first-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(format!("Cannot create private TTS scratch directory: {e}")),
            }
        }
        Err("Cannot allocate TTS scratch directory".into())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        // Only the absolute, exclusively-created directory owned by this guard.
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn native_path(path: &Path) -> Result<String, String> {
    let path = path
        .canonicalize()
        .map_err(|e| format!("Cannot resolve {}: {e}", path.display()))?;
    let s = path
        .to_str()
        .ok_or("Native model path must be UTF-8")?
        .trim_start_matches("\\\\?\\")
        .to_string();
    if s.contains([',', '\0']) {
        return Err("Native model paths cannot contain a comma or NUL".into());
    }
    Ok(s)
}
pub struct Kokoro {
    directory: PathBuf,
    prepared: PathBuf,
    adapter: KokoroAdapter,
    speaker_count: i32,
    _scratch: Scratch,
}
impl Kokoro {
    pub fn new(directory: impl AsRef<Path>) -> Result<Self, String> {
        let directory = directory.as_ref().to_path_buf();
        for file in [
            "model.onnx",
            "voices.bin",
            "tokens.txt",
            "espeak-ng-data/phondata",
            "espeak-ng-data/phontab",
            "espeak-ng-data/phonindex",
        ] {
            let p = directory.join(file);
            if !p.is_file() {
                return Err(format!(
                    "Missing Kokoro asset {}. See models/README.md and scripts/setup.ps1",
                    p.display()
                ));
            }
        }
        let adapter = KokoroAdapter::from_tokens(&directory.join("tokens.txt"))?;
        let scratch = Scratch::new()?;
        let prepared = scratch.0.join("model.ipa.onnx");
        // sherpa 1.13.8 defaults lang to model metadata voice, bypassing lexicon.
        // Clear ONLY this metadata field in a private copy, never model weights.
        let metadata = prepare_lexicon_model(&directory.join("model.onnx"), &prepared)?;
        let speaker_count = metadata
            .get("n_speakers")
            .and_then(|n| n.parse().ok())
            .filter(|n: &i32| *n > 0)
            .ok_or("Kokoro model has invalid n_speakers metadata")?;
        Ok(Self {
            directory,
            prepared,
            adapter,
            speaker_count,
            _scratch: scratch,
        })
    }
}
impl TtsBackend for Kokoro {
    fn name(&self) -> &str {
        "Kokoro / sherpa-onnx CPU"
    }
    fn synthesize(&self, ipa: &IpaSequence, options: SynthesisOptions) -> Result<Audio, String> {
        options.validate()?;
        if options.speaker >= self.speaker_count {
            return Err(format!("Speaker must be 0..{}", self.speaker_count - 1));
        }
        let phones = self.adapter.adapt(ipa)?;
        let scratch = Scratch::new()?;
        let mut lexicon = String::new();
        let mut text = String::new();
        let mut word: Vec<String> = Vec::new();
        let mut n = 0;
        let flush =
            |word: &mut Vec<String>, n: &mut usize, lexicon: &mut String, text: &mut String| {
                if word.is_empty() {
                    return;
                }
                // Alphabetic aliases avoid native number normalization and OOV lookup.
                let alias = format!(
                    "ipaword{}{}",
                    (b'a' + (*n / 26) as u8) as char,
                    (b'a' + (*n % 26) as u8) as char
                );
                lexicon.push_str(&format!("{alias} {}\n", word.join(" ")));
                text.push_str(&alias);
                text.push(' ');
                word.clear();
                *n += 1;
            };
        for token in &phones.symbols {
            if token == " " {
                flush(&mut word, &mut n, &mut lexicon, &mut text);
            } else if token.chars().all(|c| ".,!?;:—…".contains(c)) {
                flush(&mut word, &mut n, &mut lexicon, &mut text);
                text.push_str(token);
                text.push(' ');
            } else {
                word.push(token.clone());
            }
        }
        flush(&mut word, &mut n, &mut lexicon, &mut text);
        let lexicon_path = scratch.0.join("edited-ipa.txt");
        fs::write(&lexicon_path, lexicon).map_err(|e| format!("Cannot write IPA lexicon: {e}"))?;
        let config = OfflineTtsConfig {
            model: OfflineTtsModelConfig {
                kokoro: OfflineTtsKokoroModelConfig {
                    model: Some(native_path(&self.prepared)?),
                    voices: Some(native_path(&self.directory.join("voices.bin"))?),
                    tokens: Some(native_path(&self.directory.join("tokens.txt"))?),
                    data_dir: Some(native_path(&self.directory.join("espeak-ng-data"))?),
                    lexicon: Some(native_path(&lexicon_path)?),
                    lang: None,
                    ..Default::default()
                },
                num_threads: 2,
                debug: std::env::var_os("IPA_TTS_DEBUG").is_some(),
                ..Default::default()
            },
            max_num_sentences: 1,
            silence_scale: 1.0,
            ..Default::default()
        };
        // Recreate per utterance because the upstream frontend caches its lexicon.
        let tts = OfflineTts::create(&config)
            .ok_or("sherpa-onnx could not initialize Kokoro; check compatible model assets")?;
        let generation = GenerationConfig {
            sid: options.speaker,
            speed: options.speed,
            silence_scale: 1.0,
            ..Default::default()
        };
        let generated = tts
            .generate_with_config(&text, &generation, None::<fn(&[f32], f32) -> bool>)
            .ok_or("Kokoro inference failed")?;
        let audio = Audio {
            sample_rate: generated.sample_rate() as u32,
            samples: generated.samples().to_vec(),
        };
        audio.wav_bytes()?;
        Ok(audio)
    }
}

// Minimal streaming protobuf reader. Never parses graph weights or changes them.
fn varint(reader: &mut impl Read) -> std::io::Result<(u64, Vec<u8>)> {
    let mut value = 0u64;
    let mut raw = Vec::new();
    for shift in (0..70).step_by(7) {
        let mut b = [0];
        reader.read_exact(&mut b)?;
        if shift == 63 && b[0] > 1 {
            return Err(std::io::Error::other("protobuf varint overflow"));
        }
        raw.push(b[0]);
        value |= ((b[0] & 127) as u64) << shift;
        if b[0] & 128 == 0 {
            return Ok((value, raw));
        }
    }
    Err(std::io::Error::other("invalid protobuf varint"))
}
fn encoded(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    while value >= 128 {
        bytes.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}
fn prepare_lexicon_model(
    source: &Path,
    destination: &Path,
) -> Result<HashMap<String, String>, String> {
    fn prepare(source: &Path, destination: &Path) -> std::io::Result<HashMap<String, String>> {
        let mut input = File::open(source)?;
        let length = input.metadata()?.len();
        let mut out = File::create(destination)?;
        let mut metadata = HashMap::new();
        while input.stream_position()? < length {
            let (tag, tag_raw) = varint(&mut input)?;
            if tag == 0 {
                return Err(std::io::Error::other("zero protobuf field tag"));
            }
            if tag >> 3 == 14 && tag & 7 == 2 {
                let (len, _) = varint(&mut input)?;
                if len > 1_048_576 || len > length - input.stream_position()? {
                    return Err(std::io::Error::other("invalid ONNX metadata length"));
                }
                let mut record = vec![0; len as usize];
                input.read_exact(&mut record)?;
                let mut cursor = std::io::Cursor::new(&record);
                let mut values = HashMap::new();
                while cursor.position() < len {
                    let (t, _) = varint(&mut cursor)?;
                    let (l, _) = varint(&mut cursor)?;
                    if t & 7 != 2 || l > len - cursor.position() {
                        return Err(std::io::Error::other("invalid ONNX metadata entry"));
                    }
                    let mut v = vec![0; l as usize];
                    cursor.read_exact(&mut v)?;
                    values.insert(t >> 3, String::from_utf8(v).map_err(std::io::Error::other)?);
                }
                let key = values
                    .get(&1)
                    .ok_or_else(|| std::io::Error::other("missing metadata key"))?;
                let value = values.get(&2).cloned().unwrap_or_default();
                if metadata.insert(key.clone(), value).is_some() {
                    return Err(std::io::Error::other("duplicate ONNX metadata key"));
                }
                if key == "voice" {
                    record = b"\x0a\x05voice\x12\x00".to_vec();
                }
                out.write_all(&tag_raw)?;
                out.write_all(&encoded(record.len() as u64))?;
                out.write_all(&record)?;
            } else {
                out.write_all(&tag_raw)?;
                let size = match tag & 7 {
                    0 => {
                        let (_, raw) = varint(&mut input)?;
                        out.write_all(&raw)?;
                        continue;
                    }
                    1 => 8,
                    5 => 4,
                    2 => {
                        let (n, raw) = varint(&mut input)?;
                        out.write_all(&raw)?;
                        n
                    }
                    _ => return Err(std::io::Error::other("unsupported ONNX protobuf wire type")),
                };
                if size > length - input.stream_position()? {
                    return Err(std::io::Error::other("truncated ONNX model"));
                }
                // io::copy uses a bounded buffer, even for a 325 MB graph.
                let copied = std::io::copy(&mut (&mut input).take(size), &mut out)?;
                if copied != size {
                    return Err(std::io::Error::other("truncated ONNX model"));
                }
            }
        }
        out.flush()?;
        if metadata.get("model_type").map(String::as_str) != Some("kokoro")
            || metadata
                .get("version")
                .and_then(|v| v.parse::<u32>().ok())
                .unwrap_or(0)
                < 2
            || !metadata.contains_key("voice")
        {
            return Err(std::io::Error::other("Expected sherpa Kokoro >= 1.0 (metadata version >= 2), with voice metadata; v0.19 is unsupported"));
        }
        Ok(metadata)
    }
    prepare(source, destination)
        .map_err(|e| format!("Cannot prepare IPA-first model {}: {e}", source.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preparation_changes_only_voice_metadata() {
        fn entry(key: &str, value: &str) -> Vec<u8> {
            let mut r = vec![10];
            r.extend(encoded(key.len() as u64));
            r.extend(key.bytes());
            r.push(18);
            r.extend(encoded(value.len() as u64));
            r.extend(value.bytes());
            let mut out = vec![114];
            out.extend(encoded(r.len() as u64));
            out.extend(r);
            out
        }
        let scratch = Scratch::new().unwrap();
        let src = scratch.0.join("src.onnx");
        let dst = scratch.0.join("dst.onnx");
        let mut bytes = vec![58, 4, 1, 2, 3, 4]; // opaque graph bytes
        bytes.extend(entry("model_type", "kokoro"));
        bytes.extend(entry("version", "2"));
        bytes.extend(entry("voice", "en-us"));
        fs::write(&src, &bytes).unwrap();
        prepare_lexicon_model(&src, &dst).unwrap();
        let mut expected = bytes[..bytes.len() - entry("voice", "en-us").len()].to_vec();
        expected.extend(entry("voice", ""));
        assert_eq!(fs::read(&src).unwrap(), bytes);
        assert_eq!(fs::read(&dst).unwrap(), expected);
    }
    #[test]
    fn malformed_model_returns_error() {
        let scratch = Scratch::new().unwrap();
        let src = scratch.0.join("bad.onnx");
        fs::write(
            &src,
            [58, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255],
        )
        .unwrap();
        assert!(prepare_lexicon_model(&src, &scratch.0.join("out.onnx")).is_err());
    }
}
