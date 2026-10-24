<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Symbol Palette interaction and trust boundaries

The palette is a standalone Windows x64 Rust executable, using OS Win32 controls,
GDI text, RegisterHotKey, Shell_NotifyIcon and SendInput. It does not link any
TTS crate. `symbol-catalog` validates embedded JSON once and maintains a separate
NFD/lowercase search index (with ß → ss). This is not universal Unicode casefold.
Exact original strings rank first, then canonically equivalent same-case strings,
then case-insensitive matches. All query words must match. Language and category
filters intersect. Empty results disable selection-dependent actions.

The search box receives focus when opened. Arrows navigate a native multi-column
list, Enter dispatches the selected action, Esc hides and restores a valid
original target, Tab follows native controls. Mouse selects; explicit buttons
choose Copy, Insert, Favorite, Compose and Studio. A nonempty composition takes
precedence over the current candidate. Clear returns to candidate output.
Recent entries are deduplicated, newest first, capped at 30. Favorites are capped
at 1000. Both store stable catalog IDs, not a second set of symbol definitions.
Unknown IDs left by a newer/different catalog are retained but not displayed.

Settings and the action selector persist in version 2 config under
`%LOCALAPPDATA%/IPA-first-TTS/palette.json`. Version 1 migrates by adding the
default Insert action. A 128 KiB limit, schema validation, exclusive temporary
files, sync_all and same-directory MoveFileEx(REPLACE_EXISTING | WRITE_THROUGH)
protect writes. Invalid supported schemas are quarantined as `palette.corrupt-*`;
an unknown future version stops loading/writing and preserves its file. Atomic
replacement protects against partial writes; it does not claim recovery from all
hardware/filesystem failures. No personal config is bundled or committed.

## Input and focus

The hotkey captures the original HWND, PID, thread ID and a per-window property.
The property disappears with the window, preventing acceptance of a recycled
handle on the same process/thread. Windows may refuse the property on a target
with different privileges; Insert then fails closed. The palette removes its
property when releasing a live target. Abrupt process termination can leave the
inert property until the target window closes.

Insert revalidates identity, checks process integrity and held modifier keys,
hides the palette, restores the target with SetForegroundWindow, and rechecks
identity and foreground immediately before one SendInput batch. It generates
key-down/key-up events for **every UTF-16 code unit**, preserving surrogate pairs
and combining sequences. It does not synthesize a Ctrl+V paste. There is no
AttachThreadInput, elevation or UAC workaround.

Windows input is global: foreground checks reduce wrong-target risk but cannot
make activation and global event consumption atomic against another app changing
focus concurrently. SendInput's returned event count proves dispatch, not that
the target accepted the text. Unsupported target apps, remote desktop, full-screen
apps and integrity restrictions need application-specific verification. Partial
dispatch errors explicitly warn that some input may already have reached the
target; check before retrying. The status never claims text acceptance.

Copy allocates a complete CF_UNICODETEXT string and transfers ownership to
Windows only after a successful clipboard call. **Copy replaces clipboard
contents and formats.** Insert leaves all clipboard formats alone. There is no
automatic fallback that pretends to preserve non-text clipboard data. If focus
or injection fails, the palette reappears with the error and Copy remains an
explicit user choice. Opening from the tray clears the insertion target; use
the hotkey from the intended editor for Insert.

## Window and process lifecycle

Default shortcut Ctrl+Alt+Shift+I avoids common IME combinations. Settings accept
Ctrl/Alt/Shift plus A–Z, digits or F2–F24 except Windows-reserved F12. Register the
new combination under an alternate ID before releasing the old one. Conflict or
save failure leaves the original active; an initial conflict leaves tray/settings
available. Registration errors are visible. A per-user/session mutex prevents
multiple instances from competing. The second launch reveals the first. `--quit`
requests normal exit; `--background` starts with only the tray.

The main window handles per-monitor DPI and constrains position to the current
monitor's work area near the pointer. Desktop/monitor query failure retains the
existing dimensions instead of creating a zero-sized window. Font coverage is
checked against the selected system font. Missing/default-font and supplementary
coverage warnings direct the user to exact codepoints; font fallback and target
font coverage still need visual verification. No font or IPA chart is bundled.

Closing hides; tray Exit drops hotkey registrations, removes the icon, destroys
owned windows/fonts and stops/reaps a Studio child this palette launched. Explorer
restart is handled with TaskbarCreated. Existing unrelated Studio processes are
never stopped. Studio is started only by an explicit action, on a loopback port,
and is checked for startup failure. The selected string transfers in a percent-
encoded URL fragment, which Studio removes from its current history entry after
reading. It is not sent to the HTTP server. The OS/browser may retain process or
navigation records; this is not a secret transport. No telemetry or edited-text
logging is added. Studio retains its Host/Origin and input-size checks.

## Platform and language boundaries

The native palette targets Windows x64 only. System-control semantics, focus,
tray, mixed-DPI layouts and app compatibility must be checked with the native
[matrix](validation/native-checklist.md), separately from browser screenshots.
Windows API unit tests do not certify a real editor, tray or interactive desktop.
No autostart or installer is implemented; remove extracted files to uninstall.
The user may separately remove local settings if desired.

Catalog language association does not extend conservative IPA Core grammar or
the English speech adapters. Tones/orthographic characters may copy/insert while
being rejected by Studio. There is no silent deletion, implicit backend fallback
or model initialization when opening/searching the palette.
