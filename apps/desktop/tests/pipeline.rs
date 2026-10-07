// SPDX-License-Identifier: GPL-3.0-or-later
//! Real offline integrations. Explicit opt-in because they require local assets.
use ipa_core::G2pBackend;
use ipa_desktop::{Engine, EXAMPLE};
use tts_core::SynthesisOptions;

fn local_engine() -> Engine {
    let mut engine = Engine::new();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if std::env::var_os("ESPEAK_NG").is_none() {
        let portable = root.join("tools/espeak-ng/espeak-ng.exe");
        if portable.is_file() {
            engine.g2p.executable = portable;
        }
    }
    if std::env::var_os("KOKORO_MODEL_DIR").is_none() {
        engine.model_dir = root.join("models/kokoro-multi-lang-v1_0");
    }
    engine
}

#[test]
#[ignore = "requires eSpeak-NG; run cargo test -p ipa-desktop --test pipeline -- --ignored"]
fn real_espeak_text_ipa_audio() {
    let mut engine = local_engine();
    let ipa = engine.g2p.generate(EXAMPLE, "en-US").expect("real G2P");
    ipa.validate().unwrap();
    let audio = engine
        .synthesize(&ipa, "espeak", SynthesisOptions::default())
        .expect("real fallback synthesis");
    assert!(audio.duration_ms() > 500.0);
    assert!(audio.samples.iter().any(|s| s.abs() > 0.01));
    assert_eq!(&audio.wav_bytes().unwrap()[..4], b"RIFF");
}

#[cfg(feature = "kokoro")]
#[test]
#[ignore = "requires eSpeak-NG and sherpa-compatible Kokoro >=1.0 model assets"]
fn real_kokoro_text_ipa_audio_and_edits() {
    // The public executable keeps its matching runtime DLLs beside the EXE.
    // Native tests in target/debug/deps otherwise find Windows' older ORT DLL.
    let engine = local_engine();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let scratch = std::env::temp_dir().join(format!("ipa-pipeline-{}", std::process::id()));
    std::fs::create_dir(&scratch).unwrap();
    let invoke = |args: &[&str]| {
        std::process::Command::new(env!("CARGO_BIN_EXE_ipa-tts"))
            .args(args)
            .current_dir(&root)
            .env(
                "ESPEAK_NG",
                engine
                    .g2p
                    .executable
                    .canonicalize()
                    .unwrap_or_else(|_| engine.g2p.executable.clone()),
            )
            .env("KOKORO_MODEL_DIR", &engine.model_dir)
            .output()
            .unwrap()
    };
    let pangram_path = scratch.join("pangram.wav");
    let result = invoke(&["demo", pangram_path.to_str().unwrap(), "kokoro"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("IPA: ð"));
    let wav = std::fs::read(&pangram_path).unwrap();
    assert_eq!(&wav[..4], b"RIFF");
    assert_eq!(u32::from_le_bytes(wav[24..28].try_into().unwrap()), 24000);
    let samples: Vec<f32> = wav[44..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|s| i16::from_le_bytes([s[0], s[1]]) as f32 / 32768.0)
        .collect();
    assert!(samples.len() > 24000);
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    assert!(rms > 0.005, "silent output: {rms}");
    let mut edits = Vec::new();
    for (i, ipa) in ["ˈhiː", "ˈʃiː"].iter().enumerate() {
        let path = scratch.join(format!("edit-{i}.wav"));
        let result = invoke(&["synth", ipa, path.to_str().unwrap(), "kokoro"]);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let bytes = std::fs::read(path).unwrap();
        assert!(bytes.len() > 4844);
        edits.push(bytes);
    }
    assert_ne!(edits[0], edits[1], "edited phone was ignored");
    let result = invoke(&[
        "synth",
        "ɬ",
        scratch.join("invalid.wav").to_str().unwrap(),
        "kokoro",
    ]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("no token"));
    // Exclusively created by this test under the OS temp directory.
    std::fs::remove_dir_all(&scratch).unwrap();
}
