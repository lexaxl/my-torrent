# Adversarial Review — ARCHITECTURE-SPINE.md (my-torrent)

**Reviewer stance:** two spec-compliant, non-communicating builders — "Engine Unit" (owns `engine/`) and "Shell Unit" (owns `MyTorrent/`) — each reading every AD literally and building in isolation.

## Verdict

The spine is solid on FFI mechanics (typed errors, threading, naming) but leaves the *behavioral* contract underneath AD-4's polling rule and AD-5's settings hand-off genuinely underspecified, and those gaps are exactly the kind that let two compliant builders each make a locally-reasonable choice that doesn't fit together at runtime — most seriously around what "active" means, who owns the polling timer's lifecycle, and how a completed-but-still-seeding torrent stays visible.

---

## Findings

### F1 — [HIGH] Bootstrap chicken-and-egg: polling can only detect "active" by polling, but the rule says "poll only while active"

- **AD:** AD-4, interacting with AD-5's librqbit auto-resume.
- **Choice A (Shell Unit):** At launch, do one unconditional `getAllTorrents()` call to seed initial state (treated as "not the timer," exempt from the AD-4 gating), then start the recurring timer only if that seed call shows `active > 0`.
- **Choice B (Shell Unit, equally literal):** Strictly obey the rule text — never call `getAllTorrents()` unless local state already indicates something is active. On a cold launch there is no local state, so no poll fires until the user opens a window.
- **Why they clash:** librqbit auto-resumes torrents on its own (AD-5's "own resume-data mechanism") with zero explicit "add" event for the shell to hook. Under Choice B, torrents that were downloading when the app last quit resume silently in the background with no UI feedback, no menu-bar update, and no completion notification — directly contradicting the spine's own stated rationale for AD-4 ("this is what makes completion notifications and menu-bar accuracy work without an open window"). Since only one unit builds the shell, this isn't strictly cross-team, but it's a defensible-both-ways reading of the Rule as literally written, and it's the single most likely way an implementer breaks the product's core promise while believing they're spec-compliant.
- **Fix:** Add an explicit bootstrap clause to AD-4: "On app launch, perform one unconditional `getAllTorrents()` call to seed state; this call is exempt from the 'poll only while active' gate and determines whether the recurring timer starts."

### F2 — [HIGH] "Active" is never defined as a shared vocabulary — Seeding is the fault line

- **AD:** AD-4, plus the undefined `TorrentStatus` state shape referenced in the Structural Seed (`types.rs`).
- **Choice A (Engine Unit):** Exposes a `state` enum mirroring librqbit's own machine (`Queued, Checking, Downloading, Seeding, Paused, Error, Completed`), with no explicit `is_active` field — leaves "active" as an interpretation problem for the caller.
- **Choice B (Shell Unit):** Defines "active" (for the purpose of deciding whether to keep polling) as "still downloading" only — i.e. `{Downloading, Checking}` — since that's the intuitive reading of "there's progress happening."
- **Why they clash:** The moment a torrent finishes and becomes `Seeding`, Shell Unit's active-count drops to 0 and the poll timer stops entirely — even though the torrent is still seeding, uploading, and will eventually hit the seed-duration cutoff. Seed/peer stats freeze, the menu bar goes stale, and the "seeding stopped" transition (which the Settings screen's "default seed duration" implies the user can observe) is never surfaced until the user manually reopens a window. This silently guts the seed-duration feature.
- **Fix:** AD-4 should pin an explicit active/inactive partition as a named convention, e.g.: `active = {Downloading, Checking, Seeding}`; `inactive = {Paused, Error, Completed-not-seeding}`. Put this in the Consistency Conventions table, not just prose in AD-4.

### F3 — [HIGH] Nothing says the poll timer must be app-lifetime, not View-lifetime

