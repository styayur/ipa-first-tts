<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Mapping engine: repository audit

2026-10-25. Baseline main/origin/main and v0.2.0-alpha.1 resolve to
`74a2c8aa795b82d57203acfef4bc2722dc3661f2`. The worktree was clean; ignored
models/tools/dist/.cache are existing preparation materials and are preserved.
The remote is styayur/ipa-first-tts. Existing v0.1.0 and v0.2.0-alpha.1 releases
and assets must remain unchanged. v0.3.0-alpha.1 was unused at audit time.
The independent branch is feat/multilingual-language-mapping.

The conservative MPL ipa-core parses NFD IPA with explicit stress, boundaries,
ties and validation. It is neither a universal IPA inventory nor a pronunciation
dictionary. The existing eSpeak G2P entry and both speech adapters reject
non-English input. Kokoro loads lazily; Studio uses a GPL plain browser UI and
loopback tiny_http with Host/Origin, body-size and CSP checks.

The MPL symbol-catalog has 368 selected records: language associations support
search and filtering only. Its curated CC BY data and Unicode-derived material
have separate notices. The GPL palette uses Win32, windows-sys 0.61.2, atomic
settings, guarded Unicode SendInput and a local mutex. It does not load models.

The workspace currently has 120 locked external packages. Keep the patched
Apache sherpa-onnx-sys build wrapper and native ABI 1.13.8, and retain existing
GPL/MPL/data boundaries. New mapping code can use existing serde, serde_json
and unicode-normalization; no Python/ICU/cloud runtime is needed.

CI covers Windows/Linux model-independent tests, native Release builds, Clippy,
license/audit checks and real catalog browser tests. A separate workflow checks
architecture anchors and reproducible diagrams. Existing tests include 29
ordinary tests and two explicit opt-in real eSpeak/Kokoro tests. Prior native
evidence distinguishes agent editor tests, user browser/tray tests and hidden
control/API tests; it must not be relabeled as this version's new GUI testing.

Packaging already produces model-free palette/source and full offline/source
ZIPs with manifests, exact source revisions and SHA-256. Source bundles vendor
Cargo and the full bundle retains native upstream sources. The upstream sherpa
build script locates custom-target native caches via OUT_DIR unless
CARGO_TARGET_DIR is set; final packaging must preflight this path and verify
the archive against the shipped DLLs before creating any stage.
