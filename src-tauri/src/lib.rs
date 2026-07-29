mod ai;
mod commands;
mod curriculum_gen;
mod db;
mod error;
mod models;
mod pedagogy;
mod vfs;

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
            let vfs_path = dir.join("vfs.db");
            let vfs = crate::vfs::Vfs::open_or_init(&vfs_path)?;
            app.manage(vfs);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_settings,
            commands::settings::complete_wizard,
            commands::curriculum::list_subjects,
            commands::curriculum::list_chapters,
            commands::curriculum::list_concepts,
            commands::curriculum::generate_curriculum,
            commands::quiz::start_quiz,
            commands::quiz::submit_answer,
            commands::quiz::finish_quiz,
            commands::progress::get_progress,
            commands::progress::get_due,
            commands::progress::get_streak,
            commands::progress::get_gamification,
            commands::tutor::tutor_turn,
            commands::tutor::get_chat_history,
            commands::vfs::vfs_create_folder,
            commands::vfs::vfs_list_all,
            commands::vfs::vfs_list_folders,
            commands::vfs::vfs_rename_folder,
            commands::vfs::vfs_delete_folder,
            commands::vfs::vfs_list_folder_items,
            commands::vfs::vfs_remove_folder_item,
            commands::vfs::vfs_import_document,
            commands::vfs::vfs_get_document,
            commands::vfs::vfs_delete_document,
            commands::vfs::vfs_search_documents,
            commands::vfs::vfs_list_documents
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
