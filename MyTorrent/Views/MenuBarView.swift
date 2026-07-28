import SwiftUI

// 4.1 — popover content for the menu-bar `MenuBarExtra(.window)` scene
// (MyTorrentApp.swift). Reads AppModel.torrents, already kept fresh by the
// app-lifetime poller (AD-4) — no new FFI call or timer here (Scope Boundary).
struct MenuBarView: View {
    @EnvironmentObject private var appModel: AppModel
    @Environment(\.openWindow) private var openWindow
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: 0) {
            statsSection
            Divider()
            actionsSection
        }
        .frame(width: 220)
    }

    private var statsSection: some View {
        let active = appModel.activeTorrents
        return VStack(alignment: .leading, spacing: 3) {
            if active.isEmpty {
                Text("main_window.empty_state.message")
            } else {
                let downTotal = active.reduce(UInt64(0)) { $0 + $1.downSpeedBps }
                let upTotal = active.reduce(UInt64(0)) { $0 + $1.upSpeedBps }
                HStack {
                    Text("menu_bar.popover.down_speed_label")
                    Spacer()
                    Text(Formatting.speed(downTotal))
                }
                HStack {
                    Text("menu_bar.popover.up_speed_label")
                    Spacer()
                    Text(Formatting.speed(upTotal))
                }
                HStack {
                    Text("menu_bar.popover.active_label")
                    Spacer()
                    Text(activeCountText(active.count))
                }
            }
        }
        .font(.system(size: 12.5))
        .padding(12)
    }

    private var actionsSection: some View {
        VStack(alignment: .leading, spacing: 2) {
            Button("menu_bar.popover.open_window") {
                closePopover()
                // Shared with MainWindowView.handleOpenURL — always brings the
                // singleton main window to front, never a toggle.
                AppModel.activateAndOpenWindow(id: WindowID.main, openWindow: openWindow)
            }
            .buttonStyle(.borderless)

            // Reuses the existing `settings.header.title` key (already "Settings"/
            // "Настройки" for the Settings window itself) instead of a second key
            // carrying the identical translated word.
            Button("settings.header.title") {
                closePopover()
                AppModel.activateAndOpenWindow(id: WindowID.settings, openWindow: openWindow)
            }
            .buttonStyle(.borderless)

            Divider()

            // AD-7: the menu-bar is meant to be the persistent presence whose only
            // way out is an explicit Quit action — before this button, nothing in
            // the app (an LSUIElement accessory with no Dock icon/app menu) could
            // quit it once its windows were closed.
            Button("menu_bar.popover.quit") {
                NSApp.terminate(nil)
            }
            .buttonStyle(.borderless)
        }
        .padding(6)
    }

    // `MenuBarExtra(.window)`'s popover isn't a `.sheet`/`.popover`/`NavigationStack`
    // presentation, so `dismiss()` isn't guaranteed to close it (no confirmed
    // binding for this scene type) — closing the key window (the popover itself,
    // still key at the moment a button inside it is clicked) guarantees it closes
    // regardless of whether `dismiss()` turns out to be a no-op here.
    private func closePopover() {
        dismiss()
        NSApp.keyWindow?.close()
    }

    // `menu_bar.popover.active_count` carries `variations.plural` in
    // Localizable.xcstrings (RU one/few/many, EN one/other). Verified against the
    // actual compiled `.stringsdict` output (Story 4.1 code review): `String(localized:)`
    // returns the unresolved "%#@value@" template, and `String(format:)` is what
    // performs the real CLDR plural-category substitution using `count` — this
    // two-step call is the classic, correct mechanism, not a bypass of it.
    private func activeCountText(_ count: Int) -> String {
        String(format: String(localized: "menu_bar.popover.active_count"), count)
    }
}
