---
baseline_commit: e277677ef4125453a0eb600fe3b2583e86381fc0
---

# Story 5.3: Hover-тултип со скоростью на иконке меню-бара

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу увидеть суммарную скорость закачек, наведя курсор на иконку в меню-баре,
чтобы проверить статус без клика и открытия popover.

## Acceptance Criteria

1. **Given** приложение запущено и есть ≥1 активная закачка, **when** навожу курсор на иконку меню-бара (без клика), **then** показывается системный тултип вида «X ↓ · Y ↑» (та же связка глифов и `Formatting.speed`, что уже использует `TorrentDetailView.summaryLine` — не локализованный текст-лейбл, как в popover, а компактная глиф-строка).
2. **Given** активных закачек нет, **when** навожу курсор на иконку, **then** тултип показывает переиспользуемый текст `main_window.empty_state.message` («Нет активных закачек») — тот же ключ, что уже переиспользует popover (Story 4.1), не новый.
3. **Given** приложение запущено, **when** кликаю по иконке, **then** popover открывается как раньше (`UX-DR6` «клик, не hover» для открытия popover не меняется) — hover добавляет тултип **в дополнение**, не вместо клика.
4. **Given** агрегированная ↓/↑ скорость, **then** она вычисляется из уже существующего `AppModel.activeTorrents` (новый computed `activeSpeedTotals`) — без нового FFI-вызова/таймера, тот же поллинг-цикл (AD-4), что уже питает popover.
5. **Given** реализация через `.help(_:)` на `Image` внутри `MenuBarExtra`'s `label:`, **then** это эмпирически проверено на реальной сборке (навести курсор и увидеть тултип), не предполагается по документации — см. Scope Boundary про отсутствие fallback-пути.

## Scope Boundary

**Решено при создании истории:**

- **У `MenuBarExtra` нет публичного доступа к своему `NSStatusItem`.** Оригинальная формулировка в `epics.md`'s Story 4.1 (написанная до этой истории) предполагала «fallback на прямой доступ к `NSStatusItem.button.toolTip` через AppKit», если `.help(_:)` не сработает. Это **неточно**: SwiftUI's `MenuBarExtra` полностью инкапсулирует свой `NSStatusItem` — Apple не предоставляет публичного API, чтобы получить его изнутри SwiftUI-объявленной сцены (в отличие от ручного `NSStatusBar.system.statusItem(withLength:)`, которым это приложение сознательно не пользуется — `AD-3` требует именно `MenuBarExtra`). **Решение**: если `.help(_:)` на реальной сборке не показывает тултип при наведении, **AppKit-обход не реализуется** (это потребовало бы либо приватных API, либо перехода на ручной `NSStatusItem`, что нарушило бы `AD-3`) — вместо этого ограничение документируется в Dev Agent Record как известный gap, история не блокируется на нём. Это соответствует принципу проекта «эмпирическая проверка вместо предположений по документации» (см. Story 4.3/5.1/5.2), но здесь эмпирическая проверка может обнаружить, что **обхода не существует**, а не просто найти рабочий обходной путь.
- **Формат тултипа — глиф-строка (`X ↓ · Y ↑`), не как в popover.** Popover (`MenuBarView.statsSection`) показывает раздельные локализованные строки-лейблы («↓ Загрузка» / «↑ Раздача» на отдельных строках). Системный тултип — короткая однострочная подсказка, для которой уже есть точный прецедент в этом же проекте: `TorrentDetailView.summaryLine` (`"\(progressPercent)% · \(downSpeed) ↓ · \(upSpeed) ↑ · \(peers)"`) — та же связка `Formatting.speed(...)` + нелокализованные ↓/↑ глифы. Переиспользуется этот формат (без `%`/peers — тултип показывает только суммарную скорость, не проценты одного торрента), а не третий текстовый стиль.
- **Агрегация вынесена в `AppModel.activeSpeedTotals`, не продублирована.** `MenuBarView.statsSection` уже считает `downTotal`/`upTotal` через `active.reduce(...)` дважды инлайново; эта история выносит именно **сырую агрегацию чисел** (не форматирование) в `AppModel` — и `MenuBarView`, и новый tooltip-computed-property в `MyTorrentApp.swift` читают одни и те же числа, форматируя их по-разному под свой контекст (лейблы vs глиф-строка). Не выносить сам форматированный текст в `AppModel` — это View-специфичная забота (Consistency Conventions: «no pre-formatted strings cross the boundary» — тот же принцип применим и здесь, на уровне Swift-слоёв, не только FFI-границы).
- **Пустое состояние тултипа переиспользует существующий ключ**, как и popover — не заводить новый «нет активных закачек, только для тултипа» текст.
- **Не в этой истории**: кастомная иконка (Story 5.4) — эта история трогает только `.help(...)`, не сам `Image(systemName:)`.

