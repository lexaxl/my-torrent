# Product Brief Dialog: my-torrent

**Agent:** Saga (Product Brief Analyst)
**Project:** my-torrent
**Started:** 2026-07-27
**Status:** completed
**Last Updated:** 2026-07-27

---

## About This Dialog

This dialog tracks the Product Brief discovery process - the conversations, reflections, decisions, and synthesis that led to the documented brief.

---

## Project Context

**Client/Stakeholder:** Alex (owner/sole stakeholder)
**Designer:** Alex (Project Manager role, working with Saga/Freya)
**Sign-off Authority:** Alex
**Project Type:** internal (personal/hobby project)

**Working Relationship:**
Personal-stakes hobby project. Balanced involvement — Saga asks structured questions, Alex decides. Recommendations should always include rationale (not just options, not pure direction). Minimal documentation, trust-based, casual tone.

---

## Progress Tracker

- [x] Phase 0 — Project Setup (greenfield, complex/desktop, complete brief, simplified strategic analysis)
- [x] Step 1a — Client Profile (solo hobby project; driver: dissatisfaction with qBittorrent/Transmission; menu-bar-first interaction hinted)
- [x] [Vision Capture](02-vision.md) — What we're building and why
- [x] [User Definition](03-users.md) — Who we're building for
- [x] [Product Concept](04-concept.md) — The founding structural idea
- [x] [Positioning](07-positioning.md) — Market position and differentiation
- [x] Business Model (documented in `01-product-brief.md` — free/open-source, no monetization)
- [x] Success Metrics (documented in `decisions.md` — resource usage, daily adoption, GitHub feedback, 1-week v1 timeline)
- [x] Competitive Landscape (documented in `decisions.md` — unfair advantage corrected to "fresh unencumbered codebase")
- [x] Constraints (documented in `decisions.md` — $0/unsigned, Rust+native UI fixed, timeline flexible)
- [x] Platform Strategy (documented in `decisions.md` — Apple Silicon only, macOS-permanent, 3 v1 integrations)
- [x] Tone of Voice (documented in `01-product-brief.md` — quiet, plain, native, calm)
- [x] Step 12 — Product Brief Synthesis (narrative presented, confirmed without adjustment, document finalized)

**Note:** This generic template originally listed additional checklist items (Core Features, Inspiration & References, Launch Requirements, Timeline & Phases, Review & Synthesis as separate numbered files) that don't correspond 1:1 to the actual Complete Brief step sequence used for this project (Init → Client Profile → Vision → Positioning → Business Model → Target Users → Product Concept → Success Criteria → Competitive Landscape → Constraints → Platform Strategy → Tone of Voice → Synthesis). All substantive content those items would have covered is captured above or intentionally out of scope for a solo hobby brief (no separate launch/timeline workshop beyond the single 1-week target already documented).

---

## Key Decisions

See [decisions.md](decisions.md) for detailed decision log (9 entries).

**Major decisions:**
1. Greenfield project — starting from scratch, no existing code or materials.
2. Native macOS desktop torrent client, mapped to WDS "Web Application" complexity tier for workflow purposes only.
3. Simplified strategic analysis — Trigger Map (Phase 2) will stay light / folded into the brief rather than run as a full separate workshop.
4. Solo hobby project — client-profile kept thin, no agency-style stakeholder mapping.
5. Internal driver: dissatisfaction with qBittorrent (deprecated/heavy) and Transmission (light but inconvenient) — wants lightweight, native, pleasant.
6. Menu-bar-first assumption corrected during Vision: main window is primary, menu-bar is quick access.
7. Business model: fully free, open-source, no monetization — routed to B2C-only path.
8. Success criteria set honestly (resource usage, daily adoption, any community feedback), with a flagged feasibility risk on the 1-week timeline.
9. Unfair advantage corrected from "open source" (not differentiating — alternatives are open source too) to "fresh, unencumbered codebase."
10. Constraints resolved the timeline-vs-architecture tension: timeline is flexible, Rust+native-UI architecture and $0 budget are fixed.
11. Platform locked to Apple Silicon, last 1-2 macOS versions, permanently macOS-only, with 3 specific v1 native integrations.
12. Tone of voice proposed by agent from context (quiet, plain, native, calm) and confirmed without changes.

---

## Reflection Quality

**Total Checkpoints:** 6
**Confirmed First Try:** 5 (Vision, Positioning, Product Concept, Target Users, Success Criteria)
**Required Correction:** 1 (Competitive Landscape — unfair advantage claim corrected)

This measures how well the agent understood the user's intent.

---

## Dialog Artifacts

All dialog files are timestamped and track the natural conversation flow, not just the final outputs.

**Purpose:** Enable future agents (or humans) to understand WHY decisions were made, not just WHAT was decided.

---

**Generated Artifacts:**
- [wds-project-outline.yaml](../../_progress/wds-project-outline.yaml)
- [Product Brief documentation](../01-product-brief.md)
