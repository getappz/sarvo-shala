use crate::error::AppError;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub mod queries;

pub struct Db {
    inner: Arc<Mutex<rusqlite::Connection>>,
}

impl Clone for Db {
    fn clone(&self) -> Self {
        Db {
            inner: self.inner.clone(),
        }
    }
}

impl Db {
    pub fn open_or_init(path: &Path) -> Result<Db, AppError> {
        let conn = rusqlite::Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        let uv: i32 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;
        if uv == 0 {
            conn.execute_batch(include_str!("schema.sql"))?;
            conn.pragma_update(None, "user_version", 1)?;
        }
        Ok(Db {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with<T>(
        &self,
        f: impl FnOnce(&rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Result<T, AppError> {
        let conn = self.inner.lock().unwrap();
        Ok(f(&conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_db() -> std::path::PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let mut p = std::env::temp_dir();
        p.push(format!("swayam_test_{}_{}.db", std::process::id(), n));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn open_twice_is_idempotent() {
        let p = temp_db();
        let db1 = Db::open_or_init(&p).unwrap();
        let db2 = Db::open_or_init(&p).unwrap();
        let uv1: i32 = db1
            .with(|c| c.query_row("PRAGMA user_version", [], |r| r.get(0)))
            .unwrap();
        let uv2: i32 = db2
            .with(|c| c.query_row("PRAGMA user_version", [], |r| r.get(0)))
            .unwrap();
        assert_eq!(uv1, 1);
        assert_eq!(uv2, 1);
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn schema_has_tables() {
        let p = temp_db();
        let db = Db::open_or_init(&p).unwrap();
        for t in ["concepts", "review_cards", "settings", "chat_messages"] {
            let count: i32 = db
                .with(|c| {
                    c.query_row(
                        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                        [t],
                        |r| r.get(0),
                    )
                })
                .unwrap();
            assert_eq!(count, 1, "expected table {t} to exist");
        }
        let _ = std::fs::remove_file(&p);
    }
}
