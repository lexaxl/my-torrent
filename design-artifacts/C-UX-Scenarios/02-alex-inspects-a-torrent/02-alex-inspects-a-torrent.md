---
design_intent: S
design_status: not-started
---

# 02: Alex Inspects a Torrent

**Project:** my-torrent
**Created:** 2026-07-27
**Method:** Whiteport Design Studio (WDS)

---

## Transaction (Q1)

**What this scenario covers:**
Check what's inside a specific torrent — files, tracker status, connected peers — without leaving the simplicity of the main list.

---

## Business Goal (Q2)

**Goal:** Trust / community credibility
**Objective:** Competitive Landscape (Product Brief) — reinforces the "not a black box" positioning; indirectly supports daily adoption, since control-on-demand reduces frustration.

---

## User & Situation (Q3)

**Persona:** Alex (Primary — sole ICP, per Product Brief)
**Situation:** Mid-download, notices something atypical (e.g. slow speed) or simply wants to confirm which files are included before the download finishes.

---

## Driving Forces (Q4)

**Hope:** Sees exactly what's happening without digging through menus.

**Worry:** Will have to remember some obscure shortcut or menu item to find this.

---

## Device & Starting Point (Q5 + Q6)

**Device:** Desktop (Mac)
**Entry:** From the main window — clicks a row for a specific torrent in the downloads list.

---

## Best Outcome (Q7)

**User Success:**
Instantly sees the files included, tracker status, and connected peers for that torrent; understands status, closes it, moves on.

**Business Success:**
Reinforces trust that the app isn't hiding anything — supports the "power without bloat" positioning.

---

## Shortest Path (Q8)

1. **Главное окно (список закачек)** — Alex clicks a row for a specific torrent
2. **Экран деталей торрента** — sees files included, tracker status, and connected peers ✓

---

## Trigger Map Connections

**Persona:** Alex (Primary — only persona, per Product Brief ICP)

**Driving Forces Addressed:**
- ✅ **Want:** Control by request without complicating the main screen (Product Concept: "depth on demand")
- ❌ **Fear:** Feeling like a "black box" / complex, hard-to-find configuration

**Business Goal:** Trust/community credibility (Competitive Landscape) — the unfair advantage argument (fresh, unencumbered codebase) only holds if the app is genuinely transparent, not opaque.

---

## Scenario Steps

| Step | Folder | Purpose | Exit Action |
|------|--------|---------|-------------|
| 2.1 | `2.1-main-window/` | See the downloads list, identify the torrent to inspect | Clicks a row → opens torrent detail |
| 2.2 | `2.2-torrent-detail/` | See files, tracker status, and peers for that torrent | Final — scenario success ✓ |

**First step** (2.1) includes full entry context (Q3 + Q4 + Q5 + Q6).
