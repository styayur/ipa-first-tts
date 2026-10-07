# Third-party components

First-party components have the scoped licenses documented in
[licenses/README.md](licenses/README.md): GPL-3.0-or-later application, MPL-2.0
reusable crates, MIT automation and CC-BY-4.0 authored documentation. External
assets retain their original terms. Preserve upstream notices when redistributing
them. Upstream license texts and notices are copied **unchanged** into
`licenses/upstream/`; model-bundle LICENSE is also retained next to the model.

| Component | Upstream | License / notes |
|---|---|---|
| eSpeak-NG 1.52.0 | https://github.com/espeak-ng/espeak-ng | GPL-3.0-or-later; invoked as a separate executable. Check upstream source and distribution obligations. |
| sherpa-onnx 1.13.8 | https://github.com/k2-fsa/sherpa-onnx/tree/v1.13.8 | sherpa's own source is Apache-2.0. The TTS native DLL links GPL eSpeak-NG; the aggregate application is distributed under GPL-3.0-or-later, without relicensing upstream source. |
| Kokoro multi-lang v1.0 bundle | https://github.com/k2-fsa/sherpa-onnx/releases/tag/tts-models | Bundle's LICENSE is Apache-2.0. eSpeak data and other bundled components retain their own terms. |
| ONNX Runtime 1.28.2 | https://github.com/microsoft/onnxruntime/tree/v1.28.2 | MIT plus upstream ThirdPartyNotices; use the DLLs shipped with the selected sherpa native bundle. |
| piper-phonemize fork | https://github.com/csukuangfj/piper-phonemize/tree/f3ff95afc03640bc1399e113e83361192a2fafb4 | MIT for its own source; links GPL eSpeak-NG. Also retains uni-algo notices. |
| eSpeak-NG linked inside sherpa | https://github.com/csukuangfj/espeak-ng/tree/ed530aa113046142eb5115cf2fc9157854d0ffe1 | GPL-3.0-or-later; separate from the v1.52.0 command-line executable. Source includes ucd-tools and generated-data inputs. |
| lessmsi 2.12.9 (setup only) | https://github.com/activescott/lessmsi | MIT. |

Rust dependency licenses and exact versions are available through Cargo.lock and
each crate's upstream manifest. Playwright/Chromium are optional development test
tools, not distributed or required by the application.

Additional native components (Kaldi decoder, kaldifst, OpenFst, kaldi-native-fbank,
KissFFT, simple-sentencepiece, nlohmann/json, hclust-cpp, Asio and websocketpp)
retain the notices in `licenses/upstream/`. Exact pinned versions, source URLs and
SHA-256 values are listed in [licenses/native-sources.json](licenses/native-sources.json).
They are build dependencies of the official sherpa native distribution; inclusion
of their source does not imply that the application enables unrelated ASR features.

## Corresponding source / rebuilding

Every binary Release provides `ipa-first-tts-v0.1.0-corresponding-source.zip` at
[the same Release location](https://github.com/styayur/ipa-first-tts/releases/tag/v0.1.0).
It contains the first-party source used for the executable, Cargo.lock, the
unmodified vendored Rust dependencies, pinned native source snapshots (including
both GPL eSpeak distributions), notices, and native upstream build scripts.
These materials are available without charge. Source links in native-sources.json
provide additional equivalent public download locations for the upstream source.

The native binaries are **unmodified official** sherpa 1.13.8 Windows x64 shared
MT Release files, and the eSpeak CLI is extracted **unmodified** from the official
1.52.0 MSI. Binary hashes are recorded in each package's `MANIFEST.json`.
Native source snapshots preserve original archives and symlinks; when rebuilding
on Windows, use a tar/extraction environment that supports upstream symlinks.

`SOURCE_BUILD.md` inside the source archive describes Rust offline builds using
the included matching native archive and rebuilding native DLLs via the pinned
sherpa CMake scripts (`SHERPA_ONNX_ENABLE_TTS=ON`, static CRT, shared libraries,
Release, PortAudio off). Native reconfiguration may fetch pinned non-system
build prerequisites and test dependencies according to upstream scripts; this
does not make application synthesis dependent on network access.

Kokoro weights and model data are third-party Apache/GPL materials as supplied
in their original bundle. No claims of ownership, model training or relicensing
are made. The included model is copied unchanged; only a runtime temporary copy's
voice metadata is adjusted, as described in models/README.md.
