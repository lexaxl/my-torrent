# Design Log

**Project:** my-torrent
**Started:** 2026-07-27
**Method:** Whiteport Design Studio (WDS)

---

## Backlog

> Business-value items. Add links to detail files if needed.

- [x] Complete product brief — Phase 1
- [ ] ~~Define trigger map~~ — skipped per Phase 0 config (`strategic_analysis: simplified` = strategic context folded into brief, Phase 2 skipped entirely)
- [x] Outline user scenarios — Phase 3 (4 scenarios, 5 screens covered)
- [x] UX Design (page specifications, wireframes) — Phase 4 (all 4 scenarios, all 5 screen-appearances specified)

---

## Current

| Task | Started | Agent |
|------|---------|-------|
| — | — | — |

**Rules:** Mark what you start. Complete it when done (move to Log). One task at a time per agent.

---

## Design Loop Status

> Per-page design progress. Updated by agents at every design transition.

| Scenario | Step | Page | Status | Updated |
|----------|------|------|--------|---------|
| 01-alexs-daily-download | 1.1 | Главное окно | specified | 2026-07-27 |
| 02-alex-inspects-a-torrent | 2.1 | Главное окно (shared w/ 1.1) | specified | 2026-07-27 |
| 02-alex-inspects-a-torrent | 2.2 | Экран деталей торрента | specified | 2026-07-27 |
| 03-alex-sets-defaults-once | 3.1 | Настройки | specified | 2026-07-27 |
| 04-alex-glances-at-the-menu-bar | 4.1 | Меню-бар | specified | 2026-07-27 |

**Status values:** `discussed` → `wireframed` → `specified` → `explored` → `building` → `built` → `approved` | `removed`

**How to use:**
- **Append a row** when a page reaches a new status (do not overwrite — latest row per page is current status)
- **Read on startup** to see where the project stands and what to suggest next

---

## Log

### 2026-07-27 — Project initialized (Phase 0)
- Type: greenfield
- Complexity: complex (mapped from "macOS desktop app" — closest WDS category: Web Application)
- Tech stack: not yet decided
- Root folder: `design-artifacts/` (pre-existing from BMad install, reused as-is)

### 2026-07-27 — Product Brief complete (Phase 1)
- Native macOS torrent client, menu-bar quick access + main window as primary surface, per-torrent detail drill-down, zero-friction add flow.
- Free/open-source, no monetization. Target: macOS users who value minimalism.
- Success: near-zero idle resource use, daily personal adoption, any community feedback post open-source release.
- Constraints: $0 budget (unsigned build), Rust protocol engine + native macOS UI (fixed, protected over timeline), Apple HIG visual, Apple Silicon only, last 1-2 macOS versions, macOS-only permanently.
- Timeline: ~1 week for v1 — explicitly flagged as flexible/aspirational given protocol implementation complexity; architecture and budget take priority over hitting this date.
- Full detail: `A-Product-Brief/01-product-brief.md`, dialog trail in `A-Product-Brief/dialog/`.
- Next: Phase 2 (Trigger Mapping) — kept simplified per Phase 0 config.

### 2026-07-27 — UX Scenarios complete (Phase 3)

**Agent:** Saga (Scenario Outline)
**Scenarios:** 4 scenarios covering 4 screens (5 screen-appearances — main window used in 2 scenarios)
**Quality:** Excellent (7/7 completeness, 7/7 quality, 6/6 mistakes avoided, 4/4 best practices — all 4 scenarios)

**Artifacts Created:**
- `C-UX-Scenarios/00-ux-scenarios.md` — Scenario index with coverage matrix
- `C-UX-Scenarios/01-alexs-daily-download/01-alexs-daily-download.md` — Alex's Daily Download
- `C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/1.1-main-window.md` — Main window page spec
- `C-UX-Scenarios/02-alex-inspects-a-torrent/02-alex-inspects-a-torrent.md` — Alex Inspects a Torrent
- `C-UX-Scenarios/02-alex-inspects-a-torrent/2.1-main-window/2.1-main-window.md` — Main window (entry) page spec
- `C-UX-Scenarios/02-alex-inspects-a-torrent/2.2-torrent-detail/2.2-torrent-detail.md` — Torrent detail page spec
- `C-UX-Scenarios/03-alex-sets-defaults-once/03-alex-sets-defaults-once.md` — Alex Sets Defaults Once
- `C-UX-Scenarios/03-alex-sets-defaults-once/3.1-settings/3.1-settings.md` — Settings page spec
- `C-UX-Scenarios/04-alex-glances-at-the-menu-bar/04-alex-glances-at-the-menu-bar.md` — Alex Glances at the Menu Bar
- `C-UX-Scenarios/04-alex-glances-at-the-menu-bar/4.1-menu-bar/4.1-menu-bar.md` — Menu bar page spec

