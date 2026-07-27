use crate::db::Db;
use crate::error::AppError;
use crate::models::{Folder, FolderItem, VfsDocument, VfsDocumentMatch};
use crate::vfs::folders;
use crate::vfs::Vfs;
use tauri::State;

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[tauri::command]
pub fn vfs_create_folder(
    state: State<Db>,
    parent_id: Option<i64>,
    title: String,
) -> Result<Folder, AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::BadInput("title must not be empty".into()));
    }
    folders::create_folder(&state, parent_id, title, now())
}

#[tauri::command]
pub fn vfs_list_folders(state: State<Db>, parent_id: Option<i64>) -> Result<Vec<Folder>, AppError> {
    folders::list_folders(&state, parent_id)
}

#[tauri::command]
pub fn vfs_rename_folder(state: State<Db>, folder_id: i64, title: String) -> Result<(), AppError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(AppError::BadInput("title must not be empty".into()));
    }
    folders::rename_folder(&state, folder_id, title, now())
}

#[tauri::command]
pub fn vfs_delete_folder(state: State<Db>, folder_id: i64) -> Result<(), AppError> {
    folders::delete_folder(&state, folder_id)
}

#[tauri::command]
pub fn vfs_list_folder_items(
    state: State<Db>,
    folder_id: Option<i64>,
) -> Result<Vec<FolderItem>, AppError> {
    folders::list_items(&state, folder_id)
}

#[tauri::command]
pub fn vfs_remove_folder_item(
    state: State<Db>,
    folder_id: Option<i64>,
    item_type: String,
    item_id: String,
) -> Result<bool, AppError> {
    folders::remove_item(&state, folder_id, &item_type, &item_id)
}

/// Stores `content` in the sidecar vfs document store and files it into
/// `folder_id` (root if `None`) in one call -- the two writes are against
/// different SQLite databases, so this is best-effort, not transactional:
/// on a crash between them the document exists but isn't filed, which is
/// recoverable (it still shows up in search) rather than silently lost.
#[tauri::command]
pub fn vfs_import_document(
    db: State<Db>,
    vfs: State<Vfs>,
    folder_id: Option<i64>,
    path: String,
    title: String,
    content: String,
) -> Result<VfsDocument, AppError> {
    if path.trim().is_empty() {
        return Err(AppError::BadInput("path must not be empty".into()));
    }
    let doc = vfs.put_document(&path, &title, &content)?;
    folders::add_item(&db, folder_id, "document", &doc.id, now())?;
    Ok(doc)
}

#[tauri::command]
pub fn vfs_get_document(vfs: State<Vfs>, id: String) -> Result<Option<VfsDocument>, AppError> {
    vfs.get_document(&id)
}

#[tauri::command]
pub fn vfs_delete_document(vfs: State<Vfs>, id: String) -> Result<bool, AppError> {
    vfs.delete_document(&id)
}

#[tauri::command]
pub fn vfs_search_documents(
    vfs: State<Vfs>,
    query: String,
    limit: i64,
) -> Result<Vec<VfsDocumentMatch>, AppError> {
    if query.trim().is_empty() {
        return Err(AppError::BadInput("query must not be empty".into()));
    }
    let limit = limit.clamp(1, 100) as usize;
    vfs.search_documents(&query, limit)
}
