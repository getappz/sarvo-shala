//! Generic folder tree, stored in the main app db (see `schema.sql`).
//! `folder_id`/`parent_id` of `None` means "at the root". Follows the same
//! `db.with(..)` repository style as `db::queries`.

use crate::db::Db;
use crate::error::AppError;
use crate::models::{Folder, FolderItem};
use rusqlite::OptionalExtension;

fn map_folder(r: &rusqlite::Row) -> rusqlite::Result<Folder> {
    Ok(Folder {
        id: r.get(0)?,
        parent_id: r.get(1)?,
        title: r.get(2)?,
        sort_order: r.get(3)?,
        created_at: r.get(4)?,
        updated_at: r.get(5)?,
    })
}

pub fn get_folder(db: &Db, id: i64) -> Result<Folder, AppError> {
    db.with(|c| {
        c.query_row(
            "SELECT id, parent_id, title, sort_order, created_at, updated_at FROM folders WHERE id = ?1",
            [id],
            map_folder,
        )
        .optional()
    })?
    .ok_or_else(|| AppError::NotFound(format!("folder {id}")))
}

pub fn create_folder(
    db: &Db,
    parent_id: Option<i64>,
    title: &str,
    now: i64,
) -> Result<Folder, AppError> {
    if let Some(pid) = parent_id {
        get_folder(db, pid)?; // NotFound rather than a raw FK-constraint error
    }
    let id = db.with(|c| {
        c.execute(
            "INSERT INTO folders (parent_id, title, sort_order, created_at, updated_at)
             VALUES (?1, ?2, 0, ?3, ?3)",
            (parent_id, title, now),
        )?;
        Ok(c.last_insert_rowid())
    })?;
    get_folder(db, id)
}

pub fn rename_folder(db: &Db, id: i64, title: &str, now: i64) -> Result<(), AppError> {
    let changed = db.with(|c| {
        c.execute(
            "UPDATE folders SET title = ?1, updated_at = ?2 WHERE id = ?3",
            (title, now, id),
        )
    })?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("folder {id}")));
    }
    Ok(())
}

/// Deletes the folder itself; children folders cascade-delete (FK
/// `ON DELETE CASCADE`) and any items directly in it fall back to root
/// (FK `ON DELETE SET NULL`) rather than being destroyed.
pub fn delete_folder(db: &Db, id: i64) -> Result<(), AppError> {
    let changed = db.with(|c| c.execute("DELETE FROM folders WHERE id = ?1", [id]))?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("folder {id}")));
    }
    Ok(())
}

