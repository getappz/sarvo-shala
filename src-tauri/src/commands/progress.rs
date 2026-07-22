use crate::db::queries;
use crate::db::Db;
use crate::error::AppError;
use crate::models::ConceptProgress;
use tauri::State;

#[tauri::command]
pub fn get_progress(
    state: State<Db>,
    chapter_id: Option<i64>,
) -> Result<Vec<ConceptProgress>, AppError> {
    if let Some(id) = chapter_id {
        if id <= 0 {
            return Err(AppError::BadInput(format!(
                "chapter_id must be positive, got {id}"
            )));
        }
    }
    queries::get_progress(&state, chapter_id)
}

#[tauri::command]
pub fn get_due(state: State<Db>, now: i64) -> Result<Vec<ConceptProgress>, AppError> {
    queries::get_due(&state, now)
}

#[tauri::command]
pub fn get_streak(state: State<Db>, today: String) -> Result<i64, AppError> {
    queries::get_streak(&state, &today)
}

#[tauri::command]
pub fn get_gamification(
    state: State<Db>,
    today: String,
) -> Result<crate::models::GamificationSummary, AppError> {
    queries::get_gamification(&state, &today)
}

#[cfg(test)]
mod tests {
    use crate::db::queries;
    use crate::db::seed;
    use crate::db::Db;

    fn seeded() -> (Db, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("swayam_prog_{}.db", n));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();
        seed::seed_if_empty(&db).unwrap();
        (db, p)
    }

    #[test]
    fn streak_counts_consecutive() {
        let (db, p) = seeded();
        for d in ["2026-07-18", "2026-07-19", "2026-07-20"] {
            queries::upsert_study_day(&db, d).unwrap();
        }
        assert_eq!(queries::get_streak(&db, "2026-07-20").unwrap(), 3);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn streak_broken_by_gap() {
        let (db, p) = seeded();
        for d in ["2026-07-18", "2026-07-20"] {
            queries::upsert_study_day(&db, d).unwrap();
        }
        // today present, but yesterday (19) missing -> only the tail run counts
        assert_eq!(queries::get_streak(&db, "2026-07-20").unwrap(), 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn due_orders_soonest_first() {
        let (db, p) = seeded();
        queries::upsert_review_card(&db, 1, 1.0, 5.0, 1, 0, Some(0), Some(1000)).unwrap();
        queries::upsert_review_card(&db, 2, 1.0, 5.0, 1, 0, Some(0), Some(500)).unwrap();
        let due = queries::get_due(&db, 2000).unwrap();
        assert_eq!(due.len(), 2);
        assert_eq!(due[0].concept_id, 2); // due 500 before 1000
        assert_eq!(due[1].concept_id, 1);
        let _ = std::fs::remove_file(&p);
    }
}
