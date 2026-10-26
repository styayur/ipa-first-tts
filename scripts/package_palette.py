# SPDX-License-Identifier: MIT
"""Stage a model-free Release palette and its corresponding Rust source.

Unverified local builds require --candidate and cannot use release asset names.
Final packaging requires the native matrix evidence, an exact clean commit and
an already built Release executable. Never overwrites an existing stage or ZIP.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import zipfile

ROOT=Path(__file__).resolve().parents[1]
VERSION=tomllib.loads((ROOT/'Cargo.toml').read_text(encoding='utf-8'))['workspace']['package']['version']
NATIVE_GATES=['notepad','browser','vscode','hotkey_toggle','tray','focus_restore','continuous_insert','copy_exact','favorites_recent','bilingual','resize_dpi','exit_cleanup','single_instance']
MAPPING_GATES=['mapping_ui','mapping_copy','mapping_to_palette']

def check_evidence(evidence,revision):
    if evidence.get('version')!=VERSION or evidence.get('source_revision')!=revision:
        raise SystemExit('Native evidence must identify the exact package version/commit')
    for gate in NATIVE_GATES+MAPPING_GATES:
        if evidence.get('checks',{}).get(gate,{}).get('status')!='passed':raise SystemExit('Native release gate not passed: '+gate)
def digest(path):
    with path.open('rb') as file: return hashlib.file_digest(file,'sha256').hexdigest()
def archive_tree(directory,destination):
    with zipfile.ZipFile(destination,'x',zipfile.ZIP_DEFLATED,compresslevel=6,strict_timestamps=False) as archive:
        for file in sorted(directory.rglob('*')):
            if file.is_file(): archive.write(file,file.relative_to(directory.parent).as_posix())
def inventory(stage,revision):
    files=[{'path':p.relative_to(stage).as_posix(),'bytes':p.stat().st_size,'sha256':digest(p)} for p in sorted(stage.rglob('*')) if p.is_file()]
    (stage/'MANIFEST.json').write_text(json.dumps({'version':VERSION,'source_revision':revision,'files':files},indent=2)+'\n',encoding='utf-8')
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--candidate',action='store_true')
    parser.add_argument('--evidence',type=Path,default=ROOT/f'docs/validation/v{VERSION}-native.json')
    parser.add_argument('--output',type=Path,default=ROOT/'dist')
    parser.add_argument('--vc-redist-dir',type=Path)
    parser.add_argument('--target-dir',type=Path,default=ROOT/'target',help='Isolated Cargo build directory, useful while a development EXE is running')
    args=parser.parse_args()
    if os.name!='nt': raise SystemExit('Windows x64 packaging only')
    if subprocess.check_output(['git','status','--porcelain'],cwd=ROOT).strip(): raise SystemExit('Commit source changes before packaging')
    revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    evidence=json.loads(args.evidence.read_text(encoding='utf-8'))
    if not args.candidate:
        check_evidence(evidence,revision)
    subprocess.run(['cargo','build','-p','ipa-palette','--release','--locked','--target-dir',str(args.target_dir.resolve())],cwd=ROOT,check=True)
    executable=args.target_dir.resolve()/'release/ipa-symbol-palette.exe'
    if not executable.is_file(): raise SystemExit('Run cargo build -p ipa-palette --release --locked first')
    redist=args.vc_redist_dir
    if redist is None:
        choices=sorted(Path(os.environ['ProgramFiles(x86)']).glob('Microsoft Visual Studio/2022/*/VC/Redist/MSVC/*/x64/Microsoft.VC143.CRT'))
        if not choices: raise SystemExit('Supply licensed VC runtime redistributable directory')
        redist=choices[-1]
    args.output.mkdir(parents=True,exist_ok=True)
    suffix='-candidate' if args.candidate else ''
    stage=args.output/f'ipa-symbol-palette-v{VERSION}-windows-x64{suffix}'
    stage.mkdir()
    shutil.copy2(executable,stage/executable.name)
    shutil.copy2(args.target_dir.resolve()/'release/ipa-map.exe',stage/'ipa-map.exe')
    for file in ['vcruntime140.dll','vcruntime140_1.dll','msvcp140.dll']: shutil.copy2(redist/file,stage/file)
    for name in ['LICENSE','NOTICE','THIRD_PARTY_NOTICES.md']: shutil.copy2(ROOT/name,stage/name)
    shutil.copytree(ROOT/'licenses',stage/'licenses')
    shutil.copytree(ROOT/'crates/symbol-catalog/data',stage/'catalog-data')
    shutil.copytree(ROOT/'crates/language-mapping/data',stage/'mapping-data')
    shutil.copy2(ROOT/'docs/language-mapping.md',stage/'LANGUAGE_MAPPING.md')
    shutil.copy2(ROOT/'apps/palette/palette.example.json',stage/'palette.example.json')
    (stage/'START.txt').write_text(f'''IPA Symbol Palette v{VERSION} — Windows x64{' LOCAL CANDIDATE / NOT RELEASE-VERIFIED' if args.candidate else ''}
Run ipa-symbol-palette.exe. No model, Rust, Python, Node or WebView is required.
The global shortcut works only while this app is running. Launch the EXE first;
window × keeps it resident, whereas Exit unregisters the shortcut. No autostart.
Ctrl+Alt+Shift+I toggles the window; search immediately, arrows select, Enter
inserts (change Enter action to Copy), Esc hides. Tray: show/hide, Studio,
Settings, Exit. Closing hides; the visible Exit button ends the background app.
The title shows the version and the tray tooltip shows your saved shortcut.
Launching another location warns when an older copy already owns the instance.
Settings changes the shortcut; conflicts preserve the previous registration.
English / 简体中文 switches the interface. All languages is a catalog filter,
not a speech capability. The catalog is a curated selection, not full coverage.
Copy replaces the clipboard. Insert uses Windows Unicode input and never
changes the clipboard. Release modifier keys before inserting. Permissions,
remote sessions and target apps can reject input: use Copy explicitly.
Use the hotkey from the target editor to capture it; tray opening is copy-only.
Compose + appends the entire selected string to the composition field. A
nonempty composition overrides the selected candidate for Copy/Insert/Studio.
Supplementary characters may require a font installed in your target app.
IPA Studio is optional and requires the full sibling ipa-tts package.
Mapping / 语言映射 opens the native local mapping window. Ctrl+Enter converts.
Choose language, typed direction/scheme and script. Candidate ambiguity and
source limits remain visible. Copy replaces the clipboard; Palette fills the
composition and does not inject input. English IPA can be sent to Studio.
ipa-map.exe languages
ipa-map.exe romanize --language ja --scheme cldr49-kana-subset --input コーヒー
Embedded mapping needs no model/network. French/Spanish/general English G2P
requires optional eSpeak; the light package does not ship it or a speech model.
See LANGUAGE_MAPPING.md. Romanization is not IPA, P2G is bounded dictionary search.
Config: %LOCALAPPDATA%/IPA-first-TTS/palette.json (not included in this ZIP).
palette.example.json is a default configuration example, never loaded automatically.
Corrupt settings are preserved as palette.corrupt-*; unknown future versions
are never overwritten. No autostart, installer, accounts or telemetry.
Source revision: {revision}
SHA-256 is an integrity checksum, NOT Authenticode signing. Binary is unsigned.
''',encoding='utf-8')
    shutil.copy2(args.evidence,stage/'NATIVE_VALIDATION.json')
    environment={'SystemRoot':os.environ['SystemRoot'],'WINDIR':os.environ['WINDIR'],
                 'PATH':str(Path(os.environ['SystemRoot'])/'System32'),'TEMP':tempfile.gettempdir(),'TMP':tempfile.gettempdir()}
    with tempfile.TemporaryDirectory(prefix='ipa-palette-clean-') as temporary:
        clean=Path(temporary)
        for file in stage.iterdir():
            if file.suffix.lower() in ('.exe','.dll'): shutil.copy2(file,clean/file.name)
        check=clean/'self-check.json'
        subprocess.run([str(clean/executable.name),'--check',str(check)],cwd=clean,env=environment,check=True,timeout=20)
        result=json.loads(check.read_text(encoding='utf-8'))
        assert result['symbols']==368 and result['tie']=='t͡ʃ' and result['model_required'] is False
        mapped=subprocess.run([str(clean/'ipa-map.exe'),'romanize','--language','ja','--scheme','cldr49-kana-subset','--input','コーヒー'],cwd=clean,env=environment,check=True,capture_output=True,text=True,encoding='utf-8',timeout=20)
        mapping_result=json.loads(mapped.stdout)
        assert mapping_result['candidates'][0]['text']=='kōhī' and mapping_result['output_type']=='romanization'
    verification={'source_revision':revision,'candidate':args.candidate,'release_executable_sha256':digest(executable),
                  'clean_directory_self_check':result,'clean_directory_mapping_check':mapping_result,'sanitized_path':True,'physical_network_disabled':False,
                  'clean_vm_without_development_tools_tested':False,'authenticode_signed':False}
    (stage/'RELEASE_VERIFICATION.json').write_text(json.dumps(verification,indent=2,ensure_ascii=False)+'\n',encoding='utf-8')
    inventory(stage,revision)
    binary=args.output/(stage.name+'.zip'); archive_tree(stage,binary)
    # The palette needs no native TTS corresponding sources. Include tracked
    # first-party source, the local Apache patch and locked Cargo vendor sources.
    vendor=ROOT/f'.cache/vendor-v{VERSION}'
    if not vendor.is_dir(): raise SystemExit('Prepare a fresh versioned cargo vendor directory: '+str(vendor))
    source=args.output/f'ipa-symbol-palette-v{VERSION}-source{suffix}.zip'
    tracked=subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).decode().split('\0')
    top=f'ipa-symbol-palette-v{VERSION}-source/'
    with zipfile.ZipFile(source,'x',zipfile.ZIP_DEFLATED,compresslevel=6,strict_timestamps=False) as archive:
        for name in filter(None,tracked): archive.write(ROOT/name,top+name)
        for file in sorted(vendor.rglob('*')):
            if file.is_file(): archive.write(file,top+'vendor/'+file.relative_to(vendor).as_posix())
        archive.writestr(top+'.cargo/config.toml','[source.crates-io]\nreplace-with="vendored-sources"\n[source.vendored-sources]\ndirectory="vendor"\n')
        archive.writestr(top+'PALETTE_SOURCE_BUILD.txt',f'Source: {revision}\nBuild: cargo build -p ipa-palette --release --offline --locked\nRust stable MSVC + licensed C++ Build Tools / Windows SDK required for compilation only.\nThird-party licenses retain their scope.\n')
    sums=args.output/f'SHA256SUMS-v{VERSION}-palette{suffix}.txt'
    with sums.open('x',encoding='ascii') as file:
        for asset in [binary,source]: file.write(f'{digest(asset)}  {asset.name}\n')
    for asset in [binary,source]:
        with zipfile.ZipFile(asset) as archive: assert archive.testzip() is None
        print(asset.name,asset.stat().st_size,digest(asset))
if __name__=='__main__': main()
