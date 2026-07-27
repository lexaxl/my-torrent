# Key Decisions Log

**Project:** my-torrent
**Format:** Append-only decision log

---

## Decision 1: Solo hobby project, no organisation structure

**Date:** 2026-07-27
**Step:** Step 1a — Client Profile
**Session:** 1

**Context:**
WDS's client-profile step assumes an agency/client relationship (organisation, stakeholders, decision culture). This is a solo hobby project.

**What was decided:**
Skip the full organisational interrogation. Alex is sole owner, sole user, sole decision-maker, fast/individual decision style, no approval chain.

**Why:**
Matches Phase 0 config (stakes: personal_hobby, documentation: minimal). Asking a full corporate checklist would waste time and produce fabricated/irrelevant content.

**Impact:**
Client-profile.md is intentionally thin. Future steps should stay light-touch rather than agency-grade.

**Alternatives considered:**
- Full client-profile interrogation — rejected, not applicable to a solo project.

**Documented in:** `dialog/client-profile.md`

---

## Decision 2: Internal driver — dissatisfaction with existing macOS torrent clients

**Date:** 2026-07-27
**Step:** Step 1a — Client Profile
**Session:** 1

**Context:**
Asked what triggered wanting a custom torrent client instead of using an existing one.

**What was decided:**
Core driver documented: qBittorrent is being deprecated on macOS and is heavy; Transmission is light but not pleasant/convenient; other clients are confusing. Alex wants something very lightweight, native-feeling, and visually pleasant.

**Why:**
This is the emotional/practical "why" behind the whole project — it will anchor the Vision step next and should keep the product honest (lightweight + native + pleasant, not feature-bloat).

**Impact:**
Vision and positioning should emphasize: lightweight, native macOS feel, polished progress visualization, unobtrusive presence (see Decision 3 — menu bar).

**Alternatives considered:**
- "Pure learning exercise, no real pain point" — rejected; user named concrete frustrations with specific existing tools.

**Documented in:** `dialog/client-profile.md`, `dialog/00-context.md`

---

## Decision 3: Menu bar presence is a defining product idea (not just a feature)

**Date:** 2026-07-27
**Step:** Step 1a — Client Profile
**Session:** 1

**Context:**
While describing the driver, Alex volunteered a concrete interaction model: the app should live in the macOS menu bar, and hovering should reveal download/upload speed.

**What was decided:**
Capture this now as a strong signal for Vision/positioning — this is not a generic torrent client with a window, it's positioned as a lightweight menu-bar-first utility.

**Why:**
This came up unprompted and specifically — a strong signal of the product's core interaction model, distinct from qBittorrent/Transmission's traditional always-open-window approach.

**Impact:**
Will need explicit confirmation during Vision/Positioning whether the main downloads-list window (with progress bars, speeds, seed/leech counts — already confirmed as a hard requirement) is a secondary window opened on demand, while the menu bar icon + hover tooltip is the primary at-a-glance surface.

**Alternatives considered:**
- Traditional always-visible main window only (qBittorrent/Transmission style) — likely rejected given explicit "лёгкое" (lightweight) and menu-bar framing, to be confirmed in Vision step.

**Documented in:** `dialog/client-profile.md`

---

## Decision 4: Business model — free, open source, no monetization

**Date:** 2026-07-27
**Step:** Step 5 — Business Model
**Session:** 1

**Context:**
Asked whether `my-torrent` would be fully free/open-source or allow for future donations/sponsorship.

**What was decided:**
Fully free and open-source, with no monetization layer at all — not even an optional donations/sponsorship option for now.

**Why:**
Matches the personal/hobby stakes and the stated aspiration (public release to gather feedback, not revenue). Keeps scope focused on product quality rather than payment/licensing infrastructure.

**Impact:**
No B2B business-customer profile needed — routes directly to Target Users (Step 7, B2C-only path). Distribution strategy (Constraints/Platform Strategy) should lean toward open-source channels (e.g. GitHub) rather than a paid App Store model, to be confirmed later.

**Alternatives considered:**
- Optional donations/sponsorship (GitHub Sponsors, Buy Me a Coffee) — explicitly declined for now.
- Paid app (Mac App Store) — not raised as a serious option given the hobby/open-source framing.

**Documented in:** `01-product-brief.md` → Business Model

---

## Decision 5: Success criteria — resource usage, daily adoption, community feedback, 1-week v1 timeline

**Date:** 2026-07-27
**Step:** Step 8 — Success Criteria
**Session:** 1

**Context:**
Explored success from multiple angles: resource footprint, personal adoption, community/open-source feedback, and timeline.

**What was decided:**
- Near-zero CPU/memory when idle.
- Daily personal use replacing Transmission, no fixed check-in date.
- Any real GitHub engagement (issues/stars) after open-source release counts as success.
- First basic version targeted within ~1 week.

**Why:**
Keeps success criteria honest and personal-scale rather than inventing vanity metrics for a hobby project with no monetization.

**Impact:**
The 1-week timeline is flagged as a hard constraint that should push the Architecture/Platform Requirements phase toward using an existing BitTorrent protocol library rather than a from-scratch implementation, and should sharply limit what counts as "core features" for v1.

**Alternatives considered:**
- Numeric adoption/download targets for the open-source release — rejected, not meaningful for a first hobby release.

**Documented in:** `01-product-brief.md` → Success Criteria

---

## Decision 6: Unfair advantage — corrected from "open source" to "fresh, unencumbered codebase"

**Date:** 2026-07-27
**Step:** Step 9 — Competitive Landscape
**Session:** 1

**Context:**
User's first answer to "what's your unfair advantage" was "open source stays an advantage." Agent flagged that Transmission and qBittorrent are already open source, so this claim wasn't actually differentiating, and proposed an alternative framing.

