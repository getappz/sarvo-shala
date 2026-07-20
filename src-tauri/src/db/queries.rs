use crate::db::Db;
use crate::error::AppError;
use crate::models::{AnswerReview, Chapter, Concept, Question, QuestionOption, Subject};
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

// ---------- Quiz ----------

pub fn insert_quiz_session(db: &Db, chapter_id: i64, started_at: i64) -> Result<i64, AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO quiz_sessions (chapter_id, started_at, finished_at, score) VALUES (?1,?2,NULL,NULL)",
            (chapter_id, started_at),
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn insert_quiz_answer(
    db: &Db,
    session_id: i64,
    question_id: i64,
    chosen_option_id: i64,
    correct: bool,
    answered_at: i64,
) -> Result<(), AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO quiz_answers (session_id, question_id, chosen_option_id, correct, answered_at) VALUES (?1,?2,?3,?4,?5)",
            (session_id, question_id, chosen_option_id, if correct { 1 } else { 0 }, answered_at),
        )?;
        Ok(())
    })?;
    Ok(())
}

pub fn concept_mastery_pknown(db: &Db, concept_id: i64) -> Result<f64, AppError> {
    let v: Option<f64> = db.with(|c| {
        c.query_row(
            "SELECT p_known FROM concept_mastery WHERE concept_id = ?1",
            [concept_id],
            |r| r.get(0),
        )
        .optional()
    })?;
    Ok(v.unwrap_or(0.2))
}

/// Returns the question id for a concept at the given difficulty layer (1..3), if any.
pub fn question_for_concept_layer(
    db: &Db,
    concept_id: i64,
    layer: i64,
) -> Result<Option<i64>, AppError> {
    let v: Option<i64> = db.with(|c| {
        c.query_row(
            "SELECT id FROM questions WHERE concept_id = ?1 AND difficulty_layer = ?2",
            (concept_id, layer),
            |r| r.get(0),
        )
        .optional()
    })?;
    Ok(v)
}

pub fn update_mastery(
    db: &Db,
    concept_id: i64,
    p_known: f64,
    attempts: i64,
    correct: i64,
    updated_at: i64,
) -> Result<(), AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO concept_mastery (concept_id, p_known, attempts, correct, updated_at)
             VALUES (?1,?2,?3,?4,?5)
             ON CONFLICT(concept_id) DO UPDATE SET
               p_known = excluded.p_known,
               attempts = excluded.attempts,
               correct = excluded.correct,
               updated_at = excluded.updated_at",
            (concept_id, p_known, attempts, correct, updated_at),
        )?;
        Ok(())
    })?;
    Ok(())
}

pub fn get_review_card(
    db: &Db,
    concept_id: i64,
) -> Result<
    Option<(f64, f64, u32, u32, Option<i64>, Option<i64>)>,
    AppError,
> {
    let v: Option<(f64, f64, i64, i64, Option<i64>, Option<i64>)> = db.with(|c| {
        c.query_row(
            "SELECT stability, difficulty, reps, lapses, last_review, due
             FROM review_cards WHERE concept_id = ?1",
            [concept_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
        )
        .optional()
    })?;
    Ok(v.map(|(s, d, reps, laps, lr, due)| (s, d, reps as u32, laps as u32, lr, due)))
}

pub fn upsert_review_card(
    db: &Db,
    concept_id: i64,
    stability: f64,
    difficulty: f64,
    reps: u32,
    lapses: u32,
    last_review: Option<i64>,
    due: Option<i64>,
) -> Result<(), AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO review_cards (concept_id, stability, difficulty, reps, lapses, last_review, due)
             VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(concept_id) DO UPDATE SET
               stability = excluded.stability,
               difficulty = excluded.difficulty,
               reps = excluded.reps,
               lapses = excluded.lapses,
               last_review = excluded.last_review,
               due = excluded.due",
            (concept_id, stability, difficulty, reps, lapses, last_review, due),
        )?;
        Ok(())
    })?;
    Ok(())
}

