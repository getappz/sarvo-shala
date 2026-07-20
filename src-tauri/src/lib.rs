mod commands;
mod db;
mod error;
mod models;
mod pedagogy;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      let dir = app.path().app_data_dir()?;
      std::fs::create_dir_all(&dir)?;
      let db_path = dir.join("sarvo-shala.db");
      let db = crate::db::Db::open_or_init(&db_path)?;
      crate::db::seed::seed_if_empty(&db)?;
      app.manage(db);
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      commands::settings::get_settings,
      commands::settings::complete_wizard,
      commands::curriculum::list_subjects,
      commands::curriculum::list_chapters,
      commands::curriculum::list_concepts,
      commands::quiz::start_quiz,
      commands::quiz::submit_answer,
      commands::quiz::finish_quiz
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
