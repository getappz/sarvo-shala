use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseQuality {
    Correct,
    Partial,
    Wrong,
    Confused,
    NoResponse,
}

pub trait Classifier: Send + Sync {
    fn classify(&self, student_msg: &str, expected_keywords: &[String]) -> ResponseQuality;
}

pub struct HeuristicClassifier;

impl Classifier for HeuristicClassifier {
    fn classify(&self, student_msg: &str, expected_keywords: &[String]) -> ResponseQuality {
        let m = student_msg.trim().to_lowercase();

        if m.is_empty()
            || m.contains("idk")
            || m.contains("i don't know")
            || m.contains("pata nahi")
        {
            return ResponseQuality::NoResponse;
        }

        let words = m.split_whitespace().count();
        if m.contains('?') && words < 4 {
            return ResponseQuality::Confused;
        }

        if expected_keywords.is_empty() {
            return ResponseQuality::Wrong;
        }

        let present = expected_keywords
            .iter()
            .filter(|k| m.contains(&k.to_lowercase()))
            .count();
        let frac = present as f64 / expected_keywords.len() as f64;

        if frac >= 0.6 {
            ResponseQuality::Correct
        } else if frac >= 0.25 {
            ResponseQuality::Partial
        } else {
            ResponseQuality::Wrong
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kw() -> Vec<String> {
        ["pressure", "force", "area", "weight"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn classifier_thresholds() {
        let c = HeuristicClassifier;
        let k = kw();
        assert_eq!(
            c.classify("idk", &k),
            ResponseQuality::NoResponse,
            "idk -> NoResponse"
        );
        assert_eq!(
            c.classify("pata nahi", &k),
            ResponseQuality::NoResponse,
            "pata nahi -> NoResponse"
        );
        assert_eq!(
            c.classify("what?", &k),
            ResponseQuality::Confused,
            "short question -> Confused"
        );
        assert_eq!(
            c.classify("pressure force area and weight are all related", &k),
            ResponseQuality::Correct,
            "all keywords -> Correct"
        );
        assert_eq!(
            c.classify("pressure and force matter", &k),
            ResponseQuality::Partial,
            "half keywords -> Partial"
        );
        assert_eq!(
            c.classify("i like cats", &k),
            ResponseQuality::Wrong,
            "no keywords -> Wrong"
        );
    }
}
