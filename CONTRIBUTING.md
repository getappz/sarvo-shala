# Contributing to Sarvo Shala

Thanks for your interest in Sarvo Shala — contributions are welcome.

## Quick start

### Prerequisites

- Node.js (for the frontend) and npm
- Rust (stable) via [rustup](https://rustup.rs/) (for the Tauri backend)
- Tauri CLI prerequisites for your OS: see the [Tauri docs](https://v2.tauri.app/start/prerequisites/)

### Setup

```bash
git clone https://github.com/getappz/sarvo-shala.git
cd sarvo-shala

npm install
npm run dev        # frontend only, in the browser
# or
cargo tauri dev     # full app, with the Rust backend
```

### Quality bar (required)

```bash
npm run check              # TypeScript build check
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings  # currently advisory in CI, not yet blocking (see below)
cargo test
```

Clippy is not yet a hard CI gate: the codebase currently has intentional
dead-code scaffolding (inference/retriever/speech trait seams) that
`-D warnings` flags. Run it locally and fix what you introduce; it will
become blocking once the scaffolding is wired up.

## Repo structure

```text
swayam/
├── src/                    # React/TypeScript frontend
├── src-tauri/               # Tauri app shell + Rust backend
│   ├── src/
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/
├── index.html
└── package.json
```

## Pull requests

- Keep PRs focused (one theme per PR)
- Include a short test plan (commands you ran)
- All checks must pass before merging

## Contributor License Agreement (CLA)

Before your first pull request can be merged, you need to sign our
[Contributor License Agreement](CLA.md). It is a one-time, automated step: the
CLA Assistant bot comments on your PR, and you sign by replying:

> I have read the CLA Document and I hereby sign the CLA

The CLA keeps Sarvo Shala AGPL-3.0-licensed for everyone while allowing the
maintainer to also offer it (or works based on it) under separate commercial
terms — for example, as a hosted/cloud service.

## License

Sarvo Shala is distributed under the GNU Affero General Public License v3.0
(AGPL-3.0); by contributing, your contributions are licensed to the public under
the same terms (see the [CLA](CLA.md) for the full grant).
