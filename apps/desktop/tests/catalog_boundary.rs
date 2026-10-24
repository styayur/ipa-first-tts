// SPDX-License-Identifier: GPL-3.0-or-later
use ipa_core::IpaSequence;
use symbol_catalog::Catalog;
use tts_core::PhonemeAdapter;
#[test]
fn catalog_coverage_is_not_parser_or_backend_support() {
    let catalog = Catalog::bundled().unwrap();
    for text in ["¿", "\u{1df00}", "˥"] {
        assert_eq!(catalog.search(text, None, None)[0].text, text);
        assert!(IpaSequence::parse(text, None).is_err());
    }
    let phone = catalog.search("ɬ", None, None)[0];
    let ipa = IpaSequence::parse(&phone.text, Some("en-US")).unwrap();
    let adapter = tts_kokoro::KokoroAdapter::from_token_text("  1\na 2\n").unwrap();
    assert!(adapter.adapt(&ipa).unwrap_err().contains("no token"));
    assert_eq!(
        catalog.search("t͡ʃ", None, None)[0].codepoints,
        ["U+0074", "U+0361", "U+0283"]
    );
}
