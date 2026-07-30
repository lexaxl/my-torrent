import AppKit
import SwiftUI

extension Color {
    // Story 5.2 — `color-success` (#1F9254 light / #3FB873 dark), design system
    // `00-design-system.md`. First design-system color token used in code; no
    // Assets.xcassets exists yet in this project (see Story 5.2 Scope Boundary) —
    // a dynamic NSColor keeps this addition scoped to exactly the one token
    // needed, instead of standing up a whole color asset catalog for it.
    static let torrentSuccess = Color(nsColor: NSColor(name: nil, dynamicProvider: { appearance in
        appearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua
            ? NSColor(red: 0x3F / 255, green: 0xB8 / 255, blue: 0x73 / 255, alpha: 1)
            : NSColor(red: 0x1F / 255, green: 0x92 / 255, blue: 0x54 / 255, alpha: 1)
    }))

    // Story 5.4 — `color-accent` (#4C5FD5 light / #7C8CF0 dark), design system
    // `00-design-system.md`. Used for the "active" state of the menu-bar pump
    // glyph (`PumpShape` in MyTorrentApp.swift), same dynamic-`NSColor` pattern
    // as `torrentSuccess` above.
    static let torrentAccent = Color(nsColor: NSColor(name: nil, dynamicProvider: { appearance in
        appearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua
            ? NSColor(red: 0x7C / 255, green: 0x8C / 255, blue: 0xF0 / 255, alpha: 1)
            : NSColor(red: 0x4C / 255, green: 0x5F / 255, blue: 0xD5 / 255, alpha: 1)
    }))

    // Story 5.4 — muted gray for the menu-bar pump glyph's idle/outline
    // state. `Color.secondary` (SwiftUI's own environment-resolved semantic
    // color) was tried first and rendered invisible specifically inside
    // `MenuBarExtra`'s label — isolated empirically (an explicit `Color.red`
    // `.stroke()` in the same spot worked fine, so `.stroke()` itself wasn't
    // the problem). `NSColor.secondaryLabelColor`, bridged the same
    // AppKit-appearance-resolved way as `torrentAccent`/`torrentSuccess`
    // rather than through SwiftUI's environment, is what actually renders.
    static let torrentMuted = Color(nsColor: .secondaryLabelColor)
}
