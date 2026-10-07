<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# License boundaries

Copyright (c) 2026 **Stya Yur Open Source Studio** for first-party materials.
This repository follows the studio's [License Policy](https://github.com/styayur/styayur/blob/main/LICENSE_POLICY.md).

| Scope | License | Text |
|---|---|---|
| `apps/desktop/**`: complete application, CLI, browser UI, application tests | **GPL-3.0-or-later** | [GPL](GPL-3.0-or-later.txt) |
| `crates/ipa-core/**`, `crates/g2p-espeak/**`, `crates/tts-core/**`, `crates/tts-kokoro/**`, `crates/alignment-core/**`: reusable libraries, including reusable adapters | **MPL-2.0** | [MPL](MPL-2.0.txt) |
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
