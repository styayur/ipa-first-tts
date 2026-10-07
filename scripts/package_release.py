# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Stya Yur Open Source Studio
"""Build Windows offline ZIP + corresponding source ZIP; verify the staged app.

Prerequisites: scripts/setup.ps1, prepare_native_sources.py, cargo vendor --locked
.cache/vendor. Output is exclusively placed in this project's ignored dist/.
"""
from pathlib import Path
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tomllib
import wave
import zipfile

ROOT = Path(__file__).resolve().parents[1]
VERSION = tomllib.loads((ROOT / 'Cargo.toml').read_text(encoding='utf-8'))['workspace']['package']['version']
DIST = ROOT / 'dist'
SOURCE_BUILD = r'''# Corresponding source and build instructions

First-party source revision: {revision}
Release: v{version}
First-party license boundaries are in licenses/README.md. Vendor and native
sources are unchanged upstream materials with their original notices.

## Build the application from vendored source (Windows x64)

Install Rust stable MSVC, Visual Studio C++ Build Tools and Windows SDK. The
original release was built with rustc 1.98.1 and cargo 1.98.1. Extract this ZIP,
open its top-level directory, and run:

```powershell
$env:SHERPA_ONNX_ARCHIVE_DIR = (Resolve-Path native-binaries).Path
cargo build --workspace --release --offline --locked
```

The included .cargo/config.toml points at vendor/. The matching official native
archive is included in native-binaries/ so sherpa's build script can unpack it
without downloading anything. This rebuilds first-party source and the Rust
dependencies while using the unmodified official native DLLs. Models/eSpeak CLI
are in the separate offline binary package, not this source archive.

## Build native runtime from source

native-sources/ contains pinned source archives, with original upstream build
files and symlinks intact. Exact URLs and hashes are in licenses/native-sources.json.
Use an extraction environment supporting upstream symlinks. The sherpa Windows
build recipe is in sherpa-onnx-v1.13.8/.github/workflows/windows-x64.yaml.
From the extracted sherpa source directory, in a Visual Studio developer shell:

```powershell
cmake -S . -B build -A x64 -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=ON `
  -DSHERPA_ONNX_ENABLE_TTS=ON -DSHERPA_ONNX_USE_STATIC_CRT=ON `
  -DSHERPA_ONNX_ENABLE_PORTAUDIO=OFF -DBUILD_ESPEAK_NG_EXE=OFF `
  -DCMAKE_INSTALL_PREFIX=build/install
cmake --build build --config Release --parallel 2
cmake --install build --config Release
```

Use the included pinned archives in the CMake scripts' documented local search
paths or explicit FetchContent/ExternalProject overrides. Native reconfiguration
may otherwise download prerequisites specified by upstream (compiler/system SDKs
are not included). espeak-ng ed530aa is linked inside sherpa; the independent CLI
comes from espeak-ng 1.52.0, whose Windows build instructions are in its source.
ONNX Runtime 1.28.2 source and notices are included; its own build dependencies
and build recipe are described in onnxruntime's upstream repository. The native
1.28.2 DLL originally comes from csukuangfj/onnxruntime-libs, as pinned in sherpa's
cmake/onnxruntime-win-x64.cmake. Building native source does not claim bit-for-bit
reproducibility across different toolchains.
'''


