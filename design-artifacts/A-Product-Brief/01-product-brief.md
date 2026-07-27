# Project Brief: my-torrent

> Complete Strategic Foundation

**Created:** 2026-07-27
**Author:** Alex
**Brief Type:** Complete

---

## Strategic Summary

`my-torrent` is a native, minimalist macOS torrent client, built to be the lightweight, pleasant alternative that qBittorrent (heavy, being deprecated on macOS) and Transmission (light but inconvenient) don't offer. It's a hobby project for Alex, with no monetization, aimed first at daily personal use and eventually at public/open-source release to gather community feedback.

The product concept is "zero friction on entry, depth on demand": dropping a `.torrent` starts it downloading immediately with sensible global defaults (no add-dialog), while a main window lists all active downloads (progress, speed, seeders/leechers) and a per-torrent detail view holds the complexity (files, trackers, peers). A menu-bar icon provides quick access; the main window is the primary information surface.

Success is measured honestly and personally: near-zero idle resource usage, genuine daily use in place of Transmission, and any real community feedback after an open-source release — not revenue or adoption numbers. The real competitive edge isn't the open-source license (both major alternatives already have that) but having a fresh, small, unencumbered codebase that makes strict minimalism enforceable long-term.

The build is constrained to a $0 budget (unsigned distribution), a Rust protocol engine paired with a genuinely native macOS UI (Apple HIG look), Apple Silicon only, and the last 1–2 macOS versions — permanently macOS-only. The ~1-week v1 timeline is explicitly the flexible constraint: if trade-offs are needed, the timeline moves before the architecture or budget commitments do.

---

## Vision

Build the torrent client macOS deserves: install it, forget it's running, and always know at a glance how your downloads are doing. `my-torrent` is a minimalist, native menu-bar-first torrent client for people who want qBittorrent's power without the bloat, and Transmission's lightness without the awkwardness — open by default, built to earn daily use, and shared publicly to see if it clicks for others too.

**Key Insights from Discussion:**
- Primary audience: macOS users who value minimalism and simplicity — not necessarily power users, just people tired of clunky or heavy torrent clients.
- Success is deliberately narrow and honest: trivial to install, and Alex actually uses it every day. Runs continuously in the background with minimal resource footprint.
- Aspiration beyond personal use: open-source / public release specifically to gather feedback from other users — this is not a closed personal tool.
- Interaction model confirmed: the **main window is the primary information surface** (opened by clicking the app or the menu-bar icon) — showing the downloads list, progress bars, speeds, seed/leech counts. The **menu-bar icon is the quick-access entry point**, not a replacement for the window.

---

## Positioning

**Positioning Statement:**
Для пользователей macOS, которые устали от тяжёлых или неудобных торрент-клиентов, `my-torrent` — нативная лёгкая утилита для скачивания торрентов, которая всегда под рукой (меню-бар) и не грузит систему. В отличие от qBittorrent (тяжёлый, уходит с платформы) и Transmission (лёгкий, но неудобный), мы делаем базовые вещи — открытие `.torrent`, прогресс, скорости, сиды/личи — по-настоящему хорошо и нативно, прежде чем думать о дополнительных фичах.

**Components:**

- **Target Customer:** Пользователи macOS, которые ценят минимализм и простоту (не обязательно технически продвинутые)
- **Their Need:** Существующие клиенты либо тяжёлые/уходят с платформы (qBittorrent), либо лёгкие, но неудобные (Transmission), либо запутанные (прочие)
- **Product Category:** Нативная лёгкая утилита-торрент-клиент для macOS (menu-bar + основное окно)
- **Key Benefit:** Приятный, лёгкий, нативный клиент, который работает в фоне без нагрузки на систему
- **Alternatives:** qBittorrent, Transmission, прочие торрент-клиенты
- **Differentiator:** Дисциплина фокуса — довести базовые вещи (открытие `.torrent`, прогресс, скорости, сиды/личи) до по-настоящему хорошего нативного качества, прежде чем добавлять фичи

**Strategic Rationale:**
Рынок macOS-клиентов сейчас поляризован: qBittorrent тяжёлый и покидает платформу, Transmission лёгкий, но неудобный, остальные — нишевые и запутанные. Это создаёт понятный зазор для клиента, который берёт лёгкость Transmission и добавляет продуманный, приятный UX и нативную интеграцию (menu-bar + окно), сознательно откладывая фичи ради качества основ.

---

## Business Model

**Type:** Free / Open Source, no monetization

**Rationale:**
This is a personal/hobby project (see Client Profile). The stated aspiration is public release to gather community feedback, not revenue — Alex explicitly ruled out monetization (no paid tiers, no donations/sponsorship layer at this stage).

**Implications:**
- No B2B component — routes straight to Target Users (end users only), no separate business-customer profile needed.
- No pressure to build monetization infrastructure (accounts, payments, licensing) — keeps scope aligned with the "basics done well first" positioning.
- Success is measured by usage/adoption and feedback quality, not revenue metrics.
- Distribution likely via GitHub / open-source channels rather than the paid Mac App Store model — to confirm at Platform Strategy / Constraints.

