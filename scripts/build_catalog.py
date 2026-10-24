# SPDX-License-Identifier: MIT
"""Reproduce curated data. Input: official Unicode 17 UnicodeData.txt (not a chart).
Run: python scripts/build_catalog.py .cache/UnicodeData-17.0.0.txt
The selection, phonetic labels and examples are authored here; no chart is copied.
"""
import json
import sys
import unicodedata
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
names = {chr(int(line.split(';')[0], 16)): line.split(';')[1]
         for line in Path(sys.argv[1]).read_text(encoding='utf-8').splitlines()}
date = '2026-10-24'
sources = [
    dict(id='unicode17', name='Unicode Character Database 17.0.0', url='https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt', license='Unicode-3.0', accessed=date),
    dict(id='ipa-reference', name='IPA official chart (consulted, not reproduced)', url='https://www.internationalphoneticassociation.org/content/ipa-chart', license='Reference only; chart CC-BY-SA-4.0; no chart artwork copied', accessed=date),
    dict(id='cldr47', name='CLDR 47 language exemplar references (curated selection)', url='https://github.com/unicode-org/cldr/tree/release-47/common/main', license='Unicode-3.0', accessed=date),
]
langs = [('und','IPA / language-neutral','IPA',['国际音标','ipa']), ('en','English','English',['英语']), ('fr','French','français',['法语']), ('de','German','Deutsch',['德语']), ('es','Spanish','español',['西班牙语']), ('pt','Portuguese','português',['葡萄牙语']), ('pl','Polish','polski',['波兰语']), ('cs','Czech','čeština',['捷克语']), ('tr','Turkish','Türkçe',['土耳其语']), ('vi','Vietnamese','Tiếng Việt',['越南语'])]
symbols = []
def add(text, kind, category, name, aliases=(), languages=('und',), description=None, examples=()):
    symbols.append(dict(id=category+'-'+'-'.join(f'{ord(c):04x}' for c in text), text=text,
        kind=kind, category=category, tags=[category, 'ipa' if languages==('und',) else 'latin'],
        languages=list(languages), scripts=['Zinh'] if kind=='diacritic' else ['Latn'], name=name,
        aliases=list(aliases), codepoints=[f'U+{ord(c):04X}' for c in text],
        description=description or name, examples=list(examples),
        sources=['unicode17','ipa-reference' if languages==('und',) else 'cldr47'], license='CC-BY-4.0'))

consonants = '''p|voiceless bilabial plosive
b|voiced bilabial plosive
t|voiceless alveolar plosive
d|voiced alveolar plosive
ʈ|voiceless retroflex plosive
ɖ|voiced retroflex plosive
c|voiceless palatal plosive
ɟ|voiced palatal plosive
k|voiceless velar plosive
ɡ|voiced velar plosive
q|voiceless uvular plosive
ɢ|voiced uvular plosive
ʔ|glottal stop
m|bilabial nasal
ɱ|labiodental nasal
n|alveolar nasal
ɳ|retroflex nasal
ɲ|palatal nasal
ŋ|velar nasal
ɴ|uvular nasal
ʙ|bilabial trill
r|alveolar trill
ʀ|uvular trill
ⱱ|labiodental flap
ɾ|alveolar tap
ɽ|retroflex flap
ɸ|voiceless bilabial fricative
β|voiced bilabial fricative
f|voiceless labiodental fricative
v|voiced labiodental fricative
θ|voiceless dental fricative
ð|voiced dental fricative
s|voiceless alveolar fricative
z|voiced alveolar fricative
ʃ|voiceless postalveolar fricative
ʒ|voiced postalveolar fricative
ʂ|voiceless retroflex fricative
ʐ|voiced retroflex fricative
ç|voiceless palatal fricative
ʝ|voiced palatal fricative
x|voiceless velar fricative
ɣ|voiced velar fricative
χ|voiceless uvular fricative
ʁ|voiced uvular fricative
ħ|voiceless pharyngeal fricative
ʕ|voiced pharyngeal fricative
h|voiceless glottal fricative
ɦ|voiced glottal fricative
ɬ|voiceless alveolar lateral fricative
ɮ|voiced alveolar lateral fricative
ʋ|labiodental approximant
ɹ|alveolar approximant
ɻ|retroflex approximant
j|palatal approximant
ɰ|velar approximant
l|alveolar lateral approximant
ɭ|retroflex lateral approximant
ʎ|palatal lateral approximant
ʟ|velar lateral approximant
w|voiced labial velar approximant
ɥ|labial palatal approximant
ʘ|bilabial click
ǀ|dental click
ǃ|alveolar click
ǂ|palatal click
ǁ|lateral click
ɓ|bilabial implosive
ɗ|alveolar implosive
ʄ|palatal implosive
ɠ|velar implosive
ʛ|uvular implosive'''
for line in consonants.splitlines():
    text, label = line.split('|')
    aliases = [names[text].lower()]
    if text=='ʃ': aliases += ['esh','sh']
    if text=='ʒ': aliases += ['ezh','zh']
    if text=='ɲ': aliases += ['ny','enye','palatal n']
    if text=='ʁ': aliases += ['uvular r']
    add(text,'phoneme','consonants',label,aliases)
