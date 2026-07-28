use crate::db::Db;
use crate::error::AppError;
use crate::models::{
    AnswerReview, Chapter, ChatMessage, Concept, Question, QuestionOption, Subject,
};
use rusqlite::OptionalExtension;

/// Write several settings in a single transaction. Used by the wizard so a
/// crash between writes cannot leave the settings half-applied.
pub fn set_settings_bulk(db: &Db, pairs: &[(&str, &str)]) -> Result<(), AppError> {
    db.with(|c| {
        let tx = c.transaction()?;
        for (key, value) in pairs {
            tx.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                (key, value),
            )?;
        }
        tx.commit()?;
        Ok::<(), rusqlite::Error>(())
    })?;
    Ok(())
}

pub fn get_setting(db: &Db, key: &str) -> Result<Option<String>, AppError> {
    let v: Option<String> = db.with(|c| {
        c.query_row("SELECT value FROM settings WHERE key = ?1", [key], |r| {
            r.get(0)
        })
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
    let ch: Chapter = db
        .with(|c| {
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

/// Rows of `chapter_text` for retrieval scoring: (id, body, ord).
pub fn chapter_text_rows(db: &Db, chapter_id: i64) -> Result<Vec<(i64, String, i64)>, AppError> {
    let rows = db.with(|c| {
        let mut stmt =
            c.prepare("SELECT id, body, ord FROM chapter_text WHERE chapter_id = ?1 ORDER BY ord")?;
        let mapped = stmt.query_map([chapter_id], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

#[cfg(test)]
pub fn correct_option(db: &Db, question_id: i64) -> Result<i64, AppError> {
    let id: i64 = db
        .with(|c| {
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

fn question_with_options(db: &Db, qid: i64) -> Result<Question, AppError> {
    let q: Question = db
        .with(|c| {
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
        let mut stmt =
            c.prepare("SELECT id, body FROM question_options WHERE question_id = ?1 ORDER BY ord")?;
        let mapped =
            stmt.query_map([qid], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;

    // Deterministic shuffle of options by question id (never leaks is_correct).
    // Works for any number of options; guarded against schema drift where a
    // question does not have exactly 4 options.
    if opts.is_empty() {
        return Ok(Question {
            options: Vec::new(),
            ..q
        });
    }
    let mut idx: Vec<usize> = (0..opts.len()).collect();
    shuffle_n(&mut idx, qid);
    let ordered: Vec<QuestionOption> = idx
        .iter()
        .enumerate()
        .map(|(i, &src)| QuestionOption {
            id: opts[src].0,
            ord: (i + 1) as i64,
            body: opts[src].1.clone(),
        })
        .collect();

    Ok(Question {
        options: ordered,
        ..q
    })
}

/// Deterministic Fisher-Yates permutation of `arr` seeded by `seed`.
fn shuffle_n(arr: &mut [usize], seed: i64) {
    let n = arr.len();
    if n < 2 {
        return;
    }
    let mut s: u64 = seed.unsigned_abs() | 1;
    for i in (1..n).rev() {
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

/// Atomically insert a graded answer for `(session_id, question_id)` unless
/// one already exists. Returns `true` if this call performed the insert
/// (i.e. it "won" the race) and `false` if an answer was already present.
///
/// A single `INSERT OR IGNORE` relying on the `uq_quiz_answer` unique index
/// is used instead of a separate exists-check followed by an insert: two
/// `db.with` calls would each release the connection mutex in between,
/// leaving a window where a fast double-click (or two concurrent IPC calls)
/// could both pass the check before either inserted. One statement under one
/// `db.with` closure closes that window -- only one caller ever gets `true`,
/// so only one caller ever runs the follow-up BKT/FSRS mastery update.
pub fn insert_quiz_answer_if_absent(
    db: &Db,
    session_id: i64,
    question_id: i64,
    chosen_option_id: i64,
    correct: bool,
    answered_at: i64,
) -> Result<bool, AppError> {
    let inserted = db.with(|c| {
        let n = c.execute(
            "INSERT OR IGNORE INTO quiz_answers (session_id, question_id, chosen_option_id, correct, answered_at) VALUES (?1,?2,?3,?4,?5)",
            (session_id, question_id, chosen_option_id, if correct { 1 } else { 0 }, answered_at),
        )?;
        Ok(n > 0)
    })?;
    Ok(inserted)
}

/// The chosen option id for an already-graded answer.
pub fn existing_answer_choice(
    db: &Db,
    session_id: i64,
    question_id: i64,
) -> Result<Option<i64>, AppError> {
    let v: Option<i64> = db.with(|c| {
        c.query_row(
            "SELECT chosen_option_id FROM quiz_answers WHERE session_id = ?1 AND question_id = ?2",
            (session_id, question_id),
            |r| r.get(0),
        )
        .optional()
    })?;
    Ok(v)
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

type ReviewCardRow = (f64, f64, u32, u32, Option<i64>, Option<i64>);
type RawReviewCard = (f64, f64, i64, i64, Option<i64>, Option<i64>);

pub fn get_review_card(db: &Db, concept_id: i64) -> Result<Option<ReviewCardRow>, AppError> {
    let v: Option<RawReviewCard> = db.with(|c| {
        c.query_row(
            "SELECT stability, difficulty, reps, lapses, last_review, due
             FROM review_cards WHERE concept_id = ?1",
            [concept_id],
            |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            },
        )
        .optional()
    })?;
    Ok(v.map(|(s, d, reps, laps, lr, due)| (s, d, reps as u32, laps as u32, lr, due)))
}

#[allow(clippy::too_many_arguments)]
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
        c.execute("INSERT OR IGNORE INTO study_days (day) VALUES (?1)", (day,))?;
        Ok(())
    })?;
    Ok(())
}

fn build_answer_review(
    db: &Db,
    question_id: i64,
    chosen_option_id: i64,
) -> Result<AnswerReview, AppError> {
    let (correct_option_id, explanation, trap): (i64, String, String) = db
        .with(|c| {
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

pub fn quiz_session_summary(db: &Db, session_id: i64) -> Result<(i64, i64, f64), AppError> {
    // The aggregate below always returns one row (even for a missing session),
    // so guard with an explicit existence check rather than relying on
    // `.optional()` — otherwise a typo'd session_id silently returns 0/0.
    let exists: i64 = db.with(|c| {
        c.query_row(
            "SELECT count(*) FROM quiz_sessions WHERE id = ?1",
            [session_id],
            |r| r.get(0),
        )
    })?;
    if exists == 0 {
        return Err(AppError::NotFound(format!("session {session_id}")));
    }
    let (total, correct): (i64, i64) = db
        .with(|c| {
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
        let mut stmt = c.prepare(
            "SELECT question_id, chosen_option_id FROM quiz_answers WHERE session_id = ?1",
        )?;
        let mapped = stmt.query_map([session_id], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?;
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

pub fn question_concept_layer(db: &Db, question_id: i64) -> Result<(i64, i64), AppError> {
    let v: (i64, i64) = db
        .with(|c| {
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

// ---------- Progress ----------

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Parse a `YYYY-MM-DD` string into the same day-number epoch used by the
/// quiz `day_string` helper (days since 0000-03-01).
fn parse_day(s: &str) -> Option<i64> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return None;
    }
    let y: i64 = parts[0].parse().ok()?;
    let m: i64 = parts[1].parse().ok()?;
    let d: i64 = parts[2].parse().ok()?;
    Some(days_from_civil(y, m, d) + 719_163)
}

fn map_progress(r: &rusqlite::Row) -> rusqlite::Result<(i64, String, f64, Option<i64>, i64)> {
    Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
}

pub fn get_progress(
    db: &Db,
    chapter_id: Option<i64>,
) -> Result<Vec<crate::models::ConceptProgress>, AppError> {
    let sql = if chapter_id.is_some() {
        "SELECT c.id, c.name, COALESCE(m.p_known,0.2), rc.due, COALESCE(m.attempts,0)
         FROM concepts c
         LEFT JOIN concept_mastery m ON m.concept_id = c.id
         LEFT JOIN review_cards rc ON rc.concept_id = c.id
         WHERE c.chapter_id = ?1
         ORDER BY c.id"
    } else {
        "SELECT c.id, c.name, COALESCE(m.p_known,0.2), rc.due, COALESCE(m.attempts,0)
         FROM concepts c
         LEFT JOIN concept_mastery m ON m.concept_id = c.id
         LEFT JOIN review_cards rc ON rc.concept_id = c.id
         ORDER BY c.id"
    };
    let rows = db.with(|c| {
        let mut stmt = c.prepare(sql)?;
        let mapped = if let Some(ch) = chapter_id {
            stmt.query_map([ch], map_progress)?
        } else {
            stmt.query_map([], map_progress)?
        };
        let mut out = Vec::new();
        for row in mapped {
            let (id, name, pk, due, att) = row?;
            out.push(crate::models::ConceptProgress {
                concept_id: id,
                name,
                p_known: pk,
                due,
                attempts: att,
            });
        }
        Ok(out)
    })?;
    Ok(rows)
}

pub fn get_due(db: &Db, now: i64) -> Result<Vec<crate::models::ConceptProgress>, AppError> {
    let rows = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT c.id, c.name, COALESCE(m.p_known,0.2), rc.due, COALESCE(m.attempts,0)
             FROM review_cards rc
             JOIN concepts c ON c.id = rc.concept_id
             LEFT JOIN concept_mastery m ON m.concept_id = c.id
             WHERE rc.due IS NOT NULL AND rc.due <= ?1
             ORDER BY rc.due ASC",
        )?;
        let mapped = stmt.query_map([now], map_progress)?;
        let mut out = Vec::new();
        for row in mapped {
            let (id, name, pk, due, att) = row?;
            out.push(crate::models::ConceptProgress {
                concept_id: id,
                name,
                p_known: pk,
                due,
                attempts: att,
            });
        }
        Ok(out)
    })?;
    Ok(rows)
}

pub fn get_streak(db: &Db, today: &str) -> Result<i64, AppError> {
    let days: Vec<String> = db.with(|c| {
        let mut stmt = c.prepare("SELECT day FROM study_days")?;
        let mapped = stmt.query_map([], |r| r.get::<_, String>(0))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    let mut set: std::collections::HashSet<i64> = std::collections::HashSet::new();
    for d in &days {
        if let Some(n) = parse_day(d) {
            set.insert(n);
        }
    }
    let today_num = parse_day(today)
        .ok_or_else(|| AppError::BadInput(format!("invalid day string: {today}")))?;
    let mut streak = 0i64;
    let mut cur = if set.contains(&today_num) {
        today_num
    } else {
        today_num - 1
    };
    while set.contains(&cur) {
        streak += 1;
        cur -= 1;
    }
    Ok(streak)
}

pub fn get_mastery(db: &Db, concept_id: i64) -> Result<(f64, i64, i64), AppError> {
    let v: (f64, i64, i64) = db
        .with(|c| {
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

/// XP/level/badges, derived purely from data already written by the grading
/// path -- never a live snapshot. The mastery-XP term counts concepts that
/// have *ever* earned a practiced-or-mastered milestone (`concept_mastery_milestones`,
/// an append-only log) rather than concepts whose *current* p_known happens
/// to clear the bar; badges are read from `earned_badges`, likewise
/// append-only. Both are only ever written by [`record_gamification_events`],
/// called from the grading path right after a mastery update -- so a later
/// wrong review answer that lowers a concept's live p_known cannot make XP,
/// level, or an earned badge disappear (see design spec §6's anti-guilt-pattern
/// stance).
pub fn get_gamification(db: &Db) -> Result<crate::models::GamificationSummary, AppError> {
    let xp_from_answers: i64 = db.with(|c| {
        c.query_row(
            "SELECT COALESCE(SUM(CASE WHEN correct = 1 THEN 10 ELSE 0 END), 0) FROM quiz_answers",
            [],
            |r| r.get(0),
        )
    })?;

    let milestoned_concepts: i64 = db.with(|c| {
        c.query_row(
            "SELECT COUNT(DISTINCT concept_id) FROM concept_mastery_milestones",
            [],
            |r| r.get(0),
        )
    })?;
    let xp_from_mastery = milestoned_concepts * 5;

    let xp = xp_from_answers + xp_from_mastery;
    let level = xp / 100 + 1;
    let xp_into_level = xp % 100;
    let xp_for_next_level: i64 = 100;

    let badges: Vec<String> = db.with(|c| {
        let mut stmt = c.prepare("SELECT badge_id FROM earned_badges ORDER BY earned_at")?;
        let mapped = stmt.query_map([], |r| r.get::<_, String>(0))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;

    Ok(crate::models::GamificationSummary {
        xp,
        level,
        xp_into_level,
        xp_for_next_level,
        badges,
    })
}

/// Permanently record a concept crossing into `level` for the first time
/// (`practiced` or `mastered`). Idempotent: re-recording an already-earned
/// level is a no-op, and there is deliberately no corresponding "unearn" --
/// this table is append-only so a later mastery dip never revokes it.
pub fn record_mastery_milestone_if_new(
    db: &Db,
    concept_id: i64,
    level: &str,
    at: i64,
) -> Result<bool, AppError> {
    let changed = db.with(|c| {
        c.execute(
            "INSERT OR IGNORE INTO concept_mastery_milestones (concept_id, level, earned_at)
             VALUES (?1, ?2, ?3)",
            (concept_id, level, at),
        )
    })?;
    Ok(changed > 0)
}

/// Permanently record a badge as earned. Same append-only, idempotent
/// contract as [`record_mastery_milestone_if_new`].
pub fn record_badge_if_new(db: &Db, badge_id: &str, at: i64) -> Result<bool, AppError> {
    let changed = db.with(|c| {
        c.execute(
            "INSERT OR IGNORE INTO earned_badges (badge_id, earned_at) VALUES (?1, ?2)",
            (badge_id, at),
        )
    })?;
    Ok(changed > 0)
}

pub fn chapter_id_for_concept(db: &Db, concept_id: i64) -> Result<i64, AppError> {
    let v: i64 = db
        .with(|c| {
            c.query_row(
                "SELECT chapter_id FROM concepts WHERE id = ?1",
                [concept_id],
                |r| r.get(0),
            )
            .optional()
        })?
        .ok_or_else(|| AppError::NotFound(format!("concept {concept_id}")))?;
    Ok(v)
}

/// A chapter counts as fully mastered only once it has at least one concept
/// and every concept in it has *ever* earned the `mastered` milestone. The
/// `is_empty` guard matters: `.all()` over an empty iterator is vacuously
/// true, so without it a chapter with zero seeded concepts would award the
/// badge to a student who has touched nothing in it.
pub fn chapter_fully_mastered(db: &Db, chapter_id: i64) -> Result<bool, AppError> {
    let concept_ids = concept_ids_for_chapter(db, chapter_id)?;
    if concept_ids.is_empty() {
        return Ok(false);
    }
    for cid in &concept_ids {
        let mastered: bool = db.with(|c| {
            c.query_row(
                "SELECT 1 FROM concept_mastery_milestones WHERE concept_id = ?1 AND level = 'mastered'",
                [cid],
                |_| Ok(()),
            )
            .optional()
            .map(|v| v.is_some())
        })?;
        if !mastered {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Called from the grading path (`submit_answer_impl`) right after a
/// concept's mastery has just been updated -- anchors milestones/badges to
/// the moment they were actually earned instead of re-deriving them from a
/// live snapshot that can regress.
pub fn record_gamification_events(
    db: &Db,
    concept_id: i64,
    day: &str,
    at: i64,
) -> Result<(), AppError> {
    use crate::pedagogy::mastery_level::{level_for, MasteryLevel};

    let (p_known, attempts, _correct) = get_mastery(db, concept_id)?;
    let level = level_for(attempts, p_known);

    if matches!(level, MasteryLevel::Practiced | MasteryLevel::Mastered) {
        record_mastery_milestone_if_new(db, concept_id, "practiced", at)?;
    }
    if level == MasteryLevel::Mastered {
        record_mastery_milestone_if_new(db, concept_id, "mastered", at)?;
        let chapter_id = chapter_id_for_concept(db, concept_id)?;
        if chapter_fully_mastered(db, chapter_id)? {
            record_badge_if_new(db, &format!("chapter_mastered:{chapter_id}"), at)?;
        }
    }

    let streak = get_streak(db, day)?;
    if streak >= 7 {
        record_badge_if_new(db, "streak_7", at)?;
    }
    if streak >= 30 {
        record_badge_if_new(db, "streak_30", at)?;
    }
    Ok(())
}

pub fn insert_chat_message(
    db: &Db,
    chapter_id: i64,
    role: &str,
    body: &str,
    fsm_state: &str,
    created_at: i64,
) -> Result<i64, AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO chat_messages (chapter_id, role, body, fsm_state, created_at) VALUES (?1,?2,?3,?4,?5)",
            (chapter_id, role, body, fsm_state, created_at),
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn chat_history(db: &Db, chapter_id: i64) -> Result<Vec<ChatMessage>, AppError> {
    let rows = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, chapter_id, role, body, fsm_state, created_at FROM chat_messages WHERE chapter_id = ?1 ORDER BY id",
        )?;
        let mapped = stmt.query_map([chapter_id], |r| {
            Ok(ChatMessage {
                id: r.get(0)?,
                chapter_id: r.get(1)?,
                role: r.get(2)?,
                body: r.get(3)?,
                fsm_state: r.get(4)?,
                created_at: r.get(5)?,
            })
        })?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

// ---------- Curriculum generation inserts ----------

pub fn insert_subject(db: &Db, grade: i64, name: &str, name_hi: &str) -> Result<i64, AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO subjects (grade, name, name_hi) VALUES (?1,?2,?3)",
            (grade, name, name_hi),
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn insert_chapter(
    db: &Db,
    subject_id: i64,
    ord: i64,
    title: &str,
    title_hi: &str,
    summary: &str,
) -> Result<i64, AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO chapters (subject_id, ord, title, title_hi, summary) VALUES (?1,?2,?3,?4,?5)",
            (subject_id, ord, title, title_hi, summary),
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn insert_concept(
    db: &Db,
    chapter_id: i64,
    ord: i64,
    name: &str,
    key_formula: &str,
    common_error: &str,
    exam_weight: i64,
) -> Result<i64, AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO concepts (chapter_id, ord, name, key_formula, common_error, exam_weight) VALUES (?1,?2,?3,?4,?5,?6)",
            (chapter_id, ord, name, key_formula, common_error, exam_weight),
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn insert_chapter_text(
    db: &Db,
    chapter_id: i64,
    concept_id: i64,
    ord: i64,
    body: &str,
) -> Result<i64, AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO chapter_text (chapter_id, concept_id, ord, body) VALUES (?1,?2,?3,?4)",
            (chapter_id, concept_id, ord, body),
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn insert_question(
    db: &Db,
    concept_id: i64,
    bloom_level: &str,
    difficulty_layer: i64,
    prompt: &str,
    trap: &str,
    explanation: &str,
) -> Result<i64, AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO questions (concept_id, qtype, bloom_level, difficulty_layer, prompt, trap, explanation) VALUES (?1,'mcq',?2,?3,?4,?5,?6)",
            (concept_id, bloom_level, difficulty_layer, prompt, trap, explanation),
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn insert_question_option(
    db: &Db,
    question_id: i64,
    ord: i64,
    body: &str,
    is_correct: bool,
) -> Result<i64, AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO question_options (question_id, ord, body, is_correct) VALUES (?1,?2,?3,?4)",
            (question_id, ord, body, if is_correct { 1 } else { 0 }),
        )?;
        Ok(c.last_insert_rowid())
    })
}

pub fn insert_concept_prereq(db: &Db, concept_id: i64, prereq_id: i64) -> Result<(), AppError> {
    db.with(|c| {
        c.execute(
            "INSERT INTO concept_prereqs (concept_id, prereq_id) VALUES (?1,?2)",
            (concept_id, prereq_id),
        )?;
        Ok(())
    })
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
        set_settings_bulk(&db, &[("language", "hi"), ("grade", "9")]).unwrap();
        assert_eq!(get_setting(&db, "language").unwrap().as_deref(), Some("hi"));
        assert_eq!(get_setting(&db, "grade").unwrap().as_deref(), Some("9"));
        assert_eq!(get_setting(&db, "missing").unwrap(), None);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn insert_quiz_answer_if_absent_only_inserts_once() {
        let (db, p) = temp_db();
        crate::db::seed::seed_if_empty(&db).unwrap();
        let session = insert_quiz_session(&db, 1, 1_000_000).unwrap();
        let correct_id = correct_option(&db, 1).unwrap();

        let first =
            insert_quiz_answer_if_absent(&db, session, 1, correct_id, true, 1_000_001).unwrap();
        assert!(first, "first insert should win and report true");

        // Same (session, question) again -- simulates a double-click or a
        // racing concurrent call that lost the atomic insert.
        let second =
            insert_quiz_answer_if_absent(&db, session, 1, correct_id, true, 1_000_002).unwrap();
        assert!(!second, "second insert should be a no-op and report false");

        let count: i64 = db
            .with(|c| {
                c.query_row(
                    "SELECT count(*) FROM quiz_answers WHERE session_id = ?1 AND question_id = ?2",
                    (session, 1),
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(count, 1, "only one row should ever be inserted");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn chapter_fully_mastered_is_false_for_a_chapter_with_no_concepts() {
        let (db, p) = temp_db();
        crate::db::seed::seed_if_empty(&db).unwrap();
        // Insert a chapter with zero concepts -- `.all()` over an empty
        // iterator is vacuously true, which is exactly the bug this guards
        // against: an untouched, content-less chapter must never register
        // as "mastered".
        db.with(|c| {
            c.execute(
                "INSERT INTO chapters (id, subject_id, ord, title) VALUES (99, 1, 99, 'Empty')",
                [],
            )
        })
        .unwrap();
        assert!(!chapter_fully_mastered(&db, 99).unwrap());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn badges_and_mastery_xp_survive_a_later_mastery_regression() {
        let (db, p) = temp_db();
        crate::db::seed::seed_if_empty(&db).unwrap();

        // Concepts 1, 2, 3 are the whole of chapter 1. Master all three,
        // recording gamification events the same way the grading path does.
        for cid in [1, 2, 3] {
            update_mastery(&db, cid, 0.9, 5, 5, 1000).unwrap();
            record_gamification_events(&db, cid, "2026-07-20", 1000).unwrap();
        }

        let before = get_gamification(&db).unwrap();
        assert!(
            before.badges.contains(&"chapter_mastered:1".to_string()),
            "chapter 1 should be marked mastered once all its concepts are: {:?}",
            before.badges
        );
        assert!(
            before.xp >= 15,
            "expected mastery XP for 3 concepts, got {}",
            before.xp
        );

        // A later spaced-repetition review question on concept 1 comes back
        // wrong: BKT lowers its p_known well below the mastered threshold.
        update_mastery(&db, 1, 0.3, 6, 5, 2000).unwrap();
        record_gamification_events(&db, 1, "2026-07-20", 2000).unwrap();

        let after = get_gamification(&db).unwrap();
        assert!(
            after.badges.contains(&"chapter_mastered:1".to_string()),
            "an earned badge must not disappear because of a later wrong review answer: {:?}",
            after.badges
        );
        assert_eq!(
            after.xp, before.xp,
            "XP must not regress from a live mastery dip once a milestone is earned"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn only_mastered_concepts_earn_the_mastered_milestone() {
        let (db, p) = temp_db();
        crate::db::seed::seed_if_empty(&db).unwrap();

        // p_known 0.6 is "practiced", not "mastered" (see mastery_level).
        update_mastery(&db, 1, 0.6, 3, 2, 1000).unwrap();
        record_gamification_events(&db, 1, "2026-07-20", 1000).unwrap();

        let g = get_gamification(&db).unwrap();
        assert!(
            !g.badges.iter().any(|b| b.starts_with("chapter_mastered:")),
            "a merely-practiced concept must not complete its chapter: {:?}",
            g.badges
        );
        assert_eq!(
            g.xp, 5,
            "one practiced-or-above concept should award the +5 mastery bonus once"
        );
        let _ = std::fs::remove_file(&p);
    }
}
