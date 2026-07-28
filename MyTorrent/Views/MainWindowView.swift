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
    @State private var torrentPendingRemoval: TorrentStatus?

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
        .confirmationDialog(
            "main_window.row.context_menu.remove_confirm_title",
            isPresented: Binding(
                get: { torrentPendingRemoval != nil },
                set: { if !$0 { torrentPendingRemoval = nil } }
            ),
            presenting: torrentPendingRemoval
        ) { torrent in
            Button("main_window.row.context_menu.remove", role: .destructive) {
                Task { await appModel.removeTorrent(torrent.id) }
            }
        } message: { _ in
            Text("main_window.row.context_menu.remove_confirm_message")
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
                openWindow(id: "settings")
            } label: {
                Image(systemName: "gearshape")
            }
            .buttonStyle(.borderless)
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
            Text(Formatting.speed(torrent.downSpeedBps))
                .frame(width: speedColumnWidth, alignment: .trailing)
            Text(Formatting.speed(torrent.upSpeedBps))
                .frame(width: speedColumnWidth, alignment: .trailing)
            // Total connected peers, not a seeds/leechers split — librqbit doesn't
            // expose that breakdown (see Story 1.3 Scope Boundary).
            Text("\(torrent.peersConnected)")
                .frame(width: seedLeechColumnWidth, alignment: .trailing)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        // `.contentShape` — UX spec wants the entire row clickable, not just where
        // text/controls render (there's no Spacer filling the gaps between the
        // fixed-width columns otherwise). `.simultaneousGesture`, not
        // `.onTapGesture` — on macOS a plain `.onTapGesture` alongside
        // `.contextMenu` on the same view doesn't reliably fire (the context-menu
        // recognizer wins the gesture), confirmed while manually verifying this
        // story; `.simultaneousGesture` runs independently instead of competing.
        .contentShape(Rectangle())
        .simultaneousGesture(
            TapGesture().onEnded {
                openWindow(id: "torrent-detail", value: torrent.id)
            }
        )
        .contextMenu {
            Button {
                Task {
                    // Look up the live status at click time rather than trusting
                    // the `torrent` value captured when this menu was built — the
                    // 1s poll loop can change it while the menu is open, and acting
                    // on a stale snapshot here (unlike the label below, which macOS
                    // doesn't let us update once the menu is showing) would call
                    // the wrong one of pause/resume and silently fail.
                    let liveStatus = appModel.torrents.first(where: { $0.id == torrent.id })?.status
                    if liveStatus == "paused" {
                        await appModel.resumeTorrent(torrent.id)
                    } else {
                        await appModel.pauseTorrent(torrent.id)
                    }
                }
            } label: {
                Text(torrent.status == "paused" ? "main_window.row.context_menu.resume" : "main_window.row.context_menu.pause")
            }
            .disabled(torrent.status == "error")
            Button(role: .destructive) {
                torrentPendingRemoval = torrent
            } label: {
                Text("main_window.row.context_menu.remove")
            }
            Button {
                Task { await appModel.revealInFinder(torrent.id) }
            } label: {
                Text("main_window.row.context_menu.reveal_in_finder")
            }
        }
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
