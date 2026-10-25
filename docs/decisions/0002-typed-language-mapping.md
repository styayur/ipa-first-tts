<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# ADR 0002: typed local mapping edges, explicit uncertainty

2026-10-25. Accepted after repository and upstream data/license review.

Add one reusable MPL `language-mapping` crate with logical registry, typed
mapping model, romanization, dictionary P2G, IPA normalization and optional
eSpeak process adapter modules. Splitting these into many crates would add
dependency wiring without isolating additional runtimes. It depends on
ipa-core, serde/json and unicode-normalization, never GUI or speech models.
Expose the same engine through Studio, CLI and a native palette mapping window.
An additional small `ipa-map` executable lets the light package run mapping
commands without linking the speech application or its native DLLs.

Nodes include language, script, representation and scheme, rather than merely
"Latin text". An explicit compatible capability selects one edge. Ambiguous
language/scheme requests return selection requirements; no automatic path treats
romanization as IPA or changes schemes after failure. Initially supported edges
are individually exposed; nonexistent multi-step paths are unsupported.

CLDR 49 data is pinned to 96e54b273d2da4ec6fbad350ba19ff23fa24a588.
Implement named, bounded CLDR-compatible Kana and Russian alphabet subsets,
and Han default-reading romanization with the published phrase contexts.
These are not claims of complete Hepburn, ISO 9 conformance or Mandarin G2P.
Data-derived rules retain Unicode-3.0; the restricted LDML specification text is
not redistributed. Preserve raw source files, original notices and hashes.

Pin a selected CMUdict vocabulary and all its pronunciation variants for a
model-free American English dictionary edge and exact dictionary P2G candidates.
ARPAbet conversion is an explicit project adapter with its own documented vowel,
rhotic and stress policy, not a claim that ARPAbet is IPA. No confidence
percentages are invented. Homophones and heteronyms remain visible; P2G is a
bounded dictionary lookup, never a phoneme-to-letter inverse.

General English/French/Spanish G2P uses a separately selected eSpeak NG 1.52.0
adapter. Full packages already contain this GPL tool; light packages expose only
available edges and need no external tool for embedded mappings. The process
adapter uses bounded stdin/output, a timeout, literal arguments and no shell.
Reported backend output is a rule-based broad transcription, not an observed
phonetic measurement. Non-English G2P does not enable non-English synthesis.

IpaSequence remains the independent conservative parser. Normalize via it;
report grammar acceptance separately from phonology and TTS coverage. CLTS and
PHOIBLE inform this boundary and remain research references, not G2P dictionaries.
Epitran informs rule organization; no Python runtime or unreviewed tables enter
the product. ISO and ALA-LC publications are reference-only and not bundled.

All data and results retain source/version/license, rule traces, original input,
normalization, contextual limits and ambiguity. Unmapped characters fail the
whole conversion with offsets; they are not dropped or passed through as if
converted. Candidate growth is bounded and truncation is explicit. Imported
data and independent golden fixtures are validated in CI. Measurements describe
only the defined fixture set, never whole-language accuracy.
