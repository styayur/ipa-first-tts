<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Mapping workbench design

The task is a local pronunciation/transliteration workbench. Retain Studio's
existing palette: paper #f3f7fc, ink #243447, muted #5e7082, blue #275ca8,
line #ced9e7 and white #ffffff. Use the installed Segoe UI for controls and the
existing Cambria/system phonetic stack for large candidates; no downloaded font.

Keep inputs left aligned: language → typed conversion → named scheme → script,
then original text and optional context. Place selected candidates immediately
below Convert; provenance expands beside them in the same section. A candidate
is a choice, not a decorated success metric. Only compatible available edges
appear. No confidence percentage or generalized "supported language" badge.

The native secondary window follows the same order, retains standard keyboard
semantics and runs backend work off the message thread. Esc hides, Ctrl+Enter
converts. Copy is explicit. Sending to the palette fills its composition without
injecting input. The existing English Studio receives only selected English IPA;
other results remain copyable/insertable through the palette.

Review against the brief: a separate dashboard, hero animation and new visual
brand would obscure the existing tool. Reuse its controls and make the candidate
string the visual focus. Test narrow layout, native minimum size, stale-result
invalidation, keyboard focus, ambiguous choices and missing optional tools.
