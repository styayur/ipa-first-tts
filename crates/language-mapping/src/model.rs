// SPDX-License-Identifier: MPL-2.0
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Representation {
    Orthography,
    Romanization,
    Ipa,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: String,
    pub organization: String,
    pub title: String,
    pub version: String,
    pub url: String,
    pub license: String,
    pub attribution: String,
    #[serde(rename = "use")]
    pub usage: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Language {
    pub id: String,
    pub tag: String,
    pub name: String,
    pub native_name: String,
    pub scripts: Vec<String>,
    pub region: Option<String>,
    pub coverage: String,
    pub sources: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Script {
    pub code: String,
    pub name: String,
    pub description: String,
    pub languages: Vec<String>,
    pub source: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Dictionary,
    P2g,
    Kana,
    Russian,
    RussianInverse,
    Han,
    Espeak,
    Ipa,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scheme {
    pub id: String,
    pub language: String,
    pub name: String,
    pub kind: Kind,
    pub input_type: Representation,
    pub output_type: Representation,
    pub input_scripts: Vec<String>,
    pub output_script: String,
    pub version: String,
    pub reversible: bool,
    pub context_dependent: bool,
    pub external_backend: bool,
    pub dictionary: bool,
    pub reverse_candidates: bool,
    pub source: String,
    pub limitations: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Matcher {
    Exact,
    Token,
    Characters,
    Context,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingRule {
    pub id: String,
    pub language: String,
    pub scheme: String,
    pub input: String,
    pub outputs: Vec<String>,
    pub matcher: Matcher,
    pub priority: u16,
    pub right_context: Option<String>,
    pub source: String,
    pub version: String,
    pub references: Vec<String>,
    pub limitations: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    pub schema_version: u32,
    pub languages: Vec<Language>,
    pub scripts: Vec<Script>,
    pub sources: Vec<Source>,
    pub schemes: Vec<Scheme>,
    pub rules: Vec<MappingRule>,
}
fn orthography() -> Representation {
    Representation::Orthography
}
fn ipa() -> Representation {
    Representation::Ipa
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingRequest {
    pub language: String,
    pub input: String,
    #[serde(default = "orthography")]
    pub input_type: Representation,
    #[serde(default = "ipa")]
    pub target: Representation,
    #[serde(default)]
    pub scheme: Option<String>,
    #[serde(default)]
    pub source_script: Option<String>,
    #[serde(default)]
    pub target_language: Option<String>,
    #[serde(default)]
    pub context: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct ConversionCapabilities {
    pub scheme: Scheme,
    pub available: bool,
    pub unavailable_reason: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct MappingEdge {
    pub scheme: String,
    pub language: String,
    pub version: String,
    pub input_type: Representation,
    pub output_type: Representation,
    pub input_script: String,
    pub output_script: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct RuleTrace {
    pub rule: String,
    pub source: String,
    pub source_version: String,
    pub normalized_scalar_range: [usize; 2],
    pub input: String,
    pub output: String,
    pub reference: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub text: String,
    pub rank: usize,
    pub ranking_basis: String,
    pub transcription: String,
    pub traces: Vec<RuleTrace>,
    pub ipa_grammar_accepted: Option<bool>,
    pub ipa_grammar_error: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Ambiguous,
    NeedsSelection,
    Unsupported,
}
#[derive(Debug, Clone, Serialize)]
pub struct MappingResult {
    pub schema_version: u32,
    pub status: Status,
    pub original_input: String,
    pub normalized_input: String,
    pub normalization: Vec<String>,
    pub language: String,
    pub input_type: Representation,
    pub output_type: Representation,
    pub scheme: Option<String>,
    pub path: Vec<MappingEdge>,
    pub candidates: Vec<Candidate>,
    pub sources: Vec<Source>,
    pub context: Option<String>,
    pub ambiguous: bool,
    pub determined: bool,
    pub truncated: bool,
    pub warnings: Vec<String>,
    pub unsupported_reason: Option<String>,
    pub selection_options: Vec<String>,
}
