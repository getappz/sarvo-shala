use crate::error::AppError;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub mod queries;
pub mod seed;

/// Current schema version. Bump when `schema.sql` gains new tables/indexes so
/// existing databases are migrated via [`MIGRATION_SQL`] instead of re-seeded.
const SCHEMA_VERSION: i32 = 4;

/// Applied to any database whose `user_version` is below [`SCHEMA_VERSION`].
/// Uses `IF NOT EXISTS` so it is safe to re-run and to run after a full
/// `schema.sql` (which already includes these objects on a fresh DB).
const MIGRATION_SQL: &str = "
  CREATE UNIQUE INDEX IF NOT EXISTS uq_quiz_answer
    ON quiz_answers(session_id, question_id);
  CREATE UNIQUE INDEX IF NOT EXISTS uq_one_correct_option
    ON question_options(question_id) WHERE is_correct = 1;
  CREATE INDEX IF NOT EXISTS ix_chapters_subject ON chapters(subject_id);
  CREATE INDEX IF NOT EXISTS ix_concepts_chapter ON concepts(chapter_id);
  CREATE INDEX IF NOT EXISTS ix_chapter_text_chapter ON chapter_text(chapter_id);
  CREATE INDEX IF NOT EXISTS ix_questions_concept ON questions(concept_id);
  CREATE INDEX IF NOT EXISTS ix_question_options_question ON question_options(question_id);
  CREATE INDEX IF NOT EXISTS ix_quiz_answers_session ON quiz_answers(session_id);
  CREATE INDEX IF NOT EXISTS ix_chat_messages_chapter ON chat_messages(chapter_id);
  CREATE INDEX IF NOT EXISTS ix_concept_mastery_concept ON concept_mastery(concept_id);
  CREATE TABLE IF NOT EXISTS concept_mastery_milestones (
    concept_id INTEGER NOT NULL REFERENCES concepts(id),
    level TEXT NOT NULL CHECK (level IN ('practiced', 'mastered')),
    earned_at INTEGER NOT NULL,
    PRIMARY KEY (concept_id, level)
  );
  CREATE TABLE IF NOT EXISTS earned_badges (
    badge_id TEXT PRIMARY KEY,
    earned_at INTEGER NOT NULL
  );
";

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
            conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        } else if uv < SCHEMA_VERSION {
            conn.execute_batch(MIGRATION_SQL)?;
            conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        Ok(Db {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn with<T>(
        &self,
        f: impl FnOnce(&mut rusqlite::Connection) -> rusqlite::Result<T>,
    ) -> Result<T, AppError> {
        // Recover from a poisoned lock instead of panicking every later
        // command: a panic while a previous `db.with(..)` closure held the
        // lock poisons the mutex, but the data underneath is still usable.
        let mut conn = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        Ok(f(&mut conn)?)
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
        p.push(format!("shala_test_{}_{}.db", std::process::id(), n));
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
        assert_eq!(uv1, SCHEMA_VERSION);
        assert_eq!(uv2, SCHEMA_VERSION);
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
