import AppKit
import SwiftUI
import os

private let logger = Logger(
    subsystem: Bundle.main.bundleIdentifier ?? "com.alex.mytorrent.MyTorrent",
    category: "startup"
)

// `Window` (singleton scene) + `.onOpenURL` don't fire on macOS — SwiftUI only
// delivers open-URL/open-file external events to `.onOpenURL` on `WindowGroup`,
// which instead opens a new window per event (wrong for this single-window app).
// `NSApplicationDelegate.application(_:open:)` is the unified replacement since
// macOS 10.13 — it receives both Finder file opens and custom URL scheme opens.
final class AppDelegate: NSObject, NSApplicationDelegate {
    var onOpenURLs: (([URL]) -> Void)? {
        didSet {
            guard onOpenURLs != nil, !pendingURLs.isEmpty else { return }
            let urls = pendingURLs
            pendingURLs = []
            onOpenURLs?(urls)
        }
    }

    // A cold launch triggered by double-clicking a .torrent file (or clicking a
    // magnet: link) can deliver this Apple Event before SwiftUI has finished
    // building the view hierarchy and set `onOpenURLs` — buffer until it's set
    // instead of silently dropping the URLs that triggered the launch.
    private var pendingURLs: [URL] = []

    func application(_ application: NSApplication, open urls: [URL]) {
        if let onOpenURLs {
            onOpenURLs(urls)
        } else {
            pendingURLs.append(contentsOf: urls)
        }
    }
}

@main
struct MyTorrentApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var appModel = AppModel()

    init() {
        logger.info("engine linked, engineVersion() = \(engineVersion(), privacy: .public)")
    }

    var body: some Scene {
        Window("MyTorrent", id: "main") {
            MainWindowView(appDelegate: appDelegate)
                .environmentObject(appModel)
        }
    }
}
