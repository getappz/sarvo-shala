use crate::curriculum_gen;
use crate::db::queries;
use crate::db::Db;
use crate::error::AppError;
use crate::models::{Chapter, Concept, Subject};
use tauri::State;

#[tauri::command]
pub fn list_subjects(state: State<Db>, grade: i64) -> Result<Vec<Subject>, AppError> {
    if !(1..=12).contains(&grade) {
        return Err(AppError::BadInput(format!(
            "grade must be between 1 and 12, got {grade}"
        )));
    }
    queries::list_subjects(&state, grade)
}

#[tauri::command]
pub fn list_chapters(state: State<Db>, subject_id: i64) -> Result<Vec<Chapter>, AppError> {
    if subject_id <= 0 {
        return Err(AppError::BadInput(format!(
            "subject_id must be positive, got {subject_id}"
        )));
    }
    queries::list_chapters(&state, subject_id)
}

#[tauri::command]
pub fn list_concepts(state: State<Db>, chapter_id: i64) -> Result<Vec<Concept>, AppError> {
    if chapter_id <= 0 {
        return Err(AppError::BadInput(format!(
            "chapter_id must be positive, got {chapter_id}"
        )));
    }
    queries::list_concepts(&state, chapter_id)
}

#[tauri::command]
pub async fn generate_curriculum(
    state: tauri::State<'_, Db>,
    grade: i64,
    name: String,
    name_hi: Option<String>,
    num_chapters: Option<i64>,
) -> Result<Subject, AppError> {
    if !(1..=12).contains(&grade) {
        return Err(AppError::BadInput(format!(
            "grade must be between 1 and 12, got {grade}"
        )));
    }
    let nch = num_chapters.unwrap_or(3).clamp(1, 10);
    let name_hi = name_hi.filter(|s| !s.is_empty()).unwrap_or_default();
    let db = (*state).clone();

    tokio::task::spawn_blocking(move || {
        curriculum_gen::generate_curriculum(&db, grade, &name, &name_hi, nch)
    })
    .await
    .map_err(|e| AppError::Engine(format!("task join: {e}")))?
}

#[cfg(test)]
mod tests {
    use crate::db::seed;
    use crate::db::Db;

    fn seeded() -> (Db, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("shala_cur_{}.db", n));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();
        seed::seed_if_empty(&db).unwrap();
        (db, p)
    }

    #[test]
    fn list_concepts_returns_three_for_chapter_one() {
        let (db, p) = seeded();
        let concepts = crate::db::queries::list_concepts(&db, 1).unwrap();
        assert_eq!(concepts.len(), 3);
        let _ = std::fs::remove_file(&p);
    }
}
