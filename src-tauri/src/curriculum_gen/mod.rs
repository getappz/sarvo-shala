use harness::{Claude, Harness, RunEvent, RunMode, RunRequest, RunTuning};
use serde::Deserialize;
use std::time::SystemTime;

use crate::db::queries;
use crate::db::Db;
use crate::error::AppError;
use crate::models::Subject;

#[derive(Debug, Deserialize)]
struct GeneratedCurriculum {
    subject: GeneratedSubject,
    chapters: Vec<GeneratedChapter>,
}

#[derive(Debug, Deserialize)]
struct GeneratedSubject {
    name: String,
    #[serde(default)]
    name_hi: String,
}

#[derive(Debug, Deserialize)]
struct GeneratedChapter {
    title: String,
    #[serde(default)]
    title_hi: String,
    summary: String,
    concepts: Vec<GeneratedConcept>,
    #[serde(default)]
    prereqs: Vec<GeneratedPrereq>,
}

#[derive(Debug, Deserialize)]
struct GeneratedConcept {
    name: String,
    #[serde(default)]
    key_formula: String,
    #[serde(default)]
    common_error: String,
    #[serde(default = "default_exam_weight")]
    exam_weight: i64,
    #[serde(default)]
    snippets: Vec<GeneratedSnippet>,
    #[serde(default)]
    questions: Vec<GeneratedQuestion>,
}

fn default_exam_weight() -> i64 {
    3
}

#[derive(Debug, Deserialize)]
struct GeneratedSnippet {
    body: String,
}

#[derive(Debug, Deserialize)]
struct GeneratedQuestion {
    bloom_level: String,
    difficulty_layer: i64,
    prompt: String,
    #[serde(default)]
    trap: String,
    #[serde(default)]
    explanation: String,
    options: Vec<String>,
    correct: usize,
}

#[derive(Debug, Deserialize)]
struct GeneratedPrereq {
    concept: String,
    prereq: String,
}

fn build_prompt(grade: i64, name: &str, name_hi: &str, num_chapters: i64) -> String {
    format!(
        r#"You are a curriculum designer for grade {grade} {name}.
Generate a complete curriculum module with exactly {num_chapters} chapters.

Return ONLY valid JSON (no markdown, no code fences, no trailing whitespace) in this exact structure:

{{
  "subject": {{ "name": "{name}", "name_hi": "{name_hi}" }},
  "chapters": [
    {{
      "title": "Chapter title in English",
      "title_hi": "अध्याय का शीर्षक हिंदी में",
      "summary": "Brief chapter summary",
      "concepts": [
        {{
          "name": "Concept name",
          "key_formula": "Formula if any, or empty string",
          "common_error": "Common student error, or empty string",
          "exam_weight": 3,
          "snippets": [
            {{ "body": "Explanatory text paragraph for this concept." }},
            {{ "body": "Another paragraph building on the concept." }}
          ],
          "questions": [
            {{
              "bloom_level": "remember",
              "difficulty_layer": 1,
              "prompt": "Question text?",
              "trap": "Common wrong intuition students have",
              "explanation": "Why the correct answer is right",
              "options": ["Correct answer", "Distractor A", "Distractor B", "Distractor C"],
              "correct": 0
            }},
            {{
              "bloom_level": "understand",
              "difficulty_layer": 2,
              "prompt": "Another question?",
              "trap": "",
              "explanation": "",
              "options": ["Option A", "Option B", "Option C", "Option D"],
              "correct": 0
            }},
            {{
              "bloom_level": "apply",
              "difficulty_layer": 3,
              "prompt": "An application question?",
              "trap": "",
              "explanation": "",
              "options": ["Option A", "Option B", "Option C", "Option D"],
              "correct": 0
            }}
          ]
        }}
      ]
    }}
  ]
}}

Constraints:
- bloom_level must be one of: remember, understand, apply, analyze, evaluate
- difficulty_layer must be 1, 2, or 3
- Each question must have exactly 4 options; correct is 0-based index into options
- Each concept should have 2-3 snippets and 3 questions (one per difficulty layer)
- Generate {num_chapters} chapters, each with 3 concepts
- key_formula can be empty string if no formula applies
- common_error can be empty string if none
- exam_weight is 1-5
- All field values must be non-null strings or numbers"#,
        grade = grade,
        name = name,
        name_hi = name_hi,
        num_chapters = num_chapters,
    )
}

