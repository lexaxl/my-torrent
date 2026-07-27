import AppKit
import SwiftUI
import os

private let logger = Logger(
    subsystem: Bundle.main.bundleIdentifier ?? "com.alex.mytorrent.MyTorrent",
    category: "open-url"
)

struct MainWindowView: View {
    @EnvironmentObject private var appModel: AppModel
    @Environment(\.openWindow) private var openWindow
    @State private var searchText: String = ""

    let appDelegate: AppDelegate

    private let nameColumnWidth: CGFloat = 220
    private let progressColumnWidth: CGFloat = 90
    private let speedColumnWidth: CGFloat = 70
    private let seedLeechColumnWidth: CGFloat = 70

    var body: some View {
        VStack(spacing: 0) {
            toolbar
            Divider()
            if appModel.torrents.isEmpty {
                emptyState
            } else {
                downloadsList
            }
        }
        .frame(minWidth: 480, minHeight: 320)
        .onAppear {
            appDelegate.onOpenURLs = { urls in
                for url in urls {
                    handleOpenURL(url)
                }
            }
        }
    }

    // `openWindow(id:)` (unlike a raw `NSApp.windows.first` lookup) is scene-aware
    // for this singleton `Window("main")` — it activates the existing window, or
    // recreates it if the user closed it while the app kept running.
    private func handleOpenURL(_ url: URL) {
        NSApp.activate(ignoringOtherApps: true)
        openWindow(id: "main")

        let source: TorrentSource
        if url.scheme == "magnet" {
            source = .magnet(uri: url.absoluteString)
        } else if url.isFileURL {
            source = .path(path: url.path)
        } else {
            logger.error("open: unsupported URL scheme \(url.scheme ?? "nil", privacy: .public)")
            return
        }

        Task {
            await appModel.addTorrent(source)
        }
    }

    private var toolbar: some View {
        HStack(spacing: 12) {
            Text("my-torrent")
                .font(.headline)

            TextField("main_window.toolbar.search_placeholder", text: $searchText)
                .textFieldStyle(.roundedBorder)
                .frame(maxWidth: 220)

            Spacer()

            Button {
                // Открывает окно настроек (3.1) — реализуется в Story 3.1/3.2.
            } label: {
                Image(systemName: "gearshape")
            }
            .buttonStyle(.borderless)
            .disabled(true)
        }
        .padding(12)
    }

    // Local, client-side filter over the already-loaded list — no network call,
    // matches UX spec `main-window-toolbar-search-field`. Deliberately not used
    // for the emptyState/downloadsList switch below: that's about "no torrents at
    // all", not "search matched nothing" — see Story 1.4 Scope Boundary.
    private var filteredTorrents: [TorrentStatus] {
        guard !searchText.isEmpty else { return appModel.torrents }
        return appModel.torrents.filter { $0.name.localizedCaseInsensitiveContains(searchText) }
    }

    private var downloadsList: some View {
        VStack(spacing: 0) {
            columnHeaders
            Divider()
            ScrollView {
                VStack(spacing: 0) {
                    ForEach(filteredTorrents, id: \.id) { torrent in
                        torrentRow(torrent)
                        Divider()
                    }
                }
            }
        }
    }

    private var columnHeaders: some View {
        HStack(spacing: 12) {
            Text("main_window.list.header.name")
                .frame(width: nameColumnWidth, alignment: .leading)
            Text("main_window.list.header.progress")
                .frame(width: progressColumnWidth, alignment: .leading)
            Text("main_window.list.header.down_speed")
                .frame(width: speedColumnWidth, alignment: .trailing)
            Text("main_window.list.header.up_speed")
                .frame(width: speedColumnWidth, alignment: .trailing)
            Text("main_window.list.header.seed_leech")
                .frame(width: seedLeechColumnWidth, alignment: .trailing)
        }
        .font(.caption)
        .foregroundStyle(.secondary)
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
    }

    private func torrentRow(_ torrent: TorrentStatus) -> some View {
        HStack(spacing: 12) {
            Text(torrent.name)
                .frame(width: nameColumnWidth, alignment: .leading)
                .lineLimit(1)
                .truncationMode(.middle)
            ProgressView(value: torrent.progressPercent, total: 100)
                .frame(width: progressColumnWidth, alignment: .leading)
            Text(Self.formatSpeed(torrent.downSpeedBps))
                .frame(width: speedColumnWidth, alignment: .trailing)
            Text(Self.formatSpeed(torrent.upSpeedBps))
                .frame(width: speedColumnWidth, alignment: .trailing)
            // Total connected peers, not a seeds/leechers split — librqbit doesn't
            // expose that breakdown (see Story 1.3 Scope Boundary).
            Text("\(torrent.peersConnected)")
                .frame(width: seedLeechColumnWidth, alignment: .trailing)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    private static let speedFormatter: ByteCountFormatter = {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .binary
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        return formatter
    }()

    private static func formatSpeed(_ bytesPerSecond: UInt64) -> String {
        guard bytesPerSecond > 0 else { return "—" }
        return "\(speedFormatter.string(fromByteCount: Int64(bytesPerSecond)))/s"
    }

    private var emptyState: some View {
        Text("main_window.empty_state.message")
            .font(.system(size: 13))
            .foregroundStyle(.secondary)
            .padding(32)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

#Preview {
    MainWindowView(appDelegate: AppDelegate())
        .environmentObject(AppModel())
}
