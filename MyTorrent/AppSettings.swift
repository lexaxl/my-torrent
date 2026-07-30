import Foundation

// Story 3.2: seed-duration mode is stored the same way, but stays Swift-only —
// librqbit's `AddTorrentOptions` has no field for it, so there's nothing on the
// Rust side to pass it to yet (see Story 3.2 Scope Boundary). Actual enforcement
// (a cumulative app-open-time timer that stops seeding) is out of scope.
enum SeedDurationMode: Equatable {
    case off
    case hours(Int)
    case indefinite
}

// Story 3.1: Swift owns settings via UserDefaults; they flow one-directionally
// into the Rust engine as call parameters (AD-5) — never read back out of Rust.
enum AppSettings {
    private static let saveLocationKey = "settings.saveLocationPath"

    static var saveLocationPath: String {
        get { UserDefaults.standard.string(forKey: saveLocationKey) ?? defaultSaveLocationPath }
        set { UserDefaults.standard.set(newValue, forKey: saveLocationKey) }
    }

    static var defaultSaveLocationPath: String {
        FileManager.default
            .urls(for: .downloadsDirectory, in: .userDomainMask)
            .first?.path
            ?? NSString(string: "~/Downloads").expandingTildeInPath
    }

    // Story 6.2: librqbit's session-persistence store. Deliberately under
    // Application Support (NOT the user-changeable download folder), so the
    // torrent list survives restarts regardless of where the save location
    // points. Derivation only — the caller (`AppModel.setUpEngine`) creates the
    // directory and owns the failure path. Two separate path components (not a
    // single "MyTorrent/session" string) so each is escaped as its own dir
    // (code-review polish, Story 6.2).
    static var sessionStateDirectory: URL {
        let base = FileManager.default
            .urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first
            ?? URL(fileURLWithPath: NSString(string: "~/Library/Application Support").expandingTildeInPath,
                   isDirectory: true)
        return base
            .appendingPathComponent("MyTorrent", isDirectory: true)
            .appendingPathComponent("session", isDirectory: true)
    }

    static let defaultSeedDurationHours = 8
    static let seedDurationHoursRange = 1...168

    private static let seedDurationKindKey = "settings.seedDurationKind"
    private static let seedDurationHoursKey = "settings.seedDurationHours"

    static var seedDurationMode: SeedDurationMode {
        get {
            switch UserDefaults.standard.string(forKey: seedDurationKindKey) {
            case "off":
                return .off
            case "indefinite":
                return .indefinite
            default:
                // "hours", or first launch with nothing stored yet — matches the UX
                // spec's default-selected state ("N hours" pre-selected, value 8).
                return .hours(seedDurationHoursIgnoringMode)
            }
        }
        set {
            switch newValue {
            case .off:
                UserDefaults.standard.set("off", forKey: seedDurationKindKey)
            case .indefinite:
                UserDefaults.standard.set("indefinite", forKey: seedDurationKindKey)
            case .hours(let hours):
                UserDefaults.standard.set("hours", forKey: seedDurationKindKey)
                UserDefaults.standard.set(clampSeedDurationHours(hours), forKey: seedDurationHoursKey)
            }
        }
    }

    static func clampSeedDurationHours(_ value: Int) -> Int {
        Swift.min(Swift.max(value, seedDurationHoursRange.lowerBound), seedDurationHoursRange.upperBound)
    }

    // **Not a general-purpose accessor — do not use this to decide whether/how long
    // to seed.** Returns a plausible hour count even when the active mode is `.off`
    // or `.indefinite`, where that number is stale and meaningless; the only correct
    // way to ask "should this seed, and for how long" is to pattern-match
    // `seedDurationMode` itself (only trust the number when the case is `.hours`).
    // This exists solely so `SettingsView` can restore the last-typed hour count when
    // the user switches back to "N hours" after visiting "Off"/"Indefinite" — the
    // hours number persists independently of which mode is active (`seedDurationMode`'s
    // getter only surfaces it while kind == "hours"), and `@State` alone can't recover
    // it across that round-trip since it only remembers hours while `.hours` is the
    // active case (bug found during Story 3.2 manual verification).
    //
    // Presence is checked via `object(forKey:) != nil` rather than "stored == 0",
    // since `UserDefaults.integer(forKey:)` returns 0 both when a key is absent and
    // when 0 was legitimately stored — using the raw integer as its own unset-sentinel
    // would silently coerce a real 0 (e.g. written by a future migration/reset path
    // that doesn't go through `seedDurationMode`'s clamping setter) back to the default.
    static var seedDurationHoursIgnoringMode: Int {
        guard UserDefaults.standard.object(forKey: seedDurationHoursKey) != nil else {
            return defaultSeedDurationHours
        }
        return clampSeedDurationHours(UserDefaults.standard.integer(forKey: seedDurationHoursKey))
    }
}
