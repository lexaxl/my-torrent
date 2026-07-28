import AppKit
import Foundation
import os

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

    // Matches the Architecture Spine's Consistency Conventions
    // (active = {Downloading, Checking, Seeding}), plus "resolving" — a magnet
    // pending background resolution (Story 1.2) isn't in that table (written
    // before magnets existed), but must count as active or the poller would
    // never catch its transition into a real status.
    private static let activeStatuses: Set<String> = ["checking", "downloading", "seeding", "resolving"]

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

    private func refreshTorrents() async {
        guard let engine else { return }
        torrents = await Task.detached { engine.getAllTorrents() }.value
        startPollingIfNeeded()
    }

    // AD-4: owned by AppModel (app-lifetime), not a View — survives every window
    // being closed. Starts on demand (bootstrap, or a newly-added active torrent)
    // rather than unconditionally, and stops itself once nothing is active anymore.
    private func startPollingIfNeeded() {
        guard pollingTask == nil else { return }
        guard torrents.contains(where: { Self.activeStatuses.contains($0.status) }) else { return }

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
            if !torrents.contains(where: { Self.activeStatuses.contains($0.status) }) {
                break
            }
        }
        pollingTask = nil
    }
}
