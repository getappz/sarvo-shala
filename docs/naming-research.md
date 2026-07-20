# Naming Research — Swayam Rename

**Date:** 2026-07-20 · **Status:** research complete, decision pending
**Why:** "Swayam" collides with the Government of India's SWAYAM MOOC platform (swayam.gov.in, Ministry of Education). Pre-excluded roots: Swayam, Vidya, Guru, Gyan, Disha.

## Method

22 candidates over 2 waves, each verified across 4 lanes:
1. **Domains** — RDAP registration check (.com / .app / .in)
2. **Web / edtech / app stores** — existing apps, edtech products, brand & media collisions
3. **GitHub / OSS** — repos, orgs, npm, crates, username availability
4. **Trademark + government** — gov.in programs/schemes/portals (decisive, given the SWAYAM lesson), Indian TM web signals, incumbent similarity

> **Caveat:** IP India registry (tmrsearch.ipindia.gov.in) is captcha-gated — TM findings are web-proxy only. A formal class 9/41/42 search is required before final adoption.

Parent-platform fallback confirmed by owner: the app can live at `<brand>.sarvo.ai` (Sarvo = business AI automation platform), so standalone domain availability is a bonus, not a filter.

## Recommendation

### 1. Ojasvi ✦ (recommended)
*Sanskrit ojas — vitality; ojasvi = brilliant, radiant.* The only candidate of 22 with **no material conflict in any lane**: no govt scheme/portal, no edtech or consumer app, no notable GitHub/npm/crates holder. Only commercial holder anywhere is a small ethnic-wear brand (ojasvi.store). `ojasvi.app` is **unregistered**; `ojasvi.sarvo.ai` available by construction. Watch-only: Gujarat's OJAS job portal shares the root (different name, low risk); GitHub username held by individuals (org name e.g. `ojasvi-app` needed). 3 syllables, non-religious, meaning ("a brilliant one") fits an app that makes students shine.

### 2. Sopan (runner-up)
*Steps of a staircase — the learning-progression metaphor.* Clean on govt, TM-web, and GitHub. `sopan.app` unregistered. Cautions: SOPAN Mumbai autism NGO (non-edtech), an oil & gas EPC firm, two small IT shops, and common-Marathi-first-name search noise (journalist Sopan Deb).

### 3. Ekagra (third)
*Single-pointed focus.* Clean on govt and GitHub (a 2-star Ekagra.ai edu repo — watch). Caution: **ekagraapp.com** — an existing Indian consumer focus/screen-time app adjacent to the student space. `ekagra.app` unregistered.

Distant 4th/5th: **Vichar** (no decisive collision but generic; tiny quote apps hold the name) and **Chintan** (two coaching apps, un-searchable common name).

## Killed candidates (17)

| Name | Fatal conflict |
|---|---|
| Boojho / Bujho | NCERT-owned textbook character (Class 6–8 Science) — same govt-property failure as Swayam, despite all domains + GitHub/npm/crates being free |
| Jigyasa | CSIR JIGYASA national programme + NCERT's new Class 6 Science textbook is titled *Jigyasa* + existing learning apps |
| Jigyasu | Deceptively similar to the two govt Jigyasa properties; existing kids' science app + JigyasuOne AI-education platform |
| Prayas | prayas.nic.in (PMO/NIC) + DST NIDHI-PRAYAS + Physics Wallah's flagship "Prayas" JEE batches |
| Manthan | manthan.gov.in (Office of the Principal Scientific Adviser) + multiple edtech companies/apps |
| Medhavi | MP govt Medhavi Vidyarthi scholarship portal + Medhavi National Scholarship app — "Medhavi" already means *scholarship* to students |
| Seekho / Sikho | Seekho: VC-funded edtech, 100M+ downloads, GitHub org registered; sikho.ai AI-learning copilot; "Seekho aur Kamao" central scheme |
| Padho | padho.ai ("AI teacher for Indian students" — near-identical product) + multiple Padho learning apps + govt phrase-space |
| Khojo | "Khojo: Short Learning Videos" live on both app stores + near-collision with khoj-ai/khoj (~20k★ OSS AI assistant) |
| Kutuhal | "Kutuhal: AI-Driven Gamified Learning Platform" (MIT Solve 2024/25, Indian syllabus — same concept) + 30-year-old science-kit brand |
| Jugnu | Search owned by a banned adult-OTT app (I&B 2025 list) + Badshah hit song — unusable for a school app |
| Socho | Socho AI (sochoai.com) + a second Socho AI-education product — direct category collisions |
| Samjho | samjho.app is a sex-ed platform (reputational hazard) + Concept Samjho exam app |
| Pucho / Poocho | pucho.ai (agentic-AI startup) + Puch AI (high-profile WhatsApp AI agent) own the name space |
| Sawaal | Sawal AI education-assessment app + KBC/Bollywood swamping |
| Jaano | "Jaano" civic-participation app live on all three Indian app stores (borderline, treated as kill) |
| Anvay | Anvay Maritime Institute app + ANVAY agentic-AI CRM + negative news association |

## Next steps

1. **Decide**: Ojasvi (recommended) vs Sopan vs Ekagra.
2. Formal IP India TM search (classes 9, 41, 42) for the chosen name — captcha-gated, manual.
3. Native-speaker pronunciation sanity check across the 7 target languages (Ojasvi's "sv" cluster is the one thing to confirm in Tamil/Bengali).
4. Register `<name>.app` (free for all three finalists) + create `<name>.sarvo.ai`; claim GitHub org (`<name>-app` style), npm/crates names (all free).
5. Execute rename: repo dir, Tauri identifier `org.swayam.app` → e.g. `ai.sarvo.ojasvi`, work item #1, spec/plan docs — before opencode's MVP-0 work goes far.
