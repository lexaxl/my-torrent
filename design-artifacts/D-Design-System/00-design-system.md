# Design System: my-torrent

> Components, tokens, and patterns that grow from actual usage — not upfront planning.

**Created:** 2026-07-27
**Phase:** 7 — Design System (optional)
**Agent:** Freya (Designer)

---

## What Belongs Here

The Design System captures reusable patterns that emerge during UX Design (Phase 4). It is not designed upfront — it crystallizes from real page specifications.

**What goes here:**
- **Design Tokens** — Colors, spacing, typography, shadows
- **Components** — Buttons, inputs, cards, navigation elements
- **Patterns** — Layouts, form structures, content blocks
- **Visual Design** — Mood boards, design concepts, color and typography explorations
- **Assets** — Logos, icons, images, graphics

**What does NOT go here:**
- Page-specific content (that lives in `C-UX-Scenarios/`)
- Business logic or API specs (that's BMM territory)
- Aspirational components nobody uses yet

**When to skip this phase:**
- Using shadcn/ui or Material UI → the library IS your design system
- Simple sites with Tailwind → tokens in `tailwind.config` are enough

> **Note:** Component library for `my-torrent` was left undecided at Phase 0 setup ("decide later"). Since this is a native macOS desktop app, standard web component libraries (shadcn/ui, Material UI) don't apply directly — revisit this choice once the UI framework (e.g. SwiftUI/AppKit, Qt, Electron+React, Tauri) is picked during Architecture/Platform Requirements.

**Learn more:**
- WDS Course Module 12: Functional Components — Patterns Emerge
- WDS Course Module 13: Design System

---

## Folder Structure

```
D-Design-System/
├── 00-design-system.md          ← This file (hub + guide)
├── 01-Visual-Design/            [Early design exploration]
│   ├── mood-boards/             [Visual inspiration, style exploration]
│   ├── design-concepts/         [NanoBanana outputs, design explorations]
│   ├── color-exploration/       [Color palette experiments]
│   └── typography-tests/        [Font pairing and hierarchy tests]
├── 02-Assets/                   [Final production assets]
│   ├── logos/                   [Brand logos and variations]
│   ├── icons/                   [Icon sets]
│   ├── images/                  [Photography, illustrations]
│   └── graphics/                [Custom graphics and elements]
└── components/                  [Emerges during Phase 4]
    ├── interactive/             [Buttons, toggles, tabs]
    ├── form/                    [Inputs, selects, checkboxes]
    ├── layout/                  [Containers, grids, sections]
    ├── content/                 [Cards, lists, media blocks]
    ├── feedback/                [Alerts, toasts, progress]
    └── navigation/              [Menus, breadcrumbs, links]
```

**01-Visual-Design/** is used early — before or during scenarios — for exploring visual direction. Mood boards, color palettes, typography tests, and AI-generated design concepts live here.

**02-Assets/** holds final, production-ready assets. Logos, icons, images, and graphics that are referenced from page specifications.

**components/** grows organically during Phase 4 as patterns emerge across page specifications.

---

## For Agents

**Workflow:** `skill:wds-7-design-system`
**Agent trigger:** `DS` (Freya)
**Router:** `./resources/wds-7-design-system/design-system-router.md`
**Templates:** `./resources/wds-7-design-system/templates/`
**Guide:** `./resources/agent-guides/freya/design-system.md`

**Before creating any component:**
1. Check if it already exists in the chosen component library
2. Look at actual usage in `C-UX-Scenarios/` page specs — extract, don't invent
3. Load the component template from the workflow templates folder

**File naming:** Number all documents with a two-digit prefix: `01-design-tokens.md`, `02-button.md`, etc. Update the sections below as each file is created.

**Harm:** Designing an abstract component library before any pages exist. Components without real usage are decoration. They waste time and create maintenance burden for patterns nobody needs.

**Help:** Extracting patterns from real page specs. When three pages use similar card layouts, that's a component. The design system documents what emerged, making future pages faster and more consistent.

---

## Spacing Scale

> **Bring your own or use ours.** If your project already has a design system with a spacing scale (Tailwind, Material, Carbon, your own tokens), use that. Map your token names below so page specs reference them consistently. If you don't have one yet, WDS provides a default 9-token scale to get started.

### Option A: Use your existing design system

Replace the table below with your system's spacing tokens. Any naming convention works — numbered (`spacing-4`), t-shirt (`sm`/`md`/`lg`), or your own. The only rule: page specs reference token names, never raw pixel values.

### Option B: WDS default scale

Nine tokens, symmetric around `space-md` (the baseline). Freya will propose pixel values during the first design session.

| Token | Value | Use |
|-------|-------|-----|
| space-3xs | 2px | Hairline gaps (icon-to-label, inline elements) |
| space-2xs | 4px | Minimal spacing (badge padding, tight lists) |
| space-xs | 8px | Tight spacing (within compact groups) |
| space-sm | 12px | Small gaps (between related elements) |
| **space-md** | **16px** | **Default element spacing (the baseline)** |
| space-lg | 24px | Comfortable spacing (card padding, form fields) |
| space-xl | 32px | Section padding |
| space-2xl | 48px | Section gaps |
| space-3xl | 64px | Page-level breathing room |

_First real values extracted from `01-Visual-Design/design-concepts/my-torrent-prototype.html` (2026-07-27) — 4px-based grid, tighter than typical web scale to match native macOS density._

### Optical adjustments

Sometimes the math is right but the eye says it's wrong. A circular image leaves white corners, a light element on a light background looks more spaced than it is. When this happens, use token math — not raw pixels:

```
space-lg - space-3xs    → "standard spacing, pulled in by a hairline"
space-xl + space-2xs    → "section padding, nudged out slightly"
```

In page specs, always annotate why:

| Padding top | **space-lg - space-3xs** (optical: circular image adds perceived whitespace) |

**Rules:**
- Adjustments always use token math: `base ± correction`
- Always annotate the reason — future readers need to know this wasn't a mistake
- If adjusting by more than one step, the base token is probably wrong — reconsider

In CSS: `calc(var(--space-lg) - var(--space-3xs))`

---

## Type Scale

> **Bring your own or use ours.** Same principle as spacing — if your project has a type system, map it here. If not, use the WDS default.

The type scale controls **visual size** — how big text looks. This is separate from semantic level (H1, H2, p) which controls **document structure**. An H2 in a sidebar might be `text-sm`. A tagline might be a `<p>` at `text-2xl`. The semantic level is for accessibility and SEO; the type token is for visual hierarchy.

Headings can have different typefaces, weights, and styles on different pages. A landing page H1 might be a serif display font at `text-3xl` italic. An admin page H1 might be clean sans-serif at `text-lg` medium. Each page spec declares its own typographic treatment — the type scale provides the shared sizing vocabulary.

### Option A: Use your existing type system

Replace the table below with your system's type tokens.

### Option B: WDS default scale

Nine tokens, symmetric around `text-md` (body text). Freya will propose sizes during the first design session.

| Token | Value | Use |
|-------|-------|-----|
| text-3xs | 9px | Fine print, legal text |
| text-2xs | 10px | Metadata, timestamps |
| text-xs | 11px | Captions, helper text (e.g. list column headers) |
| text-sm | 12px | Labels, secondary text (e.g. row speed/seed-leech values) |
| text-md | 13px | Body text (the baseline) — matches macOS system font size (`NSFont.systemFontSize`) |
| text-lg | 15px | Emphasis, lead paragraphs |
| text-xl | 17px | Subheadings (e.g. torrent name in detail header) |
| text-2xl | 20px | Section titles, display text |
| text-3xl | 24px | Hero headings, page titles |

_Scale intentionally smaller/denser than a typical web type scale — matches native macOS text density rather than web conventions. First real values extracted from the HTML prototype (2026-07-27)._

---

## Tokens

### Color

Deliberately not stock Apple system blue — a muted indigo accent, chosen so the app has a faint identity of its own while still reading as native. Semantic colors (success/warning) are separate from the accent and not counted as "the" brand color.

| Token | Light | Dark | Use |
|-------|-------|------|-----|
| color-bg | `#F5F6F8` | `#17181D` | Window/app background |
| color-surface | `#FFFFFF` | `#1E2027` | Toolbar, cards, popovers |
| color-border | `#E1E3E8` | `#2C2F38` | Hairline separators |
| color-border-strong | `#D3D6DE` | `#383C48` | Input/button borders |
| color-text-primary | `#1C1E26` | `#EDEEF2` | Primary text |
| color-text-secondary | `#62677A` | `#A7ACBD` | Secondary text (speeds, labels) |
| color-text-tertiary | `#9297A6` | `#6E7280` | Tertiary text (placeholders, hints) |
| **color-accent** | `#4C5FD5` | `#7C8CF0` | Selected tab, links, active row indicator |
| color-accent-soft | `#EEF0FC` | `#262A45` | Selected-state background tint |
| color-success | `#1F9254` | `#3FB873` | Seeding/complete state |
| color-warning | `#B7791F` | `#D2A245` | Per-torrent tracker issue indicator |

Full light + dark theme defined via CSS custom properties in the prototype — see `01-Visual-Design/design-concepts/my-torrent-prototype.html`.

### Typography (typeface)

System font stack: `-apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", sans-serif`. Intentional choice, not a placeholder — on an actual Mac this renders as true SF Pro; matching the OS is the correct native-app choice rather than shipping a custom typeface.

### Other tokens

_Shadows, radii, and any additional tokens will be documented here as they recur across more page specifications (currently only sampled once in the prototype — see Technical Notes in individual page specs for interim values like `border-radius: 12px` on windows, `7px` on inputs/buttons)._

---

## Patterns

Spacing objects are first-class — they have IDs in page specs (e.g., `hem-v-space-xl`) and live here organized by value. Each spacing value accumulates the situations where it's used. The list grows from real design decisions.

_Patterns will be documented here as spacing objects recur across pages._

---

## Components

Extracted per "extract on 2nd use" — only components confirmed across ≥2 page specs are promoted here. Others remain page-specific for now (see individual specs).

### List Row

**Used in:** `main-window-torrent-row` (1.1), `torrent-detail-files-list-row` / `-trackers-list-row` / `-peers-list-row` (2.2)

Horizontal grid row, `space-sm` vertical padding, `space-md` horizontal padding, hairline `color-border` bottom divider (last row in a list has none), `color-row-hover`-equivalent background on hover. Content columns vary per use (torrent stats vs. file/tracker/peer fields), but the row shell — padding, divider, hover — is shared.

### Icon Button

**Used in:** `main-window-toolbar-settings-button` (1.1), `torrent-detail-header-close-button` (2.2)

26×26px hit target, `border-radius: 6px`, transparent by default, `color-row-hover`-equivalent background on hover. Icon-only, no visible label (label conveyed via tooltip or context, not on-screen text).

_Components/directories in `components/` (interactive, form, layout, content, feedback, navigation) will be populated with dedicated files once a 3rd distinct component type is confirmed — currently only these two have real 2nd-use evidence._

---

_Created using Whiteport Design Studio (WDS) methodology_
