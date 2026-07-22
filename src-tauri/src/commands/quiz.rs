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
pub fn start_quiz(state: State<Db>, chapter_id: i64) -> Result<(i64, Vec<Question>), AppError> {
    // Reject a bogus chapter id instead of silently returning an empty quiz.
    queries::get_chapter(&state, chapter_id)?;
    let now = now_secs();
    let session_id = queries::insert_quiz_session(&state, chapter_id, now)?;
    let concepts = queries::concept_ids_for_chapter(&state, chapter_id)?;
    let mut questions = Vec::with_capacity(concepts.len());
    for cid in concepts {
        let p = queries::concept_mastery_pknown(&state, cid)?;
        let layer = layer_for(p);
        let qid = queries::question_for_concept_layer(&state, cid, layer)?
            .or_else(|| {
                queries::question_for_concept_layer(&state, cid, 1)
                    .ok()
                    .flatten()
            })
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
    day: String,
) -> Result<AnswerReview, AppError> {
    submit_answer_impl(&state, session_id, question_id, chosen_option_id, &day)
}

/// Core grading pipeline, split out from the `#[tauri::command]` wrapper so
/// it can be exercised directly -- including concurrently -- in tests.
pub(crate) fn submit_answer_impl(
    db: &Db,
    session_id: i64,
    question_id: i64,
    chosen_option_id: i64,
    day: &str,
) -> Result<AnswerReview, AppError> {
    let now = now_secs();
    let review = queries::grade_answer(db, question_id, chosen_option_id)?;

    // Idempotent: a fast double-click (or a genuinely concurrent duplicate
    // call) fires this twice for the same question. `insert_quiz_answer_if_absent`
    // is a single atomic `INSERT OR IGNORE`, so only the caller that actually
    // wins the race gets `true` back and runs the BKT/FSRS/score updates
    // below; everyone else returns the already-graded result for the
    // *originally stored* choice instead of re-running (and double-counting)
    // mastery.
    if !queries::insert_quiz_answer_if_absent(
        db,
        session_id,
        question_id,
        chosen_option_id,
        review.correct,
        now,
    )? {
        let chosen = queries::existing_answer_choice(db, session_id, question_id)?
            .ok_or_else(|| AppError::NotFound(format!("answer for {question_id}")))?;
        return queries::grade_answer(db, question_id, chosen);
    }

    let (concept_id, layer) = queries::question_concept_layer(db, question_id)?;

    // BKT mastery update.
    let p_known = queries::concept_mastery_pknown(db, concept_id)?;
    let new_p = bkt::update(p_known, review.correct, &bkt::Params::default());
    let (_, attempts, correct_count) = queries::get_mastery(db, concept_id)?;
    queries::update_mastery(
        db,
        concept_id,
        new_p,
        attempts + 1,
        correct_count + if review.correct { 1 } else { 0 },
        now,
    )?;

    // FSRS review-card update.
    let card_row = queries::get_review_card(db, concept_id)?;
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
        db,
        concept_id,
        updated.stability,
        updated.difficulty,
        updated.reps,
        updated.lapses,
        updated.last_review,
        updated.due,
    )?;

    // `day` is the local calendar day supplied by the frontend, so study-day
    // streaks agree with the student's local timezone (target audience is in
    // IST, where a UTC day string would mis-attribute pre-dawn study).
    queries::upsert_study_day(db, day)?;

    // Anchors any newly-earned mastery milestone/badge to this moment,
    // derived from the mastery update just above -- not re-derived later
    // from a live snapshot that could regress (see queries::record_gamification_events).
    queries::record_gamification_events(db, concept_id, day, now)?;

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
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut p = std::env::temp_dir();
        p.push(format!("swayam_quiz_{}_{}.db", std::process::id(), n));
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
        queries::update_mastery(
            &db,
            1,
            bkt::update(p0, true, &bkt::Params::default()),
            att + 1,
            cor + 1,
            1_000_000,
        )
        .unwrap();
        let p1 = queries::concept_mastery_pknown(&db, 1).unwrap();
        assert!(p1 > p0);

        let p2 = queries::concept_mastery_pknown(&db, 1).unwrap();
        let (_, att2, cor2) = queries::get_mastery(&db, 1).unwrap();
        queries::update_mastery(
            &db,
            1,
            bkt::update(p2, false, &bkt::Params::default()),
            att2 + 1,
            cor2,
            1_000_001,
        )
        .unwrap();
        let p3 = queries::concept_mastery_pknown(&db, 1).unwrap();
        assert!(p3 < p2);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn difficulty_selection_follows_mastery() {
        let (db, p) = seeded();
        // Concept 1 default p_known is 0.2 -> layer 1.
        assert_eq!(
            super::layer_for(queries::concept_mastery_pknown(&db, 1).unwrap()),
            1
        );
        // Raise to 0.9 -> layer 3.
        queries::update_mastery(&db, 1, 0.9, 1, 1, 1_000_000).unwrap();
        assert_eq!(
            super::layer_for(queries::concept_mastery_pknown(&db, 1).unwrap()),
            3
        );
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
        queries::insert_quiz_answer_if_absent(&db, session, 1, c1, true, 1_000_001).unwrap();
        queries::insert_quiz_answer_if_absent(&db, session, 2, c2, true, 1_000_002).unwrap();
        let (total, correct, score) = queries::quiz_session_summary(&db, session).unwrap();
        assert_eq!(total, 2);
        assert_eq!(correct, 2);
        assert_eq!(score, 100.0);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn concurrent_submit_answer_only_counts_once() {
        let (db, p) = seeded();
        let session = queries::insert_quiz_session(&db, 1, 1_000_000).unwrap();
        let correct_id = queries::correct_option(&db, 1).unwrap();

        // Fire two genuinely concurrent submissions for the same
        // (session, question) -- the scenario a fast double-click or two
        // in-flight IPC calls can produce. Only one may run the BKT/FSRS
        // mastery update; the other must gracefully return the same graded
        // review instead of erroring out.
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let handles: Vec<_> = (0..2)
            .map(|_| {
                let db = db.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    super::submit_answer_impl(&db, session, 1, correct_id, "2026-07-20")
                })
            })
            .collect();
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        for r in &results {
            let review = r.as_ref().expect("submit_answer_impl must not error");
            assert!(review.correct);
        }

        let answer_count: i64 = db
            .with(|c| {
                c.query_row(
                    "SELECT count(*) FROM quiz_answers WHERE session_id = ?1 AND question_id = ?2",
                    (session, 1),
                    |r| r.get(0),
                )
            })
            .unwrap();
        assert_eq!(answer_count, 1, "only one answer row despite two submits");

        let (_, attempts, _) = queries::get_mastery(&db, 1).unwrap();
        assert_eq!(attempts, 1, "mastery must only be updated by the winner");

        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn submit_answer_feeds_gamification_xp() {
        let (db, p) = seeded();
        let session = queries::insert_quiz_session(&db, 1, 1_000_000).unwrap();
        let c1 = queries::correct_option(&db, 1).unwrap();
        let c2 = queries::correct_option(&db, 2).unwrap();

        super::submit_answer_impl(&db, session, 1, c1, "2026-07-20").unwrap();
        super::submit_answer_impl(&db, session, 2, c2, "2026-07-20").unwrap();

        let g = queries::get_gamification(&db).unwrap();
        // 10 XP per correct answer is the floor; BKT convergence (exercised
        // separately in pedagogy::bkt's own tests) may also cross a concept
        // into "practiced" after just one correct answer, adding +5 -- this
        // test only needs to prove the grading path actually feeds
        // get_gamification, not pin the exact BKT curve.
        assert!(
            g.xp >= 20,
            "two correct answers should award at least 10 XP each: {:?}",
            g
        );
        let _ = std::fs::remove_file(&p);
    }
}
