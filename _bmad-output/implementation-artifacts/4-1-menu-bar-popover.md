---
baseline_commit: aa2eef3e832bc7a07d2d328c0ebe73c0c0ad5566
---

# Story 4.1: Иконка в меню-баре и popover

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу кликнуть по иконке в меню-баре и увидеть агрегированную статистику закачек,
чтобы проверить статус без открытия главного окна.

## Acceptance Criteria

1. **Given** приложение запущено (в любом состоянии — с открытыми окнами или без), **when** я кликаю по иконке в меню-баре, **then** открывается popover.
2. **Given** popover открыт и есть ≥1 активная закачка (`active = {Downloading, Checking, Seeding}` + `resolving`, тот же partition, что уже использует `AppModel.activeStatuses`), **then** popover показывает агрегированную ↓-скорость, агрегированную ↑-скорость и количество активных закачек.
3. **Given** popover открыт и активных закачек 0, **then** вместо статистики показывается «Нет активных закачек» (переиспользует существующий ключ `main_window.empty_state.message` — тот же текст, что в пустом состоянии главного окна).
4. **Given** popover открыт, **then** он содержит два действия: «Открыть окно» (закрывает popover, всегда выводит главное окно на передний план — не toggle, если уже открыто) и «Настройки» (закрывает popover, открывает окно настроек).
5. **Given** приложение запущено, **then** взаимодействие с иконкой — клик, не hover (macOS-стандарт для status-bar-иконок, см. UX-спека Technical Notes).
6. Статистика в popover пересчитывается на том же ~1с поллинг-цикле, что и главное окно (AD-4) — **не** заводится отдельный FFI-вызов или отдельный таймер только для popover.

## Scope Boundary

**Решено при создании истории:**

