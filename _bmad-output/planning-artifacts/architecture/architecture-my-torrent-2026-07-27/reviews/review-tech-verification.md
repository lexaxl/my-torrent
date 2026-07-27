# Tech Verification Review — my-torrent Architecture Spine

**Reviewed:** ARCHITECTURE-SPINE.md + .memlog.md (architecture-my-torrent-2026-07-27)
**Method:** cross-checked each memlog "(version)" entry against the spine's claims, then independently re-verified the underlying factual claims via live web search (search results current as of this review).

## Verdict

Every load-bearing, version-sensitive claim in the Stack table checks out against current sources — no wrong or abandoned technology found — but two claims (UniFFI's Swift-6 partial-support note and the arm64 GitHub Actions runner) are asserted in the spine without a matching logged research line in the memlog, and the unsigned-.dmg distribution decision doesn't account for the macOS Gatekeeper quarantine friction real users will hit.

## Per-Technology Findings

### Rust — "latest stable toolchain"
**Status: confirmed (by nature of claim, no verification needed)**
Not a fixed version pin, so it can't go stale — no memlog entry backs it and none is needed. No concern.

### librqbit — "latest (crates.io; actively maintained, docs updated June 2026)"
**Status: confirmed, one granular sub-claim needs-recheck**
Memlog line 11 logs this as verified via web search 2026-07-27 ("actively maintained... docs updated June 2026, built on tokio, full-featured, backbone of rqbit"). My independent search confirms: crates.io/docs.rs shows librqbit 8.1.1, the ikatson/rqbit GitHub repo (1.6–1.7k stars) has open issues from Dec 2025/Jan/Feb 2026 and commits indexed as recently as March 31, 2026 — genuinely active, not abandoned. The precise "docs updated June 2026" sub-detail I could not independently pin to that exact month (my sources point to a March 2026 last-indexed commit and an "~11 months old" 8.1.1 release) — this is a minor date-granularity mismatch, not a wrong claim about maintenance status. Low severity; doesn't affect the decision.

### tokio — "1.x LTS (1.47.x until Sep 2026, or 1.51.x until Mar 2027)"
**Status: confirmed**
Memlog line 12 logs this as verified 2026-07-27. Independent search confirms verbatim: tokio's current LTS lines are 1.47.x (EOL Sep 2026, MSRV 1.70) and 1.51.x (EOL Mar 2027, MSRV 1.71), consistent with tokio's published LTS policy. Numbers match exactly. Solid.

### UniFFI (mozilla/uniffi-rs) — "latest (production-quality Swift bindings)"
**Status: confirmed**
Memlog line 8 logs this as verified 2026-07-27 ("production-quality... actively maintained by Mozilla... pre-1.0 but production-used in Firefox"). Independent search confirms: current version is 0.31.0 (Jan 2026), explicitly documented upstream as production-ready but pre-1.0. Matches.

### UniFFI's Swift 6 partial support (Stack table note + Deferred item)
**Status: confirmed, but provenance gap in the memlog**
The spine states "UniFFI has only *partial* Swift 6 (strict concurrency) support as of this research" (Stack table) and repeats this in Deferred, attributing it to "current research (2026-07-27)." However, the memlog's UniFFI line (line 8) does not actually contain this sub-claim — it only covers general maintenance/production-readiness, not the Swift 6 concurrency caveat. So this specific, architecturally-relevant claim appears in the spine without a corresponding logged verification step.
That said, I independently re-verified it and it is **accurate**: mozilla/uniffi-rs has open issues (#2818 — Xcode 26/Swift 6.2 actor-isolation config; #2274 — Swift 6 async functions produce data-race compile errors) confirming UniFFI-generated code doesn't fully conform to Swift 6 strict concurrency, especially for async functions and `@MainActor`-default-isolated modules. The spine's hedge ("target Swift 5 language mode if friction appears," in Deferred) is a reasonable, low-stakes mitigation. **Flag: correct conclusion reached without a matching audit trail — fix by adding the corresponding memlog line, not by changing the spine's content.**

### Swift 6 (language)
**Status: confirmed, no issue**
Current major Swift version as of 2026; not a claim that can go stale on its own. Covered above via the UniFFI interaction.

### SwiftUI / MenuBarExtra — "macOS 13 (Ventura)+"
**Status: confirmed**
Memlog line 9 logs this as verified 2026-07-27. Independent search confirms `MenuBarExtra` requires macOS 13.0+ per Apple's own documentation. Matches, and is compatible with the Product Brief's "last 1-2 macOS versions" target.

### Xcode — "latest stable"
**Status: confirmed (by nature of claim), no memlog verification logged**
Like the Rust toolchain, this is a moving-target statement rather than a fixed version pin, so staleness risk is minimal. No memlog line covers Xcode specifically — not concerning on its own, but noted for completeness since the task asked to check every named technology.

### CI — "GitHub Actions, macOS runner (arm64)"
**Status: confirmed by independent check; not verified in the memlog trail**
The Stack table's arm64 runner row was added per memlog line 20 ("added explicit Apple Silicon-only (arm64) row to Stack, missing from initial draft") as a reconciliation fix, but that entry is not paired with a "(version)" research line confirming arm64 GitHub-hosted runner availability. Independently verified now: GitHub-hosted macOS runners have supported Apple Silicon/arm64 since the M1 runner public beta, and as of Feb 2026 macos-26 images are GA and run natively on arm64 — so the claim holds. **Flag: same provenance gap pattern as the UniFFI Swift 6 note — true, but wasn't logged as researched.**

### AD-8 — Unsigned `.dmg` via GitHub Releases (distribution reality-check)
**Status: needs-recheck — not a wrong tech fact, but a missing real-world consideration**
This isn't a version/maintenance claim but is a "reality" the architecture should reflect: macOS attaches a `com.apple.quarantine` extended attribute to anything downloaded via a browser (including a GitHub Release asset), and Gatekeeper will refuse to open an unsigned app downloaded this way ("app is damaged" / "unidentified developer"), requiring the user to right-click → Open, or run `xattr -cr`, or disable Gatekeeper. AD-8 and the Deferred section don't mention this. For a $0-budget, unsigned, hobby-project distribution model this is expected and low-severity, but it will be the very first thing a user (even the author, on a fresh Mac) hits when trying the release build — worth a one-line README/release-notes callout at implementation time. Not a blocker, not a wrong decision — flagging so it doesn't surprise anyone during the release-build phase.

## Summary Table

| Technology | Memlog verified? | Independently re-checked? | Verdict |
| --- | --- | --- | --- |
| Rust (latest stable) | n/a (non-pinned) | n/a | confirmed |
| librqbit | yes (2026-07-27) | yes | confirmed (minor date-granularity mismatch on "June 2026") |
| tokio | yes (2026-07-27) | yes | confirmed |
| UniFFI (general) | yes (2026-07-27) | yes | confirmed |
| UniFFI Swift 6 partial support | **no** (not in memlog) | yes | confirmed, but provenance gap |
| Swift 6 (language) | n/a | n/a | confirmed |
| SwiftUI / MenuBarExtra macOS 13+ | yes (2026-07-27) | yes | confirmed |
| Xcode (latest stable) | n/a (non-pinned) | n/a | confirmed |
| GitHub Actions macOS runner (arm64) | **no** (not in memlog) | yes | confirmed, but provenance gap |
| AD-8 unsigned .dmg / Gatekeeper | n/a (decision, not a fact) | yes | needs a callout — real friction not mentioned |
