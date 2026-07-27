---
design_intent: S
design_status: not-started
---

# 03: Alex Sets Defaults Once

**Project:** my-torrent
**Created:** 2026-07-27
**Method:** Whiteport Design Studio (WDS)

---

## Transaction (Q1)

**What this scenario covers:**
Configure the app's default behavior once, so future downloads never need per-torrent setup.

---

## Business Goal (Q2)

**Goal:** "Set and forget" positioning / near-zero-friction daily use
**Objective:** Success Criteria + Product Concept (Product Brief) — directly implements the confirmed hard requirement that seeding duration be a configurable parameter.

---

## User & Situation (Q3)

**Persona:** Alex (Primary — sole ICP, per Product Brief)
**Situation:** Shortly after installing the app, before relying on it daily — wants to set save location and default seeding duration once.

---

## Driving Forces (Q4)

**Hope:** Sets this once and never has to think about it again.

**Worry:** Will have to redo this every single time a torrent is added.

---

## Device & Starting Point (Q5 + Q6)

**Device:** Desktop (Mac)
**Entry:** From the main window or the menu-bar menu — opens Settings (exact menu path to be resolved at Architecture/UX Design).

---

## Best Outcome (Q7)

**User Success:**
Sets default save folder and seeding duration in one pass, closes Settings, never needs to reopen it unless something changes.

**Business Success:**
Reinforces "basics done well" and directly satisfies the confirmed requirement for configurable seeding duration.

---

## Shortest Path (Q8)

1. **Настройки** — Alex opens Settings, sets default save location and seeding duration, closes ✓

---

## Trigger Map Connections

**Persona:** Alex (Primary — only persona, per Product Brief ICP)

**Driving Forces Addressed:**
- ✅ **Want:** Configure once, never think about it again ("set and forget")
- ❌ **Fear:** Repetitive, complex configuration on every torrent add

**Business Goal:** "Set and forget" positioning (Vision/Positioning) — directly implements the confirmed requirement that seeding duration is a global, configurable default.

---

## Scenario Steps

| Step | Folder | Purpose | Exit Action |
|------|--------|---------|-------------|
| 3.1 | `3.1-settings/` | Set default save location and seeding duration | Final — scenario success ✓ |

**First step** (3.1) includes full entry context (Q3 + Q4 + Q5 + Q6).
