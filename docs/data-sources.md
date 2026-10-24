<!-- SPDX-License-Identifier: CC-BY-4.0 -->
# Catalog provenance and reproducibility

Reviewed/acquired: 2026-10-24. Catalog schema: 1. First-party selection,
descriptions, aliases and combinations: Stya Yur Open Source Studio, CC BY 4.0.
Unicode-derived names/data retain Unicode-3.0. See
[the unchanged Unicode license](../licenses/upstream/Unicode-3.0.txt) and the
[catalog data license](../crates/symbol-catalog/data/LICENSE).

Sources referenced by each record via source IDs are carried with the embedded
data and exposed through Studio's `/api/catalog` metadata route:

- Unicode Character Database **17.0.0**, [UnicodeData.txt](https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt): codepoints and Unicode names. Unicode-3.0 applies to data; fonts and chart PDFs are not extracted.
- [Official IPA chart reference](https://www.internationalphoneticassociation.org/content/ipa-chart): consulted to author short phonetic labels. No chart arrangement, artwork, audio or font is copied. IPA charts have their own CC BY-SA terms; the interactive chart has different terms. No IPA endorsement is implied.
- [CLDR 47 language exemplar files](https://github.com/unicode-org/cldr/tree/release-47/common/main): language-script references for curated orthographic selections, not a wholesale import. Unicode-3.0 applies to upstream data.

`scripts/build_catalog.py` records the actual curated selection. Run it with the
downloaded official UnicodeData.txt; it does not fetch at runtime.
The downloaded UnicodeData.txt SHA-256 was
`2e1efc1dcb59c575eedf5ccae60f95229f706ee6d031835247d843c11d96470c`.
Vietnamese precomposed tone variants are constructed with NFC during **data authoring**.
The result is committed as exact UTF-8 JSON. Search normalization never mutates
these insertion strings. Uppercase mapping that expands to multiple letters
(such as ß → SS) is not treated as a new single character; ẞ is explicit.

There are **368 records**, including IPA consonants/vowels, common combining
diacritics, stress/length/tone, boundaries, affricates/diphthongs and curated
French/German/Spanish/Portuguese/Polish/Czech/Turkish/Vietnamese characters. The
supplementary phonetic extension U+1DF00 exercises surrogate pairs and is not a
claim of IPA Core/backend acceptance. This is not exhaustive language coverage.

Every record carries stable ID, exact text, kind/category/tags, registered
language IDs, script codes, name/aliases, codepoint list, description/examples,
source IDs and first-party license. Metadata resolves each source to URL,
license and access date. The Rust loader rejects duplicate IDs, empty/control
strings, inconsistent codepoints, unknown language/source references, invalid
script casing, malformed scalars and unsupported schemas. No GUI schema or
TTS-specific vocabulary is embedded in the catalog.

To extend: add a version-1 JSON file under `crates/symbol-catalog/data/`, register
any new language/source once across the files, include per-record provenance,
and run the catalog tests. build.rs discovers sorted JSON files at build time;
new languages do not require changes to search, UI language filters or TTS.
Do not add unknown-source symbol dumps or copy protected charts.
