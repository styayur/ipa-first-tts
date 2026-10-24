# SPDX-License-Identifier: MIT
"""Verify downloaded/staged asset hashes and exact binary ZIP inventories.
python scripts/verify_assets.py path/to/SHA256SUMS.txt
Does not claim code signing, native UI acceptance or physically offline execution.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import zipfile

def sha256(stream): return hashlib.file_digest(stream,'sha256').hexdigest()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('checksums',type=Path)
    args=parser.parse_args()
    for row in args.checksums.read_text(encoding='utf-8').splitlines():
        expected,name=row.split('  ',1)
        if Path(name).name!=name or '/' in name or '\\' in name or len(expected)!=64: raise SystemExit('Invalid checksum entry')
        asset=args.checksums.parent/name
        with asset.open('rb') as stream: assert sha256(stream)==expected,asset
        with zipfile.ZipFile(asset) as archive:
            names=archive.namelist()
            assert len(names)==len(set(names)),'Duplicate ZIP paths'
            for entry in names:
                path=PurePosixPath(entry)
                assert not path.is_absolute() and '..' not in path.parts and '\\' not in entry,'Unsafe ZIP path'
                assert ':' not in entry,'Windows stream/drive path'
            assert archive.testzip() is None,'ZIP CRC failed'
            manifests=[n for n in names if n.count('/')==1 and n.endswith('/MANIFEST.json')]
            if manifests:
                assert len(manifests)==1
                prefix=manifests[0].rsplit('/',1)[0]+'/'
                manifest=json.loads(archive.read(manifests[0]))
                listed={prefix+f['path'] for f in manifest['files']}|{manifests[0]}
                assert set(names)==listed,'Unlisted or missing archive files'
                for record in manifest['files']:
                    entry=prefix+record['path']
                    assert archive.getinfo(entry).file_size==record['bytes']
                    with archive.open(entry) as stream: assert sha256(stream)==record['sha256'],entry
                assert not any('/palette.json' in n or '/.env' in n or '/target/' in n for n in names)
        print('PASS',name,expected)
if __name__=='__main__': main()
