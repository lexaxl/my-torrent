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
}
