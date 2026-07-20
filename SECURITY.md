# Security Policy

## Reporting a Vulnerability

Please report security issues privately, not as a public GitHub issue:

- **GitHub**: [Create a private security advisory](https://github.com/getappz/sarvo-shala/security/advisories/new)
- **Response time**: best-effort acknowledgment within a few days (small, early-stage project)

## What Sarvo Shala Does (and Doesn't Do)

Sarvo Shala is a **Tauri desktop app**: a React/TypeScript frontend talking to a
Rust backend over Tauri's IPC boundary, designed to run as an offline-capable AI
tutor. The main risk surfaces are:

- **The Tauri IPC boundary** — commands exposed from `src-tauri/src` to the
  frontend are the trust boundary between untrusted webview content and
  privileged Rust code; new commands should be reviewed for what they let the
  frontend do (filesystem, process, network access).
- **Local data** — learner progress (mastery, due queue, streak) is stored
  locally on the user's device. Treat this as sensitive personal data even
  though it never leaves the machine by default.
- **Inference/network calls** — the project targets offline operation; any code
  path that reaches out to a network endpoint (e.g. a hosted inference or sync
  service, once that exists) should not be silently enabled by default.

**Does NOT** (at this stage):
- Ship a hosted backend — everything currently runs on-device.
- Require elevated privileges beyond what Tauri's standard capabilities grant.

This section will be revised as the hosted/cloud-bridge service (see
internal funding/roadmap docs) comes online — that service will
have its own, separate risk surface (network-facing API, auth, multi-tenant data).

## Automated Checks

Every push and PR runs:
- `npm run check` (TypeScript)
- `cargo fmt --check`, `cargo clippy`, `cargo test` (Rust, in `src-tauri/`)
- `cargo audit` (dependency CVE scan, in `src-tauri/`)
- [zizmor](https://github.com/zizmorcore/zizmor) (GitHub Actions workflow security linter)
