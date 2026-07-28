import AppKit
import Foundation
import os
import SwiftUI
import UserNotifications

private let logger = Logger(
    subsystem: Bundle.main.bundleIdentifier ?? "com.alex.mytorrent.MyTorrent",
    category: "engine"
)

@MainActor
final class AppModel: ObservableObject {
    @Published private(set) var torrents: [TorrentStatus] = []

    // Optional, not `let` — construction is a blocking FFI call (Session::new),
    // which AD-9 requires off the main thread. `init()` can't be async (SwiftUI's
    // `@StateObject` requires a synchronous initializer), so this starts as nil
    // and is set once `setUpEngine` finishes in the background. Every method that
    // reads it guards against the brief pre-initialization window.
    private var engine: Engine?
    private var pollingTask: Task<Void, Never>?

    // nil until the first snapshot — distinguishes "no snapshot yet" (bootstrap,
    // AD-4/AC3: don't notify about torrents already seeding when the app launched)
    // from "snapshot was empty". Keyed by torrent id (infohash), value is that
    // torrent's status as of the previous snapshot.
    private var previousStatusByID: [String: String]?

    // Matches the Architecture Spine's Consistency Conventions
    // (active = {Downloading, Checking, Seeding}), plus "resolving" — a magnet
    // pending background resolution (Story 1.2) isn't in that table (written
    // before magnets existed), but must count as active or the poller would
    // never catch its transition into a real status.
    private static let seedingStatus = "seeding"
    private static let activeStatuses: Set<String> = ["checking", "downloading", seedingStatus, "resolving"]

    // Single source of truth for the active/inactive partition — `startPollingIfNeeded`/
    // `pollLoop` (below) and the menu-bar popover (Story 4.1) both read this instead of
    // each re-checking `activeStatuses` independently.
    var hasActiveTorrents: Bool {
        torrents.contains(where: { Self.activeStatuses.contains($0.status) })
    }

    // Second consumer of the active/inactive partition (Story 4.1's menu-bar
    // popover's stats section, which needs the actual elements to sum speeds —
    // `hasActiveTorrents` above covers the boolean-only cases).
    // Deliberately includes "resolving" torrents (0/0 speed, no bytes yet) even
    // though a resolving magnet reads a little oddly as an "active download" in
    // the popover — magnet resolution is normally sub-second, and forking this
    // definition into a separate "poll-active" vs "display-active" set purely to
    // hide that brief blip isn't worth the two-definitions-to-keep-in-sync risk.
    var activeTorrents: [TorrentStatus] {
        torrents.filter { Self.activeStatuses.contains($0.status) }
    }

    // Shared by MainWindowView.handleOpenURL and MenuBarView's popover actions —
    // both need to activate the app before opening a window, since either can be
    // triggered while MyTorrent (an LSUIElement accessory app) isn't frontmost.
    static func activateAndOpenWindow(id: String, openWindow: OpenWindowAction) {
        NSApp.activate(ignoringOtherApps: true)
        openWindow(id: id)
    }

    init() {
        Task { await setUpEngine(downloadDir: AppSettings.saveLocationPath) }
    }

    private func setUpEngine(downloadDir: String) async {
        do {
            engine = try await Task.detached { try Engine(downloadDir: downloadDir) }.value
        } catch {
            logger.fault("failed to initialize torrent engine: \(String(describing: error), privacy: .public)")
            fatalError("Failed to initialize torrent engine: \(error)")
        }

        // Fire-and-forget — doesn't block the bootstrap snapshot below. Safe to
        // call on every launch: if the user already granted/denied, this returns
        // the stored decision without re-prompting. Denial isn't surfaced as an
        // error to the user (same no-user-facing-error-UI convention as
        // addTorrent/mutate below) — notifications just silently don't show.
        // Code-review-noted, accepted gap: a torrent that finishes in the narrow
        // window before the user answers this (first-launch-only) system prompt
        // has its completion notification silently dropped — `add(_:)` is called
        // once per transition (no retry/queue), so it isn't re-sent once granted.
        // Not fixed: this window is a few seconds on a single first-ever launch,
        // and queuing/replaying missed completions is real added machinery for a
        // vanishingly rare race in a single-user hobby app.
        Task {
            do {
                let granted = try await UNUserNotificationCenter.current()
                    .requestAuthorization(options: [.alert, .sound])
                logger.info("notification authorization granted: \(granted, privacy: .public)")
            } catch {
                logger.error("notification authorization request failed: \(String(describing: error), privacy: .public)")
            }
        }

        // AD-4 bootstrap exemption: one unconditional snapshot at launch, regardless
        // of whether anything is known to be active yet, to discover torrents
        // librqbit auto-resumed from its own session state.
        await refreshTorrents()
    }

