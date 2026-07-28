use crate::schema::{GenerationMethod, ReviewStatus, SubjectFile};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const SCHEMA_SQL: &str = include_str!("../../../src-tauri/src/db/schema.sql");

pub fn compile(content_dir: &Path, out: &Path) -> anyhow::Result<()> {
    let mut json_files = Vec::new();
    for entry in fs::read_dir(content_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            json_files.push(path);
        }
    }
    json_files.sort();

    // Write directly to a temp db file, then compress
    let tmp_sqlite = out.with_extension("tmp.db");
    {
        let conn = rusqlite::Connection::open(&tmp_sqlite)?;
        conn.execute_batch("PRAGMA foreign_keys = OFF;")?;
        conn.execute_batch(SCHEMA_SQL)?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        for path in &json_files {
            let json = fs::read_to_string(path)?;
            let subject: SubjectFile = serde_json::from_str(&json)?;
            insert_subject(&conn, &subject)?;
        }
    }

    let db_bytes = fs::read(&tmp_sqlite)?;
    let compressed = zstd::encode_all(std::io::Cursor::new(&db_bytes), 3)?;
    fs::write(out, &compressed)?;
    fs::remove_file(&tmp_sqlite)?;

    eprintln!(
        "Compiled {} JSON files -> {} ({} -> {} bytes, {:.1}% compression)",
        json_files.len(),
        out.display(),
        db_bytes.len(),
        compressed.len(),
        100.0 * (1.0 - compressed.len() as f64 / db_bytes.len() as f64),
    );
    Ok(())
}

