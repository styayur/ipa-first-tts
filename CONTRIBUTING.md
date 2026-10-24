<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Contributing

Keep IPA Core independent of the UI and speech models. Application synthesis must consume the edited IPA; do not silently regenerate it from text, drop unsupported phones or turn estimated timing into a measured-alignment claim.

Keep symbol-catalog independent of IPA grammar, GUI and TTS. Preserve catalog
strings byte-for-byte in Copy/Insert; normalize only the search index. New data
needs stable IDs, exact scalar lists, language/source metadata and license
attribution. See [provenance](docs/data-sources.md) and [ADR](docs/decisions/0001-native-symbol-palette.md).
Do not commit personal palette settings, test configurations or generated models.

Use [Issues](https://github.com/styayur/ipa-first-tts/issues) for redacted reproductions with version, platform, backend and minimal input. Follow [SECURITY.md](SECURITY.md) for vulnerabilities. Read the [English guide](README.md), [technical reference](README.zh-CN.md) and [model setup](models/README.md) before changing a boundary.

From the root, run applicable existing checks:

```powershell
cargo fmt --all -- --check
cargo test -p ipa-core -p tts-core -p alignment-core -p g2p-espeak -p tts-kokoro --no-default-features --locked
cargo clippy -p ipa-desktop --no-default-features --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo audit
python scripts/check_dependencies.py
```

For changes to actual synthesis, explicitly run the model-dependent pipeline tests after preparing the documented assets:

```powershell
cargo test -p ipa-desktop --test pipeline -- --ignored --test-threads=1
```

State which native/model/device checks ran and which did not. UI tests and architecture evidence checks are described in the README; no screenshot or unit-test count replaces real inference evidence. Keep a PR focused and do not submit downloaded models, native tools, generated audio or user configuration. Maintainers publish releases; leave historical assets and tags intact.

Run catalog browser checks with `python scripts/run_studio_test.py --exe
target/debug/ipa-tts.exe --test scripts/test_catalog_ui.py --without-assets`.
The runner owns a direct child and a fresh port so Windows shell grandchildren
cannot leave stale test servers. Native focus/tray/Unicode changes additionally
require the [actual app matrix](docs/validation/native-checklist.md). Unit tests
must not be relabeled as desktop passes. Final packaging is gated on evidence
for its exact commit; local `--candidate` packages remain explicitly unverified.
After reviewing dependency changes, regenerate notices with
`python scripts/check_dependencies.py --write`; never override upstream terms.

Preserve file-level SPDX headers and [component license boundaries](licenses/README.md). The GPL application, MPL crates, MIT automation and CC BY documentation have separate scopes; third-party terms do not change with a contribution.