pub fn upsert_study_day(db: &Db, day: &str) -> Result<(), AppError> {
    db.with(|c| {
        c.execute(
            "INSERT OR IGNORE INTO study_days (day) VALUES (?1)",
            (day,),
        )?;
        Ok(())
    })?;
    Ok(())
}

fn build_answer_review(
    db: &Db,
    question_id: i64,
    chosen_option_id: i64,
) -> Result<AnswerReview, AppError> {
    let (correct_option_id, explanation, trap): (i64, String, String) = db.with(|c| {
        c.query_row(
            "SELECT qo.id, q.explanation, q.trap
             FROM questions q
             JOIN question_options qo ON qo.question_id = q.id AND qo.is_correct = 1
             WHERE q.id = ?1",
            [question_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("question {question_id}")))?;
    let correct = chosen_option_id == correct_option_id;
    Ok(AnswerReview {
        question_id,
        correct,
        correct_option_id,
        explanation,
        trap,
    })
}

pub fn quiz_session_summary(
    db: &Db,
    session_id: i64,
) -> Result<(i64, i64, f64), AppError> {
    let (total, correct): (i64, i64) = db.with(|c| {
        c.query_row(
            "SELECT count(*), sum(correct) FROM quiz_answers WHERE session_id = ?1",
            [session_id],
            |r| Ok((r.get(0)?, r.get::<_, i64>(1).unwrap_or(0))),
        )
        .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("session {session_id}")))?;
    let score = if total > 0 {
        (correct as f64) / (total as f64) * 100.0
    } else {
        0.0
    };
    Ok((total, correct, score))
}

pub fn quiz_session_reviews(db: &Db, session_id: i64) -> Result<Vec<AnswerReview>, AppError> {
    let qids: Vec<(i64, i64)> = db.with(|c| {
        let mut stmt = c.prepare("SELECT question_id, chosen_option_id FROM quiz_answers WHERE session_id = ?1")?;
        let mapped = stmt.query_map([session_id], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    let mut out = Vec::with_capacity(qids.len());
    for (qid, chosen) in qids {
        out.push(build_answer_review(db, qid, chosen)?);
    }
    Ok(out)
}

pub fn concept_ids_for_chapter(db: &Db, chapter_id: i64) -> Result<Vec<i64>, AppError> {
    let v: Vec<i64> = db.with(|c| {
        let mut stmt = c.prepare("SELECT id FROM concepts WHERE chapter_id = ?1 ORDER BY ord")?;
        let mapped = stmt.query_map([chapter_id], |r| r.get::<_, i64>(0))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(v)
}

pub fn question_by_id(db: &Db, qid: i64) -> Result<Question, AppError> {
    question_with_options(db, qid)
}

pub fn question_concept_layer(
    db: &Db,
    question_id: i64,
) -> Result<(i64, i64), AppError> {
    let v: (i64, i64) = db.with(|c| {
        c.query_row(
            "SELECT concept_id, difficulty_layer FROM questions WHERE id = ?1",
            [question_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("question {question_id}")))?;
    Ok(v)
}

pub fn finish_session(db: &Db, session_id: i64, score: f64) -> Result<(), AppError> {
    db.with(|c| {
        c.execute(
            "UPDATE quiz_sessions SET finished_at = strftime('%s','now'), score = ?1 WHERE id = ?2",
            (score, session_id),
        )?;
        Ok(())
    })?;
    Ok(())
}

pub fn grade_answer(
    db: &Db,
    question_id: i64,
    chosen_option_id: i64,
) -> Result<AnswerReview, AppError> {
    build_answer_review(db, question_id, chosen_option_id)
}

pub fn get_mastery(
    db: &Db,
    concept_id: i64,
) -> Result<(f64, i64, i64), AppError> {
    let v: (f64, i64, i64) = db.with(|c| {
        c.query_row(
            "SELECT p_known, attempts, correct FROM concept_mastery WHERE concept_id = ?1",
            [concept_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("mastery for concept {concept_id}")))?;
    Ok(v)
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

