use crate::ai::classify::{Classifier, HeuristicClassifier};
use crate::ai::mock::MockBackend;
use crate::ai::retriever::KeywordRetriever;
use crate::ai::socratic::{SocState, SocraticEngine};
use crate::ai::{GenRequest, InferenceBackend, Retriever, Snippet};
use crate::db::queries;
use crate::db::Db;
use crate::error::AppError;
use crate::models::{ChatMessage, StreamEvent, TutorTurnResult};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn parse_state(s: &str) -> SocState {
    match s {
        "Probe" => SocState::Probe,
        "Clarify" => SocState::Clarify,
        "Confront" => SocState::Confront,
        "Scaffold" => SocState::Scaffold,
        "Confirm" => SocState::Confirm,
        _ => SocState::Probe,
    }
}

/// Derive expected keywords from the top retrieved snippet (lowercased words
/// of length >= 4), capped at 6. Used by the heuristic classifier.
fn keywords_from(snippets: &[Snippet]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for sn in snippets.iter().take(1) {
        for w in sn.body.to_lowercase().split(|c: char| !c.is_alphanumeric()) {
            if w.len() >= 4 && !out.iter().any(|x| x == w) {
                out.push(w.to_string());
            }
            if out.len() >= 6 {
                break;
            }
        }
    }
    out
}

#[tauri::command]
pub async fn tutor_turn(
    app: AppHandle,
    state: State<'_, Db>,
    chapter_id: i64,
    student_msg: String,
    fsm_state: String,
) -> Result<TutorTurnResult, AppError> {
    let db = (*state).clone();
    run_turn(
        &db,
        chapter_id,
        &student_msg,
        &fsm_state,
        &|e: StreamEvent| {
            let _ = app.emit("tutor-stream", e);
        },
    )
    .await
}

/// Core tutor pipeline: retrieve → classify → FSM transition → mock
/// generation (streamed via `emit`) → persist. Split out so it can be
/// unit-tested without a live `AppHandle`.
pub(crate) async fn run_turn(
    db: &Db,
    chapter_id: i64,
    student_msg: &str,
    fsm_state: &str,
    emit: &(dyn Fn(StreamEvent) + Send + Sync),
) -> Result<TutorTurnResult, AppError> {
    let retriever = KeywordRetriever::new(db.clone());
    let classifier = HeuristicClassifier;
    let backend = MockBackend;

    let is_empty = student_msg.trim().is_empty();

    // Persist the student's message (skip when this is an opening probe).
    let student_msg_id = if is_empty {
        0
    } else {
        queries::insert_chat_message(
            db,
            chapter_id,
            "student",
            student_msg,
            fsm_state,
            now_secs(),
        )?
    };

    // Ground the generation with a retrieval query.
    let query = if is_empty {
        queries::get_chapter(db, chapter_id)
            .map(|c| format!("{} {}", c.title, c.summary))
            .unwrap_or_else(|_| "concept".to_string())
    } else {
        student_msg.to_string()
    };
    let snippets = retriever.retrieve(&query, chapter_id, 3)?;

    // Advance the Socratic FSM from the supplied state.
    let mut engine = SocraticEngine {
        state: parse_state(fsm_state),
        scaffold_turns: 0,
    };
    if !is_empty {
        let q = classifier.classify(student_msg, &keywords_from(&snippets));
        engine.transition(q);
    }

    let req = GenRequest {
        system: engine.directive().to_string(),
        user: student_msg.to_string(),
        state: engine.state.to_string(),
        context: snippets.iter().map(|s| s.body.clone()).collect(),
    };

    // Stream tokens to the UI while assembling the full reply.
    let full = Mutex::new(String::new());
    backend
        .generate(req, &|tok: String| {
            emit(StreamEvent::Token { text: tok.clone() });
            full.lock().unwrap().push_str(&tok);
        })
        .await?;
    let full = full.into_inner().unwrap();

    // Persist the tutor reply.
    let tutor_msg_id = queries::insert_chat_message(
        db,
        chapter_id,
        "tutor",
        &full,
        &engine.state.to_string(),
        now_secs(),
    )?;

    // Let the UI sync its local FSM state.
    emit(StreamEvent::StateChange {
        state: engine.state.to_string(),
    });

    Ok(TutorTurnResult {
        reply: full,
        fsm_state: engine.state.to_string(),
        student_msg_id,
        tutor_msg_id,
    })
}

#[tauri::command]
pub fn get_chat_history(
    state: State<'_, Db>,
    chapter_id: i64,
) -> Result<Vec<ChatMessage>, AppError> {
    queries::chat_history(&(*state).clone(), chapter_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::seed;
    use crate::models::StreamEvent;

    fn seeded() -> (Db, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("shala_tutor_{}.db", n));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();
        seed::seed_if_empty(&db).unwrap();
        (db, p)
    }

    #[test]
    fn opening_probe_persists_only_tutor_message() {
        let (db, p) = seeded();
        let noop = |_: StreamEvent| {};
        let res = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(super::run_turn(&db, 1, "", "Probe", &noop))
            .unwrap();
        assert_eq!(res.fsm_state, "Probe");
        assert!(!res.reply.is_empty());
        let hist = queries::chat_history(&db, 1).unwrap();
        // Empty student turn: only the tutor reply is stored.
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].role, "tutor");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn student_turn_persists_both_messages() {
        let (db, p) = seeded();
        // Opening.
        let noop = |_: StreamEvent| {};
        let rt = tokio::runtime::Runtime::new().unwrap();
        let _ = rt.block_on(super::run_turn(&db, 1, "", "Probe", &noop)).unwrap();
        // Student reply with keywords from the seeded pressure snippet.
        let res = rt
            .block_on(super::run_turn(
                &db,
                1,
                "pressure depends on force and area", // contains snippet keywords
                "Probe",
                &noop,
            ))
            .unwrap();
        assert!(!res.reply.is_empty());
        assert!(res.student_msg_id > 0);
        let hist = queries::chat_history(&db, 1).unwrap();
        // Opening tutor + student + tutor reply.
        assert_eq!(hist.len(), 3);
        assert_eq!(hist[1].role, "student");
        assert_eq!(hist[1].body, "pressure depends on force and area");
        let _ = std::fs::remove_file(&p);
    }
}
