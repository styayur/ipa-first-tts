// SPDX-License-Identifier: GPL-3.0-or-later
'use strict';
const $ = id => document.getElementById(id);
let revision = 0, validationId = 0, busy = false, valid = false, sequence = [], aligned = false, timer;
const audio = $('audio');
let mappingMetadata, mappingResult, mappingChoice = 0, mappingRevision = 0;
const mapLabel = type => ({orthography:'Spelling / 正字法',romanization:'Romanization / 拉丁转写',ipa:'IPA'})[type];
function mappingClear() {
  ++mappingRevision;mappingResult=null;mappingChoice=0;
  $('map-candidates').replaceChildren();$('map-copy').disabled=true;$('map-studio').hidden=true;
  $('map-status').textContent='Input or scheme changed. Convert to refresh candidates.';
  $('map-provenance').textContent='';$('map-traces').textContent='';
  if(mappingMetadata)$('map-convert').disabled=!$('map-scheme').value;
}
function mappingSchemes() {
  const language=$('map-language').value;
  const all=mappingMetadata.capabilities.filter(c=>c.scheme.language===language);
  const available=all.filter(c=>c.available);
  const pairs=[...new Set(available.map(c=>c.scheme.input_type+'|'+c.scheme.output_type))];
  $('map-direction').replaceChildren(...pairs.map(pair=>{const o=document.createElement('option');o.value=pair;const [a,b]=pair.split('|');o.textContent=mapLabel(a)+' → '+mapLabel(b);return o;}));
  const l=mappingMetadata.languages.find(l=>l.id===language);$('map-target-language').value=l.name+' (same language)';
  $('map-coverage').textContent=l.coverage+(all.some(c=>!c.available)?' · General G2P requires the optional local eSpeak tool/full package.':'');
  mappingDirections();
}
function mappingDirections() {
  const pair=$('map-direction').value;
  const schemes=mappingMetadata.capabilities.filter(c=>c.available && c.scheme.language===$('map-language').value && c.scheme.input_type+'|'+c.scheme.output_type===pair);
  $('map-scheme').replaceChildren(...schemes.map(c=>{const o=document.createElement('option');o.value=c.scheme.id;o.textContent=c.scheme.name;return o;}));mappingScheme();
}
function mappingScheme() {
  mappingClear();const scheme=mappingMetadata.capabilities.find(c=>c.scheme.id===$('map-scheme').value)?.scheme;
  $('map-script').replaceChildren(...(scheme?.input_scripts||[]).map(code=>{const o=document.createElement('option');o.value=code;o.textContent=code+' / '+mappingMetadata.scripts.find(s=>s.code===code).name;return o;}));
  $('map-convert').disabled=!scheme;
  $('map-provenance').textContent=scheme?.limitations||'No available conversion. Choose another language or install optional local tools.';
}
function mappingSelected() {
  const r=mappingResult,c=r?.candidates[mappingChoice];$('map-copy').disabled=!c;
  $('map-studio').hidden=!(c && r.output_type==='ipa' && r.language==='en-US' && c.ipa_grammar_accepted===true);
  for(const [i,b] of [...$('map-candidates').children].entries())b.setAttribute('aria-pressed',String(i===mappingChoice));
  if(!r)return;
  $('map-provenance').textContent=[...r.warnings,...r.sources.map(s=>`${s.title}\n${s.version}\n${s.license} · ${s.attribution}\n${s.url}`)].join('\n\n');
  $('map-traces').textContent=c?JSON.stringify({transcription:c.transcription,ranking:c.ranking_basis,normalization:r.normalization,rules:c.traces},null,2):'';
}
async function mappingConvert() {
  const revision=++mappingRevision;const scheme=mappingMetadata.capabilities.find(c=>c.scheme.id===$('map-scheme').value)?.scheme;if(!scheme)return;
  $('map-convert').disabled=true;$('map-copy').disabled=true;$('map-studio').hidden=true;$('map-status').textContent='Converting locally…';
  try {
    const r=await api('map',{language:$('map-language').value,target_language:$('map-language').value,input:$('map-input').value,input_type:scheme.input_type,target:scheme.output_type,scheme:scheme.id,source_script:$('map-script').value,context:$('map-context').value||null});
    if(revision!==mappingRevision)return;mappingResult=r;mappingChoice=0;
    $('map-status').textContent=r.unsupported_reason||`${r.candidates.length} candidate${r.candidates.length===1?'':'s'} · ${r.ambiguous?'Ambiguous / not guaranteed unique':'Determined within this scheme'} · ${r.scheme}`;
    $('map-candidates').replaceChildren(...r.candidates.map((c,i)=>{const b=document.createElement('button');b.type='button';b.textContent=c.text;b.setAttribute('aria-label','Candidate '+(i+1)+': '+c.text);b.addEventListener('click',()=>{mappingChoice=i;mappingSelected();});return b;}));mappingSelected();
  } catch(e){if(revision===mappingRevision)$('map-status').textContent=e.message;}
  finally {if(revision===mappingRevision)$('map-convert').disabled=false;}
}
for(const id of ['map-input','map-context'])$(id).addEventListener('input',mappingClear);
$('map-language').addEventListener('change',mappingSchemes);$('map-direction').addEventListener('change',mappingDirections);$('map-scheme').addEventListener('change',mappingScheme);$('map-script').addEventListener('change',mappingClear);
$('map-input').addEventListener('keydown',e=>{if(e.ctrlKey&&e.key==='Enter'){e.preventDefault();mappingConvert();}});$('map-convert').addEventListener('click',mappingConvert);
$('map-copy').addEventListener('click',async()=>{const c=mappingResult?.candidates[mappingChoice];if(!c)return;try{await navigator.clipboard.writeText(c.text);$('map-status').textContent='Copied exact result. Clipboard replaced.';}catch(e){$('map-status').textContent='Copy failed: '+e.message;}});
$('map-studio').addEventListener('click',()=>{const r=mappingResult,c=r?.candidates[mappingChoice];if(busy||!c||r.output_type!=='ipa'||r.language!=='en-US'||c.ipa_grammar_accepted!==true)return;$('language').value='en-US';$('ipa').value=c.text;edited();$('ipa').focus();message('Selected English IPA copied into the editor. Model support is checked separately.');});
fetch('/api/mapping').then(r=>{if(!r.ok)throw new Error('Mapping metadata unavailable');return r.json();}).then(data=>{mappingMetadata=data;$('map-language').replaceChildren(...data.languages.map(l=>{const o=document.createElement('option');o.value=l.id;o.textContent=l.name+' / '+l.native_name+' ('+l.tag+')';return o;}));mappingSchemes();}).catch(e=>{$('map-convert').disabled=true;$('map-status').textContent=e.message;});
const stress = p => (p.stress === 'Primary' ? 'ˈ' : p.stress === 'Secondary' ? 'ˌ' : '') + p.symbol;
const message = (text, error = false) => { $('status').textContent = text; $('status').className = error ? 'error' : ''; };
function controls() {
  $('map-studio').disabled=busy;
  $('generate').disabled = busy; $('synthesize').disabled = busy || !valid;
  for (const id of ['ipa','text','language','backend','speed','speaker']) $(id).disabled = busy;
  document.querySelectorAll('.symbols button').forEach(b => b.disabled = busy);
}
function render(phones) {
  sequence = phones; $('phones').replaceChildren();
  for (const p of phones) {
    const span = document.createElement('span'); span.textContent = p.kind === 'WordBoundary' ? '' : stress(p);
    span.className = 'phone' + (p.kind === 'WordBoundary' ? ' boundary' : '');
    span.title = p.kind; $('phones').appendChild(span);
  }
}
function clearAudio() {
  audio.pause(); audio.removeAttribute('src'); audio.load(); aligned = false;
  $('play').disabled = true; $('stop').disabled = true; $('download').hidden = true;
  $('progress').value = 0; $('duration').textContent = '0:00'; $('timing').textContent = 'No audio yet';
  for (const p of $('phones').children) p.classList.remove('active');
}
async function api(path, data) {
  const response = await fetch('/api/' + path, { method:'POST', headers:{'Content-Type':'application/json'}, body:JSON.stringify(data) });
  const result = await response.json(); if (!response.ok) throw new Error(result.error || 'Local request failed'); return result;
}
function input() { return { ipa:$('ipa').value, language:$('language').value }; }
async function validate() {
  const r = revision, id = ++validationId;
  if (!$('ipa').value.trim()) { valid=false; render([]); $('validation').textContent='Enter IPA'; controls(); return; }
  try {
    const result = await api('validate', input()); if (r !== revision || id !== validationId) return;
    valid=true; $('validation').textContent='Valid IPA'; $('validation').className=''; render(result.sequence.phones);
  } catch(e) {
    if (r !== revision || id !== validationId) return;
    valid=false; render([]); $('validation').textContent=e.message; $('validation').className='error';
  } finally { controls(); }
}
function edited() {
  revision++; valid=false; clearAudio(); controls(); clearTimeout(timer);
  $('validation').textContent='Checking IPA…'; timer=setTimeout(validate,200);
}
$('ipa').addEventListener('input', edited);
$('language').addEventListener('change', edited);
for (const id of ['backend','speed','speaker']) $(id).addEventListener('change', clearAudio);
$('generate').addEventListener('click', async () => {
  busy=true; clearTimeout(timer); ++validationId; controls(); clearAudio();
  message('Generating IPA locally…');
  try {
    const result=await api('g2p',{text:$('text').value,language:$('language').value});
    $('ipa').value=result.ipa; revision++; valid=true; render(result.sequence.phones);
    $('validation').textContent='Valid IPA'; $('validation').className=''; message('IPA ready. Edit any phone, then synthesize.');
  } catch(e) { message(e.message,true); }
  finally { busy=false; controls(); }
});
$('synthesize').addEventListener('click',async()=>{
  busy=true; controls(); clearAudio(); message('Synthesizing edited IPA locally. First Kokoro load may take a moment…');
  try {
    const result=await api('synthesize',{...input(),backend:$('backend').value,speaker:Number($('speaker').value),speed:Number($('speed').value)});
    render(result.alignment.sequence.phones); aligned=true; audio.src=result.audio_url;
    $('play').disabled=false; $('stop').disabled=false; $('download').href=result.audio_url; $('download').hidden=false;
    $('timing').textContent='Estimated timing'; $('duration').textContent=(result.duration_ms/1000).toFixed(2)+' s';
    message(`Audio ready · ${result.backend === 'kokoro' ? 'Kokoro / sherpa-onnx' : 'eSpeak fallback'} · ${result.sample_rate} Hz`);
  } catch(e) { message(e.message,true); }
  finally { busy=false; controls(); }
});
$('play').addEventListener('click',async()=>{
  try { if (audio.ended) audio.currentTime=0; await audio.play(); } catch(e) { message('Playback failed: '+e.message,true); }
});
$('stop').addEventListener('click',()=>{audio.pause();audio.currentTime=0;updatePlayback();});
function updatePlayback() {
  $('progress').value = audio.duration ? audio.currentTime/audio.duration : 0;
  const time=audio.currentTime*1000;
  Array.from($('phones').children).forEach((el,i)=>{
    const p=sequence[i]; el.classList.toggle('active',aligned && !audio.paused && p.kind==='Phone' && time>=p.start_ms && time<p.end_ms);
  });
}
function frame() { updatePlayback(); if (!audio.paused && !audio.ended) requestAnimationFrame(frame); }
audio.addEventListener('play',frame); audio.addEventListener('pause',updatePlayback); audio.addEventListener('ended',updatePlayback);
audio.addEventListener('error',()=>{if(audio.hasAttribute('src')) message('Cannot load generated WAV. Synthesize again if the audio has expired.',true);});
let symbols = [], symbolIndex = 0, symbolRequest = 0;
function describeSymbol() {
  const s = symbols[symbolIndex];
  $('symbol-details').textContent = s ? `${s.name} · ${s.codepoints.join(' ')} · ${s.languages.join(', ')} · ${s.description}${s.examples.length ? ' · e.g. '+s.examples.join(' / ') : ''}` : 'No results / 没有结果';
  $('symbol-copy').disabled = !s;
  Array.from($('symbol-results').children).forEach((b,i)=>b.setAttribute('aria-pressed',String(i===symbolIndex)));
}
function insertSymbol() {
  const s = symbols[symbolIndex]; if (!s || busy) return;
  const field=$('ipa'); field.setRangeText(s.text,field.selectionStart,field.selectionEnd,'end');field.focus();edited();
}
async function searchSymbols() {
  const request = ++symbolRequest;
  try {
    const result = await api('symbols',{query:$('symbol-search').value,language:$('symbol-language').value || null,category:$('symbol-category').value || null});
    if(request!==symbolRequest) return;
    symbols=result.symbols; symbolIndex=0; $('symbol-results').replaceChildren();
    symbols.forEach((s,i)=>{
      const button=document.createElement('button');button.type='button';button.textContent=s.kind==='diacritic' && /^\p{M}/u.test(s.text) ? '◌'+s.text : s.text===' ' ? '␠' : s.text;
      button.title=s.name+' · '+s.codepoints.join(' ');button.setAttribute('aria-label',s.name);button.disabled=busy;
      button.addEventListener('focus',()=>{symbolIndex=i;describeSymbol();});
      button.addEventListener('click',()=>{symbolIndex=i;describeSymbol();insertSymbol();});$('symbol-results').append(button);
    }); describeSymbol();
  } catch(e) { $('symbol-details').textContent=e.message; }
}
$('symbol-panel').addEventListener('toggle',()=>{if($('symbol-panel').open){$('symbol-search').focus();searchSymbols();}});
$('symbol-search').addEventListener('input',searchSymbols);
for(const id of ['symbol-category','symbol-language']) $(id).addEventListener('change',searchSymbols);
$('symbol-panel').addEventListener('keydown',e=>{
  if(e.key==='Escape'){e.preventDefault();$('symbol-panel').open=false;$('ipa').focus();return;}
  if(e.target===$('symbol-search') || e.target.closest('#symbol-results')){
    if(e.key==='Enter'){e.preventDefault();insertSymbol();}
    const step={ArrowRight:1,ArrowLeft:-1,ArrowDown:8,ArrowUp:-8}[e.key];
    if(step && symbols.length){e.preventDefault();symbolIndex=Math.max(0,Math.min(symbols.length-1,symbolIndex+step));describeSymbol();$('symbol-results').children[symbolIndex]?.scrollIntoView({block:'nearest'});}
  }
});
$('symbol-copy').addEventListener('click',async()=>{
  const s=symbols[symbolIndex];if(!s)return;
  try{await navigator.clipboard.writeText(s.text);message('Copied exact catalog string. Clipboard replaced.');}
  catch(e){message('Copy failed: '+e.message,true);}
});
fetch('/api/catalog').then(r=>r.json()).then(c=>{
  for(const l of c.languages){const option=document.createElement('option');option.value=l.id;option.textContent=l.id+' · '+l.native_name;$('symbol-language').append(option);}
}).catch(e=>message('Catalog unavailable: '+e.message,true));
function receiveSymbolLink(){
  if(!location.hash.startsWith('#ipa=')) return;
  try{
    const value=decodeURIComponent(location.hash.slice(5));
    if(new TextEncoder().encode(value).length>8192) throw new Error('IPA exceeds 8192 UTF-8 bytes');
    $('ipa').value=value;history.replaceState(null,'',location.pathname);edited();
  } catch(e){message('Invalid Studio symbol link: '+e.message,true);}
}
addEventListener('hashchange',receiveSymbolLink);receiveSymbolLink();
fetch('/api/status').then(r=>r.json()).then(s=>{
  if(!s.kokoro_compiled){$('backend').value='espeak';$('backend').options[0].disabled=true;message('Kokoro is disabled in this build. eSpeak fallback is selected.');}
}).catch(e=>message('Local server unavailable: '+e.message,true));