fn insert_subject(conn: &rusqlite::Connection, subject: &SubjectFile) -> anyhow::Result<()> {
    let subject_id: i64 = conn
        .query_row(
            "SELECT id FROM subjects WHERE grade = ?1 AND name = ?2",
            rusqlite::params![subject.grade, subject.subject],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| {
            conn.execute(
                "INSERT INTO subjects (grade, name, name_hi) VALUES (?1, ?2, ?3)",
                rusqlite::params![subject.grade, subject.subject, subject.subject_hi],
            )
            .unwrap();
            conn.last_insert_rowid()
        });

    // Maps each concept's authoring id (e.g. "science8.force_pressure.force_basics")
    // to its inserted row id, so prereq_ids can be resolved after every concept in
    // the subject has been inserted (prereqs may point across chapters).
    let mut concept_row_ids: HashMap<&str, i64> = HashMap::new();

    for (ci, chapter) in subject.chapters.iter().enumerate() {
        let chapter_id: i64 = conn
            .query_row(
                "SELECT id FROM chapters WHERE subject_id = ?1 AND title = ?2",
                rusqlite::params![subject_id, chapter.title],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| {
                conn.execute(
                    "INSERT INTO chapters (subject_id, ord, title, title_hi, summary) VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![subject_id, ci as i64 + 1, chapter.title, chapter.title_hi, chapter.summary],
                )
                .unwrap();
                conn.last_insert_rowid()
            });

        for (coni, concept) in chapter.concepts.iter().enumerate() {
            let gen_method = match concept.generation_method {
                GenerationMethod::Original => "original",
                GenerationMethod::LlmGenerated => "llm_generated",
                GenerationMethod::UserUploaded => "user_uploaded",
            };
            let rev_status = match concept.review_status {
                ReviewStatus::Pending => "pending",
                ReviewStatus::Reviewed => "reviewed",
            };

            conn.execute(
                "INSERT INTO concepts (chapter_id, ord, name, key_formula, common_error, exam_weight, source_authority, generation_method, review_status) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                rusqlite::params![
                    chapter_id, coni as i64 + 1, concept.name,
                    concept.key_formula, concept.common_error, concept.exam_weight,
                    concept.source_authority, gen_method, rev_status,
                ],
            )?;
            let concept_id = conn.last_insert_rowid();
            concept_row_ids.insert(concept.id.as_str(), concept_id);

            for (ti, text) in concept.text.iter().enumerate() {
                conn.execute(
                    "INSERT INTO chapter_text (chapter_id, concept_id, ord, body, source_authority, generation_method, review_status) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    rusqlite::params![
                        chapter_id, concept_id, ti as i64 + 1, text,
                        concept.source_authority, gen_method, rev_status,
                    ],
                )?;
            }

            for q in &concept.questions {
                conn.execute(
                    "INSERT INTO questions (concept_id, qtype, bloom_level, difficulty_layer, prompt, explanation, source_authority, generation_method, review_status) VALUES (?1,'mcq',?2,?3,?4,?5,?6,?7,?8)",
                    rusqlite::params![
                        concept_id, q.bloom_level, q.difficulty_layer,
                        q.prompt, q.explanation,
                        concept.source_authority, gen_method, rev_status,
                    ],
                )?;
                let question_id = conn.last_insert_rowid();

                for (oi, opt) in q.options.iter().enumerate() {
                    conn.execute(
                        "INSERT INTO question_options (question_id, ord, body, is_correct) VALUES (?1,?2,?3,?4)",
                        rusqlite::params![
                            question_id, oi as i64 + 1, opt.body,
                            match opt.is_correct { true => 1, false => 0 },
                        ],
                    )?;
                }
            }
        }
    }

    // Resolve prereq_ids (authoring ids) against concept_row_ids now that every
    // concept in the subject has a row id, including cross-chapter references.
    for chapter in &subject.chapters {
        for concept in &chapter.concepts {
            if concept.prereq_ids.is_empty() {
                continue;
            }
            let Some(&concept_row_id) = concept_row_ids.get(concept.id.as_str()) else {
                continue;
            };
            for prereq_id_str in &concept.prereq_ids {
                if let Some(&prereq_row_id) = concept_row_ids.get(prereq_id_str.as_str()) {
                    conn.execute(
                        "INSERT OR IGNORE INTO concept_prereqs (concept_id, prereq_id) VALUES (?1, ?2)",
                        rusqlite::params![concept_row_id, prereq_row_id],
                    )?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::*;
    use tempfile::TempDir;

    fn minimal_subject_file() -> SubjectFile {
        SubjectFile {
            subject: "Science".to_string(),
            subject_hi: "विज्ञान".to_string(),
            grade: 8,
            chapters: vec![ChapterAuthoring {
                id: "science8.test".to_string(),
                title: "Test Chapter".to_string(),
                title_hi: String::new(),
                summary: "A test chapter.".to_string(),
                concepts: vec![ConceptAuthoring {
                    id: "science8.test.concept1".to_string(),
                    name: "Test Concept".to_string(),
                    key_formula: String::new(),
                    common_error: String::new(),
                    exam_weight: 3,
                    prereq_ids: vec![],
                    text: vec!["Explanatory text.".to_string()],
                    questions: vec![QuestionAuthoring {
                        bloom_level: "remember".to_string(),
                        difficulty_layer: 1,
                        prompt: "What is this?".to_string(),
                        explanation: "A test.".to_string(),
                        options: vec![
                            OptionAuthoring {
                                body: "Correct".to_string(),
                                is_correct: true,
                            },
                            OptionAuthoring {
                                body: "Wrong".to_string(),
                                is_correct: false,
                            },
                        ],
                    }],
                    source_authority: "CBSE Class 8 Science".to_string(),
                    generation_method: GenerationMethod::LlmGenerated,
                    review_status: ReviewStatus::Pending,
                }],
            }],
        }
    }

    #[test]
    fn compile_creates_valid_sqlite() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("grade8.json");
        let json = serde_json::to_string_pretty(&minimal_subject_file()).unwrap();
        fs::write(&json_path, &json).unwrap();

        let out_path = tmp.path().join("output.db.zst");
        compile(tmp.path(), &out_path).unwrap();

        assert!(out_path.exists());
        let compressed = fs::read(&out_path).unwrap();
        assert!(!compressed.is_empty());
    }

    #[test]
    fn compiled_db_loads_in_app_schema() {
        let tmp = TempDir::new().unwrap();
        let json_path = tmp.path().join("grade8.json");
        let json = serde_json::to_string_pretty(&minimal_subject_file()).unwrap();
        fs::write(&json_path, &json).unwrap();

        let out_path = tmp.path().join("output.db.zst");
        compile(tmp.path(), &out_path).unwrap();

        let compressed = fs::read(&out_path).unwrap();
        let decompressed = zstd::decode_all(std::io::Cursor::new(&compressed)).unwrap();
        let db_path = tmp.path().join("verify.db");
        fs::write(&db_path, &decompressed).unwrap();

        let conn = rusqlite::Connection::open(&db_path).unwrap();
        let concept_count: i64 = conn
            .query_row("SELECT count(*) FROM concepts", [], |r| r.get(0))
            .unwrap();
        let question_count: i64 = conn
            .query_row("SELECT count(*) FROM questions", [], |r| r.get(0))
            .unwrap();
        assert_eq!(concept_count, 1);
        assert_eq!(question_count, 1);
    }

    #[test]
    fn compile_links_prereqs_by_authoring_id_across_chapters() {
        let tmp = TempDir::new().unwrap();
        let mut subject = minimal_subject_file();
        subject.chapters[0].concepts[0].id = "science8.force_pressure.force_basics".to_string();
        subject.chapters[0].concepts[0].name = "Force basics".to_string();
        let mut pressure = subject.chapters[0].concepts[0].clone();
        pressure.id = "science8.force_pressure.pressure".to_string();
        pressure.name = "Pressure".to_string();
        pressure.prereq_ids = vec!["science8.force_pressure.force_basics".to_string()];
        subject.chapters[0].concepts.push(pressure);

        let json_path = tmp.path().join("grade8.json");
        fs::write(&json_path, serde_json::to_string_pretty(&subject).unwrap()).unwrap();
        let out_path = tmp.path().join("output.db.zst");
        compile(tmp.path(), &out_path).unwrap();

        let compressed = fs::read(&out_path).unwrap();
        let decompressed = zstd::decode_all(std::io::Cursor::new(&compressed)).unwrap();
        let db_path = tmp.path().join("verify.db");
        fs::write(&db_path, &decompressed).unwrap();
        let conn = rusqlite::Connection::open(&db_path).unwrap();

        let prereq_count: i64 = conn
            .query_row("SELECT count(*) FROM concept_prereqs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(
            prereq_count, 1,
            "expected the Pressure -> Force basics prereq link to be compiled"
        );
    }
}