**Summary:** Trigger Map (Phase 2) was intentionally skipped per Phase 0 config (`strategic_analysis: simplified`); strategic context was pulled directly from the Product Brief's ICP instead, with "Alex" (the real project owner) standing in as the single persona rather than an invented Trigger Map persona. Site type classified as Dynamic App (Storyboard format) rather than a traditional page-flow site, since this is a native macOS desktop app. All 4 screens identified in scope analysis (main window, torrent detail, settings, menu bar) are covered with zero repetition. Suggest mode was used throughout — Saga drafted all 8 scenario questions per scenario from brief context, user confirmed each without corrections.

**Next:** Phase 4 — UX Design

### 2026-07-27 — UX Design complete (Phase 4, conceptual pass)

**Agent:** Freya (Suggest mode, per Phase 3 handover `design_intent: S` on all scenarios)
**Screens specified:** 5/5 screen-appearances (Главное окно ×2 — shared spec, Экран деталей торрента, Настройки, Меню-бар)

**Artifacts Created/Updated:**
- `C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/1.1-main-window.md` — full page spec + design sections
- `C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/Sketches/1.1-main-window-ascii.md` — ASCII layout (populated + empty states)
- `C-UX-Scenarios/02-alex-inspects-a-torrent/2.1-main-window/2.1-main-window.md` — lightweight spec referencing shared screen (1.1), scenario-specific exit action only
- `C-UX-Scenarios/02-alex-inspects-a-torrent/2.2-torrent-detail/2.2-torrent-detail.md` — full page spec (tabs: Files/Trackers/Peers)
- `C-UX-Scenarios/02-alex-inspects-a-torrent/2.2-torrent-detail/Sketches/2.2-torrent-detail-ascii.md` — ASCII layout (3 tab states)
- `C-UX-Scenarios/03-alex-sets-defaults-once/3.1-settings/3.1-settings.md` — full page spec
- `C-UX-Scenarios/03-alex-sets-defaults-once/3.1-settings/Sketches/3.1-settings-ascii.md` — ASCII layout
- `C-UX-Scenarios/04-alex-glances-at-the-menu-bar/4.1-menu-bar/4.1-menu-bar.md` — full page spec (click-to-popover interaction decided)
- `C-UX-Scenarios/04-alex-glances-at-the-menu-bar/4.1-menu-bar/Sketches/4.1-menu-bar-ascii.md` — ASCII layout (idle + active/popover states)

**Key decisions made during Phase 4:**
- Главное окно (main window) documented once in full (1.1) and referenced lightly from its second appearance (2.1) to avoid duplicate/conflicting specs for the same physical screen.
- Torrent detail screen structured as header + tab switcher (Files/Trackers/Peers), not one long scrollable list.
- Menu-bar interaction resolved as click-to-open popover (not hover-tooltip) — more reliable/standard for macOS status bar items; popover includes quick actions ("Открыть окно", "Настройки").
- All 5 screen-appearances given ASCII conceptual layouts (rough structural guides) rather than full visual sketches at this stage.

**Summary:** All 4 scenarios from Phase 3 now have conceptual page specifications covering purpose, entry points, mental state, user/business goals, and section-level structure. Suggest mode was used throughout, per the `design_intent: S` set at Phase 3 handover — Freya proposed each step, user confirmed with no corrections needed. This is a conceptual pass (ASCII layouts, section lists); full detailed specification (exact spacing/typography/copy) is deferred to `[P] Write Specifications`.

**Next:** `[P] Write Specifications` for full detail, or `[W] Visual Design` for styled prototypes — user's choice.

### 2026-07-27 — [P] Write Specifications: 1.1 Главное окно (full spec)

**Agent:** Freya
**Depth:** Lightweight per user's explicit choice — Object IDs, structure, interactions, states, RU/EN content, spacing/typography tokens (from `D-Design-System`). No hex colors or web-style hover/active/disabled states — those are deferred to actual UI-framework choice at Architecture, since native macOS system styling applies instead of arbitrary custom colors.

