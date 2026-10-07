// SPDX-License-Identifier: GPL-3.0-or-later
use alignment_core::{Aligner, UniformAligner};
use ipa_core::{G2pBackend, IpaSequence};
use ipa_desktop::Engine;
use serde::Deserialize;
use serde_json::json;
use std::{collections::VecDeque, io::Read};
use tiny_http::{Header, Method, Request, Response, StatusCode};
use tts_core::SynthesisOptions;

#[derive(Deserialize)]
struct Input {
    #[serde(default)]
    text: String,
    #[serde(default)]
    ipa: String,
    #[serde(default = "language")]
    language: String,
    #[serde(default = "backend")]
    backend: String,
    #[serde(default)]
    speaker: i32,
    #[serde(default = "speed")]
    speed: f32,
}
fn language() -> String {
    "en-US".into()
}
fn backend() -> String {
    "kokoro".into()
}
fn speed() -> f32 {
    1.0
}
fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("constant valid HTTP header")
}
fn respond(request: Request, status: u16, content_type: &str, bytes: Vec<u8>) {
    let response = Response::from_data(bytes).with_status_code(StatusCode(status))
        .with_header(header("Content-Type", content_type))
        .with_header(header("Cache-Control", "no-store"))
        .with_header(header("X-Content-Type-Options", "nosniff"))
        .with_header(header("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self'; media-src 'self' blob:; connect-src 'self'; frame-ancestors 'none'"));
    if let Err(e) = request.respond(response) {
        eprintln!("HTTP response failed: {e}");
    }
}
fn json_response(request: Request, result: Result<serde_json::Value, String>) {
    let (status, value) = match result {
        Ok(v) => (200, v),
        Err(e) => (400, json!({"error":e})),
    };
    respond(
        request,
        status,
        "application/json; charset=utf-8",
        value.to_string().into_bytes(),
    );
}
fn read_input(request: &mut Request) -> Result<Input, String> {
    let content_type = request
        .headers()
        .iter()
        .find(|h| h.field.equiv("Content-Type"))
        .map(|h| h.value.as_str())
        .unwrap_or("");
    if content_type.split(';').next() != Some("application/json") {
        return Err("Expected Content-Type: application/json".into());
    }
    let mut body = Vec::new();
    request
        .as_reader()
        .take(32769)
        .read_to_end(&mut body)
        .map_err(|e| format!("Cannot read request: {e}"))?;
    if body.len() > 32768 {
        return Err("Request exceeds 32 KiB limit".into());
    }
    let input: Input = serde_json::from_slice(&body).map_err(|e| format!("Invalid JSON: {e}"))?;
    if input.ipa.len() > 8192 || input.text.len() > 4096 {
        return Err("Input is too long (text <= 4096 / IPA <= 8192 UTF-8 bytes)".into());
    }
    if !matches!(input.language.as_str(), "en" | "en-US" | "en-GB") {
        return Err("Only English is enabled in this MVP".into());
    }
    Ok(input)
}
pub fn serve(mut engine: Engine, port: u16) -> Result<(), String> {
    let server = tiny_http::Server::http(("127.0.0.1", port))
        .map_err(|e| format!("Cannot bind local UI port {port}: {e}"))?;
    let expected_host = format!("127.0.0.1:{port}");
    let expected_origin = format!("http://{expected_host}");
    println!("IPA-first UI: {expected_origin}\nAll processing is local. Press Ctrl+C to stop.");
    let mut audio_cache: VecDeque<(u64, Vec<u8>)> = VecDeque::new();
    let mut next_id = 0u64;
    for mut request in server.incoming_requests() {
        // Guard the loopback API against DNS rebinding and cross-origin mutation.
        let host_ok = request
            .headers()
            .iter()
            .any(|h| h.field.equiv("Host") && h.value.as_str() == expected_host);
        let origin_ok = request
            .headers()
            .iter()
            .filter(|h| h.field.equiv("Origin"))
            .all(|h| h.value.as_str() == expected_origin);
        if !host_ok || !origin_ok {
            respond(
                request,
                403,
                "text/plain",
                b"Use the printed 127.0.0.1 URL".to_vec(),
            );
            continue;
        }
        let url = request.url().to_owned();
        if request.method() == &Method::Get {
            match url.as_str() {
                "/" => respond(
                    request,
                    200,
                    "text/html; charset=utf-8",
                    include_bytes!("../ui/index.html").to_vec(),
                ),
                "/app.js" => respond(
                    request,
                    200,
                    "text/javascript; charset=utf-8",
                    include_bytes!("../ui/app.js").to_vec(),
                ),
                "/style.css" => respond(
                    request,
                    200,
                    "text/css; charset=utf-8",
                    include_bytes!("../ui/style.css").to_vec(),
                ),
                "/license" => respond(
                    request,
                    200,
                    "text/plain; charset=utf-8",
                    include_bytes!("../../../LICENSE").to_vec(),
                ),
                "/api/status" => json_response(
                    request,
                    Ok(
                        json!({"kokoro_compiled":cfg!(feature="kokoro"),"model_dir":engine.model_dir,"espeak":engine.g2p.executable}),
                    ),
                ),
                _ => {
                    let id = url
                        .strip_prefix("/audio/")
                        .and_then(|id| id.parse::<u64>().ok());
                    if let Some((_, bytes)) = audio_cache.iter().find(|(n, _)| Some(*n) == id) {
                        respond(request, 200, "audio/wav", bytes.clone());
                    } else {
                        respond(
                            request,
                            404,
                            "text/plain",
                            b"Not found or audio expired".to_vec(),
                        );
                    }
                }
            }
        } else if request.method() == &Method::Post
            && matches!(
                url.as_str(),
                "/api/g2p" | "/api/validate" | "/api/synthesize"
            )
        {
            let result = (|| {
                let input = read_input(&mut request)?;
                let ipa = if url == "/api/g2p" {
                    engine.g2p.generate(&input.text, &input.language)?
                } else {
                    IpaSequence::parse(&input.ipa, Some(&input.language))
                        .map_err(|e| e.to_string())?
                };
                if url != "/api/synthesize" {
                    return Ok(json!({"ipa":ipa.canonical(),"sequence":ipa}));
                }
                let audio = engine.synthesize(
                    &ipa,
                    &input.backend,
                    SynthesisOptions {
                        speaker: input.speaker,
                        speed: input.speed,
                    },
                )?;
                let alignment = UniformAligner.align(&ipa, &audio)?;
                next_id += 1;
                audio_cache.push_back((next_id, audio.wav_bytes()?));
                if audio_cache.len() > 4 {
                    audio_cache.pop_front();
                }
                Ok(
                    json!({"ipa":ipa.canonical(),"alignment":alignment,"audio_url":format!("/audio/{next_id}"),"duration_ms":audio.duration_ms(),"sample_rate":audio.sample_rate,"backend":input.backend}),
                )
            })();
            json_response(request, result);
        } else {
            respond(
                request,
                405,
                "text/plain",
                b"Method or route not supported".to_vec(),
            );
        }
    }
    Ok(())
}
