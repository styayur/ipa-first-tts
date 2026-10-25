// SPDX-License-Identifier: MPL-2.0
use language_mapping::*;
use serde::Deserialize;
fn request(language: &str, scheme: &str, input: &str, target: Representation) -> MappingRequest {
    MappingRequest {
        language: language.into(),
        scheme: Some(scheme.into()),
        input: input.into(),
        input_type: Representation::Orthography,
        target,
        source_script: None,
        target_language: None,
        context: None,
    }
}
#[derive(Deserialize)]
struct Fixtures {
    cases: Vec<Fixture>,
}
#[derive(Deserialize)]
struct Fixture {
    language: String,
    scheme: String,
    input: String,
    expected: Vec<String>,
    source: String,
    reference: String,
}
#[test]
fn independent_language_fixtures_and_traces() {
    let engine = Engine::bundled().unwrap();
    let fixtures: Fixtures = serde_json::from_str(include_str!("fixtures/builtin.json")).unwrap();
    for f in fixtures.cases {
        assert!(!f.source.is_empty() && !f.reference.is_empty());
        let target = if f.language == "en-US" {
            Representation::Ipa
        } else {
            Representation::Romanization
        };
        let result = engine.convert(&request(&f.language, &f.scheme, &f.input, target), None);
        assert_eq!(
            result
                .candidates
                .iter()
                .map(|c| c.text.as_str())
                .collect::<Vec<_>>(),
            f.expected.iter().map(String::as_str).collect::<Vec<_>>(),
            "{} {} {:?}",
            f.language,
            f.input,
            result.unsupported_reason
        );
        assert_eq!(result.original_input, f.input);
        assert!(!result.sources.is_empty());
        for c in result.candidates {
            for trace in c.traces {
                assert!(
                    engine.registry().rules.iter().any(|r| r.id == trace.rule)
                        || ALGORITHM_RULES.contains(&trace.rule.as_str())
                );
                assert!(!trace.reference.is_empty());
            }
        }
    }
}
#[test]
fn homophones_heteronyms_and_exact_bounded_reverse() {
    let engine = Engine::bundled().unwrap();
    let mut req = request(
        "en-US",
        "cmudict-us-candidates",
        "/sˈi/",
        Representation::Orthography,
    );
    req.input_type = Representation::Ipa;
    let result = engine.convert(&req, None);
    assert_eq!(result.status, Status::Ambiguous);
    assert_eq!(
        result
            .candidates
            .iter()
            .map(|c| c.text.as_str())
            .collect::<Vec<_>>(),
        ["sea", "see"]
    );
    assert!(!result.determined);
    req.input = "/siː/".into();
    assert_eq!(engine.convert(&req, None).status, Status::Unsupported);
    let mut req = request("en-US", "cmudict-us-sample", "read", Representation::Ipa);
    req.context = Some("I read yesterday".into());
    let result = engine.convert(&req, None);
    assert!(result.ambiguous);
    assert_eq!(result.candidates.len(), 2);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("Context is preserved")));
}
#[test]
fn russian_roundtrip_only_for_promised_scope() {
    let engine = Engine::bundled().unwrap();
    for input in [
        "Москва",
        "ЁЙЪЬ абвгдеёжзийклмнопрстуфхцчшщъыьэюя",
        "Юлия-Щука",
    ] {
        let forward = engine.convert(
            &request(
                "ru",
                "cldr49-ru-subset",
                input,
                Representation::Romanization,
            ),
            None,
        );
        assert_eq!(forward.status, Status::Ok);
        let mut req = request(
            "ru",
            "cldr49-ru-inverse",
            &forward.candidates[0].text,
            Representation::Orthography,
        );
        req.input_type = Representation::Romanization;
        assert_eq!(engine.convert(&req, None).candidates[0].text, input);
    }
}
#[test]
fn context_original_unicode_and_typed_edges() {
    let engine = Engine::bundled().unwrap();
    let mut req = request(
        "cmn-Hans",
        "cldr49-han-readings",
        "重",
        Representation::Romanization,
    );
    req.context = Some("重庆".into());
    let result = engine.convert(&req, None);
    assert_eq!(result.candidates[0].text, "chóng");
    assert_eq!(result.candidates.len(), 1);
    assert!(!result.determined);
    let mut req = request(
        "ja",
        "cldr49-kana-subset",
        "か\u{3099}っこう",
        Representation::Romanization,
    );
    let result = engine.convert(&req, None);
    assert_eq!(result.original_input, req.input);
    assert_eq!(result.candidates[0].text, "gakkou");
    req.source_script = Some("Kana".into());
    assert_eq!(engine.convert(&req, None).status, Status::Unsupported);
    req = request("ru", "cldr49-ru-subset", "Москва", Representation::Ipa);
    assert_eq!(engine.convert(&req, None).status, Status::Unsupported);
    req = request("ja", "hepburn", "し", Representation::Romanization);
    assert_eq!(engine.convert(&req, None).status, Status::Unsupported);
    req = request("en-US", "cmudict-us-sample", "read", Representation::Ipa);
    req.scheme = None;
    assert_eq!(engine.convert(&req, None).status, Status::NeedsSelection);
    req.language = "en".into();
    assert_eq!(engine.convert(&req, None).status, Status::Unsupported);
}
#[test]
fn unsupported_never_drops_or_fakes_success() {
    let engine = Engine::bundled().unwrap();
    for (language, scheme, input, target) in [
        (
            "ja",
            "cldr49-kana-subset",
            "学校",
            Representation::Romanization,
        ),
        (
            "ja",
            "cldr49-kana-subset",
            "っ",
            Representation::Romanization,
        ),
        (
            "ja",
            "cldr49-kana-subset",
            "ー",
            Representation::Romanization,
        ),
        (
            "ja",
            "cldr49-kana-subset",
            "かな😀",
            Representation::Romanization,
        ),
        (
            "ru",
            "cldr49-ru-subset",
            "cat",
            Representation::Romanization,
        ),
        (
            "cmn-Hans",
            "cldr49-han-readings",
            "中😀",
            Representation::Romanization,
        ),
        (
            "en-US",
            "cmudict-us-sample",
            "qzxzznotaword",
            Representation::Ipa,
        ),
        ("fr", "espeak-fr-fr", "eau", Representation::Ipa),
    ] {
        let result = engine.convert(&request(language, scheme, input, target), None);
        assert_eq!(result.status, Status::Unsupported);
        assert!(result.candidates.is_empty());
        assert_eq!(result.original_input, input);
        assert!(result.unsupported_reason.is_some());
    }
}
#[test]
fn malformed_data_sources_ids_types_and_collisions_rejected() {
    assert!(Engine::from_json("{").is_err());
    let original = Registry::bundled().unwrap();
    let mut registry = original.clone();
    registry.schema_version = 2;
    assert!(Engine::from_registry(registry).is_err());
    let mut r = original.clone();
    r.rules.push(r.rules[0].clone());
    assert!(Engine::from_registry(r).is_err());
    let mut r = original.clone();
    r.rules[0].source = "absent".into();
    assert!(Engine::from_registry(r).is_err());
    let mut r = original.clone();
    r.rules[0].version = "unfixed".into();
    assert!(Engine::from_registry(r).is_err());
    let mut r = original.clone();
    r.schemes[0].output_type = Representation::Romanization;
    assert!(Engine::from_registry(r).is_err());
    let mut r = original.clone();
    r.languages[0].tag = "xx-Madeup".into();
    assert!(Engine::from_registry(r).is_err());
    let mut r = original;
    r.scripts[0].code = "IpaX".into();
    assert!(Engine::from_registry(r).is_err());
}
#[test]
fn ipa_normalization_is_not_language_or_tts_validation() {
    let engine = Engine::bundled().unwrap();
    let mut req = request("fr", "ipa-normalize-fr", "/gã t͜ʃ/", Representation::Ipa);
    req.input_type = Representation::Ipa;
    let result = engine.convert(&req, None);
    assert_eq!(result.candidates[0].text, "ɡã t͡ʃ");
    assert_eq!(result.candidates[0].ipa_grammar_accepted, Some(true));
    assert!(result.warnings.iter().any(|s| s.contains("phonology")));
    req.input = "˥".into();
    assert_eq!(engine.convert(&req, None).status, Status::Unsupported);
}
