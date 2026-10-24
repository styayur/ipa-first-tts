<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# v0.2.0-alpha.1 verification status

2026-10-24, Windows x64. This is a source candidate, not a published release.
The [native matrix](native-results.json) and [reproducible checklist](native-checklist.md)
separate real native interaction from unit/headless-browser tests.

Recorded local passes so far:

- `cargo fmt --all -- --check`
- `cargo test --workspace --locked`: 26 unit/integration boundary tests; the two existing model-dependent tests remain explicit opt-in.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`
- `cargo build --workspace --locked --release`
- `cargo test -p ipa-desktop --test pipeline --locked -- --ignored --test-threads=1`: both real eSpeak and Kokoro tests passed, including changed IPA changing generated WAV and unsupported token failure.
- `python scripts/run_studio_test.py --exe target/debug/ipa-tts.exe --test scripts/test_catalog_ui.py`: real Rust catalog/HTTP server, 368 records, aliases/NFD search, exact BMP/supplementary clipboard text, complete t͡ʃ insertion, arrows/Enter/Esc, no-results/filters, 390px layout, fragment transfer and Host/Origin/query boundaries.
- `python scripts/run_studio_test.py --exe target/debug/ipa-tts.exe --test scripts/test_ui.py`: real Kokoro generation/playback/Stop/highlighting/errors and narrow layout passed.
- `cargo audit`: zero vulnerabilities and no warnings after fixing nine baseline build-tool advisories. No ignores are used.
- `python scripts/check_dependencies.py`: 120 locked package license expressions reviewed; upstream notice bytes and inventory checked.
- Architecture source anchors and light/dark/repeated Mermaid rendering passed locally; the README diagram is updated with the new boundaries and will be checked again in CI.

Generated detailed browser results/screenshots remain under ignored `outputs/`;
the maintained native matrix records actual unavailable checks. The initial
Release palette executable was 1,036,288 bytes; later changes require rebuilding
and recording the final asset hash/size. No memory/startup latency measurements
have been made.

Native desktop tests are **blocked**: the Computer Use helper returned
`GetCursorPos: access denied (0x80070005)` twice, including a retry after browser
activation. No real Notepad/browser/VS Code input, tray operation, mixed-DPI
visual check or native clipboard pass is claimed. The user was asked to make
the Windows session interactive. Remaining required native checks block merge
and release publication even if build/unit/CI checks pass.

The packaging scripts refuse final packages without the required native matrix
for the final commit. `package_palette.py --candidate` can produce explicitly
named local candidate ZIPs with clean-directory/model-free/sanitized-PATH
self-checks. This is not a clean-machine, physically offline network or native
interaction certification. Published-asset download/SHA verification remains
unexecuted until a release exists. No signing certificate is available; checksum
verification will not be described as code signing.