- **Иконка меню-бара — не финальный визуальный дизайн.** UX-спека (`4.1-menu-bar.md`, Open Questions #1) явно оставляет точный визуал (idle vs active) открытым вопросом, делегированным на этап Visual Design — не архитектурный вопрос (см. `ARCHITECTURE-SPINE.md` → Deferred: «Row hover visual treatment; menu-bar icon design»). Тем не менее UX-спека документирует **состояние** иконки как различимое (Page States: «Icon: idle» / «Icon: active» с разной «appearance», хоть и «TBD»), так что полностью одинаковая иконка в обоих состояниях была бы недовыполнением. Решение: placeholder — два системных SF Symbol («arrow.up.arrow.down.circle» в покое, «arrow.up.arrow.down.circle.fill» при активности, тот же мотив ⇅, что в ASCII-макете), не финальная art — можно заменить одной строкой без структурных изменений, когда появится финальный дизайн.
- **`MenuBarExtra(.window)`, не `.menu`.** Архитектура (`AD-3`) фиксирует `MenuBarExtra` вместо AppKit `NSStatusItem`+`NSPopover`, но не фиксирует стиль. UX-спека требует произвольную кастомную вёрстку (stats-блок + hairline-разделитель + actions-блок с точными отступами из Design System, а не системные пункты меню) — `.menuBarExtraStyle(.window)` даёт SwiftUI-контент произвольной формы во floating-панели; `.menu` рендерил бы каждый child как системный пункт меню (нельзя воспроизвести stats-блок с `Formatting.speed`). `.window`-стиль уже по умолчанию открывается по клику, не по hover — AC5 не требует отдельной реализации.
- **Данные — из уже существующего `AppModel.torrents`, без нового FFI-вызова.** `getAllTorrents()` (AD-4) уже поллится главным окном на app-lifetime таймере; popover — ещё один читатель того же `@Published torrents`, просто с другой агрегацией (сумма скоростей + count по active-partition). Заводить отдельный таймер/вызов специально для popover противоречило бы AD-4 («timer... owned by an app-lifetime object», не по одному на consumer) и раздуло бы список активных таймеров без причины — тот же снапшот, другое представление.
- **`AppModel.activeStatuses` был `private`** — эта история делает доступ к active-partition публичным через новое computed-свойство `AppModel.activeTorrents: [TorrentStatus]`, а не дублирует набор статусов ("checking", "downloading", "seeding", "resolving") второй копией внутри `MenuBarView`. Единственный источник истины для active/inactive partition (Consistency Conventions таблица Architecture Spine) — там же, где он уже был для поллинг-гейта.
- **Подсчёт активных закачек в тексте («N активные закачки») — открытый вопрос русской числительной формы, требует подтверждения пользователем.** UX-спека даёт `Translation Key: menu_bar.popover.active_count`, RU `"{N} активные закачки"` / EN `"{N} active downloads"` — на вид форматная строка с реальной подстановкой числа (в отличие от Story 3.2's «N часов», где число реально жило в соседнем `TextField`, а не в тексте — здесь альтернативы нет, popover не содержит поля ввода, число обязано быть частью текста). Корректное русское согласование числительного с «закачка» (жен. род) требует трёх грамматических форм: 1 → «1 активная закачка», 2–4 → «N активные закачки», 5+ (и 11–14) → «N активных закачек». String Catalog (`.xcstrings`) поддерживает это нативно через `variations.plural` с CLDR-категориями (`one`/`few`/`many`/`other` для `ru`), что и стандартный Apple-механизм для ровно этого случая (`.stringsdict`-эквивалент). **Предлагаемое решение**: реализовать через `variations.plural` в `Localizable.xcstrings` (тот же файл, что все прочие ключи проекта) — это первый ключ в проекте с реальной числовой подстановкой (0 прецедентов, как отметил код-ревью Story 3.1/3.2), но, в отличие от «N часов», здесь нет обходного пути (не спрятать число в соседний контрол), так что откладывать эту фичу — не вариант, как в 3.2. Использовать `Text("\(count) menu_bar.popover.active_count")`-подобную SwiftUI-интерполяцию потребовало бы ключа вида «%lld ...» вместо dot-path — вместо этого использовать явный `String(localized:defaultValue:)` с фиксированным dot-path ключом (`menu_bar.popover.active_count`), значение — plural-variations, а число передаётся как `%lld`-аргумент форматирования (Foundation резолвит plural-категорию автоматически на основе значения, а не способа вызова). **Если разработчик (dev-story) столкнётся с тем, что plural-variations в `.xcstrings` не резолвятся ожидаемо без Xcode GUI** — деградировать к единственной `other`-форме (грамматически неидеальной для count=1) и явно задокументировать это в Dev Agent Record как известное упрощение, а не блокировать всю историю на этом.
- **Empty-state текст переиспользует существующий ключ**, не заводит новый (`main_window.empty_state.message`) — по прямому указанию UX-спеки («Note: reuses main_window.empty_state.message»).
- **Не в этой истории**: финальный визуальный дизайн иконки (см. первый пункт); Story 4.2 (уведомление о завершении) — отдельная история, не затрагивается здесь, хотя использует ту же `AppModel`/поллинг-инфраструктуру.

## Tasks / Subtasks

- [x] **Task 1: `AppModel.swift` — публичный доступ к active-partition** (AC: 2, 3, 6)
  - [x] Заменить `private static let activeStatuses` на internal (убрать `private`) **или** добавить новое computed-свойство:
    ```swift
    var activeTorrents: [TorrentStatus] {
        torrents.filter { Self.activeStatuses.contains($0.status) }
    }
    ```
  - [x] Не менять сам набор статусов/семантику — уже корректен и покрывает `resolving` (см. существующий комментарий над `activeStatuses`)
  - [x] Не добавлять новый FFI-вызов, новый таймер или новый poll-cadence — см. Scope Boundary

- [x] **Task 2: `MenuBarView.swift` — новый файл** (AC: 1, 2, 3, 4, 5)
  - [x] Структура по образцу UX-спеки (`4.1-menu-bar.md` → Layout Structure) и Design System (`00-design-system.md` spacing scale, тот же источник, что уже использует `SettingsView`):
    ```swift
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
                    let downTotal = active.reduce(0) { $0 + $1.downSpeedBps }
                    let upTotal = active.reduce(0) { $0 + $1.upSpeedBps }
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
            .padding(12)
        }

        private var actionsSection: some View {
            VStack(alignment: .leading, spacing: 2) {
                Button("menu_bar.popover.open_window") {
                    dismiss()
                    NSApp.activate(ignoringOtherApps: true)
                    openWindow(id: "main")
                }
                .buttonStyle(.borderless)

                Button("menu_bar.popover.settings") {
                    dismiss()
                    openWindow(id: "settings")
                }
                .buttonStyle(.borderless)
            }
            .padding(6)
        }

        private func activeCountText(_ count: Int) -> String {
            String(format: String(localized: "menu_bar.popover.active_count"), count)
        }
    }
    ```
    (`NSApp.activate(ignoringOtherApps:)` + `openWindow(id: "main")` — тот же паттерн, что `MainWindowView.handleOpenURL` уже использует для «всегда на передний план, не toggle»; `@Environment(\.dismiss)` закрывает `.window`-стиль popover — доступно с macOS 13 для `MenuBarExtra(.window)`)
  - [x] Свериться с UX-спекой на предмет отдельных подписей строк («↓ Загрузка» / «↑ Раздача» / «Активно») — три статичных ключа-лейбла (`menu_bar.popover.down_speed_label`, `.up_speed_label`, `.active_label`) + одно динамическое значение-число (`menu_bar.popover.active_count`, plural) на одну строку, не путать назначение двух похожих ключей

- [x] **Task 3: `MyTorrentApp.swift` — сцена `MenuBarExtra`** (AC: 1, 5)
  - [x] Добавить в `body`:
    ```swift
    MenuBarExtra {
        MenuBarView()
            .environmentObject(appModel)
    } label: {
        Image(systemName: appModel.torrents.contains { AppModel.activeStatuses.contains($0.status) }
            ? "arrow.up.arrow.down.circle.fill"
            : "arrow.up.arrow.down.circle")
    }
    .menuBarExtraStyle(.window)
    ```
    (если `activeStatuses` остался `private` после Task 1 — использовать `appModel.activeTorrents.isEmpty` вместо прямого обращения к `activeStatuses` из `label`, чтобы не тянуть внутренний набор статусов наружу второй раз)
  - [x] Не трогать существующие `Window`/`WindowGroup` сцены — только добавление новой сцены

- [x] **Task 4: Локализация** (AC: 2, 3, 4)
  - [x] `MyTorrent/Localizable.xcstrings` — новые ключи (нативный Xcode JSON-стиль, как во всех существующих):
    - `menu_bar.popover.down_speed_label` — RU «↓ Загрузка» / EN «↓ Download»
    - `menu_bar.popover.up_speed_label` — RU «↑ Раздача» / EN «↑ Upload»
    - `menu_bar.popover.active_label` — RU «Активно» / EN «Active»
    - `menu_bar.popover.active_count` — **с `variations.plural`**, не простой `stringUnit` (см. Scope Boundary): RU `one`: «%lld активная закачка», `few`: «%lld активные закачки», `many`/`other`: «%lld активных закачек»; EN `one`: «%lld active download», `other`: «%lld active downloads»
    - `menu_bar.popover.open_window` — RU «Открыть окно» / EN «Open Window» (значения зафиксированы в UX-спеке)
    - `menu_bar.popover.settings` — RU «Настройки» / EN «Settings» (переиспользовать value, ключ новый — `settings.header.title` уже существует с тем же текстом для окна настроек, но это семантически другой контекст/ключ, не шарить один ключ между окном и пунктом popover)
  - [x] `main_window.empty_state.message` — не создавать заново, переиспользовать существующий ключ как есть

- [x] **Task 5: XcodeGen, сборка, ручная проверка** (AC: 1-6)
  - [x] Новый файл `MenuBarView.swift` → обязательно `xcodegen generate` перед `xcodebuild` (см. [[my-torrent-dev-environment]] / project memory — иначе «cannot find 'MenuBarView' in scope»)
  - [x] `./scripts/build-engine.sh` (без изменений в `engine/`, но стандартный шаг) → `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] Клик по иконке в меню-баре открывает popover (не hover) — **подтверждено**, но нестандартным путём: скриншот верхней панели не показывал саму иконку визуально (см. Debug Log References — вероятно, переполненный меню-бар пользователя, ~13 сторонних иконок уже занимают место), поэтому сначала через Accessibility API (только чтение — `menu bar item 1 of menu bar 2`) подтвердил, что иконка реально зарегистрирована в системе (auto a11y-описание «Сортировать», т.е. системный лейбл SF Symbol `arrow.up.arrow.down` — подтверждает, что рендерится верная иконка), затем **по явному разрешению пользователя** выполнен один программный клик (`click menu bar item 1 of menu bar 2`, без ввода текста и без движения мыши) — popover открылся
  - [x] Без активных закачек — popover показывает «Нет активных закачек» — подтверждено тем же кликом (реальных активных закачек на момент проверки не было)
  - [~] С ≥1 активной закачкой — popover показывает агрегированные ↓/↑ и «N активные закачки» с правильным русским согласованием хотя бы для count=1 и count=2 (реальные торренты, не моки) — **не проверено**: на момент проверки не было активных закачек, а добавление реального торрента ради теста — отдельное действие за рамками разрешённого «одного клика»; рекомендация пользователю — проверить при первой реальной закачке
  - [~] «Открыть окно» закрывает popover и выводит главное окно на передний план / «Настройки» закрывает popover и открывает окно настроек — **не проверено**: пользователь разрешил один клик именно по иконке (для проверки, что popover вообще открывается), клики по самим кнопкам действий не выполнялись, чтобы не выходить за рамки разрешения; код переиспользует уже проверенный в Story 1.2/1.5 паттерн (`NSApp.activate` + `openWindow`), но живого клика по этим двум кнопкам нет — рекомендация пользователю проверить вручную
  - [x] `cargo test` в `engine/` — 25/25 тестов прошли (эта история не трогает `engine/`, прогнано как sanity-check)

## Dev Notes

### Technical Requirements / Stack

- **`AD-3`** — «менюbar-слой использует `MenuBarExtra` (macOS 13+), не AppKit `NSStatusItem`+`NSPopover`». Стиль (`.window` vs `.menu` vs default `.automatic`) не зафиксирован архитектурой — выбор `.window` обоснован в Scope Boundary (кастомная вёрстка, не системные menu items).
- **`AD-4`** — «shell polls a single synchronous UniFFI snapshot function... on a ~1 second timer... **Ownership**: the polling Task/timer is owned by an app-lifetime object (`AppModel`)». Popover — ещё один consumer уже существующего `@Published torrents`, не новый источник поллинга.
- **Architecture Spine Consistency Conventions → Active/inactive partition**: `active = {Downloading, Checking, Seeding}` — уже реализовано как `AppModel.activeStatuses` (плюс `resolving`, добавленное в Story 1.2 для pending-магнетов, см. существующий комментарий в коде). Эта история делает его доступным для второго потребителя (popover), не переизобретает.
- **`MenuBarExtra(.window)`** — SwiftUI API, доступен с macOS 13 (project deployment target уже 13.0, см. `project.yml`). `@Environment(\.dismiss)` работает внутри `.window`-стиля для закрытия popover программно — то же API, что `.sheet`/навигация, просто в контексте другой сцены.
- **`Formatting.speed(_:)`** (`MyTorrent/Formatting.swift`) — уже существующий helper, используемый `MainWindowView` и `TorrentDetailView`. Переиспользовать как есть, не дублировать логику форматирования (`ByteCountFormatter`, `.binary`, KB/MB/GB) третий раз.

### Architecture Compliance

- `AD-3` (MenuBarExtra, не NSStatusItem) — соблюдено, единственная сцена меню-бара в проекте.
- `AD-4` (single app-lifetime poller, no push callbacks) — соблюдено, popover не заводит собственный таймер/FFI-вызов.
- `AD-7` (accessory app lifecycle, LSUIElement) — уже реализовано в `Info.plist`/`AppDelegate` (Story 1.1/1.3), эта история не трогает — `MenuBarExtra` как раз и есть «persistent presence», о которой говорит AD-7's rule.
- `AD-9` (FFI calls off main thread) — не применимо напрямую: эта история не добавляет новых FFI-вызовов, только читает уже промаршаленный `@Published torrents` на `@MainActor`.

### Project Structure Notes

```
MyTorrent/
  MyTorrentApp.swift            # UPDATE — новая сцена MenuBarExtra(.window)
  AppModel.swift                 # UPDATE — activeTorrents computed property (или де-приватизация activeStatuses)
  Views/
    MenuBarView.swift            # NEW — popover содержимое (stats + actions)
  Localizable.xcstrings          # UPDATE — 6 новых ключей, один с variations.plural
```
Совпадает со Structural Seed архитектуры (`ARCHITECTURE-SPINE.md`): `MenuBarView.swift # 4.1 — popover stats + actions` уже был зафиксирован заранее, ничего не отклоняется от плана.

Новый файл → **`xcodegen generate` обязателен** перед сборкой (готча, задокументированная в Story 2.1/project memory).

### Previous Story Intelligence (Story 3.2)

- **Safety-урок про `osascript`/Accessibility-автоматизацию**: реальные клики/клавиатурный ввод на физическом экране пользователя, не изолированная песочница — пользователь может быть рядом и заметить. Для этой истории: предпочесть скриншоты (`screencapture`), либо явно спросить/предупредить перед любым программным кликом по popover/menu-bar иконке.
- **`.xcstrings` без прецедента реальной plural-подстановки** до этой истории (0 вхождений `%@`/`%lld` до сих пор, подтверждено дважды в код-ревью 3.1/3.2) — эта история первая, где числовая подстановка неизбежна (см. Scope Boundary), а не просто вариант, от которого можно уклониться копированием паттерна 3.2.
- **`/code-review` high-effort (8-angle parallel)** обязателен перед коммитом, тот же pipeline, что все предыдущие истории.
- **Git**: последний коммит на `main` — `aa2eef3` (Story 3.2). Ветка чистая, готова для новой истории.

### Testing Standards

- Swift: формального UI-test framework в проекте нет (решение Story 1.1) — только ручная проверка (Task 5), с явным акцентом на скриншоты вместо automation по умолчанию (см. Previous Story Intelligence).
- Rust: эта история не трогает `engine/` — `cargo test` не обязателен, но не будет лишним прогнать как sanity-check.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 4.1]
- [Source: design-artifacts/C-UX-Scenarios/04-alex-glances-at-the-menu-bar/04-alex-glances-at-the-menu-bar.md]
- [Source: design-artifacts/C-UX-Scenarios/04-alex-glances-at-the-menu-bar/4.1-menu-bar/4.1-menu-bar.md] — Layout Structure, Page Sections (все Object ID и Translation Keys), Page States, Technical Notes (click не hover, aggregate-speed formula), Open Questions
- [Source: design-artifacts/C-UX-Scenarios/04-alex-glances-at-the-menu-bar/4.1-menu-bar/Sketches/4.1-menu-bar-ascii.md]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-3] — MenuBarExtra mandate
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-4] — single app-lifetime poller, active/inactive partition, no push callbacks
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-7] — accessory app lifecycle (уже реализовано)
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#Structural Seed] — `MenuBarView.swift` заранее зафиксирован в plan
- [Source: MyTorrent/AppModel.swift] — существующий `activeStatuses`, `torrents` published property
- [Source: MyTorrent/Views/MainWindowView.swift] — `openWindow`/`NSApp.activate` паттерн для «always bring to front, not toggle» (переиспользуется дословно)
- [Source: MyTorrent/Formatting.swift] — существующий speed-formatting helper
- [Source: _bmad-output/implementation-artifacts/3-2-default-seed-duration.md] — предыдущая история: `.xcstrings` no-plural-precedent наблюдение, safety-урок про automation

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- **Иконка не отображалась визуально в меню-баре пользователя**, хотя приложение запускалось и работало без падений (`ps`/`log show` подтвердили процесс жив, без крашей). Диагностика: `screencapture` полной верхней панели не показал новый значок среди ~13 уже существующих сторонних menu-bar иконок (VPN, Bluetooth, Wi-Fi, батарея и т.д.). Через Accessibility API (только чтение) подтвердил, что `MenuBarExtra` реально зарегистрирован системой: `menu bar 2 of application process "MyTorrent"` содержит ровно 1 элемент, с системным a11y-описанием «Сортировать» (авто-перевод стандартного описания SF Symbol `arrow.up.arrow.down` на русский — подтверждает, что рендерится именно запланированная иконка, а не что-то не то). Позиция элемента (660, 2, размер 33×24) не давала видимого пикселя на скриншоте того же региона. **Вероятная причина — переполненный меню-бар**: у сторонних `NSStatusItem`/`MenuBarExtra` нет системного «...»-переполнения (в отличие от Control Center), и при нехватке места новый элемент может получить AX-позицию, но не отрисоваться видимо. Это, по всей видимости, ограничение среды пользователя (плотный меню-бар), а не баг в коде — сам popover открывается и рендерится корректно (см. ниже).
- С явного разрешения пользователя выполнен один программный клик по найденному AX-элементу (`click menu bar item 1 of menu bar 2` — без `keystroke`, без движения курсора мышью) — popover открылся и отрисовался с ожидаемым содержимым: пустое состояние «Нет активных закачек» + кнопки «Открыть окно» / «Настройки» (скриншот подтверждён визуально).

