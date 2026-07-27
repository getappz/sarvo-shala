use crate::schema::{GenerationMethod, ReviewStatus, SubjectFile};
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

        for concept in &chapter.concepts {
            if concept.prereq_ids.is_empty() {
                continue;
            }
            let concept_id: i64 = conn.query_row(
                "SELECT c.id FROM concepts c JOIN chapters ch ON c.chapter_id = ch.id WHERE ch.subject_id = ?1 AND c.name = ?2",
                rusqlite::params![subject_id, concept.name],
                |r| r.get(0),
            )?;
            for prereq_id_str in &concept.prereq_ids {
                if let Ok(prereq_row_id) = conn.query_row::<i64, _, _>(
                    "SELECT c2.id FROM concepts c2 JOIN chapters ch2 ON c2.chapter_id = ch2.id WHERE ch2.subject_id = ?1 AND c2.name = ?2",
                    rusqlite::params![subject_id, prereq_id_str],
                    |r| r.get(0),
                ) {
                    conn.execute(
                        "INSERT OR IGNORE INTO concept_prereqs (concept_id, prereq_id) VALUES (?1, ?2)",
                        rusqlite::params![concept_id, prereq_row_id],
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
}
