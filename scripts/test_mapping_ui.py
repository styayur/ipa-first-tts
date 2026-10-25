# SPDX-License-Identifier: MIT
"""Real local mapping HTTP/UI tests; never substitute these for native insertion."""
from pathlib import Path
import json,os
from playwright.sync_api import sync_playwright,expect
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'outputs';OUT.mkdir(exist_ok=True)
URL=os.environ.get('IPA_TEST_URL','http://127.0.0.1:17842')
with sync_playwright() as p:
 browser=p.chromium.launch(headless=True)
 context=browser.new_context(permissions=['clipboard-read','clipboard-write'],viewport={'width':1100,'height':1000})
 external=[]
 def local_only(route):
  if route.request.url.startswith(URL+'/'):route.continue_()
  else:external.append(route.request.url);route.abort()
 context.route('**/*',local_only)
 page=context.new_page();errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
 page.goto(URL);page.wait_for_load_state('networkidle')
 # Inspect actual rendered state before selecting controls.
 assert page.get_by_role('heading',name='Language mapping / 语言映射').is_visible()
 assert page.locator('#map-language option').count()==6
 assert page.locator('#map-direction').input_value()=='orthography|ipa'
 assert page.locator('#map-scheme').input_value()=='cmudict-us-sample'
 page.locator('#map-input').fill('read');page.locator('#map-input').press('Control+Enter')
 expect(page.locator('#map-candidates button')).to_have_count(2)
 expect(page.locator('#map-status')).to_contain_text('Ambiguous')
 page.locator('#map-candidates button').nth(1).click()
 expect(page.locator('#map-candidates button').nth(1)).to_have_attribute('aria-pressed','true')
 page.locator('#map-copy').click();assert page.evaluate('() => navigator.clipboard.readText()')=='ɹˈid'
 page.locator('#map-studio').click();expect(page.locator('#ipa')).to_have_value('ɹˈid');expect(page.locator('#validation')).to_have_text('Valid IPA')
 page.locator('#map-input').fill('qzxzznotaword');expect(page.locator('#map-copy')).to_be_disabled();expect(page.locator('#map-candidates button')).to_have_count(0)
 page.locator('#map-convert').click();expect(page.locator('#map-status')).to_contain_text('outside the bundled')
 page.locator('#map-language').select_option('ja');expect(page.locator('#map-direction')).to_have_value('orthography|romanization')
 page.locator('#map-input').fill('コーヒー');page.locator('#map-convert').click();expect(page.locator('#map-candidates button')).to_have_text('kōhī')
 expect(page.locator('#map-studio')).to_be_hidden();page.locator('#map-evidence summary').click();expect(page.locator('#map-provenance')).to_contain_text('Unicode-3.0')
 page.locator('#map-copy').click();assert page.evaluate('() => navigator.clipboard.readText()')=='kōhī'
 page.locator('#map-input').fill('学校');page.locator('#map-convert').click();expect(page.locator('#map-status')).to_contain_text('Unmapped character');expect(page.locator('#map-candidates button')).to_have_count(0)
 page.locator('#map-language').select_option('cmn-Hans');page.locator('#map-input').fill('重');page.locator('#map-convert').click();expect(page.locator('#map-candidates button')).to_have_count(2)
 assert page.locator('#map-candidates button').all_text_contents()==['zhòng','chóng']
 page.locator('#map-context').fill('重庆');page.locator('#map-convert').click();expect(page.locator('#map-candidates button')).to_have_count(1);expect(page.locator('#map-candidates button')).to_have_text('chóng')
 page.locator('#map-context').fill('');page.locator('#map-language').select_option('ru');page.locator('#map-input').fill('Щука');page.locator('#map-convert').click();expect(page.locator('#map-candidates button')).to_have_text('Ŝuka')
 metadata=page.request.get(URL+'/api/mapping').json()
 if not any(c['available'] and c['scheme']['id']=='espeak-fr-fr' for c in metadata['capabilities']):
  page.locator('#map-language').select_option('fr');assert page.locator('#map-direction option').all_text_contents()==['IPA → IPA'];expect(page.locator('#map-coverage')).to_contain_text('optional local')
 # Typed API failures never produce a substituted or partial candidate.
 for request in [dict(language='ru',input='Москва',target='ipa',scheme='cldr49-ru-subset'),dict(language='xx',input='cat',target='ipa'),dict(language='ja',input='学校',target='romanization',scheme='cldr49-kana-subset')]:
  response=page.request.post(URL+'/api/map',data=request);assert response.ok;result=response.json();assert result['status']=='unsupported' and not result['candidates']
 assert page.request.post(URL+'/api/map',data={'language':'ja','input':'か','target':'romanization','surprise':True}).status==400
 assert page.request.post(URL+'/api/map',data={'language':'ja','input':'か','target':'romanization'},headers={'Origin':'https://example.invalid'}).status==403
 page.set_viewport_size({'width':390,'height':844});assert page.evaluate('() => document.documentElement.scrollWidth <= innerWidth')
 page.screenshot(path=str(OUT/'mapping-narrow.png'),full_page=True)
 page.set_viewport_size({'width':1100,'height':1000});page.locator('#map-language').select_option('ja');page.locator('#map-input').fill('コーヒー');page.locator('#map-convert').click();expect(page.locator('#map-candidates button')).to_have_text('kōhī');page.screenshot(path=str(OUT/'mapping-desktop.png'),full_page=True)
 assert not errors,errors;assert not external,external
 (OUT/'mapping-ui-verification.json').write_text(json.dumps({'real_server':True,'real_embedded_mapping':True,'model_required':False,'candidate_choice_copy':True,'english_editor_transfer':True,'unicode_kana_han_russian':True,'unsupported_and_stale_results':True,'provenance':True,'narrow_390px':True,'external_requests':external,'page_errors':errors},indent=2)+'\n',encoding='utf-8')
 context.close();browser.close()
print('Real mapping UI / HTTP / provenance tests passed')