pub fn list_folders(db: &Db, parent_id: Option<i64>) -> Result<Vec<Folder>, AppError> {
    let rows = db.with(|c| {
        let sql = "SELECT id, parent_id, title, sort_order, created_at, updated_at
                    FROM folders WHERE parent_id IS ?1 ORDER BY sort_order, id";
        let mut stmt = c.prepare(sql)?;
        let mapped = stmt.query_map([parent_id], map_folder)?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

fn map_folder_item(r: &rusqlite::Row) -> rusqlite::Result<FolderItem> {
    Ok(FolderItem {
        id: r.get(0)?,
        folder_id: r.get(1)?,
        item_type: r.get(2)?,
        item_id: r.get(3)?,
        sort_order: r.get(4)?,
        created_at: r.get(5)?,
    })
}

/// Idempotent: adding the same `(folder_id, item_type, item_id)` twice
/// returns the existing row rather than duplicating it. This is enforced in
/// application code, inside one `db.with` closure (so the check-then-insert
/// is atomic under the connection mutex), rather than via the unique index --
/// SQLite treats `NULL` as distinct from `NULL` in a unique index, so an
/// index alone can't dedupe items added at the root (`folder_id = NULL`).
pub fn add_item(
    db: &Db,
    folder_id: Option<i64>,
    item_type: &str,
    item_id: &str,
    now: i64,
) -> Result<FolderItem, AppError> {
    if let Some(fid) = folder_id {
        get_folder(db, fid)?;
    }
    let id = db.with(|c| {
        let existing: Option<i64> = c
            .query_row(
                "SELECT id FROM folder_items WHERE folder_id IS ?1 AND item_type = ?2 AND item_id = ?3",
                (folder_id, item_type, item_id),
                |r| r.get(0),
            )
            .optional()?;
        if let Some(id) = existing {
            return Ok(id);
        }
        c.execute(
            "INSERT INTO folder_items (folder_id, item_type, item_id, sort_order, created_at)
             VALUES (?1, ?2, ?3, 0, ?4)",
            (folder_id, item_type, item_id, now),
        )?;
        Ok(c.last_insert_rowid())
    })?;
    db.with(|c| {
        c.query_row(
            "SELECT id, folder_id, item_type, item_id, sort_order, created_at
             FROM folder_items WHERE id = ?1",
            [id],
            map_folder_item,
        )
    })
}

pub fn remove_item(
    db: &Db,
    folder_id: Option<i64>,
    item_type: &str,
    item_id: &str,
) -> Result<bool, AppError> {
    let changed = db.with(|c| {
        c.execute(
            "DELETE FROM folder_items WHERE folder_id IS ?1 AND item_type = ?2 AND item_id = ?3",
            (folder_id, item_type, item_id),
        )
    })?;
    Ok(changed > 0)
}

pub fn list_all_folders(db: &Db) -> Result<Vec<Folder>, AppError> {
    let rows = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, parent_id, title, sort_order, created_at, updated_at
             FROM folders ORDER BY sort_order, id",
        )?;
        let mapped = stmt.query_map([], map_folder)?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

pub fn list_all_items(db: &Db) -> Result<Vec<FolderItem>, AppError> {
    let rows = db.with(|c| {
        let mut stmt = c.prepare(
            "SELECT id, folder_id, item_type, item_id, sort_order, created_at
             FROM folder_items ORDER BY sort_order, id",
        )?;
        let mapped = stmt.query_map([], map_folder_item)?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

pub fn list_items(db: &Db, folder_id: Option<i64>) -> Result<Vec<FolderItem>, AppError> {
    let rows = db.with(|c| {
        let sql = "SELECT id, folder_id, item_type, item_id, sort_order, created_at
                    FROM folder_items WHERE folder_id IS ?1 ORDER BY sort_order, id";
        let mut stmt = c.prepare(sql)?;
        let mapped = stmt.query_map([folder_id], map_folder_item)?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()
    })?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db() -> (Db, std::path::PathBuf) {
        let n = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut p = std::env::temp_dir();
        p.push(format!("shala_folders_{n}.db"));
        let _ = std::fs::remove_file(&p);
        let db = Db::open_or_init(&p).unwrap();
        (db, p)
    }

    #[test]
    fn create_and_list_root_and_nested_folders() {
        let (db, p) = temp_db();
        let root_child = create_folder(&db, None, "Notes", 1000).unwrap();
        let nested = create_folder(&db, Some(root_child.id), "Biology", 1001).unwrap();

        let roots = list_folders(&db, None).unwrap();
        assert_eq!(
            roots.iter().map(|f| f.id).collect::<Vec<_>>(),
            vec![root_child.id]
        );

        let children = list_folders(&db, Some(root_child.id)).unwrap();
        assert_eq!(
            children.iter().map(|f| f.id).collect::<Vec<_>>(),
            vec![nested.id]
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn create_folder_with_missing_parent_is_not_found() {
        let (db, p) = temp_db();
        let err = create_folder(&db, Some(999), "Orphan", 1000).unwrap_err();
        assert!(matches!(err, AppError::NotFound(_)));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn deleting_a_folder_orphans_its_items_to_root_instead_of_destroying_them() {
        let (db, p) = temp_db();
        let folder = create_folder(&db, None, "Imports", 1000).unwrap();
        add_item(&db, Some(folder.id), "document", "doc_abc", 1001).unwrap();

        delete_folder(&db, folder.id).unwrap();

        let root_items = list_items(&db, None).unwrap();
        assert_eq!(
            root_items.len(),
            1,
            "item must survive its folder's deletion"
        );
        assert_eq!(root_items[0].item_id, "doc_abc");
        assert_eq!(
            root_items[0].folder_id, None,
            "orphaned item falls back to root"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn deleting_a_folder_cascades_to_its_children() {
        let (db, p) = temp_db();
        let parent = create_folder(&db, None, "Parent", 1000).unwrap();
        let child = create_folder(&db, Some(parent.id), "Child", 1001).unwrap();

        delete_folder(&db, parent.id).unwrap();

        assert!(matches!(
            get_folder(&db, child.id),
            Err(AppError::NotFound(_))
        ));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn add_item_is_idempotent_at_root_where_folder_id_is_null() {
        // Regression test: SQLite treats NULL as distinct from NULL in a
        // unique index, so relying on the index alone would let this
        // silently duplicate instead of deduping.
        let (db, p) = temp_db();
        let first = add_item(&db, None, "document", "doc_1", 1001).unwrap();
        let second = add_item(&db, None, "document", "doc_1", 1002).unwrap();
        assert_eq!(
            first.id, second.id,
            "re-adding the same root item must not duplicate it"
        );

        let items = list_items(&db, None).unwrap();
        assert_eq!(items.len(), 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn add_item_is_idempotent_in_a_folder() {
        let (db, p) = temp_db();
        let folder = create_folder(&db, None, "Notes", 1000).unwrap();
        let first = add_item(&db, Some(folder.id), "document", "doc_1", 1001).unwrap();
        let second = add_item(&db, Some(folder.id), "document", "doc_1", 1002).unwrap();
        assert_eq!(
            first.id, second.id,
            "re-adding the same item must not duplicate it"
        );

        let items = list_items(&db, Some(folder.id)).unwrap();
        assert_eq!(items.len(), 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn list_all_returns_every_folder_and_item_regardless_of_nesting() {
        let (db, p) = temp_db();
        let parent = create_folder(&db, None, "Parent", 1000).unwrap();
        let child = create_folder(&db, Some(parent.id), "Child", 1001).unwrap();
        add_item(&db, None, "document", "doc_root", 1002).unwrap();
        add_item(&db, Some(child.id), "document", "doc_nested", 1003).unwrap();

        let all_folders = list_all_folders(&db).unwrap();
        assert_eq!(
            all_folders.iter().map(|f| f.id).collect::<Vec<_>>(),
            vec![parent.id, child.id],
            "must include folders at every nesting depth, not just roots"
        );

        let all_items = list_all_items(&db).unwrap();
        assert_eq!(
            all_items
                .iter()
                .map(|i| i.item_id.as_str())
                .collect::<Vec<_>>(),
            vec!["doc_root", "doc_nested"],
            "must include items regardless of which folder (or root) they're in"
        );
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn remove_item_reports_whether_anything_was_removed() {
        let (db, p) = temp_db();
        let folder = create_folder(&db, None, "Notes", 1000).unwrap();
        add_item(&db, Some(folder.id), "document", "doc_1", 1001).unwrap();

        assert!(remove_item(&db, Some(folder.id), "document", "doc_1").unwrap());
        assert!(!remove_item(&db, Some(folder.id), "document", "doc_1").unwrap());
        let _ = std::fs::remove_file(&p);
    }
}