---

## Ideal Customer Profile (ICP)

**Who they are:**
macOS users who value minimalism and simplicity — not defined by technical skill level, but by taste around software (Alex + like-minded people).

**Their context:**
Download multiple different files in parallel via BitTorrent; run the client continuously in the background.

**Their frustrations:**
Cluttered UI, extra windows/notifications, slow performance, high resource consumption, complex configuration — all things current clients (qBittorrent, Transmission, others) get wrong in different ways.

**What they're trying to achieve:**
"Set and forget" by default — install, open a `.torrent`, downloads and seeding just happen — with the option to dive into settings for fine control, notably a **configurable seeding duration** (per torrent and/or globally), not indefinite or immediate-stop seeding.

**How they currently solve this:**
qBittorrent (heavy, deprecated on macOS), Transmission (light, inconvenient), other unnamed clients (confusing).

### Secondary Users

None identified — single user type confirmed. May revisit post-launch if open-source community feedback surfaces a distinct, less-technical segment.

---

## Product Concept

**Core concept:**
Zero-friction entry + on-demand depth: dropping/opening a torrent starts it immediately with global defaults; per-torrent detail (files, trackers, peers) lives in a drill-down view, not the main list.

**Implementation principle:**
Main window = flat list of active downloads (one row per torrent: progress bar, download speed, upload speed, seeders, leechers). Clicking a row opens a detail view for that torrent (files inside, trackers, peers). Adding a torrent never opens a required configuration dialog — global default settings (save location, seeding duration, etc.) apply automatically, overridable later per-torrent or globally in Settings.

**Concrete example:**
Drop a `.torrent` → a row appears in the list, downloading starts immediately, seed duration follows the global default. Click the row to see which files are included, tracker status, and connected peers.

**Why this approach:**
Directly operationalizes "set and forget by default, configurable via settings" (Target Users) and "basics done well, no bloat" (Positioning). Avoids the two named frustrations with existing clients — cluttered main UI and friction-heavy add flows — via progressive disclosure: simple surface, full power one click away.

**Features that stem from this concept:**
1. Main list view — one row per torrent (progress, down/up speed, seeders, leechers), no clutter.
2. Per-torrent detail view — files inside the torrent, tracker status, peer list.
3. Global default settings (save location, seeding duration, etc.) applied automatically on add — no mandatory add-dialog.
4. Settings screen for global defaults, with possible per-torrent overrides from the detail view.

---

## Success Criteria

**Primary metrics:**
- **Resource usage:** Near-zero CPU/memory consumption while idle (background/seeding, no active transfer).
- **Personal adoption:** Alex uses it daily in place of Transmission — no fixed evaluation date, judged by whether it genuinely sticks.
- **Community feedback:** Any real engagement after open-source release (GitHub issues, stars) counts as success — no specific numeric target set.

**Timeline:**
- First basic version with core features targeted within **~1 week**.

**⚠️ Feasibility flag (recommendation, not a blocker):** A native macOS BitTorrent client with a GUI touches TCP/UDP trackers, DHT, the peer wire protocol, and `.torrent` parsing — a 1-week timebox is very tight if implementing the protocol from scratch. This should directly inform tech-stack choice at Architecture/Platform Requirements (e.g., building on an existing BitTorrent library rather than a from-scratch protocol implementation) and what counts as "core features" for v1 vs. backlog.

---

## Competitive Landscape

**Alternatives considered:**

| Alternative | Why people stick with it | Where it falls short |
|---|---|---|
| qBittorrent | Feature-rich, capable | Heavy, being deprecated on macOS |
| Transmission | Light, simple | Not convenient/pleasant to use day to day |
| Other clients (unnamed) | — | Confusing, unclear |
| Do nothing (stay on Transmission) | Already installed, "good enough" | Merely inconvenient — not critical, no urgent pain, just friction |

**Our Unfair Advantage:**
Not the open-source license itself — Transmission and qBittorrent are already open source too, so that alone isn't differentiating. The real advantage is a **fresh, small, personally-owned codebase with no 15+ years of legacy weight and no existing user base/maintainer politics to negotiate with**. Full control over roadmap makes it realistic to enforce minimalism as strict discipline — something structurally harder for the established alternatives to retrofit, even if they wanted to.

**Reality check:** Even if Transmission or qBittorrent later added a nicer UI, menu-bar mode, or lower resource usage, this advantage (freedom to keep the codebase small and opinionated) would still hold — it's about maneuverability, not a copyable feature.

---

## Constraints

**Timeline:**
~1 week targeted for first basic v1 (see Success Criteria) — **flexible**. If it conflicts with the technical/quality bar below, the timeline is what moves, not the architecture.

