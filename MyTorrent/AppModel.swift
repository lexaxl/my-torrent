import Foundation
import os

private let logger = Logger(
    subsystem: Bundle.main.bundleIdentifier ?? "com.alex.mytorrent.MyTorrent",
    category: "engine"
)

@MainActor
final class AppModel: ObservableObject {
    @Published private(set) var torrents: [TorrentStatus] = []

    private let engine: Engine
    private var pollingTask: Task<Void, Never>?

    // Matches the Architecture Spine's Consistency Conventions
    // (active = {Downloading, Checking, Seeding}), plus "resolving" — a magnet
    // pending background resolution (Story 1.2) isn't in that table (written
    // before magnets existed), but must count as active or the poller would
    // never catch its transition into a real status.
    private static let activeStatuses: Set<String> = ["checking", "downloading", "seeding", "resolving"]

    init() {
        // Hardcoded until Settings exists (Story 3.1) — AD-5 is about who owns the
        // value and how it's passed, not about it being configurable yet.
        let downloadDir = FileManager.default
            .urls(for: .downloadsDirectory, in: .userDomainMask)
            .first?.path
            ?? NSString(string: "~/Downloads").expandingTildeInPath

        do {
            engine = try Engine(downloadDir: downloadDir)
        } catch {
            logger.fault("failed to initialize torrent engine: \(String(describing: error), privacy: .public)")
            fatalError("Failed to initialize torrent engine: \(error)")
        }

        // AD-4 bootstrap exemption: one unconditional snapshot at launch, regardless
        // of whether anything is known to be active yet, to discover torrents
        // librqbit auto-resumed from its own session state.
        Task { await bootstrap() }
    }

    private func bootstrap() async {
        await refreshTorrents()
    }

    func addTorrent(_ source: TorrentSource) async {
        let engine = self.engine

        let result: Result<String, EngineError> = await Task.detached {
            do {
                return .success(try engine.addTorrent(source: source))
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

    private func refreshTorrents() async {
        let engine = self.engine
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