    func addTorrent(_ source: TorrentSource) async {
        guard let engine else {
            logger.error("addTorrent called before engine finished initializing")
            return
        }
        // Read on the MainActor before detaching, same as `source`/`engine` above —
        // whatever the Settings window currently holds at the moment of this call
        // (AD-5: settings flow one-directionally into Rust as call parameters).
        let downloadDir = AppSettings.saveLocationPath

        let result: Result<String, EngineError> = await Task.detached {
            do {
                return .success(try engine.addTorrent(source: source, downloadDir: downloadDir))
            } catch let error as EngineError {
                return .failure(error)
            } catch {
                return .failure(.Internal(message: "\(error)"))
            }
        }.value

        switch result {
        case .success:
            await refreshTorrents()
        case .failure(let error):
            logger.error("addTorrent failed: \(String(describing: error), privacy: .public)")
        }
    }

    func pauseTorrent(_ id: String) async {
        await mutate(id, label: "pauseTorrent") { engine, id in try engine.pauseTorrent(id: id) }
    }

    func resumeTorrent(_ id: String) async {
        await mutate(id, label: "resumeTorrent") { engine, id in try engine.resumeTorrent(id: id) }
    }

    func removeTorrent(_ id: String) async {
        await mutate(id, label: "removeTorrent") { engine, id in try engine.removeTorrent(id: id) }
    }

    // Shared by pauseTorrent/resumeTorrent/removeTorrent — same shape (FFI call off
    // the main thread per AD-9, refresh on success, log-and-swallow on failure per
    // the existing no-user-facing-error-UI convention).
    private func mutate(
        _ id: String,
        label: String,
        _ operation: @escaping (Engine, String) throws -> Void
    ) async {
        guard let engine else {
            logger.error("\(label, privacy: .public) called before engine finished initializing")
            return
        }
        do {
            try await Task.detached { try operation(engine, id) }.value
            await refreshTorrents()
        } catch {
            logger.error("\(label, privacy: .public) failed: \(String(describing: error), privacy: .public)")
        }
    }

    // Not a mutation, but still an FFI call — stays off the main thread per AD-9
    // regardless. Reveals (selects) the torrent's output folder in Finder rather
    // than opening it, matching "Show in Finder" semantics. Checks the folder
    // actually exists first — `NSWorkspace.selectFile` silently does nothing on a
    // missing path (e.g. right-clicking before librqbit has created it yet), and
    // that would otherwise look identical to a successful reveal.
    func revealInFinder(_ id: String) async {
        guard let engine else {
            logger.error("revealInFinder called before engine finished initializing")
            return
        }
        do {
            let path = try await Task.detached { try engine.revealPath(id: id) }.value
            guard FileManager.default.fileExists(atPath: path) else {
                logger.error("revealInFinder: output folder does not exist yet: \(path, privacy: .public)")
                return
            }
            NSWorkspace.shared.selectFile(path, inFileViewerRootedAtPath: "")
        } catch {
            logger.error("revealInFinder failed: \(String(describing: error), privacy: .public)")
        }
    }

    // A read, not a mutation — no `refreshTorrents()` afterward, and returns the
    // value instead of just success/failure, so it doesn't fit `mutate()`'s shape.
    // Still an FFI call, still off the main thread per AD-9. Called on its own ~1s
    // cadence by `TorrentDetailView` while its window is open (Story 2.1 Dev Notes
    // — separate from this class's own app-lifetime list poll).
    func fetchTorrentDetail(_ id: String) async -> TorrentDetail? {
        guard let engine else {
            logger.error("fetchTorrentDetail called before engine finished initializing")
            return nil
        }
        do {
            return try await Task.detached { try engine.getTorrentDetails(id: id) }.value
        } catch {
            logger.error("fetchTorrentDetail failed: \(String(describing: error), privacy: .public)")
            return nil
        }
    }

    // Code-review fix: `refreshTorrents()` is called from 4 independent sites
    // (bootstrap, addTorrent, mutate, pollLoop), each suspending at the FFI call
    // below before touching `torrents`/`previousStatusByID` — `@MainActor` alone
    // doesn't make that atomic across the suspension, so two overlapping calls
    // could resume out of start-order and roll the completion baseline back to a
    // stale snapshot. Chaining each call after whichever one is already in flight
    // (captured synchronously, before any `await`, so a concurrent caller always
    // sees the latest link) makes the diff-then-assign step run in strict FIFO
    // order without skipping any call's own fresh FFI read.
    private var refreshChain: Task<Void, Never>?

