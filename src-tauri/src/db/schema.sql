CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE subjects (
  id INTEGER PRIMARY KEY,
  grade INTEGER NOT NULL,
  name TEXT NOT NULL,
  name_hi TEXT NOT NULL DEFAULT ''
);

CREATE TABLE chapters (
  id INTEGER PRIMARY KEY,
  subject_id INTEGER NOT NULL REFERENCES subjects(id),
  ord INTEGER NOT NULL,
  title TEXT NOT NULL,
  title_hi TEXT NOT NULL DEFAULT '',
  summary TEXT NOT NULL DEFAULT ''
);

CREATE TABLE concepts (
  id INTEGER PRIMARY KEY,
  chapter_id INTEGER NOT NULL REFERENCES chapters(id),
  ord INTEGER NOT NULL,
  name TEXT NOT NULL,
  key_formula TEXT NOT NULL DEFAULT '',
  common_error TEXT NOT NULL DEFAULT '',
  exam_weight INTEGER NOT NULL DEFAULT 1  -- 1..5
);

CREATE TABLE concept_prereqs (
  concept_id INTEGER NOT NULL REFERENCES concepts(id),
  prereq_id  INTEGER NOT NULL REFERENCES concepts(id),
  PRIMARY KEY (concept_id, prereq_id)
);

CREATE TABLE chapter_text (
  id INTEGER PRIMARY KEY,
  chapter_id INTEGER NOT NULL REFERENCES chapters(id),
  concept_id INTEGER REFERENCES concepts(id),
  ord INTEGER NOT NULL,
  body TEXT NOT NULL            -- ORIGINAL sample content, never verbatim NCERT
);

CREATE TABLE questions (
  id INTEGER PRIMARY KEY,
  concept_id INTEGER NOT NULL REFERENCES concepts(id),
  qtype TEXT NOT NULL CHECK (qtype IN ('mcq')),   -- MVP-0: mcq only
  bloom_level TEXT NOT NULL CHECK (bloom_level IN ('remember','understand','apply','analyze','evaluate')),
  difficulty_layer INTEGER NOT NULL CHECK (difficulty_layer IN (1,2,3)),
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
  chapter_id INTEGER NOT NULL REFERENCES chapters(id),
  started_at INTEGER NOT NULL,   -- unix seconds
  finished_at INTEGER,
  score REAL
);

CREATE TABLE quiz_answers (
  id INTEGER PRIMARY KEY,
  session_id INTEGER NOT NULL REFERENCES quiz_sessions(id),
  question_id INTEGER NOT NULL REFERENCES questions(id),
  chosen_option_id INTEGER REFERENCES question_options(id),
  correct INTEGER NOT NULL,
  answered_at INTEGER NOT NULL
);

-- BKT: one row per concept; p_known updated after every graded interaction
CREATE TABLE concept_mastery (
  concept_id INTEGER PRIMARY KEY REFERENCES concepts(id),
  p_known REAL NOT NULL DEFAULT 0.2,
  attempts INTEGER NOT NULL DEFAULT 0,
  correct INTEGER NOT NULL DEFAULT 0,
  updated_at INTEGER NOT NULL
);

-- FSRS: one row per concept card
CREATE TABLE review_cards (
  concept_id INTEGER PRIMARY KEY REFERENCES concepts(id),
  stability REAL NOT NULL DEFAULT 0,
  difficulty REAL NOT NULL DEFAULT 0,
  reps INTEGER NOT NULL DEFAULT 0,
  lapses INTEGER NOT NULL DEFAULT 0,
  last_review INTEGER,           -- unix seconds
  due INTEGER                    -- unix seconds
);

CREATE TABLE study_days (
  day TEXT PRIMARY KEY            -- 'YYYY-MM-DD' local date
);

-- Append-only fact log, same spirit as study_days: once a concept reaches a
-- mastery tier it stays recorded here forever, even if its live p_known
-- later drops from a wrong spaced-repetition review answer. Gamification
-- XP/badges read from this table, never from a live p_known snapshot, so
-- they cannot regress from ordinary review (see design spec §6).
CREATE TABLE concept_mastery_milestones (
  concept_id INTEGER NOT NULL REFERENCES concepts(id),
  level TEXT NOT NULL CHECK (level IN ('practiced', 'mastered')),
  earned_at INTEGER NOT NULL,
  PRIMARY KEY (concept_id, level)
);

-- Same append-only permanence for badges: once earned, always shown.
CREATE TABLE earned_badges (
  badge_id TEXT PRIMARY KEY,
  earned_at INTEGER NOT NULL
);

CREATE TABLE chat_messages (
  id INTEGER PRIMARY KEY,
  chapter_id INTEGER NOT NULL REFERENCES chapters(id),
  role TEXT NOT NULL CHECK (role IN ('student','tutor')),
  body TEXT NOT NULL,
  fsm_state TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL
);

-- Unified data layer (VFS): a generic folder tree that can hold items from
-- outside the curriculum schema (e.g. documents stored in the sidecar
-- agentflare-store vfs.db, see src-tauri/src/vfs/). item_id is opaque and
-- deliberately not an FK -- the referenced content lives in a different
-- SQLite database, so there is nothing local to enforce.
CREATE TABLE folders (
  id INTEGER PRIMARY KEY,
  parent_id INTEGER REFERENCES folders(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

-- Deleting a folder does not delete its items -- they fall back to root
-- (folder_id NULL) rather than silently destroying imported content.
CREATE TABLE folder_items (
  id INTEGER PRIMARY KEY,
  folder_id INTEGER REFERENCES folders(id) ON DELETE SET NULL,
  item_type TEXT NOT NULL,
  item_id TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL
);

-- One graded answer per (session, question): stops a fast double-click from
-- double-counting mastery and score.
CREATE UNIQUE INDEX IF NOT EXISTS uq_quiz_answer
  ON quiz_answers(session_id, question_id);

-- Exactly one correct option per question (invariant enforced at the DB
-- level, not just by a unit test against the hardcoded seed).
CREATE UNIQUE INDEX IF NOT EXISTS uq_one_correct_option
  ON question_options(question_id) WHERE is_correct = 1;

-- FK indexes (content grows per the roadmap).
CREATE INDEX IF NOT EXISTS ix_chapters_subject ON chapters(subject_id);
CREATE INDEX IF NOT EXISTS ix_concepts_chapter ON concepts(chapter_id);
CREATE INDEX IF NOT EXISTS ix_chapter_text_chapter ON chapter_text(chapter_id);
CREATE INDEX IF NOT EXISTS ix_questions_concept ON questions(concept_id);
CREATE INDEX IF NOT EXISTS ix_question_options_question ON question_options(question_id);
CREATE INDEX IF NOT EXISTS ix_quiz_answers_session ON quiz_answers(session_id);
CREATE INDEX IF NOT EXISTS ix_chat_messages_chapter ON chat_messages(chapter_id);
CREATE INDEX IF NOT EXISTS ix_concept_mastery_concept ON concept_mastery(concept_id);
CREATE INDEX IF NOT EXISTS ix_folders_parent ON folders(parent_id);
CREATE INDEX IF NOT EXISTS ix_folder_items_folder ON folder_items(folder_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_folder_item ON folder_items(folder_id, item_type, item_id);
