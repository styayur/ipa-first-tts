# Architecture evidence

Source review: v0.2.0-alpha.1 source candidate, based on `e200900ff27c98807c7c1fad3e27c266282241da` (2026-10-24). Native execution status is tracked separately in [validation](../validation/README.md).

The marked Mermaid block in [README](../../README.md) is the only maintained diagram source. GitHub renders it natively in the reader's theme. No duplicate SVG or independent `.mmd` is committed; extracted Mermaid and SVG files are disposable verification artifacts.

The UI runs against a Rust loopback HTTP server, not Tauri. The CLI uses the same Engine directly. G2P invokes eSpeak-NG as a child process; synthesis consumes the edited IpaSequence, not the original text. Kokoro uses a vocabulary adapter, temporary phoneme lexicon and native sherpa-onnx session. eSpeak is an explicitly selected fallback, not an automatic recovery path.

The separate apps/palette Win32 executable depends only on symbol-catalog,
serde/serde_json and Windows system bindings. It owns hotkeys, tray, guarded
UTF-16 input and atomic local settings. It starts no TTS/server at launch.
Studio and palette share one validated embedded JSON catalog and Rust query
algorithm. Explicit Studio transfer starts a sibling child and opens a loopback
URL fragment; no symbol text is sent in an HTTP query. The catalog has no
dependency on ipa-core, GUI or speech. See [ADR](../decisions/0001-native-symbol-palette.md).

Alignment follows synthesis: UniformAligner divides audio duration uniformly across phones and reports TimingQuality::Estimated. No forced aligner or measured phoneme timestamps are implemented. The server retains at most four WAV results in memory; CLI synthesis writes the selected output file. Host/Origin checks protect the loopback boundary; this is not a public authenticated service.

## Source map

- [apps/desktop/src/server.rs](../../apps/desktop/src/server.rs): `expected_host`, `UniformAligner.align`, `audio_cache.len() > 4`
- [apps/desktop/src/lib.rs](../../apps/desktop/src/lib.rs): `pub fn synthesize`, `"espeak" =>`, `"kokoro" =>`
- [crates/g2p-espeak/src/lib.rs](../../crates/g2p-espeak/src/lib.rs): `Command::new`, `fn generate`
- [crates/ipa-core/src/lib.rs](../../crates/ipa-core/src/lib.rs): `IpaSequence`
- [crates/tts-kokoro/src/lib.rs](../../crates/tts-kokoro/src/lib.rs): `KokoroAdapter`
- [crates/tts-kokoro/src/sherpa.rs](../../crates/tts-kokoro/src/sherpa.rs): `lexicon`
- [crates/alignment-core/src/lib.rs](../../crates/alignment-core/src/lib.rs): `TimingQuality::Estimated`
- [apps/desktop/tests/pipeline.rs](../../apps/desktop/tests/pipeline.rs): `#[test]`

The anchors in `evidence.json` catch renamed/deleted source symbols; they do not prove call semantics. The source review above checked the actual call sites and boundaries. A significant change to data flow, persistence, authentication, recovery or process boundaries requires reviewing this diagram and updating the evidence. Routine edits do not require redrawing it.

## Verification

Requires Python 3, Node.js 22+ and network access for the documentation-only Mermaid CLI. From the repository root:

```sh
python docs/architecture/verify.py --render
```

This checks local README image references and source anchors, extracts the authoritative block, renders it twice with Mermaid CLI 11.12.0 using deterministic IDs, compares SVG bytes, validates SVG XML, and also renders the dark theme. If the bundled browser is unavailable, pass `--chrome /absolute/path/to/chrome` (or set `PUPPETEER_EXECUTABLE_PATH`). The CLI version is pinned; its transitive npm dependencies and the browser are environment-dependent, so the byte comparison proves repeatability within the same installed toolchain. Output goes to a temporary directory, never application runtime dependencies. GitHub Markdown/browser rendering still requires visual review; CLI validation alone is not evidence of GitHub rendering.

GitDiagram returned an initial diagram on 2026-10-09 for the public repository as a discovery aid. Its generated output is not imported as authoritative architecture or licensed artwork. No private source, config or credentials were submitted.

Existing repository licenses and third-party notices continue to apply. These diagrams are documentation authored from this repository's public source; no app icons, installer assets or third-party marks are replaced.
