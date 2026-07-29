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
}

extension TorrentStatus {
    var engineStatus: TorrentEngineStatus {
        TorrentEngineStatus(rawValue: status) ?? .error
    }
}
