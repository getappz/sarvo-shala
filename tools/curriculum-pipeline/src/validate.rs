use crate::schema::SubjectFile;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub concept_id: String,
    pub message: String,
}

pub fn validate(subject: &SubjectFile) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut all_concept_ids: HashSet<&str> = HashSet::new();

    for chapter in &subject.chapters {
        for concept in &chapter.concepts {
            all_concept_ids.insert(&concept.id);

            if concept.name.is_empty() {
                issues.push(ValidationIssue {
                    concept_id: concept.id.clone(),
                    message: "concept name is empty".to_string(),
                });
            }
            if concept.exam_weight < 1 || concept.exam_weight > 5 {
                issues.push(ValidationIssue {
                    concept_id: concept.id.clone(),
                    message: format!("exam_weight {} out of range [1..5]", concept.exam_weight),
                });
            }

            for (qi, q) in concept.questions.iter().enumerate() {
                if q.prompt.is_empty() {
                    issues.push(ValidationIssue {
                        concept_id: concept.id.clone(),
                        message: format!("question {} prompt is empty", qi + 1),
                    });
                }
                if q.options.is_empty() {
                    issues.push(ValidationIssue {
                        concept_id: concept.id.clone(),
                        message: format!("question {} has no options", qi + 1),
                    });
                }
                let correct_count = q.options.iter().filter(|o| o.is_correct).count();
                if correct_count != 1 {
                    issues.push(ValidationIssue {
                        concept_id: concept.id.clone(),
                        message: format!(
                            "question {} has {} correct options (must be exactly 1)",
                            qi + 1,
                            correct_count
                        ),
                    });
                }
                for (oi, o) in q.options.iter().enumerate() {
                    if o.body.is_empty() {
                        issues.push(ValidationIssue {
                            concept_id: concept.id.clone(),
                            message: format!("question {} option {} body is empty", qi + 1, oi + 1),
                        });
                    }
                }

                match q.bloom_level.as_str() {
                    "remember" | "understand" | "apply" | "analyze" | "evaluate" => {}
                    other => issues.push(ValidationIssue {
                        concept_id: concept.id.clone(),
                        message: format!("invalid bloom_level: {other}"),
                    }),
                }

                match q.difficulty_layer {
                    1..=3 => {}
                    d => issues.push(ValidationIssue {
                        concept_id: concept.id.clone(),
                        message: format!("invalid difficulty_layer: {d}"),
                    }),
                }
            }
        }
    }

    for chapter in &subject.chapters {
        for concept in &chapter.concepts {
            for prereq in &concept.prereq_ids {
                if !all_concept_ids.contains(prereq.as_str()) {
                    issues.push(ValidationIssue {
                        concept_id: concept.id.clone(),
                        message: format!("prereq_id \"{prereq}\" does not match any concept ID"),
                    });
                }
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::*;

    fn valid_concept(id: &str) -> ConceptAuthoring {
        ConceptAuthoring {
            id: id.to_string(),
            name: "Test Concept".to_string(),
            key_formula: String::new(),
            common_error: String::new(),
            exam_weight: 3,
            prereq_ids: vec![],
            text: vec!["Some explanatory text.".to_string()],
            questions: vec![QuestionAuthoring {
                bloom_level: "remember".to_string(),
                difficulty_layer: 1,
                prompt: "What is the test?".to_string(),
                explanation: "A test explanation.".to_string(),
                options: vec![
                    OptionAuthoring {
                        body: "Correct answer".to_string(),
                        is_correct: true,
                    },
                    OptionAuthoring {
                        body: "Wrong answer".to_string(),
                        is_correct: false,
                    },
                ],
            }],
            source_authority: "Test Authority".to_string(),
            generation_method: GenerationMethod::Original,
            review_status: ReviewStatus::Reviewed,
        }
    }

    fn valid_subject() -> SubjectFile {
        SubjectFile {
            subject: "Science".to_string(),
            subject_hi: "विज्ञान".to_string(),
            grade: 8,
            chapters: vec![ChapterAuthoring {
                id: "science8.test".to_string(),
                title: "Test Chapter".to_string(),
                title_hi: String::new(),
                summary: "A test chapter.".to_string(),
                concepts: vec![valid_concept("science8.test.concept1")],
            }],
        }
    }

    #[test]
    fn valid_subject_passes() {
        let subject = valid_subject();
        let issues = validate(&subject);
        assert!(issues.is_empty(), "expected no issues, got: {issues:?}");
    }

    #[test]
    fn detects_missing_correct_option() {
        let mut subject = valid_subject();
        subject.chapters[0].concepts[0].questions[0].options[0].is_correct = false;
        let issues = validate(&subject);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("0 correct options"));
    }

    #[test]
    fn detects_two_correct_options() {
        let mut subject = valid_subject();
        subject.chapters[0].concepts[0].questions[0].options[1].is_correct = true;
        let issues = validate(&subject);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("2 correct options"));
    }

    #[test]
    fn detects_broken_preref_reference() {
        let mut subject = valid_subject();
        subject.chapters[0].concepts[0]
            .prereq_ids
            .push("nonexistent.concept".to_string());
        let issues = validate(&subject);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("nonexistent.concept"));
    }

    #[test]
    fn detects_empty_name() {
        let mut subject = valid_subject();
        subject.chapters[0].concepts[0].name = "".to_string();
        let issues = validate(&subject);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("name is empty"));
    }
}
