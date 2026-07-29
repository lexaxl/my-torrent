import AppKit
import SwiftUI
import os

private let logger = Logger(
    subsystem: Bundle.main.bundleIdentifier ?? "com.alex.mytorrent.MyTorrent",
    category: "startup"
)

// Scene id constants — shared by this file's own scene declarations and every
// `openWindow(id:)` call site (MainWindowView, MenuBarView) so a typo becomes a
// compile error instead of a silently-do-nothing runtime call.
enum WindowID {
    static let main = "main"
    static let settings = "settings"
    static let torrentDetail = "torrent-detail"
}

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

    // `LSUIElement` (Info.plist) alone does not stop the app from quitting when its
    // last window closes — confirmed empirically (Story 1.3 Task 6). AD-4's polling
    // must survive every window being closed, so this must return false explicitly.
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
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
        Window("MyTorrent", id: WindowID.main) {
            MainWindowView(appDelegate: appDelegate)
                .environmentObject(appModel)
        }
        // Value-based WindowGroup (macOS 13+) — `openWindow(id:value:)` with the same
        // torrent id activates the existing window instead of opening a duplicate
        // (Story 2.1 AC4), with no manual "already open?" bookkeeping needed.
        WindowGroup(id: WindowID.torrentDetail, for: String.self) { $torrentId in
            TorrentDetailView(torrentId: torrentId ?? "")
                .environmentObject(appModel)
        }
        // Singleton scene, like "main" — settings aren't tied to a per-entity id,
        // so this is `Window`, not `WindowGroup(for:)` (Story 2.1's pattern for
        // per-torrent windows doesn't apply here).
        Window("settings.header.title", id: WindowID.settings) {
            SettingsView()
        }
        // `SettingsView` has no `ScrollView`, so if the window were resizable below
        // its content's natural size, dragging it smaller would clip the seed-duration
        // controls with no way to scroll back to them (code-review fix, Story 3.2 —
        // found when Story 3.2 added a second form section and `SettingsView`'s own
        // `.frame(minHeight:)` floor no longer had a real safety margin). `.contentSize`
        // lets the window resize freely but never below what its content needs,
        // instead of guessing a magic-number minHeight that has to be hand-kept in
        // sync with the form's content every time a section is added/removed.
        .windowResizability(.contentSize)

        // 4.1 — always-present menu-bar entry point (AD-7's persistent presence).
        // `.window` style (not `.menu`) so the popover can be an arbitrary SwiftUI
        // layout (stats block + actions block) instead of system menu-item rows;
        // it already opens on click, not hover, by default. Reads the same poll
        // data (`AppModel.torrents`) the popover itself uses — no separate poll.
        MenuBarExtra {
            MenuBarView()
                .environmentObject(appModel)
        } label: {
            // `hasActiveTorrents` short-circuits (`contains(where:)`), unlike
            // `activeTorrents.isEmpty` which would allocate the full filtered
            // array just to answer a yes/no question on every poll tick.
            Image(systemName: appModel.hasActiveTorrents
                ? "arrow.up.arrow.down.circle.fill"
                : "arrow.up.arrow.down.circle")
                // Story 5.3 — hover tooltip, additive to the existing
                // click-opens-popover behavior (UX-DR6 "click, not hover"
                // still governs the popover itself).
                .help(appModel.menuBarTooltipText)
        }
        .menuBarExtraStyle(.window)
    }
}
