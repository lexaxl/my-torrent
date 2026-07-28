import Foundation

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
}
