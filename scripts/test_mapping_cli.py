# SPDX-License-Identifier: MIT
"""Evaluate declared fixtures, not whole-language accuracy; no generated oracle."""
from pathlib import Path
import argparse,collections,json,os,subprocess,unicodedata
ROOT=Path(__file__).resolve().parents[1]
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--exe',type=Path,required=True);parser.add_argument('--with-espeak',action='store_true');args=parser.parse_args()
 executable=args.exe.resolve();env=os.environ.copy();env['KOKORO_MODEL_DIR']=str(ROOT/'.cache/intentionally-no-mapping-model')
 if not args.with_espeak:env['ESPEAK_NG']=str(ROOT/'.cache/intentionally-no-mapping-espeak')
 def call(*arguments):
  result=subprocess.run([str(executable),*arguments],cwd=ROOT,env=env,capture_output=True,text=True,encoding='utf-8',timeout=20)
  assert result.returncode==0,result.stdout+result.stderr
  return json.loads(result.stdout)
 metadata=call('languages');assert len(metadata['languages'])==6
 scores=collections.defaultdict(lambda:{'cases':0,'passed':0,'definition':'Exact expected candidate set on the declared manually selected fixture vocabulary/subset; not a held-out whole-language estimate.'})
 observations=[]
 fixtures=json.loads((ROOT/'crates/language-mapping/tests/fixtures/builtin.json').read_text(encoding='utf-8'))
 for case in fixtures['cases']:
  result=call('map','--language',case['language'],'--scheme',case['scheme'],'--input',case['input'],'--target','ipa' if case['language']=='en-US' else 'romanization')
  output=[unicodedata.normalize('NFC',c['text']) for c in result['candidates']]
  assert output==case['expected'],(case,output)
  score=scores[case['language']];score['cases']+=1;score['passed']+=1
  observations.append({'language':case['language'],'input':case['input'],'outputs':output,'reference':case['reference'],'source':fixtures['sources'][case['source']]})
 if args.with_espeak:
  fixtures=json.loads((ROOT/'crates/language-mapping/tests/fixtures/external.json').read_text(encoding='utf-8'))
  for case in fixtures['cases']:
   result=call('map','--language',case['language'],'--scheme',case['scheme'],'--input',case['input'],'--target','ipa')
   segmental=unicodedata.normalize('NFC',''.join(c for c in result['candidates'][0]['text'] if c not in 'ˈˌ.'))
   assert segmental==case['segmental']
   score=scores[case['language']];score['cases']+=1;score['passed']+=1;score['definition']=fixtures['comparison']
   observations.append({'language':case['language'],'input':case['input'],'output':result['candidates'][0]['text'],'reference':case['reference'],'source':case['source']})
 result={'version':'0.3.0-alpha.1','model_loaded':False,'external_espeak_tested':args.with_espeak,'definition':'Fixture agreement only. Counts do not estimate coverage/accuracy over natural language. Negative/schema/ambiguity tests are additional Rust tests.','language_results':dict(scores),'observations':observations,'unresolved':'English heteronyms need sense context; Han defaults have unresolved polyphony outside three phrase contexts; French/Spanish sentence/dialect accuracy is not measured. No Japanese/Mandarin/Russian G2P or non-English TTS.'}
 (ROOT/'outputs').mkdir(exist_ok=True)
 (ROOT/'outputs/mapping-evaluation.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
 print(json.dumps(result['language_results'],ensure_ascii=False,indent=2))
if __name__=='__main__':main()
