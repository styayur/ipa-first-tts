// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
mod server;
use ipa_core::{G2pBackend, IpaSequence};
use ipa_desktop::{Engine, EXAMPLE};
use std::path::Path;
use tts_core::SynthesisOptions;

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "serve".into());
    if matches!(command.as_str(), "help" | "--help" | "-h") {
        println!("IPA-first TTS\nRun from the project root.\n\n  ipa-tts serve [port]                       Local UI (default port 17842)\n  ipa-tts doctor                             Check local assets\n  ipa-tts g2p \"text\"                         Print canonical English IPA\n  ipa-tts synth \"edited IPA\" out.wav [kokoro|espeak]\n  ipa-tts demo [out.wav] [kokoro|espeak]       Pangram -> IPA -> WAV\n\nESPEAK_NG: eSpeak executable path\nKOKORO_MODEL_DIR: Kokoro asset directory\nIPA_TTS_DEBUG=1: native token diagnostics");
        return Ok(());
    }
    let mut engine = Engine::new();
    match command.as_str() {
        "serve" => {
            let port: u16 = args
                .next()
                .as_deref()
                .unwrap_or("17842")
                .parse()
                .map_err(|_| "Port must be a number from 1 to 65535")?;
            if port == 0 {
                return Err("Port must be nonzero".into());
            }
            server::serve(engine, port)
        }
        "doctor" => {
            println!("{}", engine.g2p.check()?);
            #[cfg(feature = "kokoro")]
            {
                let _ = tts_kokoro::Kokoro::new(&engine.model_dir)?;
                println!(
                    "Kokoro assets and metadata ready: {}",
                    engine.model_dir.display()
                );
            }
            #[cfg(not(feature = "kokoro"))]
            println!("Kokoro feature disabled; eSpeak fallback available");
            Ok(())
        }
        "g2p" => {
            let text = args.next().ok_or("Usage: ipa-tts g2p \"text\"")?;
            println!("{}", engine.g2p.generate(&text, "en-US")?.canonical());
            Ok(())
        }
        "demo" | "synth" => {
            let ipa = if command == "demo" {
                engine.g2p.generate(EXAMPLE, "en-US")?
            } else {
                IpaSequence::parse(
                    &args
                        .next()
                        .ok_or("Usage: ipa-tts synth \"IPA\" out.wav [backend]")?,
                    Some("en-US"),
                )
                .map_err(|e| e.to_string())?
            };
            let output = args
                .next()
                .unwrap_or_else(|| "outputs/quick-brown-fox.wav".into());
            let backend = args.next().unwrap_or_else(|| "kokoro".into());
            println!("IPA: {}", ipa.canonical());
            let audio = engine.synthesize(&ipa, &backend, SynthesisOptions::default())?;
            let path = Path::new(&output);
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create output directory: {e}"))?;
            }
            audio.write_wav(path)?;
            println!(
                "Wrote {}: {:.2} seconds, {} Hz, backend={backend}",
                path.display(),
                audio.duration_ms() / 1000.0,
                audio.sample_rate
            );
            Ok(())
        }
        _ => Err(format!("Unknown command '{command}'; use ipa-tts help")),
    }
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
