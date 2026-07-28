import AppKit
import SwiftUI

struct SettingsView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var saveLocationPath: String = AppSettings.saveLocationPath

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            form
        }
        .frame(minWidth: 420, minHeight: 160)
    }

    private var header: some View {
        HStack {
            Text("settings.header.title")
                .font(.headline)
            Spacer()
            Button {
                dismiss()
            } label: {
                Image(systemName: "xmark")
            }
            .buttonStyle(.borderless)
        }
        .padding(12)
    }

    private var form: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("settings.save_location.label")
                .font(.subheadline)
                .foregroundStyle(.secondary)
            HStack {
                Text(saveLocationPath)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Spacer()
                Button("settings.save_location.choose_button") {
                    chooseSaveLocation()
                }
            }
        }
        .padding(16)
    }

    // Native folder picker constrains the result to an existing, browsable
    // directory, but NOT necessarily a writable one (`canChooseDirectories` only
    // requires read/list access) — confirmed against AppKit's NSOpenPanel docs
    // (code-review fix, Story 3.1: this comment previously claimed writability
    // was guaranteed by the picker, which is false — e.g. a read-only mounted
    // volume or another account's shared-but-not-writable folder both pass the
    // picker's own constraints). So an explicit writability check is needed here;
    // applies instantly on selection once it passes, no save button (UX spec
    // Technical Notes — instant-apply, confirmed by user).
    private func chooseSaveLocation() {
        let panel = NSOpenPanel()
        panel.canChooseDirectories = true
        panel.canChooseFiles = false
        panel.canCreateDirectories = true
        panel.allowsMultipleSelection = false
        panel.directoryURL = URL(fileURLWithPath: saveLocationPath)

        guard panel.runModal() == .OK, let url = panel.url else { return }
        guard FileManager.default.isWritableFile(atPath: url.path) else {
            showNotWritableAlert(for: url.path)
            return
        }
        saveLocationPath = url.path
        AppSettings.saveLocationPath = url.path
    }

    private func showNotWritableAlert(for path: String) {
        let alert = NSAlert()
        alert.alertStyle = .warning
        alert.messageText = String(localized: "settings.save_location.not_writable.title")
        alert.informativeText = String(localized: "settings.save_location.not_writable.message") + "\n\n" + path
        alert.runModal()
    }
}

#Preview {
    SettingsView()
}