**Artifacts Updated:**
- `C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/1.1-main-window.md` — full implementation-ready specification (page metadata, layout structure, spacing/typography, 3 sections with 10 Object IDs, page states, 4 open questions flagged for later)

**Key decisions:**
- Adapted the web-oriented spec template: dropped SEO/responsive-breakpoint fields (N/A for native fixed-window app), used project's own `D-Design-System` space-*/text-* tokens instead of the step's separate `heading-*` token vocabulary, for consistency with the rest of the project.
- Flagged 4 open questions for later resolution (sort order, row context menu, multi-select, exact hover styling) rather than inventing answers not yet discussed.
- Deferred Design System component extraction — these are first-use components (list, table-header, progress-bar), per "extract on 2nd use" principle already documented in `D-Design-System/00-design-system.md`.

**Next:** Continue `[P] Write Specifications` for the remaining 3 screens (Экран деталей торрента, Настройки, Меню-бар), or switch to `[W] Visual Design`.

### 2026-07-27 — [P] Write Specifications: remaining 3 screens complete

**Agent:** Freya
**Screens specified:** Экран деталей торрента (2.2), Настройки (3.1), Меню-бар (4.1) — full implementation-ready specs, same lightweight depth as 1.1

**Artifacts Updated:**
- `C-UX-Scenarios/02-alex-inspects-a-torrent/2.2-torrent-detail/2.2-torrent-detail.md` — header + tab switcher (Files/Trackers/Peers) + 3 content-area sections, 15 Object IDs, Peers-empty state, 3 open questions
- `C-UX-Scenarios/03-alex-sets-defaults-once/3.1-settings/3.1-settings.md` — header + form (save location, seed duration radio group), instant-apply confirmed (no save button), 9 Object IDs, 3 open questions
- `C-UX-Scenarios/04-alex-glances-at-the-menu-bar/4.1-menu-bar/4.1-menu-bar.md` — icon + popover (stats/empty-state + actions), 9 Object IDs, reuses main-window empty-state translation key, 2 open questions

**Key decisions:**
- Settings: confirmed instant-apply on every change, no explicit save button — consistent with "set and forget" positioning.
- Torrent Detail: per-file include/exclude checkbox explicitly deferred (open question), not committed for v1 — avoids over-scoping beyond what Product Brief actually confirmed.
- Menu-bar popover reuses the exact empty-state translation key from Главное окно rather than duplicating copy.

**All 4 scenarios / 5 screen-appearances now have full page specifications.** Total open questions across all screens: 12 (sort order, row context menu, multi-select, hover styling, file-selection checkbox, presentation style ×2, seed-duration default/bounds, icon visual design, window-focus behavior).

**Next:** `[W] Visual Design` (styled prototypes) — natural next step now that all specs exist; or resolve open questions first via direct discussion; or move to Phase 3/6/7 per WDS roadmap (PRD Platform, Design System, Delivery).

### 2026-07-27 — Open questions resolved (10 of 12)

**Agent:** Freya

**Resolved:**
- Главное окно: no sorting in v1 (insertion order only); yes to basic row context menu (Pause/Resume, Remove, Show in Finder — added to spec); no multi-select in v1.
- Экран деталей торрента: presentation = separate resizable window; no per-file include/exclude checkbox in v1; no per-torrent seed-duration override (global setting only).
- Настройки: presentation = separate window (same decision as Torrent Detail); "N hours" default = 8, valid range = 1–168.
- Меню-бар: "Открыть окно" always brings the main window to front (not a show/hide toggle).

**Still open (deferred to `[W] Visual Design`, not product decisions):**
- Exact hover/selection visual treatment for torrent rows (1.1)
- Exact menu-bar icon visual design, idle vs. active states (4.1)

**Artifacts Updated:** all 4 page specs — `1.1-main-window.md` (added Row Context Menu section), `2.2-torrent-detail.md`, `3.1-settings.md`, `4.1-menu-bar.md` — Open Questions tables and relevant spec fields updated to reflect resolutions.

**Next:** `[W] Visual Design` — only 2 open items remain, both inherently visual and best resolved there.

### 2026-07-27 — Visual Design: HTML prototype, all 4 screens

