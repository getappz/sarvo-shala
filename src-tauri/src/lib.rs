mod db;
mod error;
mod models;

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
      let db_path = dir.join("swayam.db");
      let db = crate::db::Db::open_or_init(&db_path)?;
      crate::db::seed::seed_if_empty(&db)?;
      app.manage(db);
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
