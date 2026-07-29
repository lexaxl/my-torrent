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
}