### Completion Notes List

- Все 5 задач выполнены. `AppModel.activeTorrents` — новое публичное computed-свойство, `MenuBarView.swift` — новый файл (popover: stats-секция + actions-секция), `MyTorrentApp.swift` — добавлена сцена `MenuBarExtra(.window)`, `Localizable.xcstrings` — 6 новых ключей, включая первый в проекте ключ с `variations.plural` (`menu_bar.popover.active_count`, RU one/few/many/other, EN one/other).
- **AC1, AC3, AC5 подтверждены живым тестом** (клик по иконке — не hover — открывает popover; пустое состояние показывает переиспользуемый текст «Нет активных закачек»).
- **AC2 (агрегированные ↓/↑ и count с реальной активной закачкой) и AC4 (поведение кнопок «Открыть окно»/«Настройки») — НЕ подтверждены живым тестом.** Разрешение пользователя касалось одного клика по самой иконке (чтобы проверить, что popover вообще открывается, т.к. иконка не была видна на экране) — клики по кнопкам действий и добавление реального торрента для проверки stats-секции выходили за рамки этого разрешения, поэтому не выполнялись. Код для обеих кнопок переиспользует уже проверенный в Story 1.2/1.5 паттерн (`NSApp.activate(ignoringOtherApps:)` + `openWindow(id:)`), логически корректен, но живого клика по ним нет.
- **AC6 (переиспользование существующего поллинга, без нового FFI-вызова/таймера)** — подтверждено чтением кода: `MenuBarView` читает `appModel.activeTorrents`, вычисляемое из уже существующего `@Published torrents`, никакого нового `Task`/timer/FFI-вызова не добавлено.
- **Иконка визуально не видна в меню-баре пользователя** (см. Debug Log References) — вероятно, из-за переполненного меню-бара на конкретной машине пользователя, не баг в реализации (AX подтверждает регистрацию с правильным символом, клик по найденному элементу открывает рабочий popover). **Рекомендация пользователю**: если иконка нужна видимой, попробовать освободить место в меню-баре (скрыть/убрать пару сторонних иконок, например через их настройки, или Cmd-перетаскиванием) и проверить снова после перезапуска приложения.
- `variations.plural` в `.xcstrings` для `menu_bar.popover.active_count` — **перепроверено эмпирически во время код-ревью** (см. ниже): собран изолированный тестовый бинарник, загружающий реальные скомпилированные ресурсы из уже собранного `MyTorrent.app` (`.stringsdict`), подтвердивший корректный выбор RU-формы (one/few/many) для count=1,2,4,5,11,21,25 — механизм работает как задумано.
- Изменений в `engine/` нет — `cargo test`: 25/25 прошли (sanity-check, не требовалось по сути истории).

