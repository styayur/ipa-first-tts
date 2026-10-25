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
import tarfile

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
    with zipfile.ZipFile(destination, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=6, strict_timestamps=False) as archive:
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
    parser.add_argument('--candidate',action='store_true',help='Local candidate only; explicit suffix and unverified evidence retained')
    parser.add_argument('--vc-redist-dir', type=Path, help='Licensed Visual Studio x64 Microsoft.VC143.CRT directory')
    parser.add_argument('--native-evidence',type=Path,required=True,help='Final-commit native UI validation evidence')
    parser.add_argument('--target-dir',type=Path,default=ROOT/'target',help='Isolated Cargo build directory')
    parser.add_argument('--output',type=Path,default=DIST,help='Fresh output directory; existing stages/ZIPs are never overwritten')
    options = parser.parse_args()
    if os.name != 'nt': raise SystemExit('This package targets Windows x64')
    if not options.skip_build:
        subprocess.run(['cargo','build','--release','--workspace','--locked','--target-dir',str(options.target_dir.resolve())],cwd=ROOT,check=True)
    revision = subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    from package_palette import check_evidence
    native_evidence=json.loads(options.native_evidence.read_text(encoding='utf-8'))
    if not options.candidate:check_evidence(native_evidence,revision)
    tracked = subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode('utf-8').split('\0')
    tracked = [p for p in tracked if p]
    dirty = subprocess.check_output(['git','status','--porcelain'],cwd=ROOT,text=True)
    if dirty.strip(): raise SystemExit('Commit source changes before packaging: release source must match HEAD')
    vendor = ROOT / f'.cache/vendor-v{VERSION}'
    if not vendor.is_dir(): raise SystemExit(f'First run cargo vendor --locked .cache/vendor-v{VERSION}')
    native_manifest = json.loads((ROOT/'licenses/native-sources.json').read_text(encoding='utf-8'))
    native_name='sherpa-onnx-v1.13.8-win-x64-shared-MT-Release-lib.tar.bz2'
    candidates=[options.target_dir.resolve()/'sherpa-onnx-prebuilt'/native_name,*options.target_dir.resolve().glob('release/build/sherpa-onnx-sys-*/out/sherpa-onnx-prebuilt/'+native_name)]
    candidates=[p for p in candidates if p.is_file()]
    if not candidates:raise SystemExit('Matching native archive absent: set CARGO_TARGET_DIR for the upstream build script or provide its build/out cache')
    required={'onnxruntime.dll','onnxruntime_providers_shared.dll','sherpa-onnx-c-api.dll','sherpa-onnx-cxx-api.dll'}
    native_binary=None
    for candidate in candidates:
        found={}
        with tarfile.open(candidate,'r:bz2') as archive:
            for member in archive:
                name=Path(member.name).name
                if member.isfile() and name in required:
                    with archive.extractfile(member) as stream:found[name]=hashlib.file_digest(stream,'sha256').hexdigest()
        if set(found)==required and all(hash_file(options.target_dir.resolve()/'release'/n)==h for n,h in found.items()):native_binary=candidate;break
    if native_binary is None:raise SystemExit('No native source-build archive matches every shipped runtime DLL')
    for record in native_manifest:
        if hash_file(ROOT/'.cache/upstream'/record['file'])!=record['sha256']:raise SystemExit('Native source snapshot hash mismatch: '+record['file'])
    dist=options.output.resolve();dist.mkdir(parents=True,exist_ok=True)
    # Refuse to overwrite a previous package/stage instead of deleting outputs.
    suffix='-candidate' if options.candidate else ''
    stage = dist / f'ipa-first-tts-v{VERSION}-windows-x64-offline{suffix}'
    stage.mkdir()
    for name in ['ipa-tts.exe','ipa-symbol-palette.exe','ipa-map.exe','onnxruntime.dll','onnxruntime_providers_shared.dll','sherpa-onnx-c-api.dll','sherpa-onnx-cxx-api.dll']:
        copy(options.target_dir.resolve()/'release'/name, stage/name)
    for name in ['README.md','README.zh-CN.md','docs','LICENSE','NOTICE','THIRD_PARTY_NOTICES.md','licenses']:
        copy(ROOT/name,stage/name)
    copy(options.native_evidence,stage/'NATIVE_VALIDATION.json')
    copy(ROOT/'apps/palette/palette.example.json',stage/'palette.example.json')
    copy(ROOT/'tools/espeak-ng',stage/'tools/espeak-ng')
    redist = options.vc_redist_dir
    if redist is None:
        candidates = sorted(Path(os.environ['ProgramFiles(x86)']).glob(
            'Microsoft Visual Studio/2022/*/VC/Redist/MSVC/*/x64/Microsoft.VC143.CRT'))
        if not candidates: raise SystemExit('Supply --vc-redist-dir from your licensed Visual Studio installation')
        redist = candidates[-1]
    # App-local deployment: retain Microsoft binaries unchanged, never copy System32.
    for name in ['vcruntime140.dll', 'vcruntime140_1.dll', 'msvcp140.dll']:
        for destination in [stage/name, stage/'tools/espeak-ng'/name]:
            copy(redist/name, destination)
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
    mappings={}
    for language,scheme,text,target in [('ja','cldr49-kana-subset','コーヒー','romanization'),('ru','cldr49-ru-subset','Щука','romanization'),('cmn-Hans','cldr49-han-readings','重庆','romanization'),('en-US','cmudict-us-sample','read','ipa'),('fr','espeak-fr-fr','eau','ipa'),('es','espeak-es','niño','ipa')]:
        result=json.loads(run_stage(stage,['map','--language',language,'--scheme',scheme,'--input',text,'--target',target]))
        assert result['candidates'] and result['status'] in ['ok','ambiguous'];mappings[language]=result
    with wave.open(str(stage/'outputs/release-smoke.wav')) as wav:
        assert wav.getframerate()==24000 and wav.getnchannels()==1 and wav.getnframes()>24000
    verification = {'source_revision':revision,'release':f'v{VERSION}','candidate':options.candidate,'doctor':doctor,'real_kokoro_demo':demo,'mapping_checks':mappings,'native_binary_archive_sha256':hash_file(native_binary)}
    verification.update({'authenticode_signed':False,'physical_network_disabled':False,
                         'clean_vm_without_development_tools_tested':False})
    (stage/'RELEASE_VERIFICATION.json').write_text(json.dumps(verification,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    inventory = [{'path':str(p.relative_to(stage)).replace('\\','/'),'bytes':p.stat().st_size,'sha256':hash_file(p)}
                 for p in sorted(stage.rglob('*')) if p.is_file()]
    (stage/'MANIFEST.json').write_text(json.dumps({'version':VERSION,'source_revision':revision,'files':inventory},indent=2)+'\n',encoding='utf-8')
    binary_zip = dist/(stage.name+'.zip')
    if binary_zip.exists(): raise RuntimeError('Binary ZIP already exists')
    zip_tree(stage,binary_zip)
    print(f'Created {binary_zip.name}',flush=True)
    source_zip = dist/f'ipa-first-tts-v{VERSION}-corresponding-source{suffix}.zip'
    if source_zip.exists(): raise RuntimeError('Source ZIP already exists')
    top = f'ipa-first-tts-v{VERSION}-source/'
    with zipfile.ZipFile(source_zip,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=6,strict_timestamps=False) as archive:
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
        archive.write(native_binary,top+'native-binaries/'+native_binary.name,compress_type=zipfile.ZIP_STORED)
    checksums = '\n'.join(f'{hash_file(file)}  {file.name}' for file in [binary_zip,source_zip])+'\n'
    with (dist/f'SHA256SUMS-v{VERSION}-tts{suffix}.txt').open('x',encoding='ascii') as checksum_file:
        checksum_file.write(checksums)
    for file in [binary_zip,source_zip]: print(f'{file.name}: {file.stat().st_size/1024/1024:.1f} MiB',flush=True)
    print('Release packaging and real Kokoro smoke check passed',flush=True)


if __name__ == '__main__':
    main()
