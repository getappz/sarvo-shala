# Sarvo Shala

Offline AI tutor for Indian school students (grades 6–12, CBSE). Runs fully on-device: a Rust + Tauri backend carries the pedagogy (BKT mastery, FSRS-4.5 spaced repetition, a Socratic dialogue FSM) and a React frontend renders the tutor, quiz, and progress views. No network calls — the "AI" is a deterministic local mock backend that stands in for a real on-device model at the M1 milestone.

## Stack

- **Backend:** Rust, Tauri 2, `rusqlite` (bundled SQLite), `tokio`.
- **Frontend:** React + TypeScript + Vite, `@tauri-apps/api`.
- **Tooling:** [`mise`](https://mise.jdx.dev) for pinned toolchains and tasks.

## Getting started

```sh
mise install            # pin node/rust/bun per mise.toml
mise run dev            # launch the Tauri desktop app (frontend + backend)
mise run web            # frontend only, in the browser (no Tauri commands)
```

## Tasks

| Task | Command | What it checks |
|------|---------|----------------|
| `dev` | `mise run dev` | Run the desktop app in dev mode |
| `web` | `mise run web` | Run the Vite frontend only |
| `check` | `mise run check` | Type-check the frontend (`tsc --noEmit`) |
| `test` | `mise run test` | Run backend Rust tests (`cargo test`) |
| `lint` | `mise run lint` | Clippy with `-D warnings` |
| `fmt-check` | `mise run fmt-check` | Verify Rust formatting |
| `verify` | `mise run verify` | **Acceptance gate** = `check` + `test` + `lint` + `fmt-check` |

Always run `mise run verify` before marking a change done.

## Architecture

```
src-tauri/src/
  pedagogy/      bkt.rs, fsrs.rs          # mastery + spaced-repetition scheduling
  ai/            socratic.rs, classify.rs, # Socratic FSM + heuristic answer classifier
                mock.rs, retriever.rs, mod.rs
  db/            schema.sql, queries.rs, seed.rs   # SQLite store + seed
  commands/      curriculum, quiz, tutor, progress, settings  # Tauri IPC handlers
  models.rs      shared request/response + StreamEvent types
src/screens/     Wizard, Browse, Tutor, Quiz, Progress
```

### Key invariants

- **Exactly one correct option** per question — enforced by a partial unique
  index (`uq_one_correct_option`) in `schema.sql`, not just a unit test.
- **One graded answer** per `(session, question)` — unique index
  (`uq_quiz_answer`) plus idempotent `submit_answer`, so a double-click can't
  double-count mastery or score.
- **Socratic escape hatch** — `scaffold_turns` is persisted in the FSM state
  string round-tripped to/from the frontend (`"Scaffold:3"`), so a student
  stuck in `Scaffold` is routed to `Confirm` after `MAX_SCAFFOLD_TURNS`.
- **Local-day streaks** — study days are recorded with the student's local
  calendar day (passed from the frontend), so pre-dawn study in IST isn't
  attributed to the previous UTC day.
- **Seeding is transactional** — a crash mid-seed can't leave half-populated
  tables; the guard re-runs the whole seed in one transaction.

## Data

Content is sample (explicitly **not** NCERT text) and lives in
`src-tauri/src/db/seed.rs`. To reset the local database during development:

```sh
mise run clean-db
```

## License

AGPL-3.0-only
