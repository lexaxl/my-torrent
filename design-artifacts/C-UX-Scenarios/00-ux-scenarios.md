# UX Scenarios: my-torrent

> Scenario outlines connecting Product Brief context to concrete user journeys (Trigger Map was skipped per Phase 0 config — strategic context lives in the Product Brief)

**Created:** 2026-07-27
**Author:** Alex with Saga
**Method:** Whiteport Design Studio (WDS)

---

## Scenario Summary

| ID | Scenario | Persona | Pages | Priority | Status |
|----|----------|---------|-------|----------|--------|
| 01 | Alex's Daily Download | Alex | 1 | ⭐ P1 | ✅ Outlined |
| 02 | Alex Inspects a Torrent | Alex | 2 | ⭐ P1 | ✅ Outlined |
| 03 | Alex Sets Defaults Once | Alex | 1 | P2 | ✅ Outlined |
| 04 | Alex Glances at the Menu Bar | Alex | 1 | P2 | ✅ Outlined |

---

## Scenarios

### [01: Alex's Daily Download](01-alexs-daily-download/01-alexs-daily-download.md)
**Persona:** Alex — Want: "set and forget" zero-friction add / Fear: cluttered UI, add-flow friction
**Pages:** Главное окно (список закачек)
**User Value:** Opens a torrent/magnet, sees it downloading instantly with no setup
**Business Value:** The frictionless first moment that drives daily adoption

---

### [02: Alex Inspects a Torrent](02-alex-inspects-a-torrent/02-alex-inspects-a-torrent.md)
**Persona:** Alex — Want: control on demand / Fear: feeling like a "black box"
**Pages:** Главное окно (список закачек), Экран деталей торрента
**User Value:** Sees files, tracker status, and peers for a torrent without leaving the simple main view
**Business Value:** Reinforces trust/transparency behind the "power without bloat" positioning

---

### [03: Alex Sets Defaults Once](03-alex-sets-defaults-once/03-alex-sets-defaults-once.md)
**Persona:** Alex — Want: configure once, never again / Fear: repetitive per-torrent configuration
**Pages:** Настройки
**User Value:** Sets default save location and seeding duration in one pass
**Business Value:** Delivers on the "set and forget" positioning and the confirmed configurable-seeding-duration requirement

---

### [04: Alex Glances at the Menu Bar](04-alex-glances-at-the-menu-bar/04-alex-glances-at-the-menu-bar.md)
**Persona:** Alex — Want: quick glance without opening a window / Fear: intrusive UI
**Pages:** Меню-бар
**User Value:** Sees aggregate download/upload speed at a glance, without breaking focus
**Business Value:** Justifies the menu-bar quick-access layer and the "lightweight, unobtrusive" differentiator

---

## Page Coverage Matrix

| Page | Scenario | Purpose in Flow |
|------|----------|----------------|
| Главное окно (список закачек) | 01 | See a newly opened torrent start downloading with zero setup |
| Главное окно (список закачек) | 02 | Identify and select a torrent to inspect |
| Экран деталей торрента | 02 | See files, tracker status, and peers for a torrent |
| Настройки | 03 | Set default save location and seeding duration |
| Меню-бар | 04 | See aggregate download/upload speed at a glance |

**Coverage:** 4/4 screens assigned to scenarios (Главное окно appears in two scenarios by design — same screen, two distinct transactions, no duplication of scope)

---

## Next Phase

These scenario outlines feed into **Phase 4: UX Design** where each page gets:
- Detailed page specifications
- Wireframe sketches
- Component definitions
- Interaction details

---

## What Belongs Here

Scenarios organize the product into meaningful user journeys. Each scenario groups related pages. Each page gets a full specification that a developer can build from.

**Folder structure per scenario:**
```
C-UX-Scenarios/
├── 00-ux-scenarios.md          ← This file (scenario guide + page index)
├── 01-scenario-name/
│   ├── 1.1-page-name/
│   │   ├── 1.1-page-name.md   ← Page specification
│   │   └── Sketches/           ← Wireframes and concepts
│   ├── 1.2-page-name/
│   │   ├── 1.2-page-name.md
│   │   └── Sketches/
│   └── ...
├── 02-scenario-name/
│   └── ...
├── Components/                  ← Shared component specs
└── Features/
    └── Storyboards/             ← Multi-step interaction flows
```

**Learn more:**
- WDS Course Module 08: Outline Scenarios — Design Experiences Not Screens
- WDS Course Module 09: Conceptual Sketching
- WDS Course Module 10: Storyboarding
- WDS Course Module 11: Conceptual Specifications
- WDS Course Tutorial 08: From Trigger Map to Scenarios

---

## For Agents

### Scenario Outline (Saga)
**Workflow:** `skill:wds-3-scenarios`
**Agent trigger:** `SC` (Saga)

### Page Specifications (Freya)
**Workflow:** `skill:wds-4-ux-design`
**Agent trigger:** `UX` (Freya)
**Page template:** `./resources/wds-4-ux-design/templates/page-specification.template.md`
**Scenario template:** `./resources/wds-4-ux-design/templates/scenario-overview.template.md`
**Quality guide:** `./resources/agent-guides/freya/specification-quality.md`
**Object types:** `./resources/wds-4-ux-design/object-types/`

### Specification Audit (Freya)
**Workflow:** `skill:wds-4-ux-design`
**Agent trigger:** `SA` (Freya)

**Before writing any page specification:**
1. Read `A-Product-Brief/01-product-brief.md` §Ideal Customer Profile — know Alex's driving forces (Trigger Map/personas were skipped; this is the source of truth instead)
2. Read the page specification template — use it as your scaffold, not memory
3. Discuss the page purpose with the user before filling in details
4. Each page folder needs a `Sketches/` subfolder for wireframes

**Harm:** Producing page specs from memory of what the template "roughly" contains. Plausible-looking specs that use wrong structure break the pipeline — developers can't trust them, audits can't validate them, and the user must correct what should have been right.

**Help:** Reading the actual template into context, discussing page purpose with the user, then filling the template with specific content. Specs that follow the template work across projects, pass audits, and give developers confidence.

---

_Generated with Whiteport Design Studio framework_
