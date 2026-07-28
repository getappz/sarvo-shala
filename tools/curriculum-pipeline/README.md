# curriculum-pipeline

Dev-tool CLI that turns an official board PDF into original, curriculum-aligned
content for Sarvo Shala. See `docs/superpowers/specs/2026-07-22-curriculum-authoring-pipeline-design.md`
for the full design rationale (legal posture, architecture, schema).

Internal/maintainer use only — no external-contributor workflow yet.

## Prerequisites

- Rust 1.77+ (managed via mise — see root `mise.toml`)
- `ANTHROPIC_API_KEY` env var (for the `generate` stage)

## Usage

```bash
# 1. Extract plain text from an official board PDF
cargo run -- extract --pdf path/to/board-chapter.pdf --out /tmp/chapter.txt

# 2. Generate original content from the extracted text
export ANTHROPIC_API_KEY=sk-...
cargo run -- generate \
  --text /tmp/chapter.txt \
  --subject Science \
  --grade 8 \
  --chapter-title "Force and Pressure" \
  --source-authority "CBSE Class 8 Science" \
  --out /tmp/science8-force-pressure.json

# 3. Validate the generated JSON (structural checks + lint)
cargo run -- validate --file /tmp/science8-force-pressure.json

# 4. After human review, compile into a shippable SQLite database
cargo run -- compile --content-dir content/ --out dist/grade8.db.zst
```

## Verification

```bash
cd tools/curriculum-pipeline
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
```