fn extract_json(text: &str) -> Result<String, AppError> {
    let trimmed = text.trim();
    if let Some(start) = trimmed.find("```") {
        let after_fence = &trimmed[start + 3..];
        let langless = after_fence.strip_prefix("json").or_else(|| after_fence.strip_prefix("JSON")).unwrap_or(after_fence);
        if let Some(end) = langless.rfind("```") {
            return Ok(langless[..end].trim().to_string());
        }
    }
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Ok(trimmed.to_string());
    }
    Err(AppError::Engine(
        "no valid JSON found in response".into(),
    ))
}

pub fn generate_curriculum(
    db: &Db,
    grade: i64,
    name: &str,
    name_hi: &str,
    num_chapters: i64,
) -> Result<Subject, AppError> {
    let prompt = build_prompt(grade, name, name_hi, num_chapters);

    let claude = Claude::new();
    let run_id = format!(
        "curriculum-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let (_handle, rx) = claude
        .run_channel(RunRequest {
            run_id,
            prompt,
            cwd: None,
            mode: RunMode::Ask,
            tuning: RunTuning::default(),
            resume: None,
        })?;

    let mut full_text = String::new();
    for ev in rx {
        match ev {
            RunEvent::Text { delta, .. } => full_text.push_str(&delta),
            RunEvent::Exited { .. } => break,
            RunEvent::Error { message, .. } => {
                return Err(AppError::Engine(format!("Claude error: {message}")))
            }
            _ => {}
        }
    }

    let json_str = extract_json(&full_text)?;
    let generated: GeneratedCurriculum = serde_json::from_str(&json_str)
        .map_err(|e| AppError::Engine(format!("JSON parse error: {e}")))?;

    insert_curriculum(db, grade, name, name_hi, &generated)
}

fn insert_curriculum(
    db: &Db,
    grade: i64,
    name: &str,
    name_hi: &str,
    gen: &GeneratedCurriculum,
) -> Result<Subject, AppError> {
    let name = if gen.subject.name.is_empty() {
        name
    } else {
        &gen.subject.name
    };
    let name_hi = if gen.subject.name_hi.is_empty() {
        name_hi
    } else {
        &gen.subject.name_hi
    };

    // Everything below runs inside one rusqlite transaction so a failure
    // partway through (bad LLM output, a constraint violation) leaves no
    // orphaned subject/chapter/concept rows behind.
    let subject_id = db.with(|c| {
        let tx = c.transaction()?;

        let subject_id = queries::insert_subject(&tx, grade, name, name_hi)?;

        for (ch_ord, ch) in gen.chapters.iter().enumerate() {
            let chapter_id = queries::insert_chapter(
                &tx,
                subject_id,
                ch_ord as i64 + 1,
                &ch.title,
                &ch.title_hi,
                &ch.summary,
            )?;

            let mut concept_name_to_id: std::collections::HashMap<&str, i64> =
                std::collections::HashMap::new();

            for (cn_ord, cn) in ch.concepts.iter().enumerate() {
                let concept_id = queries::insert_concept(
                    &tx,
                    chapter_id,
                    cn_ord as i64 + 1,
                    &cn.name,
                    &cn.key_formula,
                    &cn.common_error,
                    cn.exam_weight,
                )?;

                concept_name_to_id.insert(&cn.name, concept_id);

                for (sn_ord, sn) in cn.snippets.iter().enumerate() {
                    queries::insert_chapter_text(
                        &tx,
                        chapter_id,
                        concept_id,
                        sn_ord as i64 + 1,
                        &sn.body,
                    )?;
                }

                for q in &cn.questions {
                    let qid = queries::insert_question(
                        &tx,
                        concept_id,
                        &q.bloom_level,
                        q.difficulty_layer,
                        &q.prompt,
                        &q.trap,
                        &q.explanation,
                    )?;

                    for (opt_ord, opt) in q.options.iter().enumerate() {
                        queries::insert_question_option(
                            &tx,
                            qid,
                            opt_ord as i64 + 1,
                            opt,
                            opt_ord == q.correct,
                        )?;
                    }
                }
            }

            for prereq in &ch.prereqs {
                if let (Some(&cid), Some(&pid)) = (
                    concept_name_to_id.get(prereq.concept.as_str()),
                    concept_name_to_id.get(prereq.prereq.as_str()),
                ) {
                    queries::insert_concept_prereq(&tx, cid, pid)?;
                }
            }
        }

        tx.commit()?;

        Ok(subject_id)
    })?;

    Ok(Subject {
        id: subject_id,
        grade,
        name: name.to_string(),
        name_hi: name_hi.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_json_strips_code_fence() {
        let input = "Here is the JSON:\n```json\n{\"subject\":{\"name\":\"test\"}}\n```\n";
        assert_eq!(extract_json(input).unwrap(), "{\"subject\":{\"name\":\"test\"}}");
    }

    #[test]
    fn extract_json_no_fence() {
        let input = "{\"key\": \"value\"}";
        assert_eq!(extract_json(input).unwrap(), "{\"key\": \"value\"}");
    }

    #[test]
    fn extract_json_no_fence_generic() {
        let input = "```\n{\"a\":1}\n```";
        assert_eq!(extract_json(input).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn extract_json_no_valid_json_returns_err() {
        let input = "this is not json at all";
        assert!(extract_json(input).is_err());
    }

    #[test]
    fn build_prompt_contains_grade_and_subject_name() {
        let prompt = build_prompt(9, "Physics", "भौतिकी", 3);
        assert!(prompt.contains("grade 9"));
        assert!(prompt.contains("Physics"));
        assert!(prompt.contains("भौतिकी"));
    }

    #[test]
    fn insert_curriculum_rolls_back_entirely_on_a_bad_question() {
        let mut p = std::env::temp_dir();
        p.push(format!("shala_curriculum_gen_{}.db", std::process::id()));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();

        let gen = GeneratedCurriculum {
            subject: GeneratedSubject {
                name: "Physics".into(),
                name_hi: String::new(),
            },
            chapters: vec![GeneratedChapter {
                title: "Motion".into(),
                title_hi: String::new(),
                summary: String::new(),
                concepts: vec![GeneratedConcept {
                    name: "Velocity".into(),
                    key_formula: String::new(),
                    common_error: String::new(),
                    exam_weight: 3,
                    snippets: vec![GeneratedSnippet {
                        body: "text".into(),
                    }],
                    // Not a valid `bloom_level` per the CHECK constraint on
                    // `questions` -- fails partway through the transaction.
                    questions: vec![GeneratedQuestion {
                        bloom_level: "not_a_real_level".into(),
                        difficulty_layer: 1,
                        prompt: "?".into(),
                        trap: String::new(),
                        explanation: String::new(),
                        options: vec!["a".into(), "b".into(), "c".into(), "d".into()],
                        correct: 0,
                    }],
                }],
                prereqs: vec![],
            }],
        };

        let result = insert_curriculum(&db, 9, "Physics", "", &gen);
        assert!(result.is_err(), "invalid bloom_level should fail the insert");

        let subject_count: i64 = db
            .with(|c| c.query_row("SELECT count(*) FROM subjects", [], |r| r.get(0)))
            .unwrap();
        assert_eq!(
            subject_count, 0,
            "a failed insert must not leave a partial subject behind"
        );

        let _ = std::fs::remove_file(&p);
    }
}
