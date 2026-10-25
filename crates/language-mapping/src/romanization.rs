// SPDX-License-Identifier: MPL-2.0
use crate::*;

pub const ALGORITHM_RULES: &[&str] = &[
    "project:separator-v1",
    "project:kana-fold-v1",
    "project:kana-gemination-v1",
    "project:kana-n-v1",
    "project:kana-length-v1",
];
fn algorithm(id: &str, input: &str, output: &str, start: usize, end: usize) -> RuleTrace {
    RuleTrace{rule:id.into(),source:"project".into(),source_version:"1".into(),normalized_scalar_range:[start,end],input:input.into(),output:output.into(),reference:"crates/language-mapping/src/romanization.rs; bounded policies derived from pinned CLDR rules, not a general ICU interpreter".into()}
}
fn prefix<'a>(
    rest: &str,
    rules: &[&'a MappingRule],
    inverse: bool,
) -> Option<(&'a MappingRule, usize)> {
    rules
        .iter()
        .filter_map(|r| {
            let token = if inverse { &r.outputs[0] } else { &r.input };
            rest.starts_with(token)
                .then_some((*r, token.chars().count()))
        })
        .max_by_key(|(r, count)| (*count, r.priority))
}
impl Engine {
    pub(crate) fn romanize(
        &self,
        scheme: &Scheme,
        input: &str,
        request: &MappingRequest,
        result: &mut MappingResult,
    ) -> Result<(), String> {
        if scheme.kind == Kind::Han {
            return self.han(input, request, result);
        }
        let kana = scheme.kind == Kind::Kana;
        let inverse = scheme.kind == Kind::RussianInverse;
        if kana
            && request.source_script.as_deref().is_some_and(|script| {
                (script == "Hira"
                    && input
                        .chars()
                        .any(|c| ('\u{30a1}'..='\u{30fa}').contains(&c)))
                    || (script == "Kana"
                        && input
                            .chars()
                            .any(|c| ('\u{3041}'..='\u{3096}').contains(&c)))
            })
        {
            return Err("Input contains kana outside the explicitly selected source script; choose Hrkt for mixed syllabaries.".into());
        }
        let folded: String = input
            .chars()
            .map(|c| {
                if kana && ('\u{3041}'..='\u{3096}').contains(&c) {
                    char::from_u32(c as u32 + 0x60).expect("kana offset")
                } else {
                    c
                }
            })
            .collect();
        if folded != input {
            result.normalization.push("Explicit Hiragana→Katakana matching fold (+0x60 for the selected block); no Kanji reading or NFKC compatibility folding".into());
            result.normalized_input = folded.clone();
        }
        let rules: Vec<_> = self
            .registry
            .rules
            .iter()
            .filter(|r| {
                r.scheme
                    == if inverse {
                        "cldr49-ru-subset"
                    } else {
                        &scheme.id
                    }
            })
            .collect();
        let chars: Vec<_> = folded.chars().collect();
        let mut offset = 0;
        let mut out = String::new();
        let mut traces = Vec::new();
        while offset < chars.len() {
            let c = chars[offset];
            let rest: String = chars[offset..].iter().collect();
            if c.is_ascii_whitespace() || c == '-' || (kana && "。、".contains(c)) {
                let value = match c {
                    '。' => ".".into(),
                    '、' => ",".into(),
                    _ => c.to_string(),
                };
                out.push_str(&value);
                traces.push(algorithm(
                    "project:separator-v1",
                    &c.to_string(),
                    &value,
                    offset,
                    offset + 1,
                ));
                offset += 1;
                continue;
            }
            if kana && c == 'ッ' {
                let next: String = chars[offset + 1..].iter().collect();
                let Some((rule, _)) = prefix(&next, &rules, false) else {
                    return Err(format!("Sokuon requires a supported following consonant at normalized scalar {offset}"));
                };
                let next = &rule.outputs[0];
                let consonant = if next.starts_with("ch") {
                    't'
                } else {
                    next.chars().next().expect("nonempty rule")
                };
                if !"bcdfghjkmnprstvwxyz".contains(consonant) {
                    return Err(format!(
                        "Unsupported sokuon context at normalized scalar {offset}"
                    ));
                }
                let value = consonant.to_string();
                out.push(consonant);
                traces.push(algorithm(
                    "project:kana-gemination-v1",
                    "ッ",
                    &value,
                    offset,
                    offset + 1,
                ));
                offset += 1;
                continue;
            }
            if kana && c == 'ン' {
                let quote = chars
                    .get(offset + 1)
                    .is_some_and(|c| "アイウエオナニヌネノヤユヨン".contains(*c));
                let value = if quote { "n'" } else { "n" };
                out.push_str(value);
                traces.push(algorithm(
                    "project:kana-n-v1",
                    "ン",
                    value,
                    offset,
                    offset + 1,
                ));
                offset += 1;
                continue;
            }
            if kana && c == 'ー' {
                if !out.chars().last().is_some_and(|c| "aeiou".contains(c)) {
                    return Err(format!("Prolonged mark requires an immediately preceding supported vowel at scalar {offset}"));
                }
                out.push('\u{0304}');
                traces.push(algorithm(
                    "project:kana-length-v1",
                    "ー",
                    "\u{0304}",
                    offset,
                    offset + 1,
                ));
                offset += 1;
                continue;
            }
            let Some((rule, count)) = prefix(&rest, &rules, inverse) else {
                return Err(format!("Unmapped character U+{:04X} at normalized scalar {offset}; no partial conversion returned",c as u32));
            };
            let token: String = chars[offset..offset + count].iter().collect();
            let value = if inverse {
                &rule.input
            } else {
                &rule.outputs[0]
            };
            out.push_str(value);
            let mut t = trace(rule, 0, &token, offset, offset + count);
            if inverse {
                t.output = rule.input.clone();
                t.reference = format!("inverse of {} (NFC bounded alphabet only)", rule.id);
            }
            traces.push(t);
            offset += count;
        }
        if out.trim().is_empty() {
            return Err("No convertible symbols in input".into());
        }
        result
            .candidates
            .push(candidate(out.nfc().collect(), "romanization", traces));
        Ok(())
    }
    fn han(
        &self,
        input: &str,
        request: &MappingRequest,
        result: &mut MappingResult,
    ) -> Result<(), String> {
        let chars: Vec<_> = input.chars().collect();
        let mut states = vec![(String::new(), Vec::new())];
        for (offset, c) in chars.iter().copied().enumerate() {
            if c.is_ascii_whitespace() || "。、.,-".contains(c) {
                let value = match c {
                    '。' => ".".into(),
                    '、' => ",".into(),
                    _ => c.to_string(),
                };
                for (out, traces) in &mut states {
                    out.push_str(&value);
                    traces.push(algorithm(
                        "project:separator-v1",
                        &c.to_string(),
                        &value,
                        offset,
                        offset + 1,
                    ));
                }
                continue;
            }
            let Some(index) = self.han.get(&c) else {
                return Err(format!("No bundled Han reading for U+{:04X} at normalized scalar {offset}; no text dropped",c as u32));
            };
            let default = &self.registry.rules[*index];
            let context = self
                .registry
                .rules
                .iter()
                .find(|r| r.matcher == Matcher::Context && r.input == c.to_string());
            let mut options = vec![default];
            if let Some(rule) = context {
                let right = rule.right_context.as_ref().expect("validated context");
                let phrase = format!("{c}{right}");
                if chars.get(offset + 1).map(char::to_string).as_ref() == Some(right)
                    || (chars.len() == 1 && request.context.as_deref() == Some(&phrase))
                {
                    options = vec![rule];
                } else if chars.len() == 1
                    && request.context.as_ref().is_none_or(|s| s.is_empty())
                    && rule.outputs != default.outputs
                {
                    options.push(rule);
                    result.warnings.push(format!("Isolated {c}: {} is supported in context {phrase}; the default reading is not unique.",rule.outputs[0]));
                }
            }
            let mut next = Vec::new();
            for (out, traces) in &states {
                for rule in &options {
                    if next.len() == MAX_CANDIDATES {
                        result.truncated = true;
                        break;
                    }
                    let mut out = out.clone();
                    let mut traces = traces.clone();
                    if out.chars().last().is_some_and(|c| {
                        c.is_alphabetic() || unicode_normalization::char::is_combining_mark(c)
                    }) {
                        out.push(' ');
                    }
                    out.push_str(&rule.outputs[0]);
                    traces.push(trace(rule, 0, &c.to_string(), offset, offset + 1));
                    next.push((out, traces));
                }
            }
            states = next;
        }
        if states.first().is_none_or(|s| s.0.trim().is_empty()) {
            return Err("No Han characters converted".into());
        }
        result.ambiguous = true;
        result.warnings.push("Han results use default character readings plus only three published phrase contexts. They are hypotheses, not unique word pronunciations; tone sandhi, full polyphony and word segmentation are not implemented. Hans profile does not certify input as exclusively simplified characters.".into());
        for (out, traces) in states {
            result
                .candidates
                .push(candidate(out.nfc().collect(), "romanization", traces));
        }
        Ok(())
    }
}
