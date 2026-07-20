use crate::db::Db;
use crate::error::AppError;
use crate::models::{Chapter, Concept, Question, QuestionOption, Subject};
use rusqlite::OptionalExtension;

pub fn set_setting(db: &Db, key: &str, value: &str) -> Result<(), AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            (key, value),
        )
    })?;
    Ok(())
}

pub fn get_setting(db: &Db, key: &str) -> Result<Option<String>, AppError> {
    let v: Option<String> = db.with(|c| {
        c.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            [key],
            |r| r.get(0),
        )
        .optional()
    })?;
    Ok(v)
}

pub fn list_subjects(db: &Db, grade: i64) -> Result<Vec<Subject>, AppError> {
    let rows = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, grade, name, name_hi FROM subjects WHERE grade = ?1 ORDER BY id",
        )?;
        let mapped = stmt.query_map([grade], |r| {
            Ok(Subject {
                id: r.get(0)?,
                grade: r.get(1)?,
                name: r.get(2)?,
                name_hi: r.get(3)?,
            })
        })?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

pub fn list_chapters(db: &Db, subject_id: i64) -> Result<Vec<Chapter>, AppError> {
    let rows = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, subject_id, ord, title, title_hi, summary FROM chapters WHERE subject_id = ?1 ORDER BY ord",
        )?;
        let mapped = stmt.query_map([subject_id], |r| {
            Ok(Chapter {
                id: r.get(0)?,
                subject_id: r.get(1)?,
                ord: r.get(2)?,
                title: r.get(3)?,
                title_hi: r.get(4)?,
                summary: r.get(5)?,
            })
        })?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

pub fn get_chapter(db: &Db, id: i64) -> Result<Chapter, AppError> {
    let ch: Chapter = db.with(|c| {
        c.query_row(
            "SELECT id, subject_id, ord, title, title_hi, summary FROM chapters WHERE id = ?1",
            [id],
            |r| {
                Ok(Chapter {
                    id: r.get(0)?,
                    subject_id: r.get(1)?,
                    ord: r.get(2)?,
                    title: r.get(3)?,
                    title_hi: r.get(4)?,
                    summary: r.get(5)?,
                })
            },
        )
        .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("chapter {id}")))?;
    Ok(ch)
}

pub fn list_concepts(db: &Db, chapter_id: i64) -> Result<Vec<Concept>, AppError> {
    let rows = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, chapter_id, ord, name, key_formula, common_error, exam_weight
             FROM concepts WHERE chapter_id = ?1 ORDER BY ord",
        )?;
        let mapped = stmt.query_map([chapter_id], |r| {
            Ok(Concept {
                id: r.get(0)?,
                chapter_id: r.get(1)?,
                ord: r.get(2)?,
                name: r.get(3)?,
                key_formula: r.get(4)?,
                common_error: r.get(5)?,
                exam_weight: r.get(6)?,
            })
        })?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

pub fn chapter_snippets(db: &Db, chapter_id: i64) -> Result<Vec<(i64, String)>, AppError> {
    let rows = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, body FROM chapter_text WHERE chapter_id = ?1 ORDER BY ord",
        )?;
        let mapped = stmt.query_map([chapter_id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

pub fn correct_option(db: &Db, question_id: i64) -> Result<i64, AppError> {
    let id: i64 = db.with(|c| {
        c.query_row(
            "SELECT id FROM question_options WHERE question_id = ?1 AND is_correct = 1",
            [question_id],
            |r| r.get(0),
        )
        .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("correct option for question {question_id}")))?;
    Ok(id)
}

pub fn questions_for_chapter(db: &Db, chapter_id: i64, limit: i64) -> Result<Vec<Question>, AppError> {
    let qids: Vec<i64> = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT q.id FROM questions q
             JOIN concepts c ON c.id = q.concept_id
             WHERE c.chapter_id = ?1 ORDER BY q.id",
        )?;
        let mapped = stmt.query_map([chapter_id], |r| r.get::<_, i64>(0))?;
        let mut v: Vec<i64> = mapped.collect::<rusqlite::Result<Vec<_>>>()?;
        if limit > 0 && (v.len() as i64) > limit {
            v.truncate(limit as usize);
        }
        Ok(v)
    })?;

    let mut out = Vec::with_capacity(qids.len());
    for qid in qids {
        out.push(question_with_options(db, qid)?);
    }
    Ok(out)
}

fn question_with_options(db: &Db, qid: i64) -> Result<Question, AppError> {
    let q: Question = db.with(|c| {
        c.query_row(
            "SELECT id, concept_id, qtype, bloom_level, difficulty_layer, prompt
             FROM questions WHERE id = ?1",
            [qid],
            |r| {
                Ok(Question {
                    id: r.get(0)?,
                    concept_id: r.get(1)?,
                    qtype: r.get(2)?,
                    bloom_level: r.get(3)?,
                    difficulty_layer: r.get(4)?,
                    prompt: r.get(5)?,
                    options: Vec::new(),
                })
            },
        )
        .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("question {qid}")))?;

    let opts: Vec<(i64, String)> = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, body FROM question_options WHERE question_id = ?1 ORDER BY ord",
        )?;
        let mapped = stmt.query_map([qid], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;

    // Deterministic shuffle of options by question id (never leaks is_correct).
    let mut idx: [usize; 4] = [0, 1, 2, 3];
    shuffle4(&mut idx, qid);
    let ordered: Vec<QuestionOption> = idx
        .iter()
        .enumerate()
        .map(|(i, &src)| QuestionOption {
            id: opts[src].0,
            ord: (i + 1) as i64,
            body: opts[src].1.clone(),
        })
        .collect();

    Ok(Question { options: ordered, ..q })
}

/// Deterministic Fisher-Yates permutation of 4 indices seeded by `seed`.
fn shuffle4(arr: &mut [usize; 4], seed: i64) {
    let mut s: u64 = (seed.unsigned_abs() | 1) as u64;
    for i in (1..4).rev() {
        s = s
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = ((s >> 33) % (i as u64 + 1)) as usize;
        arr.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (Db, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("shala_q_{}.db", n));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();
        (db, p)
    }

    #[test]
    fn settings_roundtrip() {
        let (db, p) = temp_db();
        set_setting(&db, "language", "hi").unwrap();
        set_setting(&db, "grade", "9").unwrap();
        assert_eq!(get_setting(&db, "language").unwrap().as_deref(), Some("hi"));
        assert_eq!(get_setting(&db, "grade").unwrap().as_deref(), Some("9"));
        assert_eq!(get_setting(&db, "missing").unwrap(), None);
        let _ = std::fs::remove_file(&p);
    }
}

