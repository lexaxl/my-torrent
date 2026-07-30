import AppKit

// Single home for the hand-rolled warning-alert shape (Story 6.1 code review:
// `AppDelegate.applicationShouldTerminate` was about to become the second copy
// of the boilerplate `SettingsView.showNotWritableAlert` already had).
enum Alerts {
    // Buttons are added in the order given; NSAlert makes the FIRST one the
    // default (Return key) and places it rightmost — pass the SAFE action first
    // for destructive confirmations. An empty `buttons` keeps NSAlert's
    // implicit localized OK.
    @discardableResult
    static func runWarning(
        title: String, message: String, buttons: [String] = []
    ) -> NSApplication.ModalResponse {
        let alert = NSAlert()
        alert.alertStyle = .warning
        alert.messageText = title
        alert.informativeText = message
        for buttonTitle in buttons {
            alert.addButton(withTitle: buttonTitle)
        }
        return alert.runModal()
    }
}
