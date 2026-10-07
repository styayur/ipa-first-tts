# SPDX-License-Identifier: MIT
# Copyright (c) 2026 Stya Yur Open Source Studio
"""Real local UI smoke test. Requires Python Playwright + Chromium.

Start ipa-tts serve first, then run python scripts/test_ui.py.
No mock G2P, model or audio is used.
"""
from pathlib import Path
import json
import urllib.request
from playwright.sync_api import sync_playwright, expect

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "outputs"
OUTPUT.mkdir(exist_ok=True)

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True, args=["--autoplay-policy=no-user-gesture-required"])
    page = browser.new_page(viewport={"width": 1120, "height": 1000})
    errors = []
    page.on("pageerror", lambda error: errors.append(str(error)))
    page.goto("http://127.0.0.1:17842")
    page.wait_for_load_state("networkidle")
    # Inspect rendered state before interacting.
    assert page.get_by_role("heading", name="IPA Studio").is_visible()
    assert page.get_by_role("button", name="Generate IPA", exact=True).is_enabled()
    page.get_by_role("button", name="Generate IPA", exact=True).click()
    expect(page.locator("#validation")).to_have_text("Valid IPA", timeout=15000)
    generated_ipa = page.locator("#ipa").input_value()
    assert "ð" in generated_ipa and "ˈ" in generated_ipa
    page.get_by_role("button", name="Synthesize", exact=True).click()
    expect(page.locator("#play")).to_be_enabled(timeout=90000)
    expect(page.locator("#status")).to_contain_text("Kokoro / sherpa-onnx")
    wav_url = page.locator("#download").get_attribute("href")
    wav = urllib.request.urlopen("http://127.0.0.1:17842" + wav_url).read()
    assert wav[:4] == b"RIFF" and len(wav) > 48000
    (OUTPUT / "ui-pangram.wav").write_bytes(wav)
    page.get_by_role("button", name="Play", exact=True).click()
    page.wait_for_function("() => !document.getElementById('audio').paused && document.getElementById('audio').currentTime > 0.1")
    page.wait_for_selector(".phone.active")
    page.screenshot(path=str(OUTPUT / "desktop.png"), full_page=True)
    page.get_by_role("button", name="Stop", exact=True).click()
    assert page.locator("#audio").evaluate("a => a.paused && a.currentTime === 0")
    assert page.locator(".phone.active").count() == 0
    # Editing invalidates old audio, validates Unicode, and then uses new IPA.
    page.locator("#ipa").fill("ˈʃiː")
    expect(page.locator("#play")).to_be_disabled()
    expect(page.locator("#validation")).to_have_text("Valid IPA")
    page.get_by_role("button", name="Synthesize", exact=True).click()
    expect(page.locator("#play")).to_be_enabled(timeout=90000)
    edited_url = page.locator("#download").get_attribute("href")
    edited = urllib.request.urlopen("http://127.0.0.1:17842" + edited_url).read()
    assert edited != wav
    # Valid IPA absent from the model's token inventory must produce a clear error.
    page.locator("#ipa").fill("ɬ")
    expect(page.locator("#validation")).to_have_text("Valid IPA")
    page.get_by_role("button", name="Synthesize", exact=True).click()
    expect(page.locator("#status")).to_contain_text("no token", timeout=15000)
    page.locator("#ipa").fill("a😀")
    expect(page.locator("#validation")).to_contain_text("unsupported IPA")
    expect(page.locator("#synthesize")).to_be_disabled()
    # Restore the example and ensure narrow viewport has no horizontal overflow.
    page.get_by_role("button", name="Generate IPA", exact=True).click()
    expect(page.locator("#validation")).to_have_text("Valid IPA", timeout=15000)
    page.set_viewport_size({"width": 390, "height": 844})
    assert page.evaluate("document.documentElement.scrollWidth <= window.innerWidth")
    page.screenshot(path=str(OUTPUT / "desktop-mobile.png"), full_page=True)
    assert not errors, errors
    (OUTPUT / "ui-verification.json").write_text(json.dumps({
        "generated_ipa": generated_ipa,
        "wav_bytes": len(wav),
        "playback_started": True,
        "stop_reset_playhead": True,
        "estimated_highlight_seen": True,
        "edited_ipa_changed_audio": True,
        "invalid_ipa_and_model_token_errors": True,
        "responsive_390px": True,
        "page_errors": errors,
    }, indent=2, ensure_ascii=False), encoding="utf-8")
    browser.close()
print("Real Kokoro UI smoke test passed")
