use crate::db::queries;
use crate::db::Db;
use crate::error::AppError;
use crate::models::Settings;
use tauri::State;

#[tauri::command]
pub fn get_settings(state: State<Db>) -> Result<Settings, AppError> {
    let language = queries::get_setting(&state, "language")?
        .unwrap_or_else(|| "en".to_string());
    let grade = queries::get_setting(&state, "grade")?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(8);
    let board = queries::get_setting(&state, "board")?
        .unwrap_or_else(|| "CBSE".to_string());
    let onboarded = queries::get_setting(&state, "onboarded")?
        .map(|v| v == "true")
        .unwrap_or(false);
    Ok(Settings {
        language,
        grade,
        board,
        onboarded,
    })
}

#[tauri::command]
pub fn complete_wizard(
    state: State<Db>,
    language: String,
    grade: i64,
    board: String,
) -> Result<Settings, AppError> {
    if language != "en" && language != "hi" {
        return Err(AppError::BadInput(format!(
            "language must be 'en' or 'hi', got {language}"
        )));
    }
    if !(6..=12).contains(&grade) {
        return Err(AppError::BadInput(format!(
            "grade must be between 6 and 12, got {grade}"
        )));
    }
    if board != "CBSE" {
        return Err(AppError::BadInput(format!(
            "board must be 'CBSE', got {board}"
        )));
    }
    queries::set_setting(&state, "language", &language)?;
    queries::set_setting(&state, "grade", &grade.to_string())?;
    queries::set_setting(&state, "board", &board)?;
    queries::set_setting(&state, "onboarded", "true")?;
    Ok(Settings {
        language,
        grade,
        board,
        onboarded: true,
    })
}
