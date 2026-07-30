import Foundation

// Shared closed-set representation of the engine's `TorrentStatus.status` raw
// string (`derive_status` in `engine/src/lib.rs`, plus the client-only
// `"resolving"` state for a not-yet-resolved pending magnet — Story 1.2).
//
// Story 5.1 code review found three independent places (this file's callers:
// `AppModel.activeStatuses`/`seedingStatus`, and `MainWindowView`'s subtitle
// switch + context-menu equality checks) each hardcoding this vocabulary as
// raw string literals with no shared anchor — an engine change adding a new
// status (e.g. the "Completed-not-seeding" status `ARCHITECTURE-SPINE.md`'s
// Deferred section anticipates once seed-duration enforcement ships) could
// silently fall through one call site's logic without a compiler error. This
// enum is that anchor; `.error` is the explicit, singular fallback for any
// status string not in this list — not a silently-diverging `default` at
// each call site.
enum TorrentEngineStatus: String, Hashable {
    case downloading
    case seeding
    case paused
    case checking
    case resolving
    case error

    // Both classifications below are deliberately exhaustive `switch`es with no
    // `default` (Story 6.1 code review): a `Set` literal in a caller gives no
    // compiler signal when a case is added here, which is exactly the silent
    // fall-through this enum exists to prevent — adding a case must break the
    // build at these two switches until each new status is classified.

    // The active/inactive partition (Architecture Spine Consistency Conventions:
    // active = {Downloading, Checking, Seeding}), plus `resolving` — a magnet
    // pending background resolution (Story 1.2) isn't in that table (written
    // before magnets existed), but must count as active or the poller would
    // never catch its transition into a real status.
    var isActive: Bool {
        switch self {
        case .downloading, .seeding, .checking, .resolving: true
        case .paused, .error: false
        }
    }

    // The quit-confirmation partition (Story 6.1) — deliberately NARROWER than
    // `isActive`: seeding is "active" for polling and menu-bar purposes, but a
    // finished torrent seeds indefinitely, so counting it here would make the
    // quit dialog appear on almost every quit. Only genuinely unfinished work
    // warrants the interruption warning — and it is a real interruption: the
    // session has no persistence (librqbit `SessionOptions::default()`), so the
    // torrent list does not survive an app restart.
    var blocksQuit: Bool {
        switch self {
        case .downloading, .checking, .resolving: true
        case .seeding, .paused, .error: false
        }
    }
}

extension TorrentStatus {
    var engineStatus: TorrentEngineStatus {
        TorrentEngineStatus(rawValue: status) ?? .error
    }
}