def hash_file(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def copy(source, destination):
    destination.parent.mkdir(parents=True, exist_ok=True)
    if source.is_dir(): shutil.copytree(source, destination)
    else: shutil.copy2(source, destination)


def zip_tree(path, destination):
    with zipfile.ZipFile(destination, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
        for file in sorted(path.rglob('*')):
            if file.is_file(): archive.write(file, str(file.relative_to(path.parent)).replace('\\','/'))


def run_stage(stage, args):
    environment = os.environ.copy()
    environment['ESPEAK_NG'] = str(stage / 'tools/espeak-ng/espeak-ng.exe')
    environment['KOKORO_MODEL_DIR'] = str(stage / 'models/kokoro-multi-lang-v1_0')
    output = subprocess.run([str(stage/'ipa-tts.exe'), *args], cwd=stage, env=environment,
                            capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=120)
    if output.returncode != 0: raise RuntimeError(output.stdout + output.stderr)
    return output.stdout.strip()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--skip-build', action='store_true')
    options = parser.parse_args()
    if os.name != 'nt': raise SystemExit('This package targets Windows x64')
    if not options.skip_build:
        subprocess.run(['cargo','build','--release','--workspace','--locked'],cwd=ROOT,check=True)
    revision = subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    tracked = subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode('utf-8').split('\0')
    tracked = [p for p in tracked if p]
    dirty = subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True)
    if dirty.strip(): raise SystemExit('Commit source changes before packaging: release source must match HEAD')
    vendor = ROOT / '.cache/vendor'
    if not vendor.is_dir(): raise SystemExit('First run cargo vendor --locked .cache/vendor')
    native_manifest = json.loads((ROOT/'licenses/native-sources.json').read_text(encoding='utf-8'))
    DIST.mkdir(exist_ok=True)
    # Refuse to overwrite a previous package/stage instead of deleting outputs.
    stage = DIST / f'ipa-first-tts-v{VERSION}-windows-x64-offline'
    stage.mkdir()
    for name in ['ipa-tts.exe','onnxruntime.dll','onnxruntime_providers_shared.dll','sherpa-onnx-c-api.dll','sherpa-onnx-cxx-api.dll']:
        copy(ROOT/'target/release'/name, stage/name)
    for name in ['README.md','LICENSE','NOTICE','THIRD_PARTY_NOTICES.md','licenses']:
        copy(ROOT/name,stage/name)
    copy(ROOT/'tools/espeak-ng',stage/'tools/espeak-ng')
    model = ROOT/'models/kokoro-multi-lang-v1_0'
    for name in ['model.onnx','voices.bin','tokens.txt','espeak-ng-data','LICENSE','README.md']:
        copy(model/name,stage/'models/kokoro-multi-lang-v1_0'/name)
    copy(ROOT/'models/README.md',stage/'models/README.md')
    for name in ['desktop.png','quick-brown-fox.wav','verification.json']:
        copy(ROOT/'outputs'/name,stage/'outputs'/name)
    (stage/'Start-IPA-Studio.cmd').write_text(
        '@echo off\nrem SPDX-License-Identifier: MIT\nsetlocal\ncd /d "%~dp0"\n'
        'set "ESPEAK_NG=%~dp0tools\\espeak-ng\\espeak-ng.exe"\n'
        'set "KOKORO_MODEL_DIR=%~dp0models\\kokoro-multi-lang-v1_0"\n'
        'echo Open http://127.0.0.1:17842 in your browser after the server starts.\n'
        '"%~dp0ipa-tts.exe" serve\npause\n',encoding='ascii',newline='\r\n')
    source_note = SOURCE_BUILD.format(revision=revision,version=VERSION)
    (stage/'SOURCE_BUILD.md').write_text(source_note,encoding='utf-8')
    print('Testing staged offline application...',flush=True)
    doctor = run_stage(stage,['doctor'])
    demo = run_stage(stage,['demo','outputs/release-smoke.wav','kokoro'])
    with wave.open(str(stage/'outputs/release-smoke.wav')) as wav:
        assert wav.getframerate()==24000 and wav.getnchannels()==1 and wav.getnframes()>24000
    verification = {'source_revision':revision,'release':f'v{VERSION}','doctor':doctor,'real_kokoro_demo':demo}
    (stage/'RELEASE_VERIFICATION.json').write_text(json.dumps(verification,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    inventory = [{'path':str(p.relative_to(stage)).replace('\\','/'),'bytes':p.stat().st_size,'sha256':hash_file(p)}
                 for p in sorted(stage.rglob('*')) if p.is_file()]
    (stage/'MANIFEST.json').write_text(json.dumps({'version':VERSION,'source_revision':revision,'files':inventory},indent=2)+'\n',encoding='utf-8')
    binary_zip = DIST/(stage.name+'.zip')
    if binary_zip.exists(): raise RuntimeError('Binary ZIP already exists')
    zip_tree(stage,binary_zip)
    print(f'Created {binary_zip.name}',flush=True)
    source_zip = DIST/f'ipa-first-tts-v{VERSION}-corresponding-source.zip'
    if source_zip.exists(): raise RuntimeError('Source ZIP already exists')
    top = f'ipa-first-tts-v{VERSION}-source/'
    with zipfile.ZipFile(source_zip,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6) as archive:
        for name in tracked: archive.write(ROOT/name,top+name)
        for file in sorted(vendor.rglob('*')):
            if file.is_file(): archive.write(file,top+'vendor/'+file.relative_to(vendor).as_posix())
        config = '[source.crates-io]\nreplace-with = "vendored-sources"\n[source.vendored-sources]\ndirectory = "vendor"\n'
        archive.writestr(top+'.cargo/config.toml',config)
        archive.writestr(top+'SOURCE_BUILD.md',source_note)
        for record in native_manifest:
            file = ROOT/'.cache/upstream'/record['file']
            if hash_file(file)!=record['sha256']: raise RuntimeError('Native source snapshot changed: '+file.name)
            archive.write(file,top+'native-sources/'+file.name,compress_type=zipfile.ZIP_STORED)
        native_binary = ROOT/'target/sherpa-onnx-prebuilt/sherpa-onnx-v1.13.8-win-x64-shared-MT-Release-lib.tar.bz2'
        archive.write(native_binary,top+'native-binaries/'+native_binary.name,compress_type=zipfile.ZIP_STORED)
    checksums = '\n'.join(f'{hash_file(file)}  {file.name}' for file in [binary_zip,source_zip])+'\n'
    (DIST/'SHA256SUMS.txt').write_text(checksums,encoding='ascii')
    for file in [binary_zip,source_zip]: print(f'{file.name}: {file.stat().st_size/1024/1024:.1f} MiB',flush=True)
    print('Release packaging and real Kokoro smoke check passed',flush=True)


if __name__ == '__main__':
    main()
