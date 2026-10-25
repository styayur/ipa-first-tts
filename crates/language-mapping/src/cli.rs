// SPDX-License-Identifier: MPL-2.0
use crate::{espeak::EspeakG2p, *};
use std::{collections::HashMap, io::Read};
pub const COMMANDS: &[&str] = &[
    "languages",
    "schemes",
    "map",
    "romanize",
    "ipa-to-text",
    "normalize-ipa",
    "reverse-romanize",
];
pub fn run(args: &[String]) -> Result<serde_json::Value, String> {
    let command = args.first().map(String::as_str).unwrap_or("languages");
    if !COMMANDS.contains(&command) {
        return Err(
            "Use languages, schemes, map, romanize, ipa-to-text, normalize-ipa or reverse-romanize"
                .into(),
        );
    }
    let engine = Engine::bundled()?;
    let provider = EspeakG2p::discover();
    let mut flags = HashMap::new();
    let mut index = 1;
    if args.get(1).is_some_and(|a| a == "--json") {
        if args.len() != 2 || command != "map" {
            return Err("Usage: ipa-map map --json < request.json".into());
        }
        let mut bytes = Vec::new();
        std::io::stdin()
            .take(16385)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        if bytes.len() > 16384 {
            return Err("JSON request exceeds 16 KiB".into());
        }
        let request: MappingRequest = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        return serde_json::to_value(engine.convert(&request, Some(&provider)))
            .map_err(|e| e.to_string());
    }
    while index < args.len() {
        let flag = &args[index];
        if ![
            "--language",
            "--input",
            "--scheme",
            "--from",
            "--target",
            "--script",
            "--context",
            "--target-language",
        ]
        .contains(&flag.as_str())
        {
            return Err(format!("Unknown mapping flag: {flag}"));
        }
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("Missing value for {flag}"))?;
        if flags.insert(flag.as_str(), value.as_str()).is_some() {
            return Err(format!("Duplicate mapping flag: {flag}"));
        }
        index += 2;
    }
    if command == "languages" {
        if !flags.is_empty() {
            return Err("languages takes no flags".into());
        }
        return Ok(
            serde_json::json!({"schema_version":1,"languages":engine.registry().languages,"scripts":engine.registry().scripts,"sources":engine.registry().sources,"capabilities":engine.capabilities(Some(&provider))}),
        );
    }
    let language = flags
        .get("--language")
        .ok_or("Choose --language (en-US, fr, es, ja, cmn-Hans, ru)")?;
    if command == "schemes" {
        if flags.len() != 1 || engine.registry().language(language).is_none() {
            return Err("schemes requires one valid --language".into());
        }
        return Ok(
            serde_json::json!({"schemes":engine.capabilities(Some(&provider)).into_iter().filter(|s|s.scheme.language.eq_ignore_ascii_case(language)).collect::<Vec<_>>()}),
        );
    }
    let representation = |name: &str| -> Result<Representation, String> {
        match name {
            "orthography" => Ok(Representation::Orthography),
            "romanization" => Ok(Representation::Romanization),
            "ipa" => Ok(Representation::Ipa),
            _ => Err(format!("Unknown representation: {name}")),
        }
    };
    let (input_type, target) = match command {
        "romanize" => (Representation::Orthography, Representation::Romanization),
        "ipa-to-text" => (Representation::Ipa, Representation::Orthography),
        "normalize-ipa" => (Representation::Ipa, Representation::Ipa),
        "reverse-romanize" => (Representation::Romanization, Representation::Orthography),
        _ => (
            representation(flags.get("--from").unwrap_or(&"orthography"))?,
            representation(flags.get("--target").unwrap_or(&"ipa"))?,
        ),
    };
    if command != "map" && (flags.contains_key("--from") || flags.contains_key("--target")) {
        return Err(
            "This command fixes its representations; use map to select --from/--target".into(),
        );
    }
    let request = MappingRequest {
        language: language.to_string(),
        input: flags
            .get("--input")
            .ok_or("Missing --input text")?
            .to_string(),
        input_type,
        target,
        scheme: flags.get("--scheme").map(|s| s.to_string()),
        source_script: flags.get("--script").map(|s| s.to_string()),
        context: flags.get("--context").map(|s| s.to_string()),
        target_language: flags.get("--target-language").map(|s| s.to_string()),
    };
    serde_json::to_value(engine.convert(&request, Some(&provider))).map_err(|e| e.to_string())
}
