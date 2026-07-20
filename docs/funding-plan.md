# Funding & Support Plan — Sarvo Shala

**Date:** 2026-07-20 · **Status:** DRAFT — staged plan, no applications submitted yet.
**Context:** Sarvo Shala is AGPL-3.0-licensed with a CLA (see [`LICENSE`](../LICENSE), [`CLA.md`](../CLA.md)), held by Appz Dev Solutions (https://appz.dev). The CLA is what makes the paid, cloud-hosted "bridge" service viable later without re-licensing contributors' work. This plan sequences funding/support options against that model — grants and credits first (no strings, available pre-traction), community funding once there are contributors/users, and monetization tooling (Polar.sh, Tidelift) only once the paid tier is real.

## Prerequisites (do before applying to anything)

- [ ] Push the repo to GitHub under the public org/account (every program below checks GitHub activity).
- [ ] Wire up the CLA-assistant bot referenced in `CONTRIBUTING.md`/`CLA.md`.
- [ ] Add `FUNDING.yml` once GitHub Sponsors / Open Collective usernames exist (placeholder links are worse than none — hold until Stage 3 setup).

## Stage 1 — Zero-cost infra credits (apply immediately, no traction required)

| Program | What it covers | Notes |
|---|---|---|
| Google Cloud for Startups | Cloud hosting credits | No-VC entry tier available for solo/early builders |
| AWS Activate — Founders tier | Cloud hosting credits ($1K+) | No VC partner required at this tier |
| Anthropic API credits (startup program) | Claude API usage | Relevant if the tutor's inference is Claude-backed |

These offset dev/hosting costs directly — apply as soon as the repo is public. Not project funding, but free money for infra you'd pay for anyway.

## Stage 2 — Grant funding (the real target; needs a working demo + README)

**Primary target: NLnet Foundation / NGI Zero.** EU-funded, €5k–€50k, no equity, rolling calls. Best fit here because "offline AI tutor" reads directly as digital-commons / education-access work, which is exactly NLnet's remit.
- [ ] Confirm current open NGI Zero call and its thematic fit (Commons Fund vs. other sub-funds) before drafting.
- [ ] Draft 1–2 page proposal once a demo-able build exists — frame around offline access + privacy-respecting AI tutoring, not generic "AI app" language.

Secondary (lower priority, only if a clear fit emerges):
- Sovereign Tech Agency — only if a reusable infra piece (not the app itself) spins out of this project.
- Education / Global-South-focused funders (Mozilla, Internet Society Foundation, EdTech-specific funds) — worth a look if positioning shifts explicitly toward underserved/low-connectivity learners.

## Stage 3 — Community funding (once there are contributors/users, 3–6 months out)

- **Open Collective** — transparent fiscal hosting for hosting/expense costs as the community grows. Set up `FUNDING.yml` at this point.
- **GitHub Sponsors** — zero-fee, direct-to-maintainer.
- **Polar.sh** — built specifically for the open-core / dual-license model this project already uses (AGPL core + commercial license via the CLA). Set this up when the paid cloud-hosted bridge is closer to real, not before — it's the natural home for license-key issuance and sponsor-gated features tied to that offering.

## Stage 4 — Later (only once the paid cloud tier ships and has customers)

- **Tidelift** — pays maintainers for maintenance/security guarantees; needs enterprise-ish adopters first, not a pre-launch move.
- Accelerators (GitHub Accelerator, etc.) — optional, only relevant if the project moves toward a VC-backed path rather than staying bootstrapped off grants/credits/Polar.sh revenue.

## Open decisions

- Whether to pursue a bootstrapped path (grants + credits + Polar.sh revenue from the hosted bridge) vs. a VC-backed path (accelerators, equity funding) — affects whether Stage 4 accelerators are ever worth pursuing.
- Final framing for the NLnet pitch (offline-access angle vs. Socratic-tutoring/pedagogy angle vs. both) — decide once the demo exists and README copy is settled.
