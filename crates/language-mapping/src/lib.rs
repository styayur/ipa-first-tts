// SPDX-License-Identifier: MPL-2.0
//! Typed, local conversion edges. No GUI, speech model or network dependency.
#![forbid(unsafe_code)]
pub mod cli;
pub mod espeak;
pub mod model;
mod registry;
mod romanization;
use ipa_core::IpaSequence;
pub use model::*;
pub use romanization::ALGORITHM_RULES;
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

pub const MAX_CANDIDATES: usize = 32;
pub trait G2pProvider {
    fn available(&self) -> bool;
    fn generate(&self, input: &str, language: &str) -> Result<String, String>;
    fn provenance_warning(&self) -> String;
}
pub struct Engine {
    registry: Registry,
    pub(crate) han: HashMap<char, usize>,
}
impl Engine {
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
    pub fn bundled() -> Result<Self, String> {
        Self::from_registry(Registry::bundled()?)
    }
    pub fn from_json(data: &str) -> Result<Self, String> {
        Self::from_registry(Registry::from_json(data)?)
    }
    pub fn from_registry(registry: Registry) -> Result<Self, String> {
        registry.validate()?;
        let mut han = HashMap::new();
        for (index, rule) in registry
            .rules
            .iter()
            .enumerate()
            .filter(|(_, r)| r.matcher == Matcher::Characters)
        {
            for c in rule.input.chars() {
                if han.insert(c, index).is_some() {
                    return Err(format!("Conflicting character rule: {c}"));
                }
            }
        }
        Ok(Self { registry, han })
    }
    pub fn capabilities(&self, provider: Option<&dyn G2pProvider>) -> Vec<ConversionCapabilities> {
        self.registry.schemes.iter().map(|s|{
            let available=!s.external_backend || provider.is_some_and(G2pProvider::available);
            ConversionCapabilities{scheme:s.clone(),available,unavailable_reason:(!available).then(||"Requires the optional local eSpeak NG executable and voice data; no automatic fallback.".into())}
        }).collect()
    }
    pub fn convert(
        &self,
        request: &MappingRequest,
        provider: Option<&dyn G2pProvider>,
    ) -> MappingResult {
        let normalized: String = request.input.nfc().collect();
        let mut result=MappingResult{schema_version:1,status:Status::Unsupported,original_input:request.input.clone(),normalized_input:normalized.clone(),normalization:vec!["NFC for matching; original_input retained; trace offsets count normalized Unicode scalars".into()],language:request.language.clone(),input_type:request.input_type,output_type:request.target,scheme:None,path:vec![],candidates:vec![],sources:vec![],context:request.context.clone(),ambiguous:false,determined:false,truncated:false,warnings:vec![],unsupported_reason:None,selection_options:vec![]};
        if request.input.trim().is_empty()
            || request.input.len() > 4096
            || request
                .context
                .as_ref()
                .is_some_and(|v| v.len() > 4096 || v.contains('\0'))
            || request
                .input
                .chars()
                .any(|c| c.is_control() && !c.is_whitespace())
        {
            result.unsupported_reason=Some("Input must be nonempty, <=4096 UTF-8 bytes, with no non-whitespace controls; context <=4096 bytes without NUL.".into());
            return result;
        }
        let Some(language) = self.registry.language(&request.language) else {
            result.unsupported_reason=Some(format!("Unknown/unspecified language '{}'. Select an exact registered language; English is en-US and Mandarin is cmn-Hans. No dialect or language is guessed.",request.language));
            result.selection_options = self
                .registry
                .languages
                .iter()
                .map(|l| l.tag.clone())
                .collect();
            return result;
        };
        result.language = language.tag.clone();
        if request
            .target_language
            .as_ref()
            .is_some_and(|s| !s.eq_ignore_ascii_case(&language.tag))
        {
            result.unsupported_reason =
                Some("Cross-language translation is not a mapping edge.".into());
            return result;
        }
        let options: Vec<_> = self
            .registry
            .schemes
            .iter()
            .filter(|s| {
                s.language == language.id
                    && s.input_type == request.input_type
                    && s.output_type == request.target
            })
            .collect();
        let scheme = if let Some(id) = &request.scheme {
            let Some(s) = options.iter().copied().find(|s| s.id == *id) else {
                result.unsupported_reason=Some("Unknown scheme or language/representation mismatch. The requested scheme is never substituted.".into());
                result.selection_options = options.iter().map(|s| s.id.clone()).collect();
                return result;
            };
            s
        } else if options.len() == 1 {
            options[0]
        } else {
            result.status = if options.is_empty() {
                Status::Unsupported
            } else {
                Status::NeedsSelection
            };
            result.selection_options = options.iter().map(|s| s.id.clone()).collect();
            result.unsupported_reason = Some(
                if options.is_empty() {
                    "No typed conversion path exists for this language and representation pair."
                } else {
                    "Select an explicit scheme; no automatic dictionary/backend fallback."
                }
                .into(),
            );
            return result;
        };
        let input_script = request
            .source_script
            .as_ref()
            .unwrap_or(&scheme.input_scripts[0]);
        if !scheme.input_scripts.contains(input_script) {
            result.unsupported_reason =
                Some("Source script does not match the selected scheme.".into());
            return result;
        }
        result.scheme = Some(scheme.id.clone());
        result.path.push(MappingEdge {
            scheme: scheme.id.clone(),
            language: language.tag.clone(),
            version: scheme.version.clone(),
            input_type: scheme.input_type,
            output_type: scheme.output_type,
            input_script: input_script.clone(),
            output_script: scheme.output_script.clone(),
        });
        result.sources = self
            .registry
            .sources
            .iter()
            .filter(|s| s.id == scheme.source || s.id == "project")
            .cloned()
            .collect();
        result.warnings.push(scheme.limitations.clone());
        if request.context.as_ref().is_some_and(|c| !c.is_empty()) && scheme.kind != Kind::Han {
            result.warnings.push("Context is preserved as metadata; this edge does not infer grammatical sense from a separate context field. For eSpeak supply the full sentence as input.".into());
        }
        let converted = match scheme.kind {
            Kind::Dictionary => self.dictionary(&normalized, &mut result),
            Kind::P2g => self.p2g(&normalized, &mut result),
            Kind::Kana | Kind::Russian | Kind::RussianInverse | Kind::Han => {
                self.romanize(scheme, &normalized, request, &mut result)
            }
            Kind::Ipa => match IpaSequence::parse(&normalized, Some(&language.tag)) {
                Ok(ipa) => {
                    result.normalized_input = ipa.canonical();
                    result.normalization.push("ipa-core: NFD, g→ɡ, tie aliases, whitespace/wrappers; no phonological interpretation".into());
                    result.candidates.push(candidate(
                        ipa.canonical(),
                        "normalized_notation",
                        vec![],
                    ));
                    Ok(())
                }
                Err(e) => Err(format!("Conservative IPA grammar: {e}")),
            },
            Kind::Espeak => {
                if let Some(backend) = provider.filter(|p| p.available()) {
                    match backend.generate(&normalized, &language.tag) {
                        Ok(raw) => {
                            result.warnings.push(backend.provenance_warning());
                            let output = IpaSequence::parse(&raw, Some(&language.tag))
                                .map(|p| p.canonical())
                                .unwrap_or(raw.clone());
                            let trace=RuleTrace{rule:scheme.id.clone(),source:scheme.source.clone(),source_version:scheme.version.clone(),normalized_scalar_range:[0,normalized.chars().count()],input:normalized.clone(),output:output.clone(),reference:format!("eSpeak NG voice for {}; rule/dictionary program, not per-word source alignment",language.tag)};
                            result.candidates.push(candidate(
                                output,
                                "rule_based_broad_phonetic",
                                vec![trace],
                            ));
                            result.ambiguous = true;
                            result.warnings.push("Single backend hypothesis is not a uniqueness or accuracy guarantee; no numerical confidence is available.".into());
                            Ok(())
                        }
                        Err(e) => Err(e),
                    }
                } else {
                    Err("Selected G2P backend is unavailable. Install the full package or configure ESPEAK_NG; no alternate scheme was used.".into())
                }
            }
        };
        if let Err(reason) = converted {
            result.candidates.clear();
            result.status = Status::Unsupported;
            result.unsupported_reason = Some(reason);
            return result;
        }
        if result.candidates.iter().any(|c| c.text.len() > 16384) {
            result.candidates.clear();
            result.status = Status::Unsupported;
            result.unsupported_reason = Some(
                "Output exceeds the 16 KiB limit; no partial result returned. Split the input."
                    .into(),
            );
            return result;
        }
        result.ambiguous |= result.candidates.len() > 1 || result.truncated;
        result.status = if result.ambiguous {
            Status::Ambiguous
        } else {
            Status::Ok
        };
        result.determined = !result.ambiguous;
        for (i, c) in result.candidates.iter_mut().enumerate() {
            c.rank = i + 1;
            if result.output_type == Representation::Ipa {
                match IpaSequence::parse(&c.text, Some(&language.tag)) {
                    Ok(_) => c.ipa_grammar_accepted = Some(true),
                    Err(e) => {
                        c.ipa_grammar_accepted = Some(false);
                        c.ipa_grammar_error = Some(e.to_string());
                        result.warnings.push(format!("Conservative IPA parser rejected candidate {}: {e}. Copy remains available; Studio transfer is unavailable.",i+1));
                    }
                }
            }
        }
        if result.output_type == Representation::Ipa {
            result.warnings.push("IPA grammar acceptance, language phonology and TTS backend vocabulary are separate. This engine does not enable speech languages.".into());
        }
        result
    }
    fn dictionary(&self, input: &str, result: &mut MappingResult) -> Result<(), String> {
        let key = input.trim().to_lowercase();
        result.normalization.push(
            "Exact CMU lookup trims outer whitespace and lowercases; no spelling/phoneme guessing"
                .into(),
        );
        let Some(rule) = self
            .registry
            .rules
            .iter()
            .find(|r| r.scheme == "cmudict-us-sample" && r.input == key)
        else {
            return Err("Word is outside the bundled selected CMU vocabulary; choose an explicit optional G2P scheme if needed.".into());
        };
        for (i, out) in rule.outputs.iter().enumerate() {
            result.candidates.push(candidate(
                out.clone(),
                "phonemic_approximation",
                vec![trace(rule, i, input, 0, input.chars().count())],
            ));
        }
        if rule.outputs.len() > 1 {
            result.warnings.push("Dictionary pronunciation variants are not ranked by word sense; no contextual disambiguation is claimed.".into());
        }
        Ok(())
    }
    fn p2g(&self, input: &str, result: &mut MappingResult) -> Result<(), String> {
        let normalized = IpaSequence::parse(input, Some("en-US"))
            .map_err(|e| format!("P2G IPA: {e}"))?
            .canonical();
        result.normalized_input = normalized.clone();
        result.normalization.push("Exact canonical IPA dictionary matching; stress retained; no approximate phone-to-letter replacement".into());
        for rule in self
            .registry
            .rules
            .iter()
            .filter(|r| r.scheme == "cmudict-us-sample")
        {
            for (i, out) in rule.outputs.iter().enumerate() {
                if IpaSequence::parse(out, Some("en-US"))
                    .expect("validated IPA rule")
                    .canonical()
                    == normalized
                {
                    if result.candidates.len() == MAX_CANDIDATES {
                        result.truncated = true;
                        break;
                    }
                    let mut t = trace(rule, i, input, 0, input.chars().count());
                    t.output = rule.input.clone();
                    result.candidates.push(candidate(
                        rule.input.clone(),
                        "dictionary_spelling",
                        vec![t],
                    ));
                    break;
                }
            }
        }
        if result.candidates.is_empty() {
            return Err("No exact spelling candidate in the selected dictionary. No approximate or original-text recovery is claimed.".into());
        }
        result.warnings.push("Uniqueness is only within the finite bundled dictionary, not all possible spellings. Alphabetical order is not probability.".into());
        Ok(())
    }
}
pub(crate) fn candidate(text: String, transcription: &str, traces: Vec<RuleTrace>) -> Candidate {
    Candidate{text,rank:0,ranking_basis:"Explicit rule priority/longest token; dictionary variants retain source order, reverse spellings alphabetical. No probability or numeric confidence.".into(),transcription:transcription.into(),traces,ipa_grammar_accepted:None,ipa_grammar_error:None}
}
pub(crate) fn trace(
    rule: &MappingRule,
    output: usize,
    input: &str,
    start: usize,
    end: usize,
) -> RuleTrace {
    RuleTrace {
        rule: rule.id.clone(),
        source: rule.source.clone(),
        source_version: rule.version.clone(),
        normalized_scalar_range: [start, end],
        input: input.into(),
        output: rule.outputs[output].clone(),
        reference: rule.references[output].clone(),
    }
}