**Budget:**
$0 — no Apple Developer Program enrollment. Distribute as an unsigned build with user-facing instructions for bypassing Gatekeeper (System Settings → allow anyway). **Fixed** (deliberate choice, not just default).

**Technical:**
Rust engine for the BitTorrent protocol (trackers, DHT, peer wire protocol, `.torrent` parsing) + native macOS UI on top (Swift/SwiftUI or AppKit, bridged to the Rust core — exact bridging approach to be resolved at Architecture). **Fixed** — explicitly prioritized over the 1-week timeline when the two are in tension.

**Brand/Visual:**
Apple HIG-native look and feel — no custom/distinct visual identity, blend into macOS conventions. To be detailed further at UX Design (Phase 4).

**What's flexible vs. fixed (priority order if trade-offs arise):**
1. Timeline (~1 week for v1) — **moves first**
2. Budget ($0, unsigned distribution) — fixed
3. Technical architecture (Rust core + native macOS UI) — fixed, protected even at the cost of timeline

---

## Platform & Device Strategy

**Primary Platform:** Native macOS Desktop Application

**Supported Devices:**
Mac computers on the **last 1–2 macOS versions**, **Apple Silicon only** (no Intel support).

**Device Priority:** Single-platform — macOS only, permanently. No iOS/iPadOS or cross-platform ambitions (explicitly ruled out as a future direction).

**Interaction Models:**
Mouse and keyboard (standard macOS desktop interaction). Menu-bar icon as quick-access entry point; main window as primary information surface (per Product Concept).

**Technical Requirements:**
- **Offline functionality:** N/A in the traditional sense — the app requires network connectivity to download/seed by nature (BitTorrent), but the app itself should launch/run/display state fine even with no active connections.
- **Native features needed for v1:**
  - `.torrent` file association (double-click in Finder opens the app)
  - Magnet link handling (opens directly from browser)
  - System notification on download completion
- **Explicitly deferred (not v1):** Launch-at-login (login item) — not selected for v1 scope.

**Platform Rationale:**
Matches the Constraints decision (Rust core + native macOS UI) and the target user (macOS users who value minimalism). Restricting to Apple Silicon + last 1-2 OS versions avoids legacy compatibility work that would fight the "basics done well, fast" priority and the tight timeline.

**Future Platform Plans:**
None — macOS-only is a deliberate, permanent scope boundary, not a "v1 only" constraint.

**Design Implications:**
UI should follow Apple HIG native conventions (per Constraints); file-association and magnet-link handling need corresponding OS-level entries (Info.plist URL/file type declarations) — to be handled at Architecture.

**Development Implications:**
No Intel/older-OS compatibility shims needed. `.torrent`/magnet handling and notifications require macOS system integration APIs — to be scoped precisely during Architecture/Platform Requirements alongside the Rust↔native-UI bridge design.

---

## Tone of Voice

**For UI Microcopy & System Messages**

### Tone Attributes

1. **Quiet & Unobtrusive**: The app shouldn't "shout" about itself — minimal copy, no unnecessary dialogs or exclamations. Mirrors the "background utility" nature of the product.
2. **Plain & Direct**: No marketing fluff — just state facts (progress, status, counts) clearly and concisely.
3. **Native/System-standard**: Copy reads like part of macOS itself, using conventional system phrasing rather than inventing a distinct brand voice — matches the Apple HIG-native visual direction.
4. **Calm Under Errors**: Errors are described factually, without alarm or drama — consistent with a target audience that's irritated by noise/clutter, not fragile about failures.

### Examples

**Error Messages:**
- ✅ "Трекер недоступен" ("Tracker unavailable")
- ❌ "Упс! Что-то сломалось" ("Oops! Something broke")

**Button Text:**
- ✅ "Открыть…" ("Open…")
- ❌ "Давай начнём!" ("Let's get started!")

**Empty States:**
- ✅ "Нет активных закачек" ("No active downloads")
- ❌ "Тут пока пусто 👀" ("Nothing here yet 👀")

**Success Messages:**
- ✅ "Готово" / system notification: "Загрузка завершена: {filename}" ("Done" / "Download complete: {filename}")
- ❌ "Ура, всё скачалось! 🎉" ("Yay, all downloaded! 🎉")

### Guidelines

**Do:**
- Use standard macOS terminology and phrasing conventions wherever one exists
- Keep messages short — state the fact, nothing more
- Let silence be the default (no confirmation toasts for routine actions)

**Don't:**
- Use exclamation marks, emoji, or hype language
- Anthropomorphize the app ("I'm downloading your file!")
- Dramatize errors or failures

---

*Note: Tone of Voice applies to UI microcopy (labels, buttons, errors, system messages). Strategic content (headlines, feature descriptions) is not applicable here — this is a utility app, not a marketing site.*

---

**Status:** Product Brief Complete
**Next Phase:** Trigger Mapping (Phase 2) — simplified, per Phase 0 config
**Last Updated:** 2026-07-27
