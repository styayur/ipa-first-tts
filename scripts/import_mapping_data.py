# SPDX-License-Identifier: MIT
"""Generate pinned, bounded mapping data from reviewed local upstream snapshots.
No network or runtime downloads. Fetches are a separate engineering step.
"""
from pathlib import Path
import argparse,hashlib,json,re,shutil,unicodedata as ud

ROOT=Path(__file__).resolve().parents[1]
CLDR='96e54b273d2da4ec6fbad350ba19ff23fa24a588'
CMU='74790861f652b15e4ac49015a90074ad62a27690'
WORDS='''a about above after again air all also am an and any are around as ask at away back bad be because become bed been before begin behind being below best better between big bird black blue boat book both boy bread break bring brown build but buy by call came can car care carry case cat cause change child city close cold come could country course cut day dead dear death deep did different do does dog done door down draw dream drink drive dry during each ear early earth east easy eat eight either end enough even ever every eye face fact fall far fast father fear feel feet few field find fire first fish five floor fly follow food foot for force form found four free friend from front full game gave get girl give go good got great green ground group grow had hand hard has have he head hear heard heart heat heavy help her here high him his hold home horse hot house how however human hundred i ice idea if important in into is it its just keep kind knew know land language large last late later laugh law lay lead learn least leave left less let letter life light like line list little live long look love low made make man many may me mean men might mile mind miss moment money month more morning most mother mountain move much must my name near need never new next night no north not nothing now number of off often old on once one only open or order other our out over own page paper part pass past pay people person picture place play point put question quick rain read ready real really reason red reed remember rest right river road rock room round run said same saw say school sea second see seem seen set several she ship short should show side simple since sing sit six sleep small so some something song soon sound south space speak special spring stand start state stay still stood stop story street strong student such summer sun sure system table take talk tell ten than that the their them then there these they thing think third this those though thought three through time to today together told too took top toward town tree true try turn two under until up us use usual very voice want war warm was watch water way we week well went were what when where which while white who whole why wide will wind window winter wish with without woman women word work world would write wrong year yes yet you young your'''.split()
WORDS.append('hello')

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--source-dir',type=Path,required=True);args=p.parse_args()
 src=args.source_dir;dest=ROOT/'crates/language-mapping/data';up=dest/'upstream';up.mkdir(parents=True,exist_ok=True)
 records=[]
 def copy(name,url,version,license,usage):
  raw=(src/name).read_bytes();(up/name).write_bytes(raw)
  records.append(dict(path='upstream/'+name,url=url,version=version,retrieved='2026-10-25',license=license,usage=usage,sha256=hashlib.sha256(raw).hexdigest()))
 scopes={'Hiragana-Latin.xml':'Reference for transform-chain semantics only; no full ICU rule interpreter.',
         'Latin-Katakana.xml':'Selected literal rows and common context policies adapted for the named kana subset.',
         'Hiragana-Katakana.xml':'Reference for the explicit selected Hiragana/Katakana block fold; NFKC/iteration/exception repertoire not fully implemented.',
         'Cyrillic-Latin.xml':'Modern Russian alphabet rows and NFD/case policy adapted; no other Cyrillic language or ISO conformance claim.',
         'Han-Latin.xml':'Han default-reading groups and three explicit right-context rules transformed; non-Han radical aliases excluded.',
         'Latin-NumericPinyin.xml':'Copied reference snapshot only; numeric Pinyin conversion is NOT implemented.'}
 for name in scopes:
  local='cldr-'+name if (src/('cldr-'+name)).exists() else name
  copy(local,f'https://raw.githubusercontent.com/unicode-org/cldr/{CLDR}/common/transforms/{name}',CLDR,'Unicode-3.0',scopes[name]+' Original data copied; no LDML specification publication text copied.')
 copy('cldr-LICENSE',f'https://raw.githubusercontent.com/unicode-org/cldr/{CLDR}/LICENSE',CLDR,'Unicode-3.0','Original combined notice retained, including specification/data distinction.')
 copy('cmudict-LICENSE',f'https://raw.githubusercontent.com/cmusphinx/cmudict/{CMU}/LICENSE',CMU,'BSD-2-Clause','Original CMU notice for selected dictionary entries.')
 copy('cmudict.phones',f'https://raw.githubusercontent.com/cmusphinx/cmudict/{CMU}/cmudict.phones',CMU,'BSD-2-Clause','ARPAbet inventory; not IPA.')
 # Retain only actually selected original lines, not a silently invented lexicon.
 selected=[]
 dictionary_source=src/'cmudict.dict' if (src/'cmudict.dict').exists() else src/'cmudict-selected.dict'
 for line in dictionary_source.read_text(encoding='utf-8').splitlines():
  key=line.split(' ',1)[0].split('(')[0]
  if key in WORDS:selected.append(line)
 (src/'cmudict-selected.dict').write_text('\n'.join(selected)+'\n',encoding='utf-8',newline='\n')
 copy('cmudict-selected.dict',f'https://raw.githubusercontent.com/cmusphinx/cmudict/{CMU}/cmudict.dict',CMU,'BSD-2-Clause','Selected complete entries and all variants; full upstream SHA-256 81917843c7f44ce2b094ac63873c2c7a4cf802040792c455ba3ca406891c3d22.')
 # Selected registry records are facts extracted from their official snapshots.
 iana=(src/('iana-language-subtag-registry' if (src/'iana-language-subtag-registry').exists() else 'iana-selected.txt')).read_text(encoding='utf-8')
 blocks=[b.strip() for b in iana.split('%%') if any(re.search(r'^Subtag: '+re.escape(tag)+r'$',b,re.M) for tag in ['en','fr','es','ja','cmn','ru','US','FR','ES','Hans','Latn','Cyrl','Hira','Kana','Hrkt','Zyyy'])]
 (src/'iana-selected.txt').write_text(iana.split('%%')[0]+'%%\n'+'\n%%\n'.join(blocks)+'\n',encoding='utf-8',newline='\n')
 copy('iana-selected.txt','https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry','File-Date: 2026-09-17','CC0-1.0','Selected registry records; IANA/IETF dedication: https://www.iana.org/help/licensing-terms (2021-11-10).')
 codes={'Latn','Cyrl','Hani','Hans','Hira','Kana','Hrkt','Zyyy'}
 text=(src/('iso15924.txt' if (src/'iso15924.txt').exists() else 'iso15924-selected.txt')).read_text(encoding='utf-8')
 (src/'iso15924-selected.txt').write_text('\n'.join(l for l in text.splitlines() if l.startswith('#') or l.split(';')[0] in codes)+'\n',encoding='utf-8',newline='\n')
 copy('iso15924-selected.txt','https://unicode.org/iso15924/iso15924.txt','snapshot 2026-10-25','Unicode-3.0','Selected official script identifiers; no invented IPA script code.')
 sources=[dict(id='cldr49',organization='Unicode Consortium',title='CLDR 49 transform data',version=CLDR,url='https://cldr.unicode.org/index/downloads',license='Unicode-3.0',attribution='Copyright Unicode, Inc.; retain upstream/cldr-LICENSE',use='copied/transformed data; not the LDML publication'),dict(id='cmudict',organization='Carnegie Mellon University / CMUSphinx',title='CMU Pronouncing Dictionary',version=CMU,url=f'https://github.com/cmusphinx/cmudict/tree/{CMU}',license='BSD-2-Clause',attribution='Copyright (C) 1993-2015 Carnegie Mellon University; upstream/cmudict-LICENSE',use='selected dictionary entries; explicit project ARPAbet adapter'),dict(id='project',organization='Stya Yur Open Source Studio',title='Typed mapping adapter and bounded algorithms',version='1',url='https://github.com/styayur/ipa-first-tts',license='MPL-2.0',attribution='Stya Yur Open Source Studio',use='original code and adapter policies'),dict(id='espeak152',organization='eSpeak NG contributors',title='eSpeak NG G2P adapter',version='1.52.0 / 4870adfa25b1a32b4361592f1be8a40337c58d6c',url='https://github.com/espeak-ng/espeak-ng/tree/1.52.0',license='GPL-3.0-or-later',attribution='Retain bundled eSpeak notices and corresponding source.',use='optional separate process, never model initialization'),dict(id='iana',organization='IANA',title='Language Subtag Registry',version='2026-09-17',url='https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry',license='CC0-1.0',attribution='IANA / IETF Trust; https://www.iana.org/help/licensing-terms',use='registered identifiers only'),dict(id='iso15924',organization='ISO 15924 Registration Authority / Unicode',title='Script code list',version='snapshot 2026-10-25',url='https://unicode.org/iso15924/iso15924-codes.html',license='Unicode-3.0',attribution='Unicode, Inc.',use='script identifiers, not paid romanization standards')]
 scripts=[dict(code=c,name=n,description=desc,languages=langs,source='iso15924') for c,n,desc,langs in [('Latn','Latin','Latin script; does not imply one romanization or phonology.',['en-US','fr','es','ja','cmn-Hans','ru']),('Cyrl','Cyrillic','Russian alphabet subset of Cyrillic.',['ru']),('Hrkt','Japanese syllabaries','Hiragana and Katakana; Kanji excluded.',['ja']),('Hira','Hiragana','U+3040..U+309F; selected kana.',['ja']),('Kana','Katakana','U+30A0..U+30FF; selected kana.',['ja']),('Hans','Han (Simplified variant)','Character-reading table, not a word dictionary.',['cmn-Hans']),('Hani','Han','Han script; only CLDR default readings imported.',['cmn-Hans']),('Zyyy','Common','IPA technical notation uses Common here; no invented ISO code.',['en-US','fr','es','ja','cmn-Hans','ru'])]]
 languages=[dict(id=tag,tag=tag,name=name,native_name=native,scripts=ss,region=region,coverage=coverage,sources=['iana','project']) for tag,name,native,ss,region,coverage in [('en-US','English (US)','English',['Latn','Zyyy'],'US','selected CMUdict; optional eSpeak G2P; bounded P2G'),('fr','French','français',['Latn','Zyyy'],'FR','optional eSpeak G2P only; no French synthesis'),('es','Spanish','español',['Latn','Zyyy'],'ES','optional eSpeak G2P only; no Spanish synthesis'),('ja','Japanese','日本語',['Hrkt','Hira','Kana','Zyyy'],None,'named kana romanization subset only; no Kanji G2P'),('cmn-Hans','Mandarin (Han)','普通话',['Hans','Hani','Latn','Zyyy'],None,'CLDR Han default-reading romanization and three phrase contexts; no Mandarin IPA'),('ru','Russian','русский',['Cyrl','Latn','Zyyy'],None,'CLDR Russian alphabet subset romanization and constrained inverse; no Russian IPA')]]
 next(l for l in languages if l['id']=='ja')['scripts'].append('Latn')
 schemes=[]
 def scheme(id,language,name,kind,inp,out,script,output_script,source,version='1',reversible=False,external=False,context=False,limits=''):
  schemes.append(dict(id=id,language=language,name=name,kind=kind,input_type=inp,output_type=out,input_scripts=script.split(','),output_script=output_script,version=version,reversible=reversible,context_dependent=context,external_backend=external,dictionary=kind in ['dictionary','p2g','espeak'],reverse_candidates=kind in ['dictionary','p2g'],source=source,limitations=limits))
 scheme('cmudict-us-sample','en-US','CMUdict selected US vocabulary / project ARPAbet adapter v1','dictionary','orthography','ipa','Latn','Zyyy','cmudict',CMU,context=True,limits='Selected vocabulary only. All pronunciation variants retained; word lookup cannot resolve heteronyms. Stress precedes the ARPAbet vowel, not an inferred syllable onset. Broad phonemic approximation, not measured speech.')
 scheme('cmudict-us-candidates','en-US','Exact IPA → CMUdict selected spelling candidates','p2g','ipa','orthography','Zyyy','Latn','cmudict',CMU,context=True,limits='Finite exact dictionary candidates; stress is significant. Not an inverse G2P and not all spellings.')
 scheme('cldr49-kana-subset','ja','CLDR 49 kana → Latin subset v1 (modified Hepburn family)','kana','orthography','romanization','Hrkt,Hira,Kana','Latn','cldr49','49/subset-1',limits='Common kana/digraphs, sokuon, syllabic n and prolonged-vowel mark. Explicit dji/dzu and n apostrophe policy. No Kanji, particles-as-pronunciation, halfwidth or iteration marks; not complete Hepburn or full ICU conformance.')
 scheme('cldr49-ru-subset','ru','CLDR 49 Cyrillic → Latin, Russian alphabet subset v1','russian','orthography','romanization','Cyrl','Latn','cldr49','49/subset-1',True,limits='Modern Russian alphabet, case and ASCII spaces only; not ISO 9 conformance. Inverse applies only to this exact alphabet/output repertoire, compared in NFC.')
 scheme('cldr49-ru-inverse','ru','Inverse of CLDR 49 Russian alphabet subset v1','russian_inverse','romanization','orthography','Latn','Cyrl','cldr49','49/subset-1',True,limits='Exact bounded alphabet inverse; not IPA-to-Russian spelling.')
 scheme('cldr49-han-readings','cmn-Hans','CLDR 49 Han → tone-marked Latin default readings','han','orthography','romanization','Hans,Hani','Latn','cldr49','49/subset-1',context=True,limits='Han character readings, not full word segmentation or Mandarin G2P. 三 phrase contexts 藏文/重庆/秘鲁; isolated contextual characters return alternatives. Other polyphony is not resolved. Pinyin is not IPA.')
 for lang,voice in [('en-US','en-us'),('fr','fr-fr'),('es','es')]:
  scheme('espeak-'+voice,lang,'eSpeak NG 1.52.0 '+voice+' G2P','espeak','orthography','ipa','Latn','Zyyy','espeak152','1.52.0/adapter-1',external=True,context=True,limits='Optional pinned local executable and voice data. Broad rule-based transcription; unknown words and isolated heteronyms may be wrong. No claim of whole-language accuracy or non-English synthesis. Custom executable overrides have unverified version.')
 for language in languages:
  scheme('ipa-normalize-'+language['id'],language['id'],'ipa-core conservative normalization','ipa','ipa','ipa','Zyyy','Zyyy','project',limits='NFD and project aliases/grammar only; no language phonology validation or TTS guarantee.')
 rules=[]
 def rule(id,language,scheme,input,outputs,matcher='token',source='cldr49',priority=10,right=None,references=None,limits=''):
  rules.append(dict(id=id,language=language,scheme=scheme,input=input,outputs=outputs,matcher=matcher,priority=priority,right_context=right,source=source,version=CLDR if source=='cldr49' else CMU if source=='cmudict' else '1',references=references or [id]*len(outputs),limitations=limits))
 # Parse only simple literal reverse/bidirectional kana rows. Contextual ICU
 # grammar is not interpreted by this importer; the bounded algorithms are explicit.
 kana={}
 for line in (src/'Latin-Katakana.xml').read_text(encoding='utf-8').splitlines():
  line=line.split('#')[0].strip()
  m=re.fullmatch(r'([a-z]+)\s*[↔←]\s*([\u30a1-\u30fa\u3099\u309a ]+)\s*;',line)
  if m:kana[ud.normalize('NFC',m[2].replace(' ',''))]=m[1]
 for base,roman in list(kana.items()):
  if len(base)==1 and roman.endswith('i') and roman not in ['i','wi','yi']:
   for small,vowel in [('ャ','a'),('ュ','u'),('ョ','o')]:
    result=(roman[:-1] if roman in ['shi','chi','ji','dji'] else roman[:-1]+'y')+vowel
    kana.setdefault(base+small,result)
 for key,value in sorted(kana.items()):rule('kana-'+ '-'.join(f'{ord(c):04x}' for c in key),'ja','cldr49-kana-subset',key,[value],limits='CLDR literal/common small-y context subset; input folds Hiragana to Katakana explicitly.')
 lower='абвгдеёжзийклмнопрстуфхцчшщъыьэюя'
 latin=['a','b','v','g','d','e','ë','ž','z','i','j','k','l','m','n','o','p','r','s','t','u','f','h','c','č','š','ŝ','ʺ','y','ʹ','é','û','â']
 for c,value in zip(lower,latin,strict=True):
  for char,out in [(c,value),(c.upper(),value.upper() if c not in 'ъь' else value+'\u0331')]:
   rule(f'ru-{ord(char):04x}','ru','cldr49-ru-subset',char,[out],limits='NFD Cyrillic-Latin rules composed into NFC; uppercase signs retain underline as in upstream.')
 han_seen=set()
 for row in (src/'cldr-Han-Latin.xml').read_text(encoding='utf-8').splitlines():
  m=re.fullmatch(r'\[([^\[\]\\]+)\]→([^;]+);',row.strip())
  if m:
   key,value=m.groups()
   selected_chars=[]
   for original in key:
    if not ('\u3400'<=original<='\u9fff' or '\uf900'<=original<='\ufaff' or 0x20000<=ord(original)<=0x323af):continue
    char=ud.normalize('NFC',original)
    if char not in han_seen:
     selected_chars.append(char);han_seen.add(char)
   if selected_chars:
    rule('han-'+ '-'.join(f'{ord(c):04x}' for c in value),'cmn-Hans','cldr49-han-readings',''.join(selected_chars),[value],matcher='characters',limits='Unihan kMandarin default selected by CLDR; may be polyphonic. Non-Han radical aliases excluded; NFC duplicates follow first upstream rule.')
 for char,right,output in [('藏','文','zàng'),('重','庆','chóng'),('秘','鲁','bì')]:
  rule('han-context-'+f'{ord(char):04x}','cmn-Hans','cldr49-han-readings',char,[output],matcher='context',priority=100,right=right,limits='Only the explicit right-context character in CLDR is recognized.')
 arpa={'AA':'ɑ','AE':'æ','AH':'ʌ','AO':'ɔ','AW':'aʊ','AY':'aɪ','B':'b','CH':'t͡ʃ','D':'d','DH':'ð','EH':'ɛ','ER':'ɝ','EY':'eɪ','F':'f','G':'ɡ','HH':'h','IH':'ɪ','IY':'i','JH':'d͡ʒ','K':'k','L':'l','M':'m','N':'n','NG':'ŋ','OW':'oʊ','OY':'ɔɪ','P':'p','R':'ɹ','S':'s','SH':'ʃ','T':'t','TH':'θ','UH':'ʊ','UW':'u','V':'v','W':'w','Y':'j','Z':'z','ZH':'ʒ'}
 dictionary={}
 for line in selected:
  word,phones=line.split(' ',1);phones=phones.split('#')[0].strip();out=''
  for phone in phones.split():
   stress=phone[-1] if phone[-1].isdigit() else ''
   key=phone.rstrip('012');symbol=arpa[key]
   if stress=='0' and key=='AH':symbol='ə'
   if stress=='0' and key=='ER':symbol='ɚ'
   out+=('ˈ' if stress=='1' else 'ˌ' if stress=='2' else '')+symbol
  base=word.split('(')[0]
  dictionary.setdefault(base,[]).append((out,word+' '+phones))
 for word,entries in sorted(dictionary.items()):
  unique={}
  for out,ref in entries:unique.setdefault(out,ref)
  rule('cmu-'+word.replace("'",'-'),'en-US','cmudict-us-sample',word,list(unique),matcher='exact',source='cmudict',references=list(unique.values()),limits='Exact selected dictionary entry; no contextual sense disambiguation.')
 bundle=dict(schema_version=1,languages=languages,scripts=scripts,sources=sources,schemes=schemes,rules=rules)
 (dest/'registry.json').write_text(json.dumps(bundle,ensure_ascii=False,separators=(',',':'))+'\n',encoding='utf-8',newline='\n')
 raw=(dest/'registry.json').read_bytes()
 records.append(dict(path='registry.json',url='scripts/import_mapping_data.py',version='1',retrieved='2026-10-25',license='mixed: Unicode-3.0 / BSD-2-Clause; see per-rule sources',usage='Generated selected rules and registry facts; project adapter code is MPL-2.0.',sha256=hashlib.sha256(raw).hexdigest()))
 (dest/'sources.json').write_text(json.dumps(dict(schema_version=1,files=records),indent=2,ensure_ascii=False)+'\n',encoding='utf-8',newline='\n')
 print(f'{len(rules)} rules; {len(dictionary)} dictionary spellings; {sum(len(r["input"]) for r in rules if r["matcher"]=="characters")} Han characters; {len(kana)} kana tokens; {len(raw)} generated bytes')
if __name__=='__main__':main()
