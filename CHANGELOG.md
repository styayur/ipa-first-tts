<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Changes

## 0.3.0-alpha.1 — experimental language mapping

- Independent typed local mapping engine: versioned language/script/scheme/source
  registry, explicit conversion edges, candidate provenance, uncertainty and limits.
- Pinned CLDR 49 kana/Russian alphabet subsets and Han default readings with three
  phrase contexts; selected CMUdict / explicit ARPAbet adapter and finite exact P2G.
- Optional separately selected eSpeak 1.52.0 G2P for en-US, French and Spanish;
  no new synthesis languages. Fixed Windows stdin EOF handling by appending LF.
- Shared JSON CLI (`ipa-map` and `ipa-tts`), Studio workbench and native mapping
  window with Copy, composition transfer and eligible English Studio transfer.
- Source/data byte manifests, independent per-language fixtures and evaluation
  policies; light/full packaging retains all license and capability boundaries.
- Restore minimized shortcut activation and single-instance foregrounding. Route
  sent tray callbacks through the window procedure so left-click reveals the
  palette and right-click opens its menu; also forward Explorer's tray recreation
  broadcast. Keep tray activation in explicit Copy mode.

## 0.2.0-alpha.1 — experimental preview

- Independent Rust/Win32 Symbol Palette with shared validated offline catalog
  (368 selected IPA and orthographic records), search, categories, composition,
  favorites/recent, English/Chinese controls, configurable global shortcut,
  tray lifecycle and guarded Unicode insertion. Real editor insertion and user-reported browser/tray checks passed;
  evidence and limitations are in docs/validation/native-results.json.
- Studio uses the same catalog/search and accepts complete Unicode strings;
  IPA parser and explicit English TTS adapters retain their independent limits.
- Versioned atomic user settings with migration, corruption quarantine and
  future-version protection; standalone model-free packaging with release gates.
- Updated Unicode provenance, dependency/license inventories and automated
  Unicode/catalog/config/HTTP/UI/native API tests.
- Security fixes for nine inherited build-dependency advisories; pinned sherpa
  native ABI preserved through a documented Apache-2.0 build-wrapper patch.

## 0.1.0 — published

English IPA-first Studio, eSpeak-NG G2P/fallback, Kokoro/sherpa-onnx synthesis,
explicit phoneme adapters, estimated alignment and portable Windows offline
package with corresponding source and application-local VC runtime.