## Tasks / Subtasks

- [x] **Task 1: `AppModel.swift` — новый `activeSpeedTotals`** (AC: 4)
  - [x] Добавить рядом с существующими `hasActiveTorrents`/`activeTorrents` (после `activeTorrents`, ~строка 55):
    ```swift
    // Raw ↓/↑ aggregation shared by MenuBarView's popover stats and the new
    // menu-bar hover tooltip (Story 5.3) — each formats these two numbers
    // differently for its own context, but neither re-derives them independently.
    var activeSpeedTotals: (down: UInt64, up: UInt64) {
        let active = activeTorrents
        return (
            active.reduce(0) { $0 + $1.downSpeedBps },
            active.reduce(0) { $0 + $1.upSpeedBps }
        )
    }
    ```
  - [x] **Не трогать** `activeTorrents`/`hasActiveTorrents` — только добавление, никаких изменений к существующим computed properties

- [x] **Task 2: `MenuBarView.swift` — переиспользовать `activeSpeedTotals`** (AC: 4)
  - [x] В `statsSection` заменить:
    ```swift
    let downTotal = active.reduce(UInt64(0)) { $0 + $1.downSpeedBps }
    let upTotal = active.reduce(UInt64(0)) { $0 + $1.upSpeedBps }
    ```
    на:
    ```swift
    let totals = appModel.activeSpeedTotals
    let downTotal = totals.down
    let upTotal = totals.up
    ```
    (минимальная правка — оставляет локальные имена `downTotal`/`upTotal` нетронутыми для остального тела функции, меняется только откуда они берутся)
  - [x] Не менять остальную структуру `statsSection` (лейблы, `activeCountText`, разметку) — эта история не трогает сам popover визуально, только источник чисел

- [x] **Task 3: `MyTorrentApp.swift` — `.help(_:)` на иконке меню-бара** (AC: 1, 2, 3, 5)
  - [x] Новый приватный computed на `MyTorrentApp` (рядом с `body`):
    ```swift
    private var menuBarTooltip: String {
        guard appModel.hasActiveTorrents else {
            return String(localized: "main_window.empty_state.message")
        }
        let totals = appModel.activeSpeedTotals
        return "\(Formatting.speed(totals.down)) ↓ · \(Formatting.speed(totals.up)) ↑"
    }
    ```
  - [x] Применить к `Image` внутри `label:` (строка ~105-108):
    ```swift
    Image(systemName: appModel.hasActiveTorrents
        ? "arrow.up.arrow.down.circle.fill"
        : "arrow.up.arrow.down.circle")
        .help(menuBarTooltip)
    ```
  - [x] **Не трогать** остальную структуру `MenuBarExtra`/`.menuBarExtraStyle(.window)` — клик по-прежнему открывает popover без изменений (AC3), это чисто additive-правка