**What was decided:**
Real unfair advantage is having a fresh, small, personally-owned codebase with no legacy weight and no existing user base/maintainer politics — giving full control to enforce minimalism as strict ongoing discipline, something structurally harder for established alternatives to retrofit.

**Why:**
The step explicitly requires not accepting vague/weak unfair-advantage claims. "Open source" was checked against reality (both main alternatives are already open source) and found not to hold up; the corrected framing survived the reality check (would still hold even if alternatives improved their UI/resource usage).

**Impact:**
Reinforces that the project's edge is about maintaining scope discipline over time, not a one-time feature. Should inform how "basics done well" positioning gets protected as the project evolves (e.g., resist feature creep even post-v1).

**Alternatives considered:**
- "Open source" as the unfair advantage — rejected on reality check (not unique among named alternatives).
- "Do nothing" (staying on Transmission) — confirmed as low-stakes: merely inconvenient, no critical/urgent pain, which explains why this project is genuinely optional/hobby-paced rather than solving an emergency.

**Documented in:** `01-product-brief.md` → Competitive Landscape

---

## Decision 7: Constraints — $0 budget, Rust+native-UI architecture fixed, timeline flexible

**Date:** 2026-07-27
**Step:** Step 10 — Constraints
**Session:** 1

**Context:**
Explored budget, technical stack, and brand/visual constraints, then explicitly checked which would flex under pressure.

**What was decided:**
- Budget: $0, no Apple Developer Program — unsigned build + Gatekeeper bypass instructions.
- Technical: Rust engine for the BitTorrent protocol + native macOS UI (bridging approach TBD at Architecture).
- Brand: Apple HIG-native look, no custom visual identity.
- Priority under conflict: **timeline moves first**, budget and technical architecture are protected even at the cost of the ~1-week v1 target.

**Why:**
User explicitly chose architecture/code quality over the aggressive timeline when asked to resolve the tension flagged back in Step 8 (Success Criteria) — the 1-week target was always acknowledged as tight given a Rust+native-UI bridge, not a from-scratch-protocol estimate.

**Impact:**
This resolves the feasibility flag raised in Decision 5. Architecture/Platform Requirements phase should treat "~1 week" as aspirational/soft, and should not compromise on using Rust for the protocol core or on shipping a genuinely native macOS UI (not a webview/Electron-style wrapper) to hit that date.

**Alternatives considered:**
- Compressing scope/architecture to hit 1 week strictly — explicitly rejected by user in favor of protecting quality.

**Documented in:** `01-product-brief.md` → Constraints

---

## Decision 8: Platform strategy — Apple Silicon only, macOS-permanent, v1 native integrations scoped

**Date:** 2026-07-27
**Step:** Step 10a — Platform Strategy
**Session:** 1

**Context:**
Explored minimum OS/architecture support, which native macOS integrations matter for v1, and future platform ambitions.

**What was decided:**
- Last 1-2 macOS versions, Apple Silicon only (no Intel).
- v1 native integrations: `.torrent` file association, magnet link handling, download-complete notification.
- Explicitly deferred: launch-at-login.
- macOS-only permanently — no iOS/iPadOS plans ever, not just "not in v1."

**Why:**
Keeps platform scope aligned with the tight timeline and minimalism-first positioning — no legacy compatibility burden. The three chosen integrations are the ones that most directly serve the "zero friction on entry" product concept (opening a torrent/magnet should just work from Finder/browser) and the "set and forget" goal (notification on completion).

**Impact:**
Architecture/Platform Requirements should scope Info.plist URL-scheme (magnet:) and UTI (.torrent) declarations, plus macOS UserNotifications integration, alongside the Rust↔native-UI bridge. Login-item support can be added later without being a v1 blocker.

**Alternatives considered:**
- Intel Mac support — rejected, adds compatibility surface without serving the core audience.
- Login item for v1 — deferred, not selected among the three chosen v1 integrations.
- Cross-platform/iOS future — explicitly ruled out, not merely deprioritized.

**Documented in:** `01-product-brief.md` → Platform & Device Strategy

---

## Decision 9: Tone of voice — quiet, plain, native, calm

**Date:** 2026-07-27
**Step:** Step 11 — Tone of Voice
**Session:** 1

**Context:**
Agent analyzed accumulated product context (minimalism, Apple HIG-native visual, background-utility nature, audience irritated by clutter/noise) and proposed 4 tone attributes with examples, per this step's requirement to suggest rather than ask the user to invent tone from scratch.

**What was decided:**
Tone = Quiet & Unobtrusive, Plain & Direct, Native/System-standard, Calm Under Errors. No exclamation marks, no emoji, no hype language, no anthropomorphizing, silence as default (no confirmation toasts for routine actions).

**Why:**
Directly derived from already-confirmed product characteristics rather than invented independently — keeps the brief internally consistent (minimalism value → tone, HIG-native visual → tone).

**Impact:**
UX Design (Phase 4) and later UI copywriting should follow these attributes for all microcopy — buttons, errors, empty states, notifications.

**Alternatives considered:**
- Warmer/more personal tone (e.g., "Let's get started!") — rejected as inconsistent with "quiet utility" positioning.

**Documented in:** `01-product-brief.md` → Tone of Voice

---

### Product Brief Synthesis (Step 12)

**Final narrative presented:** Yes

**Adjustments during synthesis:**
- None — narrative confirmed as presented, no corrections needed at final review.

**User confirmation:** Confirmed

**Brief generated:** `A-Product-Brief/01-product-brief.md`

**Completion:** 2026-07-27

---

_Continue appending decisions as they're made throughout the Product Brief process._
