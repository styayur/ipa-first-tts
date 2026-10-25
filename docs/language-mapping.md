<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Local language mapping / 本地语言映射

Open **Mapping / 语言映射** in the native palette or use the **Language mapping**
section in Studio. Choose a language, a typed conversion and a named scheme.
The target language stays the same: this is not translation. The native form
combines direction and scheme in one selector. Unavailable optional edges are
omitted, with an explanation of the missing eSpeak tool.

Enter text, optionally context, then Convert (Ctrl+Enter in the input field).
Select a candidate before Copy, sending it into the palette composition or
sending eligible English IPA into Studio. Filling the composition does not
inject input. Recall the palette from the intended editor before Insert.
Copy explicitly replaces the clipboard. Opening/searching/mapping with embedded
data does not require a speech model, server or network.

The palette's visible **Exit / 退出** button ends the app; the title-bar × hides it.
The shortcut hides the palette together with its mapping/settings windows.
The title identifies the version and the tray tooltip reflects the saved shortcut.
Repeated launch restores the existing palette. If another executable location is
running, a bilingual notice identifies it so you can exit the old copy first.

The light ZIP contains `ipa-symbol-palette.exe` and `ipa-map.exe`; the console
program does not link Kokoro/sherpa. The full ZIP additionally provides eSpeak
and Studio. Set ESPEAK_NG only to a trusted local executable if using an override.
Runtime versions of arbitrary overrides are unverified; published voice data is
pinned to 1.52.0. The adapter bounds input/output and reaps timed-out children.

## Actual edges and limits

| Language profile | Embedded conversion | Optional G2P | Reverse candidates | Speech |
|---|---|---|---|---|
| en-US | 443 CMUdict spellings / 549 pronunciations → explicit IPA approximation | eSpeak en-us, explicitly chosen | Exact canonical IPA → selected dictionary spellings | Existing English adapters, separately checked |
| fr | Conservative IPA normalization only | eSpeak fr-fr | Unavailable | Unavailable |
| es | Conservative IPA normalization only | eSpeak es (Spain profile) | Unavailable | Unavailable |
| ja | Common kana → `cldr49-kana-subset` Latin | Unavailable; no Kanji G2P | Unavailable | Unavailable |
| cmn-Hans | 44,498 distinct NFC Han character defaults + three CLDR phrase contexts → tone-marked Latin | Unavailable; not Mandarin IPA | Unavailable | Unavailable |
| ru | Modern Russian alphabet → `cldr49-ru-subset` Latin | Unavailable | Exact romanization inverse, **not IPA P2G** | Unavailable |

Language registration, romanization, G2P, conservative IPA grammar and tested TTS
are distinct statuses. All profiles expose the same model-independent IPA
normalizer; this says nothing about their phoneme inventories. No new synthesis
language is enabled. Only English IPA can be sent directly to the speech editor;
all other results remain copyable and transferable into the native composition.

Japanese uses the named CLDR-derived bounded subset, not a generic “Hepburn”
promise. Digraphs, sokuon, n before vowel/y/n and the prolonged sound mark have
explicit policies. `コーヒー → kōhī`, `がっこう → gakkou`, `ぢづ → djidzu`.
Written おう stays `ou`; particle は stays `ha`. These are romanization choices,
not Japanese phonetic transcription. Halfwidth kana, iteration marks, Kanji and
unmapped characters fail explicitly.

Russian maps the 33-letter modern alphabet in both cases, preserving the special
uppercase sign underlines from CLDR. `Щука → Ŝuka`, not `Shchuka`; `Юлия → Ûliâ`.
NFC round trips are guaranteed only within that repertoire plus ASCII whitespace
and hyphen. This is not ISO 9 conformance and does not infer Russian pronunciation.

Han romanization spaces individual character readings. The CLDR contexts are
`藏文`, `重庆`, `秘鲁`. Isolated `重` exposes `zhòng` and contextual `chóng`;
context exactly `重庆` selects the latter. A separate full sentence context is
not analyzed. General polyphony, word segmentation, neutral tones and tone sandhi
are unresolved. Han outputs remain explicitly tentative, even with one candidate.
The cmn-Hans profile does not certify input as exclusively simplified characters.

CMUdict gives ARPAbet, not IPA. This project's adapter maps its phone symbols,
AH0→ə, ER0→ɚ and other vowels/rhotics explicitly. IY/UW are i/u without inferred
duration; stress precedes the encoded vowel, without guessed syllabification.
`read` retains both pronunciations. No language model uses a sentence to choose
one. P2G matches canonical IPA including stress and returns alphabetically ordered
finite spellings; `/sˈi/` finds sea/see. `/siː/` is not silently made equivalent.
Uniqueness is only within the selected dictionary; the original spelling is not
recovered by reversing phones into letters.

