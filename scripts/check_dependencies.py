# SPDX-License-Identifier: MIT
"""Audit locked license expressions; retain upstream license/notice bytes.
Run --write after dependency changes; CI's default mode checks the inventory.
This is a license inventory check, separate from cargo audit's security database.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
APPROVED = {
    'MIT','Apache-2.0','MPL-2.0','GPL-3.0-or-later','ISC','BSD-3-Clause','Zlib','Unicode-3.0','CDLA-Permissive-2.0',
    'MIT OR Apache-2.0','Apache-2.0 OR MIT','MIT/Apache-2.0','Unlicense OR MIT','MIT OR Zlib OR Apache-2.0',
    'Apache-2.0 OR ISC OR MIT','Apache-2.0 AND ISC','(MIT OR Apache-2.0) AND Unicode-3.0',
    '0BSD OR MIT OR Apache-2.0','Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT',
    'MIT OR Apache-2.0 OR LGPL-2.1-or-later','Zlib OR Apache-2.0 OR MIT',
}
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--write',action='store_true')
    args=parser.parse_args()
    metadata=json.loads(subprocess.check_output(['cargo','metadata','--locked','--format-version','1'],cwd=ROOT))
    records=[]
    for package in sorted(metadata['packages'],key=lambda p:(p['name'],p['version'])):
        if package['license'] not in APPROVED:
            raise SystemExit(f"Unreviewed license: {package['name']}: {package['license']}")
        record={'name':package['name'],'version':package['version'],'license':package['license'],
                'source':package['source'] or ('first-party' if package['id'] in metadata['workspace_members'] else 'local Apache-2.0 security patch'), 'notices':[]}
        if package['id'] not in metadata['workspace_members']:
            directory=Path(package['manifest_path']).parent
            # Crate archives use both LICENSE-* and license-*; Windows globbing
            # hides this distinction while Linux correctly treats case literally.
            files=sorted(p for p in directory.iterdir() if p.is_file() and
                         p.name.upper().startswith(('LICENSE','COPYING','COPYRIGHT','NOTICE','UNLICENSE')))
            if package.get('license_file'): files.append(directory/package['license_file'])
            if not files and package['name']=='r-efi':
                # This upstream crate places its full license notices in AUTHORS.
                files=[directory/'AUTHORS',directory/'README.md']
            if not files: raise SystemExit('No upstream license file: '+package['name'])
            for source in files:
                destination=ROOT/'licenses/rust'/f"{package['name']}-{package['version']}"/source.name
                raw=source.read_bytes()
                if args.write:
                    destination.parent.mkdir(parents=True,exist_ok=True)
                    destination.write_bytes(raw)
                elif not destination.exists() or destination.read_bytes()!=raw:
                    raise SystemExit('Missing/changed notice: '+str(destination.relative_to(ROOT)))
                record['notices'].append({'path':destination.relative_to(ROOT).as_posix(),'sha256':hashlib.sha256(raw).hexdigest()})
        records.append(record)
    output=json.dumps({'format_version':1,'license_reviewed':'2026-10-25','packages':records},indent=2)+'\n'
    path=ROOT/'licenses/rust-dependencies.json'
    if args.write: path.write_text(output,encoding='utf-8')
    elif not path.exists() or path.read_text(encoding='utf-8')!=output: raise SystemExit('Dependency inventory changed; review and run --write')
    print(f'PASS: {len(records)} locked packages, reviewed license expressions and exact upstream notices')
if __name__=='__main__': main()