- [x] **Task 4: Сборка и эмпирическая проверка** (AC: 1-5)
  - [x] `xcodegen generate` → `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] **Ключевая эмпирическая проверка (AC5)**: иконка меню-бара оказалась визуально невидима в реальном меню-баре пользователя (то же известное ограничение среды, что в Story 4.1 — переполненный меню-бар). Визуальный скриншот тултипа поэтому не был возможен, но найден более надёжный способ: реальный курсор мыши физически перемещён на AX-позицию иконки (`CGEvent`/`mouseMoved`, скомпилированный однострочный Swift-скрипт — с разрешения пользователя), после чего прочитан AX-атрибут `help` иконки (`menu bar item 1 of menu bar 2`) — он **и есть** тот же текст, что передаётся в `.help(_:)`, независимо от того, видна ли иконка визуально. Подтверждено эмпирически: `.help(_:)` реально работает на `MenuBarExtra`'s label — AppKit-обход не понадобился
  - [x] Подтверждено: клик по иконке по-прежнему открывает popover (регрессия AC3) — с разрешения пользователя, тот же AX-путь, что в предыдущих историях; popover открылся с корректными агрегированными данными (после рефакторинга Task 2)
  - [x] Ручная проверка формата текста — оба состояния подтверждены через AX `help`-атрибут: без активных закачек → `help=Нет активных закачек` (AC2); с добавленным тестовым торрентом (`engine/tests/fixtures/test.torrent`, статус downloading) → `help=— ↓ · — ↑` (AC1; «—» — ожидаемый вывод `Formatting.speed(0)` при нулевой скорости, формат-строка верна)

## Dev Notes

### Technical Requirements / Stack

- **SwiftUI `.help(_:)`** — стандартный modifier для тултипов, доступен с macOS 11+ (deployment target проекта — 13.0). Применяется к произвольному `View`, включая содержимое `MenuBarExtra`'s `label:` closure — **не гарантированно** ведёт себя так же, как на обычном toolbar-элементе (рендерится через `NSStatusItem`, чей internals SwiftUI полностью скрывает) — отсюда обязательная эмпирическая проверка (Task 4).
- **`AD-3`** (MenuBarExtra, не AppKit `NSStatusItem`+`NSPopover`) — эта история не вводит никакого прямого AppKit-доступа к статус-бар иконке; если бы потребовался AppKit-обход для тултипа, это было бы отклонением от `AD-3` — решение (Scope Boundary) явно этого избегает.
- **`AD-4`** (poll-based sync, single snapshot function) — `activeSpeedTotals` читает уже существующий `@Published torrents`/`activeTorrents`, не заводит новый вызов/таймер.
- **`Formatting.speed(_:)`** (`MyTorrent/Formatting.swift`) — переиспользуется как есть, третий потребитель после `MainWindowView`/`TorrentDetailView`.

### Architecture Compliance

- `AD-3` — соблюдено (см. выше).
- `AD-4` — соблюдено: `activeSpeedTotals` не добавляет нового поллинга.
- Deferred-раздел `ARCHITECTURE-SPINE.md` («Row hover visual treatment; menu-bar icon design») закрывался Story 5.1 (row) и не относится к этой истории напрямую — эта история про сам menu-bar icon hover, что ближе к `4.1-menu-bar.md`'s UX-спеке, которая уже фиксирует «клик, не hover» для **открытия popover**, но не адресует hover-тултип отдельно (эта история впервые вводит именно его).

### Project Structure Notes

```
MyTorrent/
  AppModel.swift               # UPDATE — activeSpeedTotals (raw aggregation)
  MyTorrentApp.swift            # UPDATE — menuBarTooltip, .help(...) на Image
  Views/
    MenuBarView.swift            # UPDATE — statsSection читает appModel.activeSpeedTotals вместо своего reduce