- **AD:** AD-4 + AD-3 (MenuBarExtra), Capability Map row "04 — Alex Glances at the Menu Bar."
- **Choice A (Shell Unit):** Owns the polling `Task`/timer in a top-level `ObservableObject` (e.g. an `AppModel` instantiated by `MyTorrentApp`), independent of any View's presence — correctly keeps polling (and firing notifications / updating `MenuBarExtra`) even with every window closed.
- **Choice B (Shell Unit, idiomatic SwiftUI default):** Starts the poll loop inside `MainWindowView`'s `.task {}` / `.onAppear`, stops it in `.onDisappear` — the standard SwiftUI pattern for a view-scoped background poll.
- **Why they clash:** Closing the main window on macOS does not quit the app. Under Choice B, polling — and therefore menu-bar accuracy and completion notifications — dies the instant the user closes the window, which is precisely the scenario AD-4's own parenthetical calls out as the reason polling must run "regardless of whether any window is open." The Rule describes the desired *behavior* but never states *where in the object-ownership hierarchy* the timer must live to guarantee it, so Choice B is a fully literal, fully defensible reading that still breaks the requirement.
- **Fix:** Add a sentence to AD-4's Rule: "The polling timer/task must be owned by an app-lifetime object, not a View — it must not start/stop with window appearance."

### F4 — [MEDIUM] Mutation calls (pause/resume/remove) don't specify synchronous-vs-eventual consistency — race on immediate re-poll

- **AD:** AD-4 (polling) interacting with the State Mutation convention ("shell never mutates torrent state locally").
- **Choice A (Shell Unit):** Since the shell can't locally mutate state, it issues an out-of-band `getAllTorrents()` immediately after a successful `pause_torrent()` call to refresh the UI quickly, rather than waiting up to ~1s for the next tick.
- **Choice B (Engine Unit):** Implements `pause_torrent()` as fire-and-forget against the tokio runtime — the call returns once the request is *enqueued*, not once librqbit's internal state machine has actually applied it.
- **Why they clash:** The shell's immediate follow-up poll can legitimately still show the torrent `Downloading` (pre-mutation state), causing a visible flicker (paused → still-downloading → paused-again a tick later) or, in a worse implementation, a shell that treats "still downloading after pause call" as failure and retries the mutation. This is a genuine race in the flow, not just a UX nit, because neither side's contract says which of "call returns" vs "state has settled" is the guarantee.
- **Fix:** AD-6 or a new short AD should state whether mutation calls (`pause/resume/remove`) block until librqbit's state machine reflects the change (recommended — keeps the shell's model simple) or are fire-and-forget (in which case the shell must be told to ignore immediate re-polls and just wait for the next tick).

### F5 — [MEDIUM] Seed-duration semantics: wall-clock vs. app-open time, and whether it's an add-time constant or something the engine must keep re-evaluating

- **AD:** AD-5, Deferred item "per-torrent seed-duration override... out of v1 scope."
- **Choice A (Shell Unit / UI copy):** Assumes "default seed duration" means wall-clock hours since completion (e.g. "seeds for 48 hours after finishing," matching what a user would expect from any other torrent client), regardless of whether the app was open the whole time.
- **Choice B (Engine Unit):** Since there is no daemon (AD-1 — single in-process core, nothing runs while the app is closed), the only tractable implementation is "48 hours of cumulative *app-open* seeding time," tracked via an in-memory timer that pauses when the app quits and resumes on relaunch (comparing against a stored completion timestamp only approximates wall-clock and would need extra bookkeeping the spine never asks for).
- **Why they clash:** These are materially different behaviors (a torrent left seeding overnight with the app closed behaves totally differently under A vs B), and the spine offers no guidance for which one the engine should implement, while the Settings UI (built independently) will bake in copy/expectations for whichever one the shell author assumes.
- **Fix:** AD-5 should state explicitly whether "seed duration" is wall-clock-since-completion or cumulative-app-open-time, since AD-1 (no background daemon) makes wall-clock enforcement non-trivial and this is a real product-behavior decision, not an implementation detail.

### F6 — [MEDIUM] Snapshot granularity: does `TorrentStatus` carry Peers/Trackers/Files, or is Detail data a separate on-demand call?

