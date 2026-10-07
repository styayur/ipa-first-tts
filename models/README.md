<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Local Kokoro assets

Default directory: `models/kokoro-multi-lang-v1_0/`.

Required files:

```text
model.onnx
voices.bin
tokens.txt
espeak-ng-data/
  phondata
  phonindex
  phontab
  ... other original data files
```

Use the **sherpa-onnx** Kokoro v1.0 bundle, not a generic Kokoro ONNX file and not
the v0.19 English bundle. Model metadata must include `model_type=kokoro`,
`version>=2`, `voice`, and `n_speakers`. Keep model, tokens, voices and eSpeak data
from the same release together. v1.1/int8 bundles may be used by naming their
ONNX file `model.onnx`, but only v1.0 is verified in this MVP.

Official download:
https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/kokoro-multi-lang-v1_0.tar.bz2

`scripts/setup.ps1` downloads and extracts this bundle on Windows.
For Linux/macOS, download this archive with curl and extract with
`tar -xjf kokoro-multi-lang-v1_0.tar.bz2 -C models`.

Override the directory with `KOKORO_MODEL_DIR`. Speaker 0 is `af_alloy`;
speaker 3 is `af_heart`; IDs must match the bundle. The UI exposes numeric IDs
to avoid shipping a pronunciation or voice database.

## Why a private model copy?

sherpa-onnx 1.13.8 falls back to `metadata.voice` when `kokoro.lang` is empty.
A nonempty language bypasses the lexicon and invokes eSpeak on the synthetic
word aliases. The backend makes an exclusively owned temporary copy with ONLY
the `voice` metadata string cleared. Graph and weight bytes are copied unchanged.
An utterance-specific lexicon then maps alphabetic aliases to edited IPA tokens.
The original model is never modified. Copies and lexicons are removed on normal
shutdown. A crash can leave a directory named `ipa-first-<pid>-<counter>` in the
OS temp directory; it can be removed after the process exits.

Asset download is setup only. Generation and playback perform no network calls.
The model archive is roughly 334 MiB; allow about 1 GiB for download, extracted
assets and a temporary model copy. Third-party model/runtime licenses apply;
retain the bundle's license files if redistributing it.
