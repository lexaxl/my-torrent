---
design_intent: S
design_status: not-started
---

# 01: Alex's Daily Download

**Project:** my-torrent
**Created:** 2026-07-27
**Method:** Whiteport Design Studio (WDS)

---

## Transaction (Q1)

**What this scenario covers:**
Open a torrent and see it start downloading — with zero setup on the path.

---

## Business Goal (Q2)

**Goal:** Daily adoption — Alex uses `my-torrent` daily in place of Transmission
**Objective:** Success Criteria (Product Brief) — "genuine daily use, judged by whether it sticks." This first-impression moment (frictionless start) is the load-bearing detail behind that criterion.

---

## User & Situation (Q3)

**Persona:** Alex (Primary — sole ICP, per Product Brief; Trigger Map skipped, strategic context folded into brief)
**Situation:** At his Mac, just found something he wants to download (e.g. a Linux ISO or software distribution) — either browsing in Safari or already has a saved `.torrent` file in Finder.

---

## Driving Forces (Q4)

**Hope:** It just works immediately, no dialog gets in the way.

**Worry:** It'll make him go through a setup wizard before anything even starts downloading.

---

## Device & Starting Point (Q5 + Q6)

**Device:** Desktop (Mac)
**Entry:** Double-clicks a `.torrent` file in Finder, or clicks a magnet link in the browser — the app opens/comes to the foreground with the download row already in progress.

---

## Best Outcome (Q7)

**User Success:**
Sees the download appear in the list with a live progress bar and speed within a second — no dialog interrupted the flow.

**Business Success:**
This zero-friction first moment is exactly what turns a one-time install into daily habitual use.

---

## Shortest Path (Q8)

1. **Главное окно (список закачек)** — Alex opens a `.torrent`/magnet link; a row appears instantly, already downloading on global defaults (save folder, seed duration), showing progress bar, download speed, upload speed, seeders, leechers ✓

*Single-step scenario — the entire transaction lives on one screen. Documented as a storyboard (states: empty → row added → downloading → complete) rather than a multi-page flow.*

---

## Trigger Map Connections

**Persona:** Alex (Primary — only persona, per Product Brief ICP)

**Driving Forces Addressed:**
- ✅ **Want:** "Set and forget" — zero-friction add flow (Product Concept: no mandatory add-dialog)
- ❌ **Fear:** Cluttered UI / friction-heavy add flows (named frustration with qBittorrent/others)

**Business Goal:** Daily adoption (Success Criteria) — the frictionless-add moment is the mechanism behind this goal.

---

## Scenario Steps

| Step | Folder | Purpose | Exit Action |
|------|--------|---------|-------------|
| 1.1 | `1.1-main-window/` | Open a torrent/magnet and see it start downloading immediately | Final — scenario success ✓ |

**First step** (1.1) includes full entry context (Q3 + Q4 + Q5 + Q6).
**On-step interactions** (states within the main window — empty, row added, downloading, complete) are documented as storyboard items within the page spec.
