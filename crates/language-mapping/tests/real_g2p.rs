// SPDX-License-Identifier: MPL-2.0
use language_mapping::{cli, espeak::EspeakG2p, *};
#[test]
#[ignore = "requires pinned local eSpeak NG 1.52.0; run explicitly after scripts/setup.ps1 -SkipModel"]
fn independent_french_spanish_and_newline_regression() {
    let engine = Engine::bundled().unwrap();
    let provider = EspeakG2p {
        executable: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tools/espeak-ng/espeak-ng.exe"),
    };
    assert!(provider.available());
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/external.json")).unwrap();
    for f in fixtures["cases"].as_array().unwrap() {
        let request:MappingRequest=serde_json::from_value(serde_json::json!({"language":f["language"],"scheme":f["scheme"],"input":f["input"],"target":"ipa"})).unwrap();
        let result = engine.convert(&request, Some(&provider));
        assert!(
            !result.candidates.is_empty(),
            "{:?}",
            result.unsupported_reason
        );
        let segmental: String = result.candidates[0]
            .text
            .chars()
            .filter(|c| !"ˈˌ.".contains(*c))
            .collect();
        assert_eq!(
            segmental,
            f["segmental"].as_str().unwrap(),
            "{} independent reference {}",
            request.input,
            f["source"]
        );
        assert_eq!(result.candidates[0].ipa_grammar_accepted, Some(true));
        assert!(!result.determined);
    }
    assert_eq!(provider.generate("hello", "en-US").unwrap(), "həlˈo͡ʊ");
    assert!(provider.generate("hello", "ja").is_err());
    std::env::set_var("ESPEAK_NG", &provider.executable);
    let value = cli::run(&[
        "map".into(),
        "--language".into(),
        "fr".into(),
        "--input".into(),
        "eau".into(),
        "--target".into(),
        "ipa".into(),
    ])
    .unwrap();
    assert_eq!(value["candidates"][0]["text"], "ˈo");
}