vowels = [('i','close front unrounded'),('y','close front rounded'),('ɨ','close central unrounded'),('ʉ','close central rounded'),('ɯ','close back unrounded'),('u','close back rounded'),('ɪ','near-close near-front unrounded'),('ʏ','near-close near-front rounded'),('ʊ','near-close near-back rounded'),('e','close-mid front unrounded'),('ø','close-mid front rounded'),('ɘ','close-mid central unrounded'),('ɵ','close-mid central rounded'),('ɤ','close-mid back unrounded'),('o','close-mid back rounded'),('ə','mid central'),('ɛ','open-mid front unrounded'),('œ','open-mid front rounded'),('ɜ','open-mid central unrounded'),('ɞ','open-mid central rounded'),('ʌ','open-mid back unrounded'),('ɔ','open-mid back rounded'),('æ','near-open front unrounded'),('ɐ','near-open central'),('a','open front unrounded'),('ɶ','open front rounded'),('ɑ','open back unrounded'),('ɒ','open back rounded'),('ɚ','rhotic mid central'),('ɝ','rhotic open-mid central')]
for text,label in vowels: add(text,'phoneme','vowels',label+' vowel',[names[text].lower(), 'schwa' if text=='ə' else label])
for text,label,example in [('̥','voiceless','n̥'),('̬','voiced','s̬'),('̃','nasalized','ã'),('̩','syllabic','n̩'),('̯','non-syllabic','i̯'),('̪','dental','t̪'),('̺','apical','s̺'),('̻','laminal','s̻'),('̹','more rounded','ɔ̹'),('̜','less rounded','ɔ̜'),('̟','advanced','u̟'),('̠','retracted','i̠'),('̈','centralized','ë'),('̽','mid-centralized','e̽'),('̝','raised','e̝'),('̞','lowered','e̞'),('̘','advanced tongue root','e̘'),('̙','retracted tongue root','e̙'),('̰','creaky voice','a̰'),('̤','breathy voice','a̤'),('̚','no audible release','t̚'),('͡','tie bar above','t͡ʃ'),('͜','tie bar below','t͜ʃ')]:
    add(text,'diacritic','diacritics',label,[names[text].lower()],examples=[example],description=label+'; combine with a base, do not add the display dotted circle.')
for text,label in [('ʰ','aspirated'),('ʲ','palatalized'),('ʷ','labialized'),('ˠ','velarized'),('ˤ','pharyngealized'),('ⁿ','nasal release'),('ˡ','lateral release'),('ʼ','ejective')]:
    add(text,'diacritic','diacritics',label,[names[text].lower()],examples=['t'+text])
for text,kind,label in [('ˈ','stress','primary stress'),('ˌ','stress','secondary stress'),('ː','length','long'),('ˑ','length','half long'),('˥','tone','extra high tone'),('˦','tone','high tone'),('˧','tone','mid tone'),('˨','tone','low tone'),('˩','tone','extra low tone'),('↗','tone','global rise'),('↘','tone','global fall'),('ꜛ','tone','upstep'),('ꜜ','tone','downstep')]:
    add(text,kind,'prosody',label,[names[text].lower()],examples=['a'+text] if kind!='stress' else [text+'a'])
for text,label,aliases in [('t͡ʃ','voiceless postalveolar affricate',['tsh','ch']),('d͡ʒ','voiced postalveolar affricate',['dzh','j']),('t͡s','voiceless alveolar affricate',['ts']),('d͡z','voiced alveolar affricate',['dz']),('aɪ','closing diphthong a-i',['ai']),('aʊ','closing diphthong a-u',['au']),('eɪ','closing diphthong e-i',['ei']),('oʊ','closing diphthong o-u',['ou']),('ɔɪ','closing diphthong open-o-i',['oi']),('ã','nasalized open front vowel',['a tilde']),('˩˥','rising tone contour',['rising']),('˥˩','falling tone contour',['falling'])]:
    add(text,'combination','combinations',label,aliases,description=label+'; inserted as one complete candidate, without normalization.')
for text,kind,label in [(' ','boundary','word boundary'),('|','boundary','minor group boundary'),('‖','boundary','major group boundary'),('.','boundary','syllable break'),('/','punctuation','phonemic transcription delimiter'),('[','punctuation','phonetic transcription opening'),(']','punctuation','phonetic transcription closing')]:
    add(text,kind,'punctuation',label)
# A supplementary-plane phonetic extension exercises surrogate-pair insertion.
add('\U0001df00','phoneme','consonants','f with right hook (phonetic extension)',[names['\U0001df00'].lower()],description='Extended phonetic character; catalog support does not imply IPA Core or TTS support.')
data = dict(schema_version=1, languages=[dict(id=i,name=n,native_name=nn,aliases=a) for i,n,nn,a in langs], sources=sources, symbols=symbols)
out = ROOT/'crates/symbol-catalog/data'
(out/'ipa.json').write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
symbols = []
selection = {'fr':'œçéèêëàâîïôùûüÿæ','de':'äöüß','es':'ñáéíóúü¿¡','pt':'áàâãéêíóôõúç','pl':'ąćęłńóśźż','cs':'áčďéěíňóřšťúůýž','tr':'çğıİöşü','vi':'đăâêôơư'}
for language,texts in selection.items():
    chars = set(texts)
    chars.update(c.upper() if len(c.upper()) == 1 else 'ẞ' if c == 'ß' else c for c in texts)
    if language=='vi':
        for base in 'aăâeêioôơuưyAĂÂEÊIOÔƠUƯY':
            for tone in '\u0300\u0301\u0303\u0309\u0323': chars.add(unicodedata.normalize('NFC',base+tone))
    for char in sorted(chars):
        existing = next((s for s in symbols if s['text']==char),None)
        if existing: existing['languages'].append(language); continue
        add(char,'punctuation' if char in '¿¡' else 'orthography','orthography',names[char].lower(),[{'œ':'oe ligature','Œ':'OE ligature','ß':'eszett sharp s','ẞ':'capital sharp s'}.get(char,names[char].lower())],languages=(language,),description='Curated Latin-script character; language association is not a TTS capability claim.')
(out/'orthography.json').write_text(json.dumps(dict(schema_version=1,languages=[],sources=[],symbols=symbols),ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Wrote',len(data['symbols'])+len(symbols),'symbols')
