<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# License boundaries

Copyright (c) 2026 **Stya Yur Open Source Studio** for first-party materials.
This repository follows the studio's [License Policy](https://github.com/styayur/styayur/blob/main/LICENSE_POLICY.md).

| Scope | License | Text |
|---|---|---|
| `apps/desktop/**`, `apps/palette/**`: complete applications, CLI, browser/native UI, application tests | **GPL-3.0-or-later** | [GPL](GPL-3.0-or-later.txt) |
| `crates/ipa-core/**`, `crates/g2p-espeak/**`, `crates/tts-core/**`, `crates/tts-kokoro/**`, `crates/alignment-core/**`, `crates/symbol-catalog/**` excluding catalog data: reusable libraries | **MPL-2.0** | [MPL](MPL-2.0.txt) |
| Authored symbol catalog selection, aliases and descriptions | **CC-BY-4.0**; Unicode-derived names/data retain Unicode-3.0 | [Data provenance](../docs/data-sources.md) |
| `crates/language-mapping/` code/schema/adapter policies (excluding data and reference excerpts) | **MPL-2.0** | [MPL](MPL-2.0.txt) |
| Mapping data: CLDR / selected CMUdict / IANA / original registry descriptions | **Unicode-3.0 / BSD-2-Clause / CC0-1.0 / CC-BY-4.0** by scope | [Mapping data notice](../crates/language-mapping/data/README.md) |
| Independent Wiktionary pronunciation reference facts in mapping test fixtures | Attribution per entry; copyrightable excerpts **CC-BY-SA-4.0**, not a production lexicon | [Reference fixture](../crates/language-mapping/tests/fixtures/external.json) |
| `third_party/sherpa-onnx-sys/**` | **Apache-2.0**, upstream preserved with documented manifest patch | [License](../third_party/sherpa-onnx-sys/LICENSE) |
| `scripts/**`, `.github/workflows/**`: small scripts and CI automation | **MIT** | [MIT](MIT.txt) |
| Standalone first-party README/Markdown documentation | **CC-BY-4.0** | [CC BY 4.0](CC-BY-4.0.txt) |
| Third-party software, native runtime, model weights and data | **Their original licenses** | [Third-party inventory](../THIRD_PARTY_NOTICES.md) |

The repository root LICENSE covers the application and the combined executable;
it does not override the separate scope above. Each crate declares MPL-2.0 in
Cargo metadata and includes its own complete license. First-party files carry
SPDX identifiers. GPL-3.0-or-later means version 3 or any later version published
by the Free Software Foundation; the complete GPL v3 terms are included.

MPL-covered source remains available under MPL-2.0. No Exhibit B notice
(Incompatible With Secondary Licenses) is applied. The combined GPL application
uses the larger-work/secondary-license provisions in MPL-2.0 section 3.3.

Document attribution: Stya Yur Open Source Studio, "IPA-first TTS", with a link
to this repository and [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
When changing documentation, indicate that changes were made. License texts
themselves retain the rights and notices of their original authors; copying a
license text does not relicense that text as project documentation.

No third-party font is bundled. Model and runtime files are not committed to the
source repository. The Windows release retains third-party notices and provides
the upstream versioned source locations and included corresponding-source
archives described in THIRD_PARTY_NOTICES.md.
