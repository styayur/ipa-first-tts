# SPDX-License-Identifier: MIT
"""Check source bytes, fixed versions, licenses and independently authored fixtures."""
from pathlib import Path,PurePosixPath
import hashlib,json

ROOT=Path(__file__).resolve().parents[1]
def main():
 data=ROOT/'crates/language-mapping/data'
 manifest=json.loads((data/'sources.json').read_text(encoding='utf-8'))
 assert manifest['schema_version']==1
 listed=set()
 for record in manifest['files']:
  name=record['path'];path=PurePosixPath(name)
  assert not path.is_absolute() and '..' not in path.parts and ':' not in name
  assert name not in listed;listed.add(name)
  raw=(data/name).read_bytes();assert hashlib.sha256(raw).hexdigest()==record['sha256'],name
  assert all(record[k] for k in ['url','version','retrieved','license','usage'])
 assert {p.relative_to(data).as_posix() for p in (data/'upstream').iterdir() if p.is_file()}<=listed
 registry=json.loads((data/'registry.json').read_text(encoding='utf-8'))
 sources={s['id']:s for s in registry['sources']}
 for rule in registry['rules']:
  source=sources[rule['source']]
  assert source['version']==rule['version'] and source['license'] and source['attribution']
  assert rule['references'] and len(rule['references'])==len(rule['outputs'])
 fixtures=json.loads((ROOT/'crates/language-mapping/tests/fixtures/builtin.json').read_text(encoding='utf-8'))
 for case in fixtures['cases']:assert case['source'] in fixtures['sources'] and case['reference'] and case['expected']
 assert set(c['language'] for c in fixtures['cases'])=={'en-US','ja','ru','cmn-Hans'}
 external=json.loads((ROOT/'crates/language-mapping/tests/fixtures/external.json').read_text(encoding='utf-8'))
 assert set(c['language'] for c in external['cases'])=={'fr','es'}
 assert 'specification/data distinction' in next(r['usage'] for r in manifest['files'] if r['path'].endswith('cldr-LICENSE'))
 print(f'PASS: {len(manifest["files"])} source/data hashes; {len(registry["rules"])} sourced rules; {len(fixtures["cases"])} builtin + {len(external["cases"])} independent external fixtures')
if __name__=='__main__':main()
