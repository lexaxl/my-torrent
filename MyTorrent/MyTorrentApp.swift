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
            // Story 5.4 — custom pump glyph, replacing the generic
            // arrow.up.arrow.down.circle(.fill) SF Symbol: outline/muted when
            // idle, filled/accent when active. Drawn as a plain SwiftUI
            // `Shape` (`PumpShape` below), not a raster `Assets.xcassets`
            // image — confirmed empirically on a real build (see this
            // story's Scope Boundary for the crowded-menu-bar false-negative
            // this session hit and resolved before settling on this design).
            Group {
                if appModel.hasActiveTorrents {
                    PumpShape().fill(Color.torrentAccent)
                } else {
                    PumpShape().stroke(Color.torrentMuted, style: StrokeStyle(lineWidth: 1.5, lineJoin: .round))
                }
            }
            .frame(width: 18, height: 18)
                // Story 5.3 — hover tooltip, additive to the existing
                // click-opens-popover behavior (UX-DR6 "click, not hover"
                // still governs the popover itself).
                .help(appModel.menuBarTooltipText)
        }
        .menuBarExtraStyle(.window)
    }
}

// Story 5.4 — menu-bar pump glyph, a plain SwiftUI `Shape`. Geometry mirrors
// the app icon's pump silhouette (`AppIcon.appiconset` source) so both places
// read as the same shape — outline/muted when idle, filled/accent when
// active, matching the "простой"/"активный" pair epics.md Story 5.4 AC1
// describes.
private struct PumpShape: Shape {
    func path(in rect: CGRect) -> Path {
        // Content bounding box in the shapes' own coordinate space below is
        // x:55-145 (width 90), y:20-160 (height 140); center it in `rect`
        // with an 0.85 fill factor for a small margin. Maps points directly
        // (rather than building the path in raw coordinates and calling
        // `.applying(someComposedCGAffineTransform)`) to avoid relying on
        // `CGAffineTransform.translatedBy`/`.scaledBy` chain composition
        // order, which is easy to get backwards and silently place the whole
        // shape off in space outside the tiny menu-bar frame (hit this
        // exact bug earlier in this story).
        let scale = min(rect.width / 90, rect.height / 140) * 0.85
        func p(_ x: CGFloat, _ y: CGFloat) -> CGPoint {
            CGPoint(x: (x - 100) * scale + rect.midX, y: (y - 90) * scale + rect.midY)
        }

        var glyphPath = Path()
        glyphPath.addRoundedRect(
            in: CGRect(origin: p(55, 20), size: CGSize(width: 90 * scale, height: 16 * scale)),
            cornerSize: CGSize(width: 8 * scale, height: 8 * scale)
        )
        glyphPath.addRoundedRect(
            in: CGRect(origin: p(90, 34), size: CGSize(width: 20 * scale, height: 26 * scale)),
            cornerSize: CGSize(width: 6 * scale, height: 6 * scale)
        )
        glyphPath.addRoundedRect(
            in: CGRect(origin: p(65, 58), size: CGSize(width: 70 * scale, height: 70 * scale)),
            cornerSize: CGSize(width: 14 * scale, height: 14 * scale)
        )
        var base = Path()
        base.move(to: p(75, 128))
        base.addLine(to: p(125, 128))
        base.addLine(to: p(140, 160))
        base.addLine(to: p(60, 160))
        base.closeSubpath()
        glyphPath.addPath(base)
        return glyphPath
    }
}