    private func refreshTorrents() async {
        let previous = refreshChain
        let task = Task { [weak self] in
            await previous?.value
            await self?.performRefresh()
        }
        refreshChain = task
        await task.value
    }

    private func performRefresh() async {
        guard let engine else { return }
        let newTorrents = await Task.detached { engine.getAllTorrents() }.value
        detectCompletionsAndNotify(newTorrents: newTorrents)
        torrents = newTorrents
        startPollingIfNeeded()
    }

    // AD-4: "the shell detects a torrent's transition into Completed/Seeding by
    // diffing consecutive snapshots... No separate 'on complete' callback exists."
    // The engine has no distinct "completed" status (derive_status maps a finished
    // torrent straight to "seeding" — see Story 4.2 Scope Boundary), so a
    // transition into "seeding" from anything else is the only completion signal.
    private func detectCompletionsAndNotify(newTorrents: [TorrentStatus]) {
        defer {
            // Replaced wholesale, not merged — naturally prunes ids no longer
            // present (same pattern as the engine's own per-peer baseline map,
            // see build_peers_prunes_baseline_for_disconnected_peers).
            previousStatusByID = Dictionary(uniqueKeysWithValues: newTorrents.map { ($0.id, $0.status) })
        }
        // Bootstrap snapshot (AD-4): nothing to diff against yet — don't notify
        // about torrents already seeding when the app launched (AC3).
        guard let previousStatusByID else { return }

        // Code-review fix: require a REAL prior observation of this id (not just
        // "absent" defaulting to "wasn't seeding") before treating "seeding" as a
        // transition. Without `let previous =`, an id absent from the dictionary —
        // a torrent removed then re-added with the same infohash, or a freshly
        // added torrent for content already fully on disk — read as "not seeding"
        // and fired a notification for something that never transitioned this
        // session. Requiring an existing, non-seeding prior entry closes both.
        // Code-review-noted, accepted gap: no batching — if several torrents
        // transition in the same ~1s tick, this posts one system notification per
        // torrent back-to-back. Realistic for this single-user hobby app's torrent
        // counts; a combined "N downloads complete" banner would be over-building
        // for a case that's rare and merely a minor UX flood, not a correctness bug.
        for torrent in newTorrents where torrent.status == Self.seedingStatus {
            if let previous = previousStatusByID[torrent.id], previous != Self.seedingStatus {
                postCompletionNotification(torrentID: torrent.id, torrentName: torrent.name)
            }
        }
    }

    private func postCompletionNotification(torrentID: String, torrentName: String) {
        let content = UNMutableNotificationContent()
        content.title = String(localized: "notification.download_complete.title")
        // Clamped — some real-world .torrent files declare pathologically long
        // names, which would otherwise render as a visually broken banner.
        content.body = String(torrentName.prefix(200))
        content.sound = .default
        // Code-review fix: identifier derived from the torrent id (was a random
        // UUID) — lets a later "seeding" re-transition for the very same torrent
        // (see Scope Boundary's accepted pause/resume-through-checking case)
        // replace the prior banner instead of stacking an unrelated duplicate,
        // and leaves room for a future cancel-on-remove without a signature change.
        let request = UNNotificationRequest(
            identifier: "completion.\(torrentID)",
            content: content,
            trigger: nil
        )
        Task {
            do {
                try await UNUserNotificationCenter.current().add(request)
            } catch {
                logger.error("failed to post completion notification: \(String(describing: error), privacy: .public)")
            }
        }
    }

    // AD-4: owned by AppModel (app-lifetime), not a View — survives every window
    // being closed. Starts on demand (bootstrap, or a newly-added active torrent)
    // rather than unconditionally, and stops itself once nothing is active anymore.
    private func startPollingIfNeeded() {
        guard pollingTask == nil else { return }
        guard hasActiveTorrents else { return }

        pollingTask = Task { [weak self] in
            await self?.pollLoop()
        }
    }

    private func pollLoop() async {
        while !Task.isCancelled {
            try? await Task.sleep(for: .seconds(1))
            // Calls back into `startPollingIfNeeded()`, which no-ops here since
            // `pollingTask` is still this very task (not yet nil).
            await refreshTorrents()
            if !hasActiveTorrents {
                break
            }
        }
        pollingTask = nil
    }
}
