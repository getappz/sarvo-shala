use crate::error::AppError;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub mod queries;
pub mod seed;

/// Current schema version. Bump when `schema.sql` gains new tables/indexes so
/// existing databases are migrated via [`MIGRATION_SQL`] instead of re-seeded.
const SCHEMA_VERSION: i32 = 5;

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
  CREATE TABLE IF NOT EXISTS folders (
    id INTEGER PRIMARY KEY,
    parent_id INTEGER REFERENCES folders(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
  );
  CREATE TABLE IF NOT EXISTS folder_items (
    id INTEGER PRIMARY KEY,
    folder_id INTEGER REFERENCES folders(id) ON DELETE SET NULL,
    item_type TEXT NOT NULL,
    item_id TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at INTEGER NOT NULL
  );
  CREATE INDEX IF NOT EXISTS ix_folders_parent ON folders(parent_id);
  CREATE INDEX IF NOT EXISTS ix_folder_items_folder ON folder_items(folder_id);
  CREATE UNIQUE INDEX IF NOT EXISTS uq_folder_item ON folder_items(folder_id, item_type, item_id);
  ALTER TABLE concepts ADD COLUMN source_authority TEXT NOT NULL DEFAULT 'Sarvo Shala (original, not NCERT text)';
  ALTER TABLE concepts ADD COLUMN generation_method TEXT NOT NULL DEFAULT 'original' CHECK (generation_method IN ('original','llm_generated','user_uploaded'));
  ALTER TABLE concepts ADD COLUMN review_status TEXT NOT NULL DEFAULT 'reviewed' CHECK (review_status IN ('pending','reviewed'));
  ALTER TABLE chapter_text ADD COLUMN source_authority TEXT NOT NULL DEFAULT 'Sarvo Shala (original, not NCERT text)';
  ALTER TABLE chapter_text ADD COLUMN generation_method TEXT NOT NULL DEFAULT 'original' CHECK (generation_method IN ('original','llm_generated','user_uploaded'));
  ALTER TABLE chapter_text ADD COLUMN review_status TEXT NOT NULL DEFAULT 'reviewed' CHECK (review_status IN ('pending','reviewed'));
  ALTER TABLE questions ADD COLUMN source_authority TEXT NOT NULL DEFAULT 'Sarvo Shala (original, not NCERT text)';
  ALTER TABLE questions ADD COLUMN generation_method TEXT NOT NULL DEFAULT 'original' CHECK (generation_method IN ('original','llm_generated','user_uploaded'));
  ALTER TABLE questions ADD COLUMN review_status TEXT NOT NULL DEFAULT 'reviewed' CHECK (review_status IN ('pending','reviewed'));
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
    fn provenance_columns_exist_with_correct_defaults_after_migrating_an_old_db() {
        let p = temp_db();
        {
            let conn = rusqlite::Connection::open(&p).unwrap();
            conn.execute_batch(
                "CREATE TABLE subjects (id INTEGER PRIMARY KEY, grade INTEGER NOT NULL, name TEXT NOT NULL);
                 CREATE TABLE chapters (id INTEGER PRIMARY KEY, subject_id INTEGER NOT NULL REFERENCES subjects(id), ord INTEGER NOT NULL, title TEXT NOT NULL);
                 CREATE TABLE concepts (
                   id INTEGER PRIMARY KEY,
                   chapter_id INTEGER NOT NULL REFERENCES chapters(id),
                   ord INTEGER NOT NULL,
                   name TEXT NOT NULL,
                   key_formula TEXT NOT NULL DEFAULT '',
                   common_error TEXT NOT NULL DEFAULT '',
                   exam_weight INTEGER NOT NULL DEFAULT 1
                 );
                 CREATE TABLE chapter_text (
                   id INTEGER PRIMARY KEY,
                   chapter_id INTEGER NOT NULL REFERENCES chapters(id),
                   concept_id INTEGER REFERENCES concepts(id),
                   ord INTEGER NOT NULL,
                   body TEXT NOT NULL
                 );
                 CREATE TABLE questions (
                   id INTEGER PRIMARY KEY,
                   concept_id INTEGER NOT NULL REFERENCES concepts(id),
                   qtype TEXT NOT NULL,
                   bloom_level TEXT NOT NULL,
                   difficulty_layer INTEGER NOT NULL,
                   prompt TEXT NOT NULL,
                   trap TEXT NOT NULL DEFAULT '',
                   explanation TEXT NOT NULL DEFAULT ''
                 );
                 CREATE TABLE question_options (
                   id INTEGER PRIMARY KEY,
                   question_id INTEGER NOT NULL REFERENCES questions(id),
                   ord INTEGER NOT NULL,
                   body TEXT NOT NULL,
                   is_correct INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE TABLE quiz_sessions (
                   id INTEGER PRIMARY KEY,
                   chapter_id INTEGER NOT NULL,
                   started_at INTEGER NOT NULL,
                   finished_at INTEGER,
                   score REAL
                 );
                 CREATE TABLE quiz_answers (
                   id INTEGER PRIMARY KEY,
                   session_id INTEGER NOT NULL,
                   question_id INTEGER NOT NULL,
                   chosen_option_id INTEGER,
                   correct INTEGER NOT NULL,
                   answered_at INTEGER NOT NULL
                 );
                 CREATE TABLE chat_messages (
                   id INTEGER PRIMARY KEY,
                   chapter_id INTEGER NOT NULL,
                   role TEXT NOT NULL,
                   body TEXT NOT NULL,
                   fsm_state TEXT NOT NULL DEFAULT '',
                   created_at INTEGER NOT NULL
                 );
                 CREATE TABLE concept_mastery (
                   concept_id INTEGER PRIMARY KEY,
                   p_known REAL NOT NULL DEFAULT 0.2,
                   attempts INTEGER NOT NULL DEFAULT 0,
                   correct INTEGER NOT NULL DEFAULT 0,
                   updated_at INTEGER NOT NULL
                 );
                 INSERT INTO subjects (id, grade, name) VALUES (1, 8, 'Science');
                 INSERT INTO chapters (id, subject_id, ord, title) VALUES (1, 1, 1, 'Test Chapter');
                 INSERT INTO concepts (id, chapter_id, ord, name) VALUES (1, 1, 1, 'Old concept');
                 PRAGMA user_version = 4;",
            )
            .unwrap();
        }
        let db = Db::open_or_init(&p).unwrap();
        let (authority, method, status): (String, String, String) = db
            .with(|c| {
                c.query_row(
                    "SELECT source_authority, generation_method, review_status FROM concepts WHERE id = 1",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
            })
            .unwrap();
        assert_eq!(authority, "Sarvo Shala (original, not NCERT text)");
        assert_eq!(method, "original");
        assert_eq!(status, "reviewed");
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn schema_has_tables() {
        let p = temp_db();
        let db = Db::open_or_init(&p).unwrap();
        for t in [
            "concepts",
            "review_cards",
            "settings",
            "chat_messages",
            "folders",
            "folder_items",
        ] {
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
