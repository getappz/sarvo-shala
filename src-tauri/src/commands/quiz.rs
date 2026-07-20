use crate::db::queries;
use crate::db::Db;
use crate::error::AppError;
use crate::models::{AnswerReview, Question, QuizResult};
use crate::pedagogy::{bkt, fsrs};
use tauri::State;

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Format a unix timestamp (seconds) as a UTC `YYYY-MM-DD` day string.
fn day_string(now: i64) -> String {
    let days = now.div_euclid(86400) + 719_163; // Unix epoch -> days since 0000-03-01
    let mut y = 0i64;
    let mut m = 0i64;
    let mut d = 0i64;
    civil_from_days(days, &mut y, &mut m, &mut d);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Reversed algorithm: days since 0000-03-01 -> civil (y, m, d).
fn civil_from_days(z: i64, y: &mut i64, m: &mut i64, d: &mut i64) {
    let zz = z + 306;
    let era = if zz >= 0 { zz } else { zz - 146096 } / 146097;
    let doe = zz - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y_ = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    *m = if mp < 10 { mp + 3 } else { mp - 9 };
    *y = if *m <= 2 { y_ + 1 } else { y_ };
    *d = doy - (153 * mp + 2) / 5 + 1;
}

fn layer_for(p_known: f64) -> i64 {
    if p_known < 0.4 {
        1
    } else if p_known < 0.7 {
        2
    } else {
        3
    }
}

#[tauri::command]
pub fn start_quiz(
    state: State<Db>,
    chapter_id: i64,
) -> Result<(i64, Vec<Question>), AppError> {
    let now = now_secs();
    let session_id = queries::insert_quiz_session(&state, chapter_id, now)?;
    let concepts = queries::concept_ids_for_chapter(&state, chapter_id)?;
    let mut questions = Vec::with_capacity(concepts.len());
    for cid in concepts {
        let p = queries::concept_mastery_pknown(&state, cid)?;
        let layer = layer_for(p);
        let qid = queries::question_for_concept_layer(&state, cid, layer)?
            .or_else(|| queries::question_for_concept_layer(&state, cid, 1).ok().flatten())
            .ok_or_else(|| AppError::NotFound(format!("no question for concept {cid}")))?;
        questions.push(queries::question_by_id(&state, qid)?);
    }
    Ok((session_id, questions))
}

#[tauri::command]
pub fn submit_answer(
    state: State<Db>,
    session_id: i64,
    question_id: i64,
    chosen_option_id: i64,
) -> Result<AnswerReview, AppError> {
    let now = now_secs();
    let review = queries::grade_answer(&state, question_id, chosen_option_id)?;
    queries::insert_quiz_answer(
        &state,
        session_id,
        question_id,
        chosen_option_id,
        review.correct,
        now,
    )?;

    let (concept_id, layer) = queries::question_concept_layer(&state, question_id)?;

    // BKT mastery update.
    let p_known = queries::concept_mastery_pknown(&state, concept_id)?;
    let new_p = bkt::update(p_known, review.correct, &bkt::Params::default());
    let (_, attempts, correct_count) = queries::get_mastery(&state, concept_id)?;
    queries::update_mastery(
        &state,
        concept_id,
        new_p,
        attempts + 1,
        correct_count + if review.correct { 1 } else { 0 },
        now,
    )?;

    // FSRS review-card update.
    let card_row = queries::get_review_card(&state, concept_id)?;
    let card = match card_row {
        Some((s, d, reps, laps, lr, due)) => fsrs::Card {
            stability: s,
            difficulty: d,
            reps,
            lapses: laps,
            last_review: lr,
            due,
        },
        None => fsrs::Card {
            stability: 0.0,
            difficulty: 0.0,
            reps: 0,
            lapses: 0,
            last_review: None,
            due: None,
        },
    };
    let rating = if review.correct {
        match layer {
            3 => fsrs::Rating::Easy,
            2 => fsrs::Rating::Good,
            _ => fsrs::Rating::Hard,
        }
    } else {
        fsrs::Rating::Again
    };
    let updated = fsrs::review(&card, rating, now);
    queries::upsert_review_card(
        &state,
        concept_id,
        updated.stability,
        updated.difficulty,
        updated.reps,
        updated.lapses,
        updated.last_review,
        updated.due,
    )?;

    queries::upsert_study_day(&state, &day_string(now))?;

    Ok(review)
}

#[tauri::command]
pub fn finish_quiz(state: State<Db>, session_id: i64) -> Result<QuizResult, AppError> {
    let (total, correct, score) = queries::quiz_session_summary(&state, session_id)?;
    queries::finish_session(&state, session_id, score)?;
    let per_question = queries::quiz_session_reviews(&state, session_id)?;
    Ok(QuizResult {
        session_id,
        total,
        correct,
        score,
        per_question,
    })
}

#[cfg(test)]
mod tests {
    use crate::db::queries;
    use crate::db::seed;
    use crate::db::Db;
    use crate::pedagogy::bkt;

    fn seeded() -> (Db, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("swayam_quiz_{}.db", n));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();
        seed::seed_if_empty(&db).unwrap();
        (db, p)
    }

    #[test]
    fn grading_correct_and_wrong() {
        let (db, p) = seeded();
        // Q1 is concept 1, layer 1.
        let correct_id = queries::correct_option(&db, 1).unwrap();
        let wrong_id = queries::question_by_id(&db, 1)
            .unwrap()
            .options
            .into_iter()
            .find(|o| o.id != correct_id)
            .unwrap()
            .id;

        let right = queries::grade_answer(&db, 1, correct_id).unwrap();
        assert!(right.correct);
        let wrong = queries::grade_answer(&db, 1, wrong_id).unwrap();
        assert!(!wrong.correct);

        let p0 = queries::concept_mastery_pknown(&db, 1).unwrap();
        let (_, att, cor) = queries::get_mastery(&db, 1).unwrap();
        queries::update_mastery(&db, 1, bkt::update(p0, true, &bkt::Params::default()), att + 1, cor + 1, 1_000_000).unwrap();
        let p1 = queries::concept_mastery_pknown(&db, 1).unwrap();
        assert!(p1 > p0);

        let p2 = queries::concept_mastery_pknown(&db, 1).unwrap();
        let (_, att2, cor2) = queries::get_mastery(&db, 1).unwrap();
        queries::update_mastery(&db, 1, bkt::update(p2, false, &bkt::Params::default()), att2 + 1, cor2, 1_000_001).unwrap();
        let p3 = queries::concept_mastery_pknown(&db, 1).unwrap();
        assert!(p3 < p2);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn difficulty_selection_follows_mastery() {
        let (db, p) = seeded();
        // Concept 1 default p_known is 0.2 -> layer 1.
        assert_eq!(super::layer_for(queries::concept_mastery_pknown(&db, 1).unwrap()), 1);
        // Raise to 0.9 -> layer 3.
        queries::update_mastery(&db, 1, 0.9, 1, 1, 1_000_000).unwrap();
        assert_eq!(super::layer_for(queries::concept_mastery_pknown(&db, 1).unwrap()), 3);
        let q = queries::question_for_concept_layer(&db, 1, 3).unwrap();
        assert!(q.is_some());
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn finish_computes_score() {
        let (db, p) = seeded();
        let session = queries::insert_quiz_session(&db, 1, 1_000_000).unwrap();
        let c1 = queries::correct_option(&db, 1).unwrap();
        let c2 = queries::correct_option(&db, 2).unwrap();
        queries::insert_quiz_answer(&db, session, 1, c1, true, 1_000_001).unwrap();
        queries::insert_quiz_answer(&db, session, 2, c2, true, 1_000_002).unwrap();
        let (total, correct, score) = queries::quiz_session_summary(&db, session).unwrap();
        assert_eq!(total, 2);
        assert_eq!(correct, 2);
        assert_eq!(score, 100.0);
        let _ = std::fs::remove_file(&p);
    }
}