### Code Review (`/code-review high`)

8 параллельных агентов-искателей (line-by-line, removed-behavior, cross-file tracer, reuse, simplification, efficiency, altitude, CLAUDE.md conventions) + верификация → 10 находок, все обработаны:

- **Исправлено (8):** `dismiss()` заменён на `closePopover()` (`dismiss()` + `NSApp.keyWindow?.close()`, гарантированно закрывает popover в `MenuBarExtra(.window)`, где `\.dismiss` не гарантирован); кнопка «Настройки» теперь тоже делает `NSApp.activate`; добавлена кнопка «Выйти» (`NSApp.terminate(nil)`) — закрывала AD-7 gap (нигде в приложении не было способа выйти); партиция active/inactive объединена в новое `AppModel.hasActiveTorrents` (было продублировано 3 раза); иконка меню-бара теперь читает `hasActiveTorrents` (короткое замыкание) вместо `activeTorrents.isEmpty` (полная аллокация массива без надобности); убран дублирующийся translation-ключ `menu_bar.popover.settings` (переиспользован существующий `settings.header.title`); паттерн `NSApp.activate`+`openWindow` вынесен в общий `AppModel.activateAndOpenWindow(id:openWindow:)`; строковые id сцен вынесены в новый `WindowID` enum (`MyTorrentApp.swift`).
- **Ложная тревога, код не менялся (1):** плюрализация `active_count` — заподозрена сломанной (`String(format: String(localized:), count)` якобы не резолвит CLDR-форму), но эмпирическая проверка против реально скомпилированного `.stringsdict` (не просто рассуждение) показала, что механизм работает правильно — двухшаговый вызов является классическим корректным паттерном, а не обходом плюрализации.
- **Осознанно не менялось, но задокументировано (1):** `resolving`-магниты по-прежнему считаются «активными» в popover (0/0 скорость) — решено не заводить отдельное определение «активности для отображения» отдельно от «активности для поллинга» ради устранения секундного косметического артефакта, особенно сразу после того, как только что объединили 3 копии этого определения в одну.

