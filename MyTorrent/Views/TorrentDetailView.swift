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
        case .trackers:
            trackersList
        case .peers:
            peersList
        }
    }

    // Shared by filesList/trackersList — both list a fixed-order collection with
    // no natural per-row identity beyond position (code-review fix, Story 2.2:
    // was duplicated ScrollView/VStack/ForEach/Divider scaffolding in each). Peers
    // aren't built on this helper — its rows are keyed by peer address instead of
    // position (see peersList).
    private func rowList<Item, Row: View>(
        _ items: [Item],
        @ViewBuilder row: @escaping (Item) -> Row
    ) -> some View {
        ScrollView {
            VStack(spacing: 0) {
                ForEach(Array(items.enumerated()), id: \.offset) { _, item in
                    row(item)
                    Divider()
                }
            }
        }
    }

    private var filesList: some View {
        rowList(detail?.files ?? []) { fileRow($0) }
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

    // No status column — librqbit doesn't expose per-tracker announce status
    // publicly, only the URL list (Story 2.2 Scope Boundary). No empty-state
    // text either: not specified in the UX spec, unlike the Peers tab below.
    private var trackersList: some View {
        rowList(detail?.trackers ?? []) { trackerRow($0) }
    }

    private func trackerRow(_ tracker: Tracker) -> some View {
        Text(tracker.url)
            .lineLimit(1)
            .truncationMode(.middle)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .frame(maxWidth: .infinity, alignment: .leading)
    }

    // Role (seeder/leecher) and upload speed per peer aren't available from
    // librqbit either (Story 2.2 Scope Boundary) — address and a download speed
    // derived engine-side from polling deltas. `Peer.state` is intentionally not
    // shown: the engine only ever reports live-connected peers here (see Scope
    // Boundary), so the value is always the literal "live" — a column that can
    // never vary isn't meaningful to display (code-review fix, Story 2.2). The
    // field stays on the wire type in case a richer state becomes available later.
    //
    // Rows are keyed by address, not position (code-review fix, Story 2.2): the
    // engine rebuilds this list fresh every ~1s poll with no guaranteed order, so
    // a position-based id could make rows visually reorder with no real change.
    private var peersList: some View {
        let peers = detail?.peers ?? []
        return Group {
            if peers.isEmpty {
                Text("torrent_detail.peers.empty")
                    .font(.body)
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    VStack(spacing: 0) {
                        ForEach(peers, id: \.address) { peer in
                            peerRow(peer)
                            Divider()
                        }
                    }
                }
            }
        }
    }

    private func peerRow(_ peer: Peer) -> some View {
        HStack(spacing: 12) {
            Text(peer.address)
                .lineLimit(1)
                .truncationMode(.middle)
            Spacer()
            Text("\(Formatting.speed(peer.downSpeedBps)) ↓")
                .foregroundStyle(.secondary)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
    }
}

#Preview {
    TorrentDetailView(torrentId: "")
        .environmentObject(AppModel())
}
