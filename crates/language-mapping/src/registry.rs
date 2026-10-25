// SPDX-License-Identifier: MPL-2.0
use crate::model::*;
use std::collections::HashSet;
use unicode_normalization::UnicodeNormalization;

fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 160
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-._".contains(&c))
}
fn text(s: &str) -> bool {
    !s.is_empty() && !s.chars().any(char::is_control)
}
fn unique<'a>(
    values: impl Iterator<Item = &'a str>,
    kind: &str,
) -> Result<HashSet<&'a str>, String> {
    let mut set = HashSet::new();
    for id in values {
        if !identifier(id) || !set.insert(id) {
            return Err(format!("Invalid/duplicate {kind} ID: {id}"));
        }
    }
    Ok(set)
}
impl Registry {
    pub fn from_json(input: &str) -> Result<Self, String> {
        if input.len() > 2 * 1024 * 1024 {
            return Err("Mapping data exceeds 2 MiB".into());
        }
        let data: Self =
            serde_json::from_str(input).map_err(|e| format!("Invalid mapping data: {e}"))?;
        data.validate()?;
        Ok(data)
    }
    pub fn bundled() -> Result<Self, String> {
        Self::from_json(include_str!("../data/registry.json"))
    }
    pub fn language(&self, tag: &str) -> Option<&Language> {
        self.languages
            .iter()
            .find(|l| l.tag.eq_ignore_ascii_case(tag))
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.languages.is_empty() || self.rules.len() > 10000 {
            return Err("Unsupported mapping schema/size".into());
        }
        let langs = unique(self.languages.iter().map(|v| v.id.as_str()), "language")?;
        let scripts = unique(self.scripts.iter().map(|v| v.code.as_str()), "script")?;
        let sources = unique(self.sources.iter().map(|v| v.id.as_str()), "source")?;
        let schemes = unique(self.schemes.iter().map(|v| v.id.as_str()), "scheme")?;
        unique(self.rules.iter().map(|v| v.id.as_str()), "rule")?;
        let iana = include_str!("../data/upstream/iana-selected.txt");
        let iso = include_str!("../data/upstream/iso15924-selected.txt");
        for lang in &self.languages {
            if lang.id != lang.tag
                || !text(&lang.name)
                || !text(&lang.native_name)
                || lang.scripts.is_empty()
                || lang
                    .tag
                    .split('-')
                    .any(|part| !iana.lines().any(|l| l == format!("Subtag: {part}")))
                || lang
                    .region
                    .as_ref()
                    .is_some_and(|r| !iana.lines().any(|l| l == format!("Subtag: {r}")))
                || lang.sources.is_empty()
                || lang.sources.iter().any(|s| !sources.contains(s.as_str()))
                || lang.scripts.iter().any(|s| !scripts.contains(s.as_str()))
            {
                return Err(format!("Invalid/unregistered language: {}", lang.id));
            }
        }
        for script in &self.scripts {
            if script.code.len() != 4
                || !script.code.as_bytes()[0].is_ascii_uppercase()
                || !script.code.as_bytes()[1..]
                    .iter()
                    .all(u8::is_ascii_lowercase)
                || !iso
                    .lines()
                    .any(|l| l.starts_with(&(script.code.clone() + ";")))
                || !sources.contains(script.source.as_str())
                || script.languages.iter().any(|l| !langs.contains(l.as_str()))
            {
                return Err(format!("Invalid/unregistered script: {}", script.code));
            }
        }
        for source in &self.sources {
            if !source.url.starts_with("https://")
                || !text(&source.version)
                || !text(&source.license)
                || !text(&source.attribution)
            {
                return Err(format!("Invalid source: {}", source.id));
            }
        }
        for scheme in &self.schemes {
            let Some(language) = self.language(&scheme.language) else {
                return Err("Scheme language missing".into());
            };
            if !sources.contains(scheme.source.as_str())
                || !text(&scheme.version)
                || !text(&scheme.limitations)
                || scheme.input_scripts.is_empty()
                || scheme
                    .input_scripts
                    .iter()
                    .any(|s| !language.scripts.contains(s))
                || !scripts.contains(scheme.output_script.as_str())
                || !language.scripts.contains(&scheme.output_script)
                || scheme.external_backend != (scheme.kind == Kind::Espeak)
                || scheme.dictionary
                    != matches!(scheme.kind, Kind::Dictionary | Kind::P2g | Kind::Espeak)
                || scheme.reverse_candidates != matches!(scheme.kind, Kind::Dictionary | Kind::P2g)
                || scheme.reversible != matches!(scheme.kind, Kind::Russian | Kind::RussianInverse)
                || match scheme.kind {
                    Kind::Kana => scheme.language != "ja",
                    Kind::Russian | Kind::RussianInverse => scheme.language != "ru",
                    Kind::Han => scheme.language != "cmn-Hans",
                    Kind::Dictionary | Kind::P2g => scheme.language != "en-US",
                    Kind::Espeak => !matches!(scheme.language.as_str(), "en-US" | "fr" | "es"),
                    Kind::Ipa => false,
                }
                || (matches!(scheme.kind, Kind::Kana | Kind::Han | Kind::Russian)
                    && (scheme.input_type != Representation::Orthography
                        || scheme.output_type != Representation::Romanization))
                || (scheme.kind == Kind::P2g
                    && (scheme.input_type != Representation::Ipa
                        || scheme.output_type != Representation::Orthography))
                || (scheme.kind == Kind::RussianInverse
                    && (scheme.input_type != Representation::Romanization
                        || scheme.output_type != Representation::Orthography))
                || (matches!(scheme.kind, Kind::Espeak | Kind::Dictionary)
                    && (scheme.input_type != Representation::Orthography
                        || scheme.output_type != Representation::Ipa))
                || (scheme.kind == Kind::Ipa
                    && (scheme.input_type != Representation::Ipa
                        || scheme.output_type != Representation::Ipa))
            {
                return Err(format!("Invalid typed scheme: {}", scheme.id));
            }
        }
        let mut keys = HashSet::new();
        for rule in &self.rules {
            let Some(scheme) = self.schemes.iter().find(|s| s.id == rule.scheme) else {
                return Err("Unknown rule scheme".into());
            };
            let source = self.sources.iter().find(|s| s.id == rule.source);
            if !schemes.contains(rule.scheme.as_str())
                || rule.language != scheme.language
                || rule.source != scheme.source
                || source.is_none_or(|s| s.version != rule.version)
                || !text(&rule.input)
                || rule.input.len() > 65536
                || rule.input.nfc().collect::<String>() != rule.input
                || rule.outputs.is_empty()
                || rule.outputs.len() > 32
                || (scheme.kind != Kind::Dictionary && rule.outputs.len() != 1)
                || rule.outputs.iter().any(|s| !text(s) || s.len() > 512)
                || rule.references.len() != rule.outputs.len()
                || rule.references.iter().any(|s| !text(s))
                || (rule.matcher == Matcher::Context) != rule.right_context.is_some()
                || rule
                    .right_context
                    .as_ref()
                    .is_some_and(|s| s.chars().count() != 1)
                || !keys.insert((
                    rule.scheme.as_str(),
                    rule.input.as_str(),
                    rule.right_context.as_deref(),
                    rule.priority,
                ))
            {
                return Err(format!("Invalid/conflicting rule: {}", rule.id));
            }
            if scheme.output_type == Representation::Ipa {
                for output in &rule.outputs {
                    ipa_core::IpaSequence::parse(output, Some(&rule.language))
                        .map_err(|e| format!("Rule {} IPA: {e}", rule.id))?;
                }
            }
        }
        Ok(())
    }
}
