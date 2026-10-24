<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# ADR 0001 — Independent Win32 palette, shared offline catalog

Date: 2026-10-24. Status: accepted for implementation.

## Audit before implementation

Local main was clean at fa8a3ab1e2010122619c791c6ed15fb946ace965;
after fetch, origin/main was e200900ff27c98807c7c1fad3e27c266282241da.
Branch feat/global-symbol-palette starts at origin/main. No tracked edits or
untracked files existed. Ignored models/, tools/, .cache/ and dist/ contain
previous preparation/release materials and must be preserved.
The existing v0.1.0 annotated tag resolves to fa8a3ab; its four release assets
remain unchanged. v0.2.0-alpha.1 is unused at audit time.
Repository API reports push/admin access, no main branch protection or rulesets.
Latest main CI run 39760073153 succeeded; architecture run 39760073093 failed.
The workflows are required quality checks for this work even without protection.

ipa-core is an independent MPL library with conservative NFD parsing. G2P and
explicit Kokoro/eSpeak adapters already reject unsupported input. Studio is a
GPL plain HTML/JS application behind tiny_http, bound to 127.0.0.1 with Host and
Origin validation. Engine loads Kokoro lazily. Keep these contracts.

## Options and decision

| Option | Dependencies/runtime | Reuse and interaction | Cost/risk |
|---|---|---|---|
| Win32 controls + existing Studio | windows-sys bindings; OS user32/gdi32/shell32 only | Keep Studio/HTTP; native hotkey, tray, Unicode input and DPI | More explicit FFI/resource ownership, small standalone executable |
| Tauri + existing HTML/JS | Tauri/wry/tao, WebView2 runtime and build integration | Reuses HTML; native plugins still needed for input/focus | Larger dependency graph; offline WebView availability/distribution |
| egui or other native toolkit | Toolkit, renderer and graphics stack | Rebuild palette UI; Studio still separate | Rendering/runtime weight; accessibility and native integration work remains |

Use Win32 controls in a separate GPL apps/palette binary, with a narrow audited
unsafe Windows module. Reusable catalog is an MPL Rust crate depending only on
serde, serde_json and unicode-normalization (already present). No GUI/TTS
dependency enters ipa-core or symbol-catalog. Actual sizes and memory belong in
verification results, not estimates presented as measurements.

Versioned JSON files embed in Rust and validate at startup. Catalog output is
never normalized; NFD/case-insensitive search uses a separate string. Studio
serves the same catalog and can insert complete records into its editor.
The native palette sends to an explicitly launched sibling ipa-tts Studio via
a URL fragment; the fragment is not sent to the server or written as a file.
Studio availability is optional; light packages do not initialize or ship TTS.

## Windows safety and lifecycle

RegisterHotKey default Ctrl+Alt+Shift+I; configurable with conflict feedback.
A per-user local mutex prevents competing instances. Hiding retains tray/hotkey;
exit unregisters and deletes resources. Re-add tray after Explorer restart.
Record HWND, PID and thread ID before showing; validate all before restoring.
Check foreground again immediately before SendInput. Use UTF-16 key-down/up
packets, including surrogate pairs. Reject held modifier keys. No AttachThreadInput,
elevation, UAC bypass, or synthesized clipboard paste. Copy explicitly replaces
the clipboard; Insert never touches it. SendInput cannot guarantee application
acceptance; report dispatch separately from application-specific test evidence.

Microsoft references consulted 2026-10-24:
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-keybdinput
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-registerhotkey
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow

Config resides in LOCALAPPDATA/IPA-first-TTS/palette.json. Version validation,
size limits, exclusive temporary-file creation, flush and atomic replacement
prevent partially written settings. Quarantine corrupt supported-version files;
do not overwrite an unknown future schema. No autostart, telemetry or network.

## Verification scope

Automate catalog/schema/Unicode/search, persistence/migration, native hotkey and
UTF-16 dispatch tests. Separately test actual Notepad/browser/editor interaction,
tray and focus with visible UI. Report unavailable apps, DPI/monitor scenarios
and UIPI cases honestly. Offline build/start, real backend tests and published
asset download hashes are separate release gates.
