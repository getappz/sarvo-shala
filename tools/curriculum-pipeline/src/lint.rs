use crate::schema::SubjectFile;

const FILLER_PATTERNS: &[&str] = &[
    "core principles of",
    "real-world applications of",
    "fundamental concepts of",
    "key aspects of",
    "important role of",
    "plays a crucial role",
    "plays a vital role",
    "is essential for understanding",
    "forms the basis of",
    "is one of the most important",
    "in this chapter we will learn",
    "we will explore",
    "let us understand",
    "let us learn",
    "can be defined as",
    "refers to the process",
];

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub concept_id: String,
    pub field: String,
    pub message: String,
}

pub fn lint(subject: &SubjectFile) -> Vec<LintIssue> {
    let mut issues = Vec::new();

    for chapter in &subject.chapters {
        for concept in &chapter.concepts {
            check_text(&mut issues, &concept.id, "name", &concept.name);
            check_text(&mut issues, &concept.id, "summary", &chapter.summary);
            for (ti, t) in concept.text.iter().enumerate() {
                check_text(&mut issues, &concept.id, &format!("text[{}]", ti + 1), t);
            }
        }
    }

    issues
}

fn check_text(issues: &mut Vec<LintIssue>, concept_id: &str, field: &str, text: &str) {
    let lower = text.to_lowercase();
    for &pattern in FILLER_PATTERNS {
        if lower.contains(pattern) {
            issues.push(LintIssue {
                concept_id: concept_id.to_string(),
                field: field.to_string(),
                message: format!("possible generic filler phrase: \"{pattern}\""),
            });
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::*;

    fn clean_concept(id: &str) -> ConceptAuthoring {
        ConceptAuthoring {
            id: id.to_string(),
            name: "Force basics".to_string(),
            key_formula: String::new(),
            common_error: String::new(),
            exam_weight: 3,
            prereq_ids: vec![],
            text: vec!["A force is a push or pull that acts on an object.".to_string()],
            questions: vec![],
            source_authority: "Test".to_string(),
            generation_method: GenerationMethod::Original,
            review_status: ReviewStatus::Reviewed,
        }
    }

    #[test]
    fn clean_content_passes() {
        let subject = SubjectFile {
            subject: "Science".to_string(),
            subject_hi: String::new(),
            grade: 8,
            chapters: vec![ChapterAuthoring {
                id: "science8.test".to_string(),
                title: "Test".to_string(),
                title_hi: String::new(),
                summary: "How forces affect motion.".to_string(),
                concepts: vec![clean_concept("science8.test.c1")],
            }],
        };
        let issues = lint(&subject);
        assert!(issues.is_empty(), "expected no issues, got: {issues:?}");
    }

    #[test]
    fn detects_core_principles_of() {
        let mut concept = clean_concept("science8.test.c1");
        concept.text = vec!["The core principles of motion are important.".to_string()];
        let subject = SubjectFile {
            subject: "Science".to_string(),
            subject_hi: String::new(),
            grade: 8,
            chapters: vec![ChapterAuthoring {
                id: "science8.test".to_string(),
                title: "Test".to_string(),
                title_hi: String::new(),
                summary: String::new(),
                concepts: vec![concept],
            }],
        };
        let issues = lint(&subject);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("core principles of"));
    }
}