- **AD:** AD-4 ("a single synchronous UniFFI snapshot function... `getAllTorrents() -> [TorrentStatus]`"), Capability Map row "02 — Alex Inspects a Torrent."
- **Choice A (Shell Unit):** Reads AD-4 literally — assumes `TorrentStatus` already includes nested `peers: [Peer]`, `trackers: [Tracker]`, `files: [TorrentFile]` arrays, since the Rule names `getAllTorrents()` as *the* snapshot mechanism and TorrentDetailView is only governed by AD-4/AD-6/AD-7 (no separate AD implying another call exists).
- **Choice B (Engine Unit):** Keeps the hot-path snapshot cheap (it's polled every ~1s for every torrent) by excluding per-peer/per-tracker detail from `TorrentStatus`, exposing a separate on-demand `get_torrent_details(id)` call instead, issued only while the detail window is open.
- **Why they clash:** These produce genuinely different UniFFI interfaces. Because the interface is code-generated from whichever side authors `engine/lib.rs`/UDL first, this particular mismatch *will* be caught at compile time rather than silently misbehaving at runtime — but if both units work in parallel (Shell Unit writing `TorrentDetailView` against an assumed flat-snapshot shape before `engine/` exists), one side eats a rewrite. Worth flagging as wasted-work risk even though it's not a silent-bug risk.
- **Fix:** State explicitly in the Structural Seed or AD-4 whether Files/Trackers/Peers ride along in the bulk snapshot or require a separate per-torrent call, and whether that separate call (if any) is itself polled on its own cadence while the detail view is open.

### F7 — [MEDIUM] `add_torrent` parameter shape for `.torrent` files is unspecified (path vs. bytes)

- **AD:** AD-2, Capability Map row "01 — Alex's Daily Download," and the Deferred item on Info.plist/file-association ("no cross-builder divergence risk — one file, one owner").
- **Choice A (interface author):** `add_torrent(path: String, ...)` — shell passes a filesystem path; engine reads and parses the file itself via librqbit, consistent with AD-2's "delegated to librqbit."
- **Choice B (interface author):** `add_torrent(bytes: Data, ...)` — shell reads the `.torrent` file into memory itself (e.g. because the file-association callback hands it a URL whose lifetime/access isn't guaranteed by the time a background `Task` on another thread gets to it) and passes raw bytes across FFI.
- **Why they clash:** Both are defensible, produce different function signatures, and — as in F6 — whichever side writes the UniFFI interface first effectively dictates it, so this is a wasted-work risk during parallel development rather than a silent runtime bug. The Deferred section calls the file-association piece "one file, one owner, no divergence risk," but that claim is about the Info.plist entitlements, not about the shape of the boundary call the file handoff feeds into — those are two different things the spine conflates.
- **Fix:** Either pin the parameter shape in the Structural Seed (`lib.rs` comment: `add_torrent(source: TorrentSource)` where `TorrentSource` is an enum over path/bytes/magnet) or explicitly note in Deferred that this shape is undecided and must be settled before either unit starts on the add-flow.

### F8 — [MEDIUM] "Governed by" column in the Capability → Architecture Map reads as exhaustive but isn't

- **AD:** Capability → Architecture Map, interacting with AD-6/AD-7 ("Binds: engine/ ↔ MyTorrent boundary" / "Binds: MyTorrent/" — i.e. blanket, not per-capability).
- **Choice A (Shell Unit):** Treats each row's "Governed by" list as the complete set of ADs relevant to that capability — row 1 ("01 — Alex's Daily Download") lists only AD-1, AD-2, AD-5, so `add_torrent` isn't wrapped in a background `Task` (AD-7) and its failure path isn't treated with the same rigor as AD-6 typed errors, since neither is cited for that row.
- **Choice B (Engine Unit):** Treats AD-6 and AD-7 as blanket rules that apply to *every* FFI call regardless of whether a given capability row cites them, since their own "Binds" fields say "engine/ ↔ MyTorrent boundary" and "MyTorrent/" — i.e. everything.
- **Why they clash:** `add_torrent` is exactly the kind of blocking, disk/network-touching, fallible call AD-6 and AD-7 exist to protect — and it's the one row where their omission is most consequential (a UI freeze on adding a large `.torrent`/resolving a magnet). The table's own framing ("Governed by") invites reading it as authoritative-and-complete, which is inconsistent with how AD-6/AD-7 declare their own scope.
- **Fix:** Add a one-line note above the Capability → Architecture Map: "This table lists capability-specific ADs only; ADs bound to 'all' or a whole directory (AD-1, AD-5, AD-6, AD-7) apply everywhere regardless of whether a row cites them."

### F9 — [LOW-MEDIUM] Torrent identity: hex case and v1/v2 (hybrid) infohash not pinned

- **AD:** Consistency Conventions — "Torrent identity."
- **Choice A (Engine Unit):** `TorrentStatus.id` is whatever librqbit internally normalizes to — plausibly lowercase hex, and for a hybrid v1/v2 torrent, whichever of the two infohashes librqbit treats as primary.
- **Choice B (Shell Unit):** If the shell ever needs to compute/compare an infohash client-side (e.g. optimistic "this torrent is already in your list" dedupe check parsed straight from a pasted magnet URI's `xt=urn:btih:`/`xt=urn:btmh:` parameter, before the engine round-trip confirms it), it does so using whatever case/hash-version the URI happened to use.
- **Why they clash:** A case mismatch (uppercase magnet-URI hex vs. engine's lowercase canonical `id`) or a v1-vs-v2 hash-selection mismatch on a hybrid torrent would make a naive string-equality dedupe check silently fail, producing either a false "already added" block or a genuine duplicate entry. Low severity for a hobby app (worst case is a duplicate row), but genuinely unspecified.
- **Fix:** Add to the "Torrent identity" convention row: canonical hex is lowercase; for hybrid v1+v2 torrents the v1 infohash is canonical (or whichever the team prefers) — and note that the shell must never compute/compare infohashes client-side, only use IDs as returned by the engine.

### F10 — [LOW] Speed/rate values: computed from real elapsed time, or assumed from the ~1s poll interval?

- **AD:** AD-4 ("~1 second timer" — explicitly approximate) + Consistency Conventions ("Data & formats — speeds in bytes/sec").
- **Choice A (Engine Unit):** Computes `bytes_per_sec` using its own internal timestamps (real elapsed time between internal samples) — correct regardless of actual poll cadence.
- **Choice B (Engine Unit, cheaper to write):** Computes it as `(bytes now − bytes at last snapshot query) / 1.0s`, implicitly assuming the caller polls at exactly 1.000s — which AD-4 itself says is only approximate ("~1s"), and which will drift under FFI overhead, App Nap, or a slow tick.
- **Why it matters:** This isn't a two-unit clash so much as a shared, unwritten assumption that a single engine implementer could get wrong in a way that's invisible until cadence drifts (e.g. app backgrounded), silently corrupting displayed speeds. Worth pinning down since nothing in the spine says which computation basis is required.
- **Fix:** Add a line to the "Data & formats" convention: "Rate/speed fields must be computed from actual elapsed time internally tracked by the engine, never assumed to equal the polling interval."

### F11 — [LOW] `id` vs `info_hash` field naming friction

- **AD:** Consistency Conventions — naming (snake_case → camelCase) + torrent identity.
- **Choice A (Engine Unit):** Names the field `info_hash: String` (accurate BitTorrent terminology, matches librqbit's own vocabulary) → becomes Swift `infoHash`.
- **Choice B (Shell Unit expectation):** Wants a property literally named `id` for `Identifiable` conformance (`ForEach(torrents, id: \.id)`), per the convention's own claim that infohash "is the canonical `Torrent.id` everywhere."
- **Why it's minor:** The Models/ directory is explicitly allowed to be a "thin wrapper" that adds a computed `var id: String { infoHash }`, so this is cheap to absorb — flagging only because the convention table's phrase "canonical `Torrent.id` everywhere" could be misread as a literal field-name mandate rather than a semantic one.
- **Fix:** Optional — clarify that "`Torrent.id`" names the *concept*, not a literal Rust field name; the UniFFI-generated field may be `info_hash`/`infoHash`.

---

## Summary Table

| # | Severity | Area |
| --- | --- | --- |
| F1 | HIGH | AD-4 bootstrap/launch polling chicken-and-egg |
| F2 | HIGH | AD-4 "active" undefined — Seeding fault line |
| F3 | HIGH | AD-4 timer must be app-lifetime, not View-lifetime |
| F4 | MEDIUM | Mutation call consistency race (pause/resume/remove) |
| F5 | MEDIUM | AD-5 seed-duration: wall-clock vs. app-open time |
| F6 | MEDIUM | Snapshot granularity: Peers/Trackers/Files inline or separate call |
| F7 | MEDIUM | `add_torrent` parameter shape: path vs. bytes |
| F8 | MEDIUM | Capability Map "Governed by" read as exhaustive vs. blanket ADs |
| F9 | LOW-MEDIUM | Infohash case / v1-v2 hybrid canonicalization |
| F10 | LOW | Speed computed from real time vs. assumed poll interval |
| F11 | LOW | `id` vs `info_hash` field-name friction |
