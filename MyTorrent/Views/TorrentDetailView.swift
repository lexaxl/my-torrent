import SwiftUI

struct TorrentDetailView: View {
    let torrentId: String

    @EnvironmentObject private var appModel: AppModel
    @Environment(\.dismiss) private var dismiss
    @State private var detail: TorrentDetail?
    @State private var selectedTab: Tab = .files

    private enum Tab {
        case files, trackers, peers
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            tabSwitcher
            Divider()
            content
        }
        .frame(minWidth: 360, minHeight: 320)
        .navigationTitle(detail?.name ?? "")
        // Detail data is fetched on its own ~1s cadence, separate from AppModel's
        // app-lifetime list poll (Consistency Conventions — "Snapshot granularity").
        // `.task(id:)` ties this loop to the view's own lifecycle: it starts when
        // the window opens and is cancelled automatically when it closes, with no
        // manual start/stop bookkeeping needed.
        .task(id: torrentId) {
            while !Task.isCancelled {
                detail = await appModel.fetchTorrentDetail(torrentId)
                try? await Task.sleep(for: .seconds(1))
            }
        }
    }

    private var header: some View {
        HStack(spacing: 8) {
            Button {
                dismiss()
            } label: {
                Image(systemName: "xmark")
            }
            .buttonStyle(.borderless)

            VStack(alignment: .leading, spacing: 2) {
                Text(detail?.name ?? "")
                    .font(.headline)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Text(summaryLine)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(12)
    }

    // "72% · 4.2 MB/s ↓ · 0.3 MB/s ↑ · 12" per the UX spec's header summary format.
    // Empty (not "0%...") while `detail` hasn't loaded yet — e.g. the torrent is
    // still only `pending` (Scope Boundary) — rather than showing misleading zeros.
    private var summaryLine: String {
        guard let detail else { return "" }
        return "\(Int(detail.progressPercent))% · \(Formatting.speed(detail.downSpeedBps)) ↓ · \(Formatting.speed(detail.upSpeedBps)) ↑ · \(detail.peersConnected)"
    }

    private var tabSwitcher: some View {
        Picker("", selection: $selectedTab) {
            Text("torrent_detail.tabs.files").tag(Tab.files)
            Text("torrent_detail.tabs.trackers").tag(Tab.trackers)
            Text("torrent_detail.tabs.peers").tag(Tab.peers)
        }
        .pickerStyle(.segmented)
        .labelsHidden()
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }

    @ViewBuilder
    private var content: some View {
        switch selectedTab {
        case .files:
            filesList
        case .trackers, .peers:
            // Content wired up in Story 2.2 — the tab exists and switches (matches
            // Story 2.2's own AC, which assumes the switcher is already there), it
            // just has nothing to show yet. Same "visible, not yet functional" step
            // this project already took with the search field (Story 1.1 → 1.4).
            Color.clear
        }
    }

    private var filesList: some View {
        ScrollView {
            VStack(spacing: 0) {
                ForEach(Array((detail?.files ?? []).enumerated()), id: \.offset) { _, file in
                    fileRow(file)
                    Divider()
                }
            }
        }
    }

    private func fileRow(_ file: TorrentFile) -> some View {
        HStack(spacing: 12) {
            Text(file.name)
                .lineLimit(1)
                .truncationMode(.middle)
            Spacer()
            Text(Formatting.size(file.sizeBytes))
                .foregroundStyle(.secondary)
            ProgressView(value: file.progressPercent, total: 100)
                .frame(width: 90)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }
}

#Preview {
    TorrentDetailView(torrentId: "")
        .environmentObject(AppModel())
}