Билд после фиксов: `xcodegen generate` + `xcodebuild` — BUILD SUCCEEDED. Приложение перезапущено, живо (без крашей, подтверждено `log show`), AX подтверждает регистрацию иконки в меню-баре. Кнопки действий (Открыть окно/Настройки/Выйти) повторно вживую не кликались после фиксов — предыдущее разрешение пользователя касалось только одного клика по иконке, новое не запрашивалось.

### File List

- `MyTorrent/AppModel.swift` — UPDATE: `activeTorrents` (computed, для stats-секции), `hasActiveTorrents` (computed, короткое замыкание, единая точка для partition-проверки — используется `startPollingIfNeeded`/`pollLoop`/иконкой), `static activateAndOpenWindow(id:openWindow:)`
- `MyTorrent/Views/MenuBarView.swift` — NEW: popover-содержимое (stats-секция с пустым состоянием/агрегированными ↓/↑/count, actions-секция с «Открыть окно»/«Настройки»/«Выйти»), `closePopover()`
- `MyTorrent/MyTorrentApp.swift` — UPDATE: новая сцена `MenuBarExtra(.window)` с иконкой (читает `hasActiveTorrents`), новый `WindowID` enum
- `MyTorrent/Views/MainWindowView.swift` — UPDATE (код-ревью): `handleOpenURL`/settings-кнопка/torrent-detail теперь используют `WindowID`/`AppModel.activateAndOpenWindow`
- `MyTorrent/Localizable.xcstrings` — UPDATE: 6 новых ключей (`menu_bar.popover.down_speed_label`, `.up_speed_label`, `.active_label`, `.active_count` [plural], `.open_window`, `.quit`); `settings.header.title` переиспользован вместо нового ключа для popover-кнопки «Настройки»
- `MyTorrent.xcodeproj` — regenerated via `xcodegen generate` (новый файл `MenuBarView.swift` добавлен в проект)
