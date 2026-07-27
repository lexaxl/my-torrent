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
    }
}
