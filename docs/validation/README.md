<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# v0.2.0-alpha.1 verification status

2026-10-25, Windows x64. This is a release candidate; publication is tracked separately.
The [native matrix](native-results.json) and [reproducible checklist](native-checklist.md)
separate real native interaction from unit/headless-browser tests.

Recorded local passes so far:

- `cargo fmt --all -- --check`
- `cargo test --workspace --locked`: 28 unit/integration boundary tests, including hidden test-owned native controls; the two existing model-dependent tests remain explicit opt-in.
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

Native desktop evidence now includes real Microsoft Notepad 11.2607.14.0,
Notepad3 and an isolated VS Code profile with extensions disabled. The native
palette sent `ɲʁœçˈt͡ʃéã𝼀`; Notepad Copy and the saved VS Code file matched
exactly (13 UTF-16 units / 24 UTF-8 bytes). Notepad3 also accepted a tied affricate
and repeated compositions at the preserved caret. The user confirmed the global
shortcut; the agent recalled it from all three editors and observed search focus
and the correct target caption. The Studio action started a sibling child and
opened the local browser page with explicit transfer feedback.

The earlier locked-desktop problem was resolved. Browser native automation was
then stopped by the helper's inability to determine the current URL confidently,
including for the local Studio page. It was not bypassed. Actual browser input,
tray/menu and remaining visual checks are recorded individually in the matrix;
they are not inferred from the editor tests. Manual user verification was
requested while CI and packaging continue.

The new native-control regression creates only hidden test-owned windows,
checks bounds and at least three grid rows at three client sizes, filters/no
results/reset, composition, favorite/recent persistence and Chinese labels.
It sends no global input, changes no foreground, writes no clipboard and does
not control the user's running app. This supplements external-app tests and
does not certify mixed-monitor DPI or physical tray clicks.

The packaging scripts refuse final packages without the required native matrix
for the final commit. `package_palette.py --candidate` can produce explicitly
named local candidate ZIPs with clean-directory/model-free/sanitized-PATH
self-checks. This is not a clean-machine, physically offline network or native
interaction certification. Published-asset download/SHA verification remains
unexecuted until a release exists. No signing certificate is available; checksum
verification will not be described as code signing.
