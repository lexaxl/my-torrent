# Review: ARCHITECTURE-SPINE.md (my-torrent, 2026-07-27)

**Reviewer:** rubric-walker
**Target:** `_bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md`
**Inputs cross-checked:** `design-artifacts/A-Product-Brief/01-product-brief.md`, `design-artifacts/E-Development/deliveries/DD-001-core-download-experience.yaml`, `design-artifacts/C-UX-Scenarios/04-alex-glances-at-the-menu-bar/`

## Verdict

A solid, above-average spine for a hobby-project altitude — its 8 ADs are concrete, enforceable, and correctly cover the Rust↔Swift FFI boundary and DD-001's four scenarios, but it leaves the app's background-lifecycle model (the thing the entire "install it, forget it's running" value proposition depends on) and the completion-notification trigger mechanism completely unaddressed, which are genuine, load-bearing gaps, not nitpicks.

---

## Critical

None. Nothing in the spine is actively wrong or contradictory; the gaps below are omissions, not errors.

---

## High

### 1. App lifecycle / activation-policy model is undecided (no AD covers it)

The Product Brief's Vision is explicit: *"install it, forget it's running... runs continuously in the background."* Scenario 04 (`04-alex-glances-at-the-menu-bar.md`) restates this: *"downloads are running in the background"* while the user works elsewhere. AD-4 leans on this too — its Rule explicitly requires the poll timer to keep running *"regardless of whether any window is open."*

None of this is possible unless the app's process survives the main window closing. That depends on a concrete decision the spine never makes:
- Is the app `NSApplication.ActivationPolicy.regular` (Dock icon + standard window-close-may-quit behavior) or `.accessory` (menu-bar-only, no Dock icon)?
- Does `MyTorrentApp.swift`'s `WindowGroup` + `MenuBarExtra` combination need an explicit override of `applicationShouldTerminateAfterLastWindowClosed`, or does SwiftUI's default (kept alive by the `MenuBarExtra` scene) already give the right behavior?
- The Product Brief also says the main window is opened "by clicking the app or the menu-bar icon," implying a Dock icon exists (i.e., `.regular`, not `.accessory`) — which conflicts with the menu-bar-first framing DD-001 and Scenario 04 also lean on, and is currently invisible as a tension because no AD names either option.