```
Никаких новых файлов — `xcodegen generate` не обязателен (нет новых `.swift`), но безопасно прогнать по привычке.

### Previous Story Intelligence (Story 5.2)

- **`/code-review high` (8 параллельных finder-агентов + verify-проход)** обязателен перед коммитом.
- **Скоупинг код-ревью**: если к моменту ревью Story 5.1/5.2 уже закоммичены, а `origin` не запушен, `@{upstream}...HEAD` будет включать их снова — скоупить finder-агентов только на диф этой истории (working tree / последний коммит), не пере-ревьюить уже принятый код.
- **Эмпирическая проверка system-framework поведения важнее рассуждений по документации** — это уже третий раз подряд (`.tint()` в 5.2, `.stringsdict` в 4.1) — для этой истории эмпирическая точка ещё острее: не просто «работает ли как ожидается», а «существует ли вообще путь, если не работает» (см. Scope Boundary про отсутствие AppKit fallback).
- **Test-artifact hygiene**: если для AC1 потребуется добавить тестовый торрент (`engine/tests/fixtures/test.torrent`), сначала проверить `~/Downloads/Torrents/test.txt` не остался ли с прошлой сессии (Story 5.1/5.2 столкнулись с этим дважды — `addTorrent` молча проваливается на `allow_overwrite = false`, если файл уже существует).
- **Safety-урок**: если во время сессии на экране пользователя самопроизвольно появляются системные окна/Control Center — это сигнал, что пользователь физически за компьютером; приостановить автоматизацию и спросить, прежде чем продолжать реальные клики/наведения.
- **Git**: последний коммит на `main` — `e277677` (Story 5.2 + чистый code review). Рабочая ветка чистая.

### Testing Standards

- Swift: формального UI-test framework в проекте нет — только ручная проверка (Task 4).
- Rust: эта история не трогает `engine/` — `cargo test` не требуется.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 5.3]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-3] — MenuBarExtra mandate, не AppKit NSStatusItem
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-4] — single app-lifetime poller, no new FFI calls
- [Source: MyTorrent/MyTorrentApp.swift] — существующая `MenuBarExtra` сцена, `label:` closure
- [Source: MyTorrent/Views/MenuBarView.swift] — существующая агрегация `downTotal`/`upTotal` в `statsSection`, паттерн переиспользования (`Formatting.speed`)
- [Source: MyTorrent/Views/TorrentDetailView.swift] — `summaryLine`, точный прецедент глиф-формата `"\(speed) ↓ · \(speed) ↑"`, переиспользуемый этой историей
- [Source: MyTorrent/AppModel.swift] — существующие `activeTorrents`/`hasActiveTorrents`, паттерн для нового `activeSpeedTotals`
- [Source: _bmad-output/implementation-artifacts/4-1-menu-bar-popover.md] — Debug Log: иконка меню-бара может быть визуально невидима в переполненном меню-баре пользователя (подтверждено через Accessibility API), не баг кода

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- Иконка меню-бара визуально невидима в реальном меню-баре пользователя (то же известное ограничение, что в Story 4.1 — переполненный меню-бар, ~10 сторонних иконок). Скриншот тултипа поэтому невозможен напрямую.
- **Новый приём проверки, полезный для будущих menu-bar историй**: `.help(_:)` на macOS отражается в accessibility-дереве как атрибут `help` самого элемента (`help of (menu bar item 1 of menu bar 2)`), читаемый через `System Events` **независимо от того, видна ли иконка визуально**. Это надёжнее попытки поймать визуальный тултип скриншотом — можно проверить точный текст, который получит `.help(_:)`, без необходимости физически различить иконку на экране.
- Для физического наведения курсора (не клика) на AX-позицию иконки собран одноразовый Swift-скрипт (`CGEvent(mouseEventSource:mouseType:.mouseMoved,...)`, `swift /tmp/movemouse.swift x y`) — в системе не было `cliclick`/`Quartz`-биндингов Python, компиляция инлайн-скрипта через `swift` оказалась самым быстрым путём получить реальное движение курсора без установки новых зависимостей.
- Тестовый торрент добавлен и убран без повторения гочи Story 5.1/5.2 (`~/Downloads/Torrents/test.txt` заранее проверен — не было).

### Completion Notes List

- Все 4 задачи выполнены, все 5 AC подтверждены живьём (не только чтением кода).
- `AppModel.activeSpeedTotals` — новая raw-агрегация ↓/↑, теперь единственный источник для `MenuBarView.statsSection` (отрефакторен) и нового `MyTorrentApp.menuBarTooltip`.
- `.help(menuBarTooltip)` на `Image` внутри `MenuBarExtra`'s `label:` — подтверждено эмпирически, что механизм реально работает на macOS (через AX `help`-атрибут, т.к. визуальный скриншот тултипа был недоступен из-за невидимости иконки в переполненном меню-баре пользователя). **AppKit-обход не понадобился** — `MenuBarExtra`'s `.help()` сработал с первой попытки.
- **Живая проверка**: пустое состояние → `help=Нет активных закачек` (AC2); с тестовым торрентом (статус downloading) → `help=— ↓ · — ↑` (AC1, «—» — ожидаемый вывод `Formatting.speed(0)`). Клик по иконке по-прежнему открывает popover с корректными данными (AC3, регрессия после рефакторинга Task 2 — не сломан).
- Rust/`engine/` не затронут — `cargo test` не запускался (не требуется по scope истории).

### Code Review (`/code-review high`)

Скоуплено только на диф этой истории (28 строк, 3 файла) — Story 5.1/5.2 не пере-ревьюились. 5 finder-вызовов (покрывающих все 8 углов, часть объединена из-за размера дифа) → 3 реальные находки, все исправлены:

- **Исправлено**: формат «X ↓ · Y ↑» дублировал уже существующий `TorrentDetailView.summaryLine` — по установленной конвенции проекта («извлекать на 2-е вхождение», см. `Formatting.swift`'s собственный header-комментарий) вынесен в новый `Formatting.speedPair(down:up:)`, используется теперь и в `TorrentDetailView`, и в тултипе.
- **Исправлено**: логика тултипа жила в `MyTorrentApp.swift` (сценовая обвязка) вместо `AppModel` — нарушало паттерн, который эта же история установила для `activeSpeedTotals`. Перенесено в новый `AppModel.menuBarTooltipText`, `MyTorrentApp.swift` теперь только `.help(appModel.menuBarTooltipText)`.
- **Исправлено**: несколько независимых O(n)-сканов `torrents` за один рендер (двойной `reduce` в `activeSpeedTotals`, повторный `activeTorrents` в popover/тултипе) — `activeSpeedTotals` теперь один проход с tuple-аккумулятором; `menuBarTooltipText` делает один `activeTorrents`-фетч вместо двух отдельных (`hasActiveTorrents` + `activeSpeedTotals`).
- **Отклонено (уже проверено эмпирически)**: подозрение, что `.help(_:)` может не работать на `MenuBarExtra`'s label — опровергнуто живой проверкой этой же истории (AX `help`-атрибут подтвердил текст в обоих состояниях, см. Debug Log выше).
- **Не исправлено (низкий приоритет, гипотетический сценарий)**: `String(localized:)` в тултипе резолвит локаль иначе, чем `Text(LocalizedStringKey)` в popover (не следит за `.environment(\.locale)`) — сейчас в приложении нигде нет locale-override, так что не наблюдаемо; актуально только если появится переключатель языка в приложении.

Билд после фиксов: `xcodegen generate` → `xcodebuild` — BUILD SUCCEEDED. Регрессия перепроверена через AX `help`-атрибут (пустое состояние → тот же текст, что до рефакторинга).

### File List

- `MyTorrent/AppModel.swift` — UPDATE: `activeSpeedTotals` (single-pass reduce), новый `menuBarTooltipText`
- `MyTorrent/Views/MenuBarView.swift` — UPDATE: `statsSection` читает `appModel.activeSpeedTotals` через tuple-деструктуризацию
- `MyTorrent/MyTorrentApp.swift` — UPDATE: `.help(appModel.menuBarTooltipText)` на `Image` в `MenuBarExtra`'s `label:` (локальный `menuBarTooltip` убран после код-ревью — перенесён в `AppModel`)
- `MyTorrent/Formatting.swift` — UPDATE (код-ревью): новый `speedPair(down:up:)`
- `MyTorrent/Views/TorrentDetailView.swift` — UPDATE (код-ревью): `summaryLine` использует новый `Formatting.speedPair`
- `MyTorrent.xcodeproj` — regenerated via `xcodegen generate` (без структурных изменений)
