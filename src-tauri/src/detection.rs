use crate::{
    config::Config,
    layout::{Engine, Layout},
};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Deserialize)]
pub struct Profile {
    pub language: String,
    script: String,
    characters: String,
    words: Vec<String>,
    bigrams: Vec<String>,
    trigrams: Vec<String>,
}
pub fn profiles() -> &'static [Profile] {
    static P: OnceLock<Vec<Profile>> = OnceLock::new();
    P.get_or_init(|| {
        [
            include_str!("../../language_profiles/en.json"),
            include_str!("../../language_profiles/fa.json"),
            include_str!("../../language_profiles/ar.json"),
            include_str!("../../language_profiles/ru.json"),
            include_str!("../../language_profiles/de.json"),
            include_str!("../../language_profiles/fr.json"),
            include_str!("../../language_profiles/es.json"),
        ]
        .iter()
        .filter_map(|s| serde_json::from_str(s).ok())
        .collect()
    })
}
fn word(s: &str) -> String {
    s.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase()
}
pub fn recognized(s: &str) -> bool {
    let w = word(s);
    !w.is_empty() && profiles().iter().any(|p| p.words.contains(&w))
}
fn script(c: char) -> &'static str {
    match c as u32 {
        0x0600..=0x06ff | 0x0750..=0x077f | 0xfb50..=0xfdff | 0xfe70..=0xfeff => "arabic",
        0x0400..=0x052f => "cyrillic",
        0x0041..=0x024f => "latin",
        _ => "other",
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Evidence {
    pub score: f64,
    pub lexical: f64,
}
pub fn score(text: &str, language: &str) -> Evidence {
    let Some(p) = profiles().iter().find(|p| p.language == language) else {
        return Evidence {
            score: 0.2,
            lexical: 0.0,
        };
    };
    let lower = text.to_lowercase();
    let chars: Vec<_> = lower.chars().filter(|c| c.is_alphabetic()).collect();
    if chars.is_empty() {
        return Evidence {
            score: 0.0,
            lexical: 0.0,
        };
    }
    let valid =
        chars.iter().filter(|c| script(**c) == p.script).count() as f64 / chars.len() as f64;
    let freq =
        chars.iter().filter(|c| p.characters.contains(**c)).count() as f64 / chars.len() as f64;
    let words: Vec<_> = lower
        .split_whitespace()
        .map(word)
        .filter(|w| !w.is_empty())
        .collect();
    let hits = words.iter().filter(|w| p.words.contains(w)).count();
    let lexical = hits as f64 / words.len().max(1) as f64;
    let big = p
        .bigrams
        .iter()
        .filter(|b| lower.contains(b.as_str()))
        .count() as f64
        / p.bigrams.len().max(1) as f64;
    let tri = p
        .trigrams
        .iter()
        .filter(|b| lower.contains(b.as_str()))
        .count() as f64
        / p.trigrams.len().max(1) as f64;
    let punctuation = lower
        .chars()
        .filter(|c| !c.is_alphanumeric() && !c.is_whitespace())
        .count() as f64
        / lower.chars().count().max(1) as f64;
    let mut value = if lexical >= 0.5 {
        0.65 + 0.33 * lexical
    } else {
        0.32 * valid + 0.15 * freq + 0.12 * big + 0.16 * tri + 0.15 * lexical
    };
    value *= valid;
    value -= punctuation.min(0.3) * 0.12;
    if chars.len() < 3 {
        value = value.min(0.55);
    }
    Evidence {
        score: value.clamp(0.0, 0.99),
        lexical,
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub source: String,
    pub target: String,
    pub language: String,
    pub name: String,
    pub text: String,
    pub confidence: f64,
    pub unmapped: usize,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub candidates: Vec<Candidate>,
    pub action: String,
    pub original_confidence: f64,
}

// A configured cross-script pair is a keyboard instruction, not a dictionary lookup.
// Use it for unknown/unfinished text while retaining protection for recognizable correct words.
fn configured_pair(
    engine: &mut Engine,
    text: &str,
    layouts: &[Layout],
    config: &Config,
) -> Result<Option<Candidate>, String> {
    if text
        .split_whitespace()
        .any(|chunk| word(chunk).chars().count() >= 3 && recognized(chunk))
    {
        return Ok(None);
    }
    let letters: String = text.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() {
        return Ok(None);
    }
    for (source, target) in [
        (&config.source, &config.target),
        (&config.target, &config.source),
    ] {
        let (Some(from), Some(to)) = (
            layouts.iter().find(|l| &l.id == source),
            layouts.iter().find(|l| &l.id == target),
        ) else {
            continue;
        };
        let (Some(a), Some(b)) = (
            profiles().iter().find(|p| p.language == from.language),
            profiles().iter().find(|p| p.language == to.language),
        ) else {
            continue;
        };
        if a.script == b.script || !letters.chars().all(|c| script(c) == a.script) {
            continue;
        }
        if engine.convert(&letters, source, target)?.unmapped != 0 {
            continue;
        }
        let converted = engine.convert(text, source, target)?;
        if converted.text == text {
            continue;
        }
        let confidence = score(&converted.text, &to.language).score;
        return Ok(Some(Candidate {
            source: source.clone(),
            target: target.clone(),
            language: to.language.clone(),
            name: to.name.clone(),
            text: converted.text,
            confidence,
            unmapped: converted.unmapped,
            reason: "layout_pair".into(),
        }));
    }
    Ok(None)
}

pub fn analyze(
    engine: &mut Engine,
    text: &str,
    layouts: &[Layout],
    config: &Config,
    current: &str,
) -> Result<Analysis, String> {
    if text.trim().is_empty() {
        return Err("empty_text".into());
    }
    if text.len() > 1_000_000 {
        return Err("text_too_large".into());
    }
    let original = profiles()
        .iter()
        .map(|p| score(text, &p.language).score)
        .fold(0.0f64, f64::max);
    let mut candidates = Vec::new();
    let preferred: Vec<_> = layouts
        .iter()
        .filter(|l| config.preferred_layouts.is_empty() || config.preferred_layouts.contains(&l.id))
        .collect();
    for from in &preferred {
        for to in &preferred {
            if from.id == to.id {
                continue;
            }
            let mut result = String::new();
            let mut unmapped = 0;
            let mut changed = false;
            for chunk in text.split_inclusive(char::is_whitespace) {
                if recognized(chunk) || !chunk.chars().any(char::is_alphabetic) {
                    result.push_str(chunk);
                    continue;
                }
                let c = engine.convert(chunk, &from.id, &to.id)?;
                unmapped += c.unmapped;
                changed |= c.text != chunk;
                result.push_str(&c.text);
            }
            if !changed
                || result == text
                || candidates
                    .iter()
                    .any(|c: &Candidate| c.text == result && c.language == to.language)
            {
                continue;
            }
            let evidence = score(&result, &to.language);
            let count = text.chars().filter(|c| !c.is_whitespace()).count().max(1);
            let confidence =
                (evidence.score - (unmapped as f64 / count as f64) * 0.3).clamp(0.0, 0.99);
            candidates.push(Candidate {
                source: from.id.clone(),
                target: to.id.clone(),
                language: to.language.clone(),
                name: to.name.clone(),
                text: result,
                confidence,
                unmapped,
                reason: if evidence.lexical >= 0.5 {
                    "word_evidence"
                } else {
                    "pattern_evidence"
                }
                .into(),
            });
        }
    }
    candidates.sort_by(|a, b| {
        b.confidence
            .total_cmp(&a.confidence)
            .then_with(|| (b.source == current).cmp(&(a.source == current)))
            .then_with(|| a.target.cmp(&b.target))
    });
    let action = if original >= 0.86
        && candidates
            .first()
            .is_none_or(|c| c.confidence < original + 0.08)
    {
        "unchanged"
    } else if let Some(best) = candidates.first() {
        let next = candidates
            .get(1)
            .map_or(original, |c| c.confidence.max(original));
        if best.confidence >= config.threshold
            && best.confidence - next >= 0.12
            && best.reason == "word_evidence"
            && best.unmapped == 0
        {
            "apply"
        } else {
            "choose"
        }
    } else {
        "unchanged"
    };
    let mut action = action;
    if action != "apply" {
        if let Some(candidate) = configured_pair(engine, text, layouts, config)? {
            candidates.retain(|c| c.text != candidate.text);
            candidates.insert(0, candidate);
            action = "apply";
        }
    }
    candidates.truncate(8);
    Ok(Analysis {
        candidates,
        action: action.into(),
        original_confidence: original,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn correct_language_scores_above_gibberish() {
        assert!(score("سلام دنیا", "fa").score > score("سغحل دنطا", "fa").score);
        assert!(score("hello world", "en").score > 0.9);
        assert!(score("привет мир", "ru").score > 0.9);
    }
    #[test]
    fn short_inputs_never_claim_high_confidence() {
        assert!(score("a", "en").score <= 0.55);
        assert_eq!(score("123!", "en").score, 0.0);
    }
    #[test]
    fn profiles_are_all_valid() {
        assert_eq!(profiles().len(), 7);
        assert!(recognized("hello!"));
        assert!(recognized("سلام"));
        assert!(!recognized("sghl"));
    }
    #[test]
    fn valid_text_and_mixed_text_are_preserved() {
        let mut e = Engine::default();
        let ls = crate::layout::installed();
        let c = Config::default();
        let a = analyze(&mut e, "hello world", &ls, &c, "").expect("analysis");
        assert_eq!(a.action, "unchanged");
        let a = analyze(&mut e, "سلام my friend 123!", &ls, &c, "").expect("analysis");
        assert_eq!(a.action, "unchanged");
    }
}
