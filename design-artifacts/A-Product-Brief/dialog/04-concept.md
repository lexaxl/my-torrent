# Step 4: Product Concept

**Completed:** 2026-07-27
**Session:** 1

---

## Purpose

Capture the designer's STRUCTURAL vision - the founding principle or key feature that defines the product concept.

**Not just requirements - the IDEA.**

---

## Concept Exploration

**Agent asked:**
"Как вы видите организацию главного окна и процесса добавления торрента? Один плоский список, или у каждой закачки есть деталка? Диалог настроек при добавлении, или бросил файл — и оно просто качается?"

**User described:**
"Для каждой закачки есть отдельный экран/деталка с доп. инфой (файлы внутри, трекеры, пиры). Максимально «бросил файл — и оно просто начало качаться» с настройками по умолчанию, которые можно поменять глобально."

---

## Deep Dive

### Core Structural Idea

**The founding principle:**
"Zero friction on entry, depth on demand" — adding a torrent has no configuration step (drop file → it downloads immediately using global defaults); all complexity (files inside the torrent, trackers, peers) lives one level down, in a per-torrent detail view, kept out of the main list.

**Concrete example:**
User drops a `.torrent` file onto the app (or opens it) → it immediately appears in the main downloads list and starts downloading with the current global defaults (save location, seed duration, etc. — no per-add dialog interrupts the flow). If they want to inspect what's inside, check tracker/peer status, or override something for this specific download, they click into that torrent's detail screen.

### Why This Matters

**User's rationale:**
Not separately re-explored — this directly restates and operationalizes the already-confirmed "set and forget by default, configurable via settings" goal (Step 7 / Target Users) and the "basics done well, no bloat" differentiator (Step 3 / Positioning). Agent synthesized rationale and user confirmed without correction.

**Problem it solves:**
Avoids the two failure modes named as frustrations with existing clients: (1) a cluttered main list/UI trying to show everything at once, and (2) friction-heavy add flows. Progressive disclosure — simple surface, full power one click away — serves the "minimalism" value of the target user without removing capability.

---

## Reflection Checkpoint

**Agent's synthesis:**
"Организующий принцип — 'ноль трения на входе, глубина по запросу': добавление торрента максимально бесфрикционно (бросил `.torrent`/magnet-ссылку — оно тут же начало качаться на глобальных настройках по умолчанию, без диалога), а вся сложность (файлы внутри, трекеры, пиры) убрана на отдельный экран деталей конкретной закачки, не загромождая главный список."

**User response:**
- [x] Confirmed
- [ ] Corrected

**Corrections (if any):**
None.

---

## Concept Documentation

**Core concept:**
Zero-friction entry + on-demand depth: dropping/opening a torrent starts it immediately with global defaults; per-torrent detail (files, trackers, peers) lives in a drill-down view, not the main list.

**Implementation principle:**
Main window = flat list of active downloads (one row per torrent: progress bar, download speed, upload speed, seeders, leechers — per the original hard UI requirement). Clicking a row opens a detail view for that torrent (files inside, trackers, peers). Adding a torrent never opens a required configuration dialog — global default settings apply automatically, overridable later per-torrent or globally in Settings.

**Example:** Drop a `.torrent` → row appears in the list, downloading starts, seed duration follows the global default. Click the row to see which files are included, tracker status, and connected peers.

---

## Related Features

Features that stem from this concept:
1. Main list view — one row per torrent (progress, down/up speed, seeders, leechers) — no clutter, no secondary panels.
2. Per-torrent detail view — files inside the torrent, tracker status, peer list — reached by clicking a list row.
3. Global default settings (save location, seeding duration, etc.) applied automatically on add — no mandatory add-dialog.
4. Settings screen for global defaults, with the possibility of per-torrent overrides from the detail view (specifics to be refined during UX Design).

---

**Documented in:** `wds-project-outline.yaml` → `product_concept`
**Impacts:** Navigation structure, information architecture, feature priorities
