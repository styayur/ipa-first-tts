<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Reproducible Windows native validation

Run a Release build from the exact recorded commit in an extracted clean
directory. Record Windows build, monitor/DPI, app versions, executable hashes,
screenshots/observed text and whether each item was manual or automated. Native
results belong in `native-results.json`; never fill `passed` from this checklist.
Unit tests and headless browser tests are separate evidence.

1. Start the palette with no model directories and a fresh `--config` path. Check that no TTS process or external listener starts. Search immediately; verify ɲ, ʁ, œ, French/fr, Deutsch/de, 法语, U+0252, tsh and no-results. Inspect labels, combining dotted-circle display and exact codepoints.
2. In a new unsaved **Notepad** document, place the caret. Invoke Ctrl+Alt+Shift+I; confirm search focus. Select and Insert `ɲʁœçˈt͡ʃ`, precomposed `é`, decomposed `ã`, and U+1DF00. Verify actual codepoints, not only appearance. Repeat insertion and confirm caret restoration. No text may arrive in the palette's search box or another window.
3. Repeat in a mainstream **browser** local textarea and a new unsaved **VS Code** document if installed. Record acceptance/rejection per string and application. If unavailable, record why; do not substitute a mocked edit control.
4. From search use all arrows, Enter, Esc and Tab; test copy versus insert default action. Close/reopen repeatedly by shortcut and window close. Verify close hides and keeps the hotkey, whereas Exit unregisters it.
5. Favorite a symbol, insert/copy several, quit and restart; verify favorites and recent order. Switch English/简体中文 and verify identical output. Compose base + mark; copy and inspect the exact sequence including tied candidates and surrogate pairs.
6. Copy explicitly; verify all intended codepoints. For Insert, set clipboard content first and verify it is unchanged. No automatic paste fallback exists. Close the target while the palette is open; Insert must fail and offer Copy. Move focus elsewhere before inserting; verify the guarded focus behavior.
7. Change the shortcut twice. Independently reserve the new combination and check conflict feedback while the old one remains registered. Exit and re-register the freed shortcut. Start two instances and verify one tray owner. Restart after abnormal termination using only the test process; confirm OS releases hotkeys/mutex. Do not terminate unrelated apps.
8. Use every tray menu action; open Studio only with a full package and verify transfer, explicit unsupported-symbol error and stopping only the owned Studio child on Exit. Explorer/taskbar restart should restore the icon if safely available to test.
9. Resize to the minimum and a larger window. Check main UI and Settings at 100%, 150%, 200% DPI when available. Move between available monitors with differing DPI and verify work-area limits. Inspect missing-font warnings; do not count a missing-glyph box as a passing glyph.
10. Record higher-integrity/UIPI, remote desktop and exclusive-full-screen scenarios as passed/failed/unverified independently. Do not elevate the app or change security settings to force a pass.

Final asset verification additionally requires downloading the published ZIP,
rechecking SHA-256, checking every manifest entry and repeating core launch,
search, Copy and Insert from a fresh extraction. A sanitized PATH test does not
prove execution on a separate machine without installed development tools, and
no observed network traffic does not prove a physically disconnected network.