This is exactly the kind of decision the spine altitude exists to fix: a SwiftUI implementer could plausibly build a default `WindowGroup` app that quits the process when its last window closes, silently breaking the entire background/idle value proposition (and invalidating AD-4's "even with no window open" clause) without anyone noticing until it ships. Recommend adding an AD (or at minimum a Deferred entry naming the two options and which one is intended) that fixes: Dock-icon presence, and explicit "closing the main window does not terminate the app or its downloads."

---

## Medium

### 2. Completion-notification trigger mechanism is unspecified, and collides with AD-4's poll-stop condition

DD-001's acceptance criteria require a system notification exactly once per completed download. AD-4 forbids the core from pushing callbacks/events ("no UniFFI callback/foreign-callback interfaces"), so detecting "this torrent just finished" must be done shell-side by diffing consecutive snapshots — but this responsibility, and the dedup logic needed to avoid firing the notification on every subsequent poll, is never assigned anywhere (not in AD-4, not in the Capability → Architecture Map row for "file association / magnet / completion notification," which is marked `Deferred (exact entitlements)` — a different concern).

Worse, there's a latent race with AD-4 itself: AD-4 stops the poll timer entirely once zero torrents are "active." A torrent that completes with the global seed-duration default set to "off" (an explicit DD-001 edge case) transitions to a non-active/done state at essentially the same moment it completes. Depending on how "active" is defined for the timer-stop check, the poll cycle that would have observed the completed→done transition (and fired the notification) could be the same cycle that also causes the timer to stop — leaving the notification-firing responsibility unclear both in *who* does it and *whether the polling cadence guarantees it fires at all*. This is squarely a cross-cutting FFI/shell divergence risk DD-001 requires to work, not an implementation nicety.

Recommend either an addition to AD-4's Rule (define "active" to include "completed-this-poll-and-not-yet-observed," or have the core retain a torrent in the active/pollable set for one extra tick after completion) or a new short AD assigning notification-diff ownership to the shell explicitly.

### 3. App Sandbox / entitlements posture is never stated

The spine fixes unsigned/no-notarization distribution (AD-8) but never states whether the app runs inside the macOS App Sandbox. This is a real cross-cutting decision: it affects the Rust core's ability to do arbitrary outbound TCP/UDP (peer wire protocol, DHT, tracker UDP) and the Swift shell's ability to write to an arbitrary user-chosen save path (`defaultSavePath` in DD-001's `AppSettings`), which is incompatible with a sandboxed app without security-scoped bookmarks. Given $0/no-App-Store distribution this is almost certainly "not sandboxed," but that inference is never written down, and it's the kind of assumption a builder could get wrong late (e.g., by leaving Xcode's default App Sandbox capability enabled), causing silent network/file failures that look like protocol bugs. Worth one explicit line, even for a hobby project.

---

## Low

### 4. Panic/crash behavior across the FFI boundary is silent (adjacent to AD-6)

AD-6 fixes typed `Result<T, EngineError>` for *fallible* calls, but says nothing about Rust-side panics (e.g., inside `librqbit` or its `tokio` tasks) propagating across UniFFI. UniFFI has specific, non-default-obvious behavior here (panics can abort the process or be converted depending on configuration). For a hobby app this is genuinely low-stakes, but a one-line note ("panics are not caught/converted; a core panic is expected to crash the app") would remove ambiguity cheaply and is consistent with the spine's own precedent of naming other UniFFI rough edges (it already does this for Swift 6 strict concurrency).

### 5. "Logging/observability strategy — not fixed" (Deferred) is appropriately deferred, not a gap

Checked as a candidate for "actually load-bearing" per the review rubric: this is correctly left in Deferred. It doesn't create cross-unit incompatibility (either side can add `tracing`/`print` independently without affecting the other), and the spine already flags it for revisit if debugging needs grow. No action needed — noted here only to record that it was checked, not to fault it.

---

## Checklist-by-item summary

1. **Divergence points fixed / missed** — The FFI-boundary divergence points (process model, protocol library, UI framework, sync model, state ownership, error shape, threading, release pipeline) are all fixed and well-chosen. Missed: app lifecycle/activation-policy (High #1) and notification-trigger ownership (Medium #2), both of which are real cross-cutting decisions DD-001 depends on.
2. **Rule enforceability** — All 8 ADs have concrete, checkable Rules (specific function signatures, specific frameworks named, specific thread-marshaling behavior). AD-2's "does not reimplement it" is the softest phrasing but is acceptable at this project's scale.
3. **Deferred section audit** — Reviewed every entry; none is load-bearing enough to cause cross-unit divergence. All are either single-owner implementation details, library-internal concerns, or correctly out-of-v1-scope. No reclassification needed.
4. **Tech currency** — Shows real evidence of being checked, not templated from training data: librqbit is dated ("docs updated June 2026"), tokio carries specific LTS branch/expiry dates (1.47.x/Sep 2026, 1.51.x/Mar 2027), and UniFFI's Swift 6 strict-concurrency limitation is called out with a research date (2026-07-27) and a concrete fallback (target Swift 5 language mode). This is good practice, not stale.
5. **Capability → Architecture Map coverage** — All 4 DD-001 scenarios (01–04) are mapped, plus supporting rows for live stats, file/magnet/notification integration, and build/release. Complete at the row level; the notification row's "Deferred (exact entitlements)" undersells that the *trigger logic*, not just the entitlements, is unaddressed (see Medium #2).
6. **Initiative-altitude envelope coverage** — Deployment & release pipeline is decided (AD-8, GitHub Actions → GitHub Releases, unsigned .dmg). Environments/infra are correctly N/A for a single-binary desktop app. Operations/observability is explicitly deferred (acceptable). The one envelope dimension left genuinely silent (neither decided nor deferred) is app lifecycle/process-termination behavior (High #1) and, to a lesser extent, sandboxing posture (Medium #3).