eSpeak outputs are broad rule-based hypotheses, not recorded narrow phonetics.
Unknown words, dialects and isolated heteronyms can be wrong. Output language
switch annotations fail instead of being silently removed. Latin-script checks
reject uncovered characters before invoking it. Engine and fixture metrics do
not claim universal language accuracy.

## CLI and typed results

```powershell
ipa-map languages
ipa-map schemes --language ja
ipa-map romanize --language ja --scheme cldr49-kana-subset --input コーヒー
ipa-map romanize --language cmn-Hans --scheme cldr49-han-readings --input 重 --context 重庆
ipa-map map --language en-US --scheme cmudict-us-sample --input read --target ipa
ipa-map ipa-to-text --language en-US --input /sˈi/
ipa-map map --language fr --scheme espeak-fr-fr --input eau --target ipa
ipa-map reverse-romanize --language ru --scheme cldr49-ru-inverse --input Ŝuka
ipa-map normalize-ipa --language fr --input "/gã t͜ʃ/"
```

`ipa-tts` accepts the same commands before creating its speech Engine. `map --json`
reads one strict request from stdin (16 KiB). JSON representation names are
`orthography`, `romanization`, `ipa`; requests name language, input, optional
scheme/source_script/target_language/context and the input/output types.
No language/dialect is inferred: `en` and `zh` ask for an explicit registered
profile. A unique compatible scheme may be selected; multiple possible schemes
return needs_selection. No failure substitutes a different scheme.

Results retain original input, normalized input, normalization policy, typed
path, rule traces, source versions/licenses, candidates, ambiguity, determined
state and warnings. Trace offsets count normalized Unicode scalars, not bytes
or original UTF-16 positions. Ranking is explicit priority/longest token or source
order/alphabetical spelling, not invented confidence. Input/context are bounded
to 4096 UTF-8 bytes, backend stdout to 64 KiB and stderr to 16 KiB with an 8 s
timeout; candidate sets are capped at 32. Unknown characters fail the whole
conversion, not a partial success. Context is preserved even when no sense model
uses it. CLI exit 2 denotes unsupported/selection/invalid requests.

## Reference tests and provenance

The 25 manually authored embedded fixtures cover en-US 6, Japanese 9, Russian 5
and Han 5 cases, plus negative/schema/context/round-trip tests. They check the
declared adapter and subset against pinned source entries, not a held-out
population. Six independent Wiktionary reference facts cover French 3 and
Spanish 3 cases. That metric compares NFC segments after removing stress and
syllable dots **only in the test metric**; it does not assess stress, allophony,
sentence accuracy or dialect coverage. Full result text preserves those marks.

Run `python scripts/test_mapping_cli.py --exe target/debug/ipa-map.exe
--with-espeak` for an observed per-language report in outputs/mapping-evaluation.json.
Real backend tests are explicit: `cargo test -p language-mapping --test real_g2p
--locked -- --ignored --test-threads=1`. Ordinary tests do not need eSpeak/models.

Data is pinned and embedded, with `sources.json` byte hashes and per-rule
provenance. Code is MPL, applications GPL, CLDR data Unicode-3.0, CMUdict BSD,
IANA records CC0; original descriptions/docs CC BY. See the data README,
research source register and THIRD_PARTY_NOTICES. Updating a dataset requires
independent review of sources, license, regeneration, fixtures and capability
claims; no runtime “latest rules” download exists.

## 简体中文使用要点

原生面板点“语言映射”，依次选择语言、转换方向/具名方案、源文字，输入文本后按
Ctrl+Enter。有多候选时需自行选择；“送入面板”只填写组合框，随后仍需明确复制或插入。
英语 IPA 可送至 Studio；其他语言不会因此获得语音合成能力。

内置数据可离线执行英语有限词典、日语假名、俄语字母和汉字默认读音转写，无需模型。
法语、西语和通用英语 G2P 需要完整包中的 eSpeak；缺少工具时不展示该转换选项。
“已登记语言”“有转写规则”“有 G2P”“IPA 可解析”“TTS 已验证”是不同状态。

`read` 保留多个读音；`重` 保留候选，语境 `重庆` 才选 `chóng`。汉字转写不是普通话 IPA，
不处理通用多音字、分词与变调。`コーヒー → kōhī` 是具名假名方案；`がっこう` 输出
`gakkou`，不擅自改为 `gakkō`。俄语 `Щука → Ŝuka`，该子集不宣称符合 ISO 9。

反向 IPA 查拼写仅限所附词典，保留重音；不会逐音素替换字母后声称还原了原文。
每个结果均显示来源、版本、规则、规范化过程和限制；排序不是概率。
输入失败不静默丢字，也不换方案。25 个内置参考用例及 6 个独立法/西语参考只衡量
声明子集的用例一致性，不是整种语言的准确率；新语种的 TTS 仍未开放。
