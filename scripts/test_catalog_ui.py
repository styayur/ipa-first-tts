# SPDX-License-Identifier: MIT
"""Real Rust catalog HTTP/UI tests. No model or desktop injection is required.
Start a real no-default-features ipa-tts serve, then run this script.
"""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/'outputs'
OUT.mkdir(exist_ok=True)
URL = os.environ.get('IPA_TEST_URL','http://127.0.0.1:17842')
with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    context = browser.new_context(permissions=['clipboard-read','clipboard-write'],viewport={'width':1000,'height':960})
    external_requests=[]
    def local_only(route):
        if route.request.url.startswith(URL+'/'): route.continue_()
        else: external_requests.append(route.request.url); route.abort()
    context.route('**/*',local_only)
    page = context.new_page()
    errors = []
    page.on('pageerror',lambda e:errors.append(str(e)))
    page.goto(URL)
    page.wait_for_load_state('networkidle')
    # Inspect the rendered interface before using its selectors.
    assert page.get_by_role('heading',name='IPA Studio').is_visible()
    assert page.locator('#symbol-panel summary').inner_text() == 'Symbol catalog / 字符目录'
    page.locator('#symbol-panel summary').click()
    expect(page.locator('#symbol-results button')).to_have_count(368)
    page.locator('#symbol-search').fill('tsh')
    expect(page.locator('#symbol-results button')).to_have_count(1)
    expect(page.locator('#symbol-details')).to_contain_text('U+0074 U+0361 U+0283')
    page.locator('#symbol-search').press('Enter')
    expect(page.locator('#ipa')).to_have_value('t͡ʃ')
    expect(page.locator('#validation')).to_have_text('Valid IPA')
    page.locator('#symbol-search').fill('e\u0301')
    page.locator('#symbol-language').select_option('fr')
    expect(page.locator('#symbol-results button').first).to_have_text('é')
    page.locator('#symbol-copy').click()
    expect(page.locator('#status')).to_contain_text('Copied exact catalog string')
    assert page.evaluate('() => navigator.clipboard.readText()') == 'é'
    page.locator('#symbol-search').fill('definitely-no-matching-symbol-999')
    expect(page.locator('#symbol-results button')).to_have_count(0)
    expect(page.locator('#symbol-details')).to_have_text('No results / 没有结果')
    expect(page.locator('#symbol-copy')).to_be_disabled()
    page.locator('#symbol-search').fill('')
    page.locator('#symbol-language').select_option('de')
    page.locator('#symbol-category').select_option('orthography')
    expect(page.locator('#symbol-results button')).to_have_count(8)
    assert set(page.locator('#symbol-results button').all_inner_texts()) == set('äöüÄÖÜßẞ')
    page.locator('#symbol-search').focus()
    page.locator('#symbol-search').press('ArrowRight')
    assert page.locator('#symbol-results button[aria-pressed=true]').count()==1
    selected = page.locator('#symbol-results button[aria-pressed=true]').inner_text()
    page.locator('#ipa').fill('a')
    page.locator('#symbol-search').focus()
    page.locator('#symbol-search').press('Enter')
    expect(page.locator('#ipa')).to_have_value('a'+selected)
    page.locator('#symbol-search').press('Escape')
    assert not page.locator('#symbol-panel').evaluate('e=>e.open')
    page.locator('#symbol-panel summary').click()
    page.locator('#symbol-language').select_option('')
    page.locator('#symbol-category').select_option('')
    page.locator('#symbol-search').fill('U+1DF00')
    expect(page.locator('#symbol-results button')).to_have_count(1)
    page.locator('#symbol-copy').click()
    expect(page.locator('#status')).to_contain_text('Copied exact catalog string')
    assert page.evaluate('() => navigator.clipboard.readText()') == '\U0001df00'
    # Unsupported catalog symbols remain copyable; IPA Core rejects explicitly.
    page.locator('#ipa').fill('')
    page.locator('#symbol-results button').click()
    expect(page.locator('#ipa')).to_have_value('\U0001df00')
    expect(page.locator('#validation')).to_contain_text('unsupported IPA')
    page.screenshot(path=str(OUT/'catalog-desktop.png'),full_page=True)
    page.set_viewport_size({'width':390,'height':844})
    assert page.evaluate('() => document.documentElement.scrollWidth <= innerWidth')
    page.screenshot(path=str(OUT/'catalog-narrow.png'),full_page=True)
    # Fragment transfer preserves bytes in editor and is removed from address/history.
    page.goto(URL+'/#ipa=%74%CD%A1%CA%83')
    page.wait_for_load_state('networkidle')
    expect(page.locator('#ipa')).to_have_value('t͡ʃ')
    assert page.evaluate('() => location.hash') == ''
    # Verify API security using the real server, not interception or mocks.
    assert context.request.get(URL+'/api/catalog',headers={'Host':'evil.example'}).status==403
    assert context.request.post(URL+'/api/symbols',headers={'Origin':'https://evil.example'},data={'query':'a'}).status==403
    assert context.request.post(URL+'/api/symbols',data={'query':'x'*1025}).status==400
    assert context.request.post(URL+'/api/symbols',data={'query':'a','unexpected':1}).status==400
    assert context.request.post(URL+'/api/validate',data={'ipa':'a','language':'fr'}).status==400
    if os.environ.get('IPA_EXPECT_MISSING_MODELS')=='1':
        assert context.request.post(URL+'/api/synthesize',data={'ipa':'a','backend':'kokoro'}).status==400
        assert context.request.post(URL+'/api/symbols',data={'query':'tsh'}).json()['symbols'][0]['text']=='t͡ʃ'
    assert not external_requests,external_requests
    assert not errors,errors
    (OUT/'catalog-ui-verification.json').write_text(json.dumps({'catalog_count':368,'real_rust_server':True,'missing_assets_fixture':os.environ.get('IPA_EXPECT_MISSING_MODELS')=='1','model_independent':True,'search_normalization':True,'exact_clipboard_bmp_and_supplementary':True,'complete_tied_insertion':True,'unsupported_symbol_error':True,'keyboard_enter_arrows_escape':True,'narrow_390px':True,'host_origin_guards':True,'studio_fragment':True,'external_requests':external_requests,'page_errors':errors},indent=2)+'\n',encoding='utf-8')
    browser.close()
print('Real offline catalog UI and HTTP boundary checks passed')
