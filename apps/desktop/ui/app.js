// SPDX-License-Identifier: GPL-3.0-or-later
'use strict';
const $ = id => document.getElementById(id);
let revision = 0, validationId = 0, busy = false, valid = false, sequence = [], aligned = false, timer;
const audio = $('audio');
const stress = p => (p.stress === 'Primary' ? 'ˈ' : p.stress === 'Secondary' ? 'ˌ' : '') + p.symbol;
const message = (text, error = false) => { $('status').textContent = text; $('status').className = error ? 'error' : ''; };
function controls() {
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
document.querySelectorAll('.symbols button').forEach(b=>b.addEventListener('click',()=>{
  const field=$('ipa'),start=field.selectionStart,end=field.selectionEnd;
  field.setRangeText(b.textContent,start,end,'end');field.focus();edited();
}));
fetch('/api/status').then(r=>r.json()).then(s=>{
  if(!s.kokoro_compiled){$('backend').value='espeak';$('backend').options[0].disabled=true;message('Kokoro is disabled in this build. eSpeak fallback is selected.');}
}).catch(e=>message('Local server unavailable: '+e.message,true));
