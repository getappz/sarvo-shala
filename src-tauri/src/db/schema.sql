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

CREATE TABLE chat_messages (
  id INTEGER PRIMARY KEY,
  chapter_id INTEGER NOT NULL REFERENCES chapters(id),
  role TEXT NOT NULL CHECK (role IN ('student','tutor')),
  body TEXT NOT NULL,
  fsm_state TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL
);