**Agent:** Freya
**Tool:** HTML/CSS prototype (chosen over Figma/Stitch/Nano Banana/Excalidraw — no external MCP connector configured in this environment; HTML is buildable and reviewable directly via Artifact)

**Artifact Created:**
- `D-Design-System/01-Visual-Design/design-concepts/my-torrent-prototype.html` — single-file interactive prototype covering all 4 v1 screens (Главное окно incl. empty-state toggle, Экран деталей торрента with working Files/Trackers/Peers tabs, Настройки, Меню-бар popover)
- Published as Artifact: https://claude.ai/code/artifact/1390a924-4cd6-4f83-864d-ef26661e058a

**Design tokens established (first real values — supersede the placeholder "—" rows in `D-Design-System/00-design-system.md`):**
- Accent: muted indigo `#4C5FD5` (light) / `#7C8CF0` (dark) — deliberately not stock Apple blue
- Semantic: success (seeding) `#1F9254`/`#3FB873`, warning (tracker issue) `#B7791F`/`#D2A245`
- Typography: system font stack (`-apple-system, BlinkMacSystemFont, "SF Pro Text"...`) — intentional choice so the prototype renders as true SF Pro on an actual Mac, not a webfont substitute
- Full light + dark theme support via CSS custom properties

**Still open:** exact hover/selection treatment for torrent rows, and menu-bar icon design — both are now visible as concrete choices in the prototype (subtle row hover, generic "⇅" icon placeholder) but not yet finalized/tested.

**Next:** Review prototype with user; once approved, extract these first real design-system values into `D-Design-System/00-design-system.md` (Tokens/Components sections, currently placeholders) — per "extract, don't invent" now that a 2nd context (multiple screens) confirms the patterns. Then likely Phase 6 (Design System formalization) or Phase 3/2 (PRD Platform / Architecture) to move toward implementation.

### 2026-07-27 — Design System tokens extracted

**Agent:** Freya

**Artifacts Updated:**
- `D-Design-System/00-design-system.md` — filled in Spacing Scale (4px-grid, space-3xs=2px … space-3xl=64px), Type Scale (9px–24px, native-macOS-dense rather than web-scale, text-md=13px matches `NSFont.systemFontSize`), Color tokens (bg/surface/border/text + accent `#4C5FD5`/`#7C8CF0` + semantic success/warning, light+dark), and 2 first Components (List Row, Icon Button) — both confirmed via 2nd-use across page specs

**Next:** Decide project direction — continue deeper into WDS (Phase 6 Design System formalization, Phase 3 PRD Platform) or pivot to Architecture/implementation planning (tech-stack decisions for the Rust↔native-UI bridge, outside WDS's own scope).

### 2026-07-27 — Design Delivery: DD-001 Core Download Experience packaged and handed off

**Agent:** Freya

**Artifacts Created:**
- `E-Development/deliveries/DD-001-core-download-experience.yaml` — all 4 scenarios packaged as a single delivery (solo project, one release)
- `E-Development/test-scenarios/TS-001-core-download-experience.yaml` — 4 happy-path, 1 error-state, 3 edge-case, 3 design-system, 2 accessibility, 2 performance tests, sign-off criteria
- `E-Development/deliveries/DD-001-handoff-log.md` — artifact verification record (adapted: skipped the standard multi-role handoff-dialog role-play, not applicable to a solo project)

**Key decisions:**
- Packaged as one delivery rather than 4 — appropriate scale for a solo, single-release hobby project, not a multi-team parallel-stream product.
- Handover steps 03-05 (designer↔architect dialog, weekly check-ins, communication channel) adapted/skipped as inapplicable; replaced with an honest artifact-verification log instead of fabricated role-play.
- DD-001 status set to `ready`, not `in_development` — nothing has been built yet.

**Phase 4 (UX Design) is now fully complete: scenarios → specifications → visual prototype → design system tokens → packaged delivery.**

**Next:** Architecture phase (tech-stack decisions: SwiftUI vs. AppKit, Rust↔native-UI bridge mechanism) — outside WDS's own scope, would need a different track (e.g. BMad Method's Architecture workflow, already installed alongside WDS in this project per Phase 0 setup).

---

## About This Folder

- **This file** — Single source of truth for project progress
- **agent-experiences/** — Compressed insights from design discussions (dated files)
- **wds-project-outline.yaml** — Project configuration from Phase 0 setup

**Do not modify `wds-project-outline.yaml`** — it is the source of truth for project configuration.
