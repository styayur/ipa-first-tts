// SPDX-License-Identifier: MPL-2.0
//! Offline symbol strings; intentionally independent of IPA parsing and speech.
#![forbid(unsafe_code)]
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use unicode_normalization::UnicodeNormalization;
include!(concat!(env!("OUT_DIR"), "/catalog.rs"));

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Language {
    pub id: String,
    pub name: String,
    pub native_name: String,
    pub aliases: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub url: String,
    pub license: String,
    pub accessed: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Phoneme,
    Diacritic,
    Stress,
    Tone,
    Length,
    Orthography,
    Punctuation,
    Combination,
    Boundary,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Symbol {
    pub id: String,
    pub text: String,
    pub kind: SymbolKind,
    pub category: String,
    pub tags: Vec<String>,
    pub languages: Vec<String>,
    pub scripts: Vec<String>,
    pub name: String,
    pub aliases: Vec<String>,
    pub codepoints: Vec<String>,
    pub description: String,
    pub examples: Vec<String>,
    pub sources: Vec<String>,
    pub license: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DataFile {
    pub schema_version: u32,
    pub languages: Vec<Language>,
    pub sources: Vec<Source>,
    pub symbols: Vec<Symbol>,
}
pub struct Catalog {
    pub symbols: Vec<Symbol>,
    pub languages: Vec<Language>,
    pub sources: Vec<Source>,
    index: Vec<String>,
    ids: HashMap<String, usize>,
}
pub fn codepoints(text: &str) -> Vec<String> {
    text.chars()
        .map(|c| format!("U+{:04X}", c as u32))
        .collect()
}
pub fn parse_codepoint(value: &str) -> Result<char, String> {
    let hex = value
        .strip_prefix("U+")
        .ok_or("Expected U+ hexadecimal scalar")?;
    if !(4..=6).contains(&hex.len()) || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("Expected 4–6 hexadecimal digits".into());
    }
    u32::from_str_radix(hex, 16)
        .ok()
        .and_then(char::from_u32)
        .ok_or_else(|| "Invalid Unicode scalar".into())
}
fn fold(value: &str) -> String {
    value
        .nfd()
        .flat_map(char::to_lowercase)
        .collect::<String>()
        .replace('ß', "ss")
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 80
        && value
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
impl Catalog {
    pub fn bundled() -> Result<Self, String> {
        Self::from_jsons(DATA)
    }
    pub fn from_jsons(files: &[&str]) -> Result<Self, String> {
        let mut symbols = Vec::new();
        let mut languages = Vec::new();
        let mut sources = Vec::new();
        for file in files {
            let data: DataFile =
                serde_json::from_str(file).map_err(|e| format!("Catalog JSON: {e}"))?;
            if data.schema_version != 1 {
                return Err("Unsupported catalog schema version".into());
            }
            symbols.extend(data.symbols);
            languages.extend(data.languages);
            sources.extend(data.sources);
        }
        let mut language_ids = HashSet::new();
        for language in &languages {
            if !identifier(&language.id)
                || !(2..=3).contains(&language.id.len())
                || !language.id.bytes().all(|c| c.is_ascii_lowercase())
                || !language_ids.insert(language.id.clone())
                || language.name.is_empty()
            {
                return Err(format!("Invalid/duplicate language: {}", language.id));
            }
        }
        let mut source_ids = HashSet::new();
        for source in &sources {
            if !identifier(&source.id)
                || !source_ids.insert(source.id.clone())
                || !source.url.starts_with("https://")
                || source.license.is_empty()
                || source.accessed.len() != 10
            {
                return Err(format!("Invalid/duplicate source: {}", source.id));
            }
        }
        let mut ids = HashMap::new();
        let mut index = Vec::new();
        for (i, s) in symbols.iter().enumerate() {
            let valid = identifier(&s.id)
                && ids.insert(s.id.clone(), i).is_none()
                && !s.text.is_empty()
                && s.text.len() <= 128
                && !s.text.chars().any(char::is_control)
                && s.codepoints == codepoints(&s.text)
                && s.codepoints.iter().all(|c| parse_codepoint(c).is_ok())
                && !s.languages.is_empty()
                && s.languages.iter().all(|l| language_ids.contains(l))
                && !s.scripts.is_empty()
                && s.scripts.iter().all(|s| {
                    s.len() == 4
                        && s.as_bytes()[0].is_ascii_uppercase()
                        && s.as_bytes()[1..].iter().all(|b| b.is_ascii_lowercase())
                })
                && identifier(&s.category)
                && !s.name.is_empty()
                && !s.description.is_empty()
                && !s.sources.is_empty()
                && s.sources.iter().all(|id| source_ids.contains(id))
                && s.license == "CC-BY-4.0";
            if !valid {
                return Err(format!("Invalid/duplicate symbol: {}", s.id));
            }
            let names = s
                .languages
                .iter()
                .filter_map(|id| languages.iter().find(|l| &l.id == id))
                .map(|l| {
                    format!(
                        "{} {} {} {}",
                        l.id,
                        l.name,
                        l.native_name,
                        l.aliases.join(" ")
                    )
                })
                .collect::<Vec<_>>()
                .join(" ");
            index.push(fold(&format!(
                "{} {} {} {} {} {} {} {} {} {}",
                s.text,
                s.name,
                s.aliases.join(" "),
                s.category,
                s.tags.join(" "),
                names,
                s.codepoints.join(" "),
                s.description,
                s.examples.join(" "),
                s.scripts.join(" ")
            )));
        }
        if symbols.is_empty() {
            return Err("Catalog is empty".into());
        }
        Ok(Self {
            symbols,
            languages,
            sources,
            index,
            ids,
        })
    }
    pub fn get(&self, id: &str) -> Option<&Symbol> {
        self.ids.get(id).map(|i| &self.symbols[*i])
    }
    /// Search normalization affects only the index, never the returned insertion text.
    pub fn search(
        &self,
        query: &str,
        language: Option<&str>,
        category: Option<&str>,
    ) -> Vec<&Symbol> {
        let folded = fold(query.trim());
        let language_query = self
            .languages
            .iter()
            .find(|l| {
                !folded.is_empty()
                    && std::iter::once(l.id.as_str())
                        .chain([l.name.as_str(), l.native_name.as_str()])
                        .chain(l.aliases.iter().map(String::as_str))
                        .any(|name| fold(name) == folded)
            })
            .map(|l| l.id.as_str());
        let words: Vec<_> = folded.split_whitespace().collect();
        let mut found: Vec<_> = self
            .symbols
            .iter()
            .enumerate()
            .filter(|(i, s)| {
                language.is_none_or(|l| s.languages.iter().any(|id| id == l))
                    && language_query.is_none_or(|l| s.languages.iter().any(|id| id == l))
                    && category.is_none_or(|c| s.category == c)
                    && words.iter().all(|w| self.index[*i].contains(w))
            })
            .map(|(_, s)| s)
            .collect();
        let canonical_query: String = query.trim().nfd().collect();
        found.sort_by_key(|s| {
            (
                u8::from(s.text != query.trim()),
                u8::from(s.text.nfd().collect::<String>() != canonical_query),
                u8::from(fold(&s.text) != folded),
            )
        });
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn mutated(edit: impl FnOnce(&mut DataFile)) -> Result<Catalog, String> {
        let c = Catalog::bundled().unwrap();
        let mut data = DataFile {
            schema_version: 1,
            symbols: c.symbols,
            languages: c.languages,
            sources: c.sources,
        };
        edit(&mut data);
        Catalog::from_jsons(&[&serde_json::to_string(&data).unwrap()])
    }
    #[test]
    fn catalog_schema_and_provenance() {
        let c = Catalog::bundled().unwrap();
        assert!(c.symbols.len() >= 200);
        for s in &c.symbols {
            assert_eq!(codepoints(&s.text), s.codepoints);
        }
        assert!(mutated(|d| d.symbols.push(d.symbols[0].clone())).is_err());
        assert!(mutated(|d| d.symbols[0].text.clear()).is_err());
        assert!(mutated(|d| d.symbols[0].languages = vec!["EN_invalid!".into()]).is_err());
        assert!(mutated(|d| d.symbols[0].codepoints = vec!["U+D800".into()]).is_err());
        assert!(mutated(|d| d.schema_version = 99).is_err());
        assert!(mutated(|d| d.symbols[0].sources = vec!["unknown".into()]).is_err());
    }
    #[test]
    fn scalars_are_neither_utf8_bytes_nor_utf16_units() {
        assert_eq!(parse_codepoint("U+0252").unwrap(), 'ɒ');
        assert_eq!(parse_codepoint("U+1DF00").unwrap(), '\u{1df00}');
        for bad in ["0252", "U+DFFF", "U+110000", "U+ZZZZ", "U+0000000"] {
            assert!(parse_codepoint(bad).is_err());
        }
        let text = "t͡ʃ\u{1df00}";
        assert_eq!(text.chars().count(), 4);
        assert_eq!(text.encode_utf16().count(), 5);
        assert_eq!(text.len(), 9);
        assert_eq!(codepoints(text), ["U+0074", "U+0361", "U+0283", "U+1DF00"]);
    }
    #[test]
    fn search_and_output_remain_separate() {
        let c = Catalog::bundled().unwrap();
        assert!(c
            .search("de", None, None)
            .iter()
            .all(|s| s.languages.iter().any(|l| l == "de")));
        for query in [
            "ɲ", "ʁ", "œ", "U+0252", "ESH", "French", "fr", "vowels", "t͡ʃ", "tsh", "Deutsch",
            "法语",
        ] {
            assert!(!c.search(query, None, None).is_empty(), "{query}");
        }
        assert_eq!(c.search("e\u{301}", Some("fr"), None)[0].text, "é");
        assert_eq!(c.search("t͡ʃ", None, None)[0].text, "t͡ʃ");
        assert!(c.search("no-such-symbol-xyz", None, None).is_empty());
        assert!(c
            .search("", Some("de"), Some("orthography"))
            .iter()
            .any(|s| s.text == "ß"));
        assert!(c.search("", Some("de"), Some("vowels")).is_empty());
        // Catalog entries are not promises of IPA grammar or backend acceptance.
        assert!(!c.search("¿", Some("es"), None).is_empty());
    }
}
