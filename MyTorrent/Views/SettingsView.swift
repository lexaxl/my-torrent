import AppKit
import SwiftUI

// Story 3.2: tag type for `Picker`'s `selection` — `SeedDurationMode` itself carries
// an associated `Int` for the "hours" case, which `.tag()`/`Picker` selection can't
// bind to directly. This is selection-only; the actual value lives in `seedDurationMode`.
private enum SeedDurationKind: Hashable {
    case off, hours, indefinite
}

struct SettingsView: View {
    @Environment(\.dismiss) private var dismiss
    @State private var saveLocationPath: String = AppSettings.saveLocationPath
    // `didSet` is the single write-through point to `AppSettings` (code-review fix,
    // Story 3.2) — every assignment to this property persists automatically, instead
    // of each of `seedDurationKindBinding`'s/`seedDurationHoursBinding`'s setters
    // separately remembering to write `AppSettings.seedDurationMode` afterward.
    @State private var seedDurationMode: SeedDurationMode = AppSettings.seedDurationMode {
        didSet { AppSettings.seedDurationMode = seedDurationMode }
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            form
        }
        .frame(minWidth: 420, minHeight: 200)
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
        VStack(alignment: .leading, spacing: 16) {
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

            VStack(alignment: .leading, spacing: 4) {
                Text("settings.seed_duration.label")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                Picker("", selection: seedDurationKindBinding) {
                    Text("settings.seed_duration.off").tag(SeedDurationKind.off)
                    Text("settings.seed_duration.hours").tag(SeedDurationKind.hours)
                    Text("settings.seed_duration.indefinite").tag(SeedDurationKind.indefinite)
                }
                .pickerStyle(.radioGroup)
                .labelsHidden()

                if case .hours = seedDurationMode {
                    HStack {
                        TextField("", value: seedDurationHoursBinding, format: .number)
                            .frame(width: 48)
                            .multilineTextAlignment(.trailing)
                        Text("settings.seed_duration.hours_hint")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .padding(16)
    }

    // `Picker`'s `selection` needs a value it can compare/tag directly; `SeedDurationMode`
    // carries an `Int` payload on `.hours`, so this bridges to/from the tag-only
    // `SeedDurationKind`. Persisting to `AppSettings` happens via `seedDurationMode`'s
    // own `didSet`, not here — this only ever needs to update the local `@State`.
    private var seedDurationKindBinding: Binding<SeedDurationKind> {
        Binding(
            get: {
                switch seedDurationMode {
                case .off: return .off
                case .hours: return .hours
                case .indefinite: return .indefinite
                }
            },
            set: { newKind in
                switch newKind {
                case .off:
                    seedDurationMode = .off
                case .indefinite:
                    seedDurationMode = .indefinite
                case .hours:
                    // Switching to "N hours" without typing a number — restore the
                    // last-persisted hour count. Reading local `seedDurationMode` here
                    // would lose the value across an off/indefinite round-trip, since
                    // @State only remembers hours while .hours is the active case;
                    // `seedDurationHoursIgnoringMode` survives mode switches because
                    // it's a separate persisted key (code-review fix, Story 3.2).
                    seedDurationMode = .hours(AppSettings.seedDurationHoursIgnoringMode)
                }
            }
        )
    }

    private var seedDurationHoursBinding: Binding<Int> {
        Binding(
            get: {
                if case .hours(let hours) = seedDurationMode { return hours }
                // Unreachable today (the TextField using this binding only renders
                // behind the same `.hours` check), but falls back to the persisted
                // value rather than the bare default, matching seedDurationKindBinding's
                // restore logic above (code-review fix, Story 3.2).
                return AppSettings.seedDurationHoursIgnoringMode
            },
            set: { newValue in
                seedDurationMode = .hours(AppSettings.clampSeedDurationHours(newValue))
            }
        )
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
        Alerts.runWarning(
            title: String(localized: "settings.save_location.not_writable.title"),
            message: String(localized: "settings.save_location.not_writable.message") + "\n\n" + path
        )
    }
}

#Preview {
    SettingsView()
}
