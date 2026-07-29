import Foundation

// Shared by MainWindowView (row speeds) and TorrentDetailView (header speeds,
// per-file sizes) — was duplicated as a private helper in MainWindowView until
// Story 2.1 needed the same formatting in a second file.
enum Formatting {
    private static let byteCountFormatter: ByteCountFormatter = {
        let formatter = ByteCountFormatter()
        formatter.countStyle = .binary
        formatter.allowedUnits = [.useKB, .useMB, .useGB]
        return formatter
    }()

    static func speed(_ bytesPerSecond: UInt64) -> String {
        guard bytesPerSecond > 0 else { return "—" }
        return "\(byteCountFormatter.string(fromByteCount: Int64(bytesPerSecond)))/s"
    }

    static func size(_ bytes: UInt64) -> String {
        byteCountFormatter.string(fromByteCount: Int64(bytes))
    }

    // Shared by TorrentDetailView.summaryLine and the menu-bar hover tooltip
    // (Story 5.3) — was duplicated inline in both until this, the second
    // occurrence, per this file's own extract-on-second-use convention above.
    static func speedPair(down: UInt64, up: UInt64) -> String {
        "\(speed(down)) ↓ · \(speed(up)) ↑"
    }

    private static let etaFormatter: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour, .minute]
        formatter.maximumUnitCount = 1
        formatter.unitsStyle = .abbreviated
        return formatter
    }()

    // nil when ETA isn't meaningful (Story 5.1 AC2): no speed, nothing left to
    // download, or the interval rounds under a minute — omit the fragment rather
    // than show "0 min left".
    static func eta(remainingBytes: UInt64, downSpeedBps: UInt64) -> String? {
        guard downSpeedBps > 0, remainingBytes > 0 else { return nil }
        let seconds = Double(remainingBytes) / Double(downSpeedBps)
        guard seconds >= 60 else { return nil }
        return etaFormatter.string(from: seconds)
    }
}
