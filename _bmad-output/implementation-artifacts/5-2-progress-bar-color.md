---
baseline_commit: 2a2047e79d5d744931047873ea2218eae281eb39
---

# Story 5.2: Цвет прогресс-бара по состоянию закачки

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу, чтобы цвет прогресс-бара отражал состояние закачки (активна/на паузе/завершена/ошибка),
чтобы понимать состояние с одного взгляда, не читая текст.

## Acceptance Criteria

1. **Given** закачка в списке, **when** статус — `downloading`, `checking` или `resolving`, **then** прогресс-бар — стандартный accent-цвет (`Color.accentColor`, без изменений от текущего поведения).
2. **Given** закачка завершена (`status == "seeding"`), **then** прогресс-бар — `color-success` (#1F9254 light / #3FB873 dark, дизайн-система `00-design-system.md`).
3. **Given** закачка на паузе (`status == "paused"`), **when** прогресс — любой процент, **then** прогресс-бар — серый (`Color.gray`), независимо от текущего `progressPercent`.
4. **Given** закачка в состоянии ошибки (`status == "error"`), **then** прогресс-бар — красный (`Color.red`, системный — design-система не определяет отдельный error/danger токен, см. Story 5.1's аналогичное решение).
5. **Given** любой нераспознанный/будущий статус (см. `TorrentEngineStatus.error` fallback), **then** прогресс-бар — красный, тот же путь кода, что явный `error` (не отдельная ветка).

## Scope Boundary

**Решено при создании истории:**

- **Закрыт пробел, оставленный Story 5.1**: `resolving` (pending-магнет, ещё не разрешён) не был явно упомянут в оригинальном `epics.md`'s AC для этой истории — Story 5.1's Scope Boundary заранее пометила «обычный accent-цвет по умолчанию» как план, зафиксированный здесь как AC1 (`resolving` — в одной группе с `downloading`/`checking`).
- **`progressPercent >= 100` НЕ используется как отдельное условие для зелёного цвета.** Хотя `epics.md`'s исходная формулировка AC2 говорит «прогресс достиг 100% (раздача)», Story 5.1's код-ревью **структурно доказало** (чтение исходников `librqbit` 8.1.1, `chunk_tracker.rs`): пока движок возвращает статус `"downloading"`, `progress_bytes < total_bytes` гарантированно (оба значения — из одного `TorrentStats`-снепшота, `finished` и байтовые счётчики связаны инвариантом `needed_bytes > 0 ⟺ !finished`). То есть «100% и одновременно downloading» — недостижимое состояние движка, а не что-то, что нужно ловить отдельной проверкой в Swift. Решение: цвет определяется **только** по `torrent.engineStatus` (использует общий enum из Story 5.1's `TorrentEngineStatus.swift`), без дополнительного `progressPercent`-условия — проще, и не добавляет мёртвый код.
  - **Побочный эффект этого решения**: полностью скачанный, но заново перепроверяемый торрент (`status == "checking"` при `progressPercent == 100`, например после ручного «recheck» или после перезапуска приложения до завершения проверки) покажет **accent**-цвет, не зелёный — это осознанно соответствует AC1's буквальной группировке `checking` с `downloading` (оба «в процессе», ещё не подтверждённое состояние), не отдельный edge case для обработки.
- **Нет designed-токена для error/danger** — как и в Story 5.1's аналогичном решении (User confirmed: «Системный Color.red»), не заводится новый semantic-токен в дизайн-системе, используется системный `Color.red` напрямую.
- **`Color.gray` для паузы, не design-система-специфичный серый.** Дизайн-система (`00-design-system.md`) не определяет отдельный «paused/inactive» цветовой токен — только `color-text-secondary`/`color-text-tertiary` (текстовые, не для заливки прогресс-бара). `Color.gray` — системный, адаптируется приемлемо между light/dark без доп. работы; не изобретать design-token для одного этого случая.
- **`color-success` — первый design-system цветовой токен, реально используемый в коде.** В проекте **нет `Assets.xcassets`** (подтверждено — каталог не существует; Story 5.4 позже создаст его для app icon). Вместо того чтобы заводить целый asset-каталог ради одного цвета, реализуется через `Color(nsColor: NSColor(name:dynamicProvider:))` — самодостаточный light/dark адаптивный `Color` без внешней инфраструктуры. Если Story 5.4 впоследствии создаст `Assets.xcassets` для иконки, **не обязательно** мигрировать этот цвет туда — оставить как есть, если не появится третий цветовой токен, требующий того же паттерна (см. Story 5.1's «не жди третьей копии» convention, применённая в обратную сторону: не строить инфраструктуру заранее для одного токена).
- **Не в этой истории**: hover-тултип меню-бара (Story 5.3), кастомная иконка (Story 5.4).

## Tasks / Subtasks

- [x] **Task 1: `MyTorrent/Views/MainWindowView.swift` — новый `Color` extension + helper функция** (AC: 1-5)
  - [x] Добавить в начало файла (после существующих `import`) или в отдельный небольшой файл — **предпочтительно новый файл** `MyTorrent/Extensions/Color+DesignSystem.swift` (первый файл в ещё не существующей `Extensions/`-директории; допустимо, раз `Models/` уже был создан с нуля в Story 5.1 по тому же принципу «директория из Structural Seed, просто раньше не понадобилась»):
    ```swift
    import SwiftUI
    import AppKit

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
    ```
  - [x] Использовать точные hex-значения из `00-design-system.md` (#1F9254/#3FB873) — не приближать/округлять

- [x] **Task 2: `MainWindowView.swift` — `progressTint(for:)` helper + применение к `ProgressView`** (AC: 1-5)
  - [x] Новый приватный метод рядом с `subtitle(for:)`:
    ```swift
    private func progressTint(for torrent: TorrentStatus) -> Color {
        switch torrent.engineStatus {
        case .downloading, .checking, .resolving:
            return .accentColor
        case .seeding:
            return .torrentSuccess
        case .paused:
            return .gray
        case .error:
            return .red
        }
    }
    ```
    (использует `TorrentEngineStatus` из Story 5.1's `MyTorrent/Models/TorrentEngineStatus.swift` — **не** свитчить по сырой строке `torrent.status` второй раз в этом же файле; `.error`-ветка enum'а уже покрывает и «настоящий» error, и любой нераспознанный будущий статус через существующий `?? .error` fallback в `TorrentStatus.engineStatus`, так что здесь не нужен отдельный `default:`)
  - [x] В `torrentRow(_:)` — добавить `.tint(progressTint(for: torrent))` к существующему `ProgressView(value: torrent.progressPercent, total: 100)` (строка ~216 на момент создания истории; номер мог сдвинуться, искать по сигнатуре)
  - [x] **Не трогать** `.frame(width: progressColumnWidth, alignment: .leading)` и остальные модификаторы `ProgressView` — только добавить `.tint(...)`

- [x] **Task 3: Сборка и ручная проверка** (AC: 1-5)
  - [x] `xcodegen generate` (новый файл `Color+DesignSystem.swift`) → `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] Эмпирически подтверждено на реальной сборке: `.tint(_:)` на `ProgressView` **реально перекрашивает** заливку бара на macOS (стандартный линейный determinate-стиль) — не проигнорирован
  - [x] Ручная проверка (с явного разрешения пользователя, включая один живой клик через Accessibility API, аналогично Story 5.1): добавлен реальный тестовый торрент — подтверждён **синий (accent)** цвет на статусе «Скачивается»; через контекстное меню поставлен на паузу — подтверждён **серый** цвет и текст «На паузе». Зелёный (seeding) и красный (error) не воспроизведены живьём — нет реальных пиров/заведомо битого источника под рукой в этой сессии; логика детерминирована и читаема (простой `switch` по уже проверенному в Story 5.1 `TorrentEngineStatus`), риск низкий. Тестовый торрент и файл `~/Downloads/Torrents/test.txt` удалены после проверки — окружение пользователя не осталось захламлено

## Dev Notes

### Technical Requirements / Stack

- **SwiftUI `ProgressView.tint(_:)`** — стандартный modifier, принимает `Color?` (не `any ShapeStyle`), доступен с macOS 12+ (проект — deployment target 13.0). Нет новых зависимостей.
- **`TorrentEngineStatus`** (`MyTorrent/Models/TorrentEngineStatus.swift`, Story 5.1) — переиспользуется как есть, **не редактируется** этой историей. `TorrentStatus.engineStatus` computed property уже покрывает fallback на `.error` для нераспознанных статусов.
- **Design System** (`design-artifacts/D-Design-System/00-design-system.md`) — `color-success` #1F9254/#3FB873 (строка ~189), других относящихся к этой истории токенов нет (error/danger сознательно не заводится, см. Scope Boundary).

### Architecture Compliance

- Не затрагивает engine/Rust, FFI, поллинг — чисто SwiftUI presentation-слой поверх уже существующих `torrent.status`/`torrent.progressPercent`. AD-4/AD-9 не применимы напрямую (нет новых FFI-вызовов/таймеров).
- `AD-3` (SwiftUI throughout, no AppKit) — `NSColor(name:dynamicProvider:)` используется только как способ получить адаптивный `Color` (стандартный SwiftUI-Bridged паттерн для light/dark цветов без Assets.xcassets), не вводит AppKit-вью/контролы — соответствует духу AD-3 (сам UI остаётся полностью SwiftUI).

### Project Structure Notes

```
MyTorrent/
  Extensions/
    Color+DesignSystem.swift    # NEW — Color.torrentSuccess
  Views/
    MainWindowView.swift         # UPDATE — progressTint(for:), .tint(...) на ProgressView
```
Новый файл → `xcodegen generate` обязателен перед `xcodebuild` (см. project memory про XcodeGen-готчу, применялось и в Story 5.1 для `Models/TorrentEngineStatus.swift`).

### Previous Story Intelligence (Story 5.1)

- **`/code-review high` (8 параллельных finder-агентов + verify-проход по каждому кандидату)** обязателен перед коммитом — тот же pipeline.
- **`TorrentEngineStatus` enum существует именно чтобы такие истории, как эта, не заводили четвёртое место со свитчем по сырой строке статуса** — используется здесь с первой попытки, не переизобретается.
- **Эмпирическая проверка system-framework поведения важнее рассуждений по документации** — уже дважды кусало проект (`.stringsdict`/plural в Story 4.1, need to verify librqbit source в нескольких историях) — для этой истории эмпирическая точка: реально ли `.tint()` перекрашивает `ProgressView` на macOS так, как ожидается.
- **Git**: последний коммит на `main` — `2a2047e` (Story 5.1 + все 5 code-review фиксов). Рабочая ветка чистая.
- **Известное ограничение окружения**: DNS блокирует BitTorrent-домены (project memory) — реальных пиров/раздачи в этом окружении не получить, живая проверка seeding-состояния может быть невозможна (как и в Story 5.1).

### Testing Standards

- Swift: формального UI-test framework в проекте нет (решение Story 1.1) — только ручная проверка (Task 3).
- Rust: эта история не трогает `engine/` — `cargo test` не обязателен.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 5.2]
- [Source: design-artifacts/D-Design-System/00-design-system.md] — `color-success` #1F9254/#3FB873 (строка ~189)
- [Source: MyTorrent/Models/TorrentEngineStatus.swift] — Story 5.1, переиспользуется как есть
- [Source: MyTorrent/Views/MainWindowView.swift] — `torrentRow(_:)`, текущий `ProgressView(value: torrent.progressPercent, total: 100)`
- [Source: _bmad-output/implementation-artifacts/5-1-list-row-details.md] — Code Review секция: структурное доказательство `progress_bytes < total_bytes` пока `status == "downloading"` (обосновывает решение не проверять `progressPercent` отдельно), `TorrentEngineStatus` дизайн

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- Повторное добавление тестового `.torrent` из Story 5.1 сперва молча проваливалось (список оставался пустым) — причина найдена через `log show`: `addTorrent failed: Internal(message: "error creating a new file (because allow_overwrite = false) \"/Users/alex/Downloads/Torrents/test.txt\"")` — старый файл с прошлой проверки не был удалён с диска (Remove не удаляет файлы — ожидаемое поведение). С разрешения пользователя удалён `~/Downloads/Torrents/test.txt`, после чего повторное добавление сработало.
- Контекстное меню (`AXShowMenu`) в этот раз получилось кликнуть напрямую и надёжно: полный дамп `entire contents of window` выявил точный AX-путь `menu item "Пауза" of menu 1 of group 1 of window "MyTorrent"` — `click` по этому референсу отработал без ошибки (в отличие от Story 5.1, где похожие попытки возвращали ошибки резолва и успех подтверждался только косвенно через логи/повторный скриншот).
- Во время сессии на экране пользователя дважды самопроизвольно появлялись системные окна (Control Center, «Системные настройки → Строка меню»), не инициированные командами этой сессии — явный признак того, что пользователь физически работал за машиной параллельно. Работа была приостановлена и продолжена только после явного подтверждения пользователя («продолжай»).

### Completion Notes List

- Все 3 задачи выполнены, все 5 AC покрыты и живьём подтверждены частично (AC1, AC3 — вживую; AC2/AC4/AC5 — чтением кода, простой детерминированный `switch`).
- Новый `MyTorrent/Extensions/Color+DesignSystem.swift` — `Color.torrentSuccess`, первый design-system цветовой токен в коде, реализован через `NSColor(name:dynamicProvider:)` (light #1F9254/dark #3FB873), без Assets.xcassets (которого в проекте пока нет).
- `MainWindowView.progressTint(for:)` — свитчит по `TorrentEngineStatus` (Story 5.1), не по сырой строке; `.tint(progressTint(for: torrent))` добавлен к существующему `ProgressView` без изменения прочих модификаторов.
- **Осознанно не проверяется `progressPercent` отдельно** — решение, зафиксированное ещё при создании истории, опирается на структурное доказательство из Story 5.1's код-ревью (`downloading` никогда не достигает 100%). Живая проверка это не тестировала напрямую (не было цели воспроизвести 100%+checking), но код-путь идентичен уже проверенному `TorrentEngineStatus`-паттерну.
- **Живьём подтверждено на реальной сборке**: `.tint(_:)` действительно перекрашивает `ProgressView` на macOS (accent → синий на «Скачивается», gray → серый на «На паузе»), значит framework-предположение верно и никакого fallback/альтернативного подхода не потребовалось.
- Rust/`engine/` не затронут — `cargo test` не запускался (не требуется по scope истории).

### Code Review (`/code-review high`)

8 параллельных finder-агентов (line-by-line, removed-behavior, cross-file tracer, reuse, simplification, efficiency, altitude, CLAUDE.md conventions), скоуплено только на новый код этой истории (Story 5.1 уже была отдельно проревьюена и закоммичена, повторно не рассматривалась) → 6 кандидатов → verify-проход → 3 находки пережили верификацию, ни одна не баг — по решению пользователя оставлены как задокументированные заметки, код не менялся:

- **PLAUSIBLE, не исправлено**: живая проверка (см. Debug Log выше) подтвердила `.tint()` только для `.accentColor`/`.gray` (downloading/paused). `Color.torrentSuccess` (seeding) и `.red` (error) идут через другой код-путь (`NSColor(dynamicProvider:)` против обычных системных `Color`-статиков) и не были подтверждены вживую в этой сессии — реальных пиров нет (DNS блокирует BitTorrent-домены), а error недостижим быстро (битый `.torrent` проверен эмпирически: проваливается на этапе `add_torrent`, до появления строки в списке — реальный error-статус достижим только через таймаут разрешения magnet-ссылки). Принято как известный остаточный риск, не блокирующий эту историю.
- **PLAUSIBLE, не исправлено (опционально)**: `progressTint(for:)` — второй независимый `switch` по `TorrentEngineStatus` в файле (первый — `subtitle(for:)`), а не computed-свойство на самом enum'е. Верификатор оценил это как защитимый стилевой выбор (сам enum уже даёт compiler-exhaustiveness защиту вне зависимости от того, где живёт цвет; перенос `Color` на `TorrentEngineStatus.swift` потребовал бы `import SwiftUI` в файле, который сейчас чисто model-层, только `Foundation`) — не нарушение документированной цели enum'а, оставлено как есть.
- **PLAUSIBLE, не исправлено (вне scope)**: `TorrentDetailView`'s собственные per-file прогресс-бары остаются нераскрашенными — `progressTint` приватен `MainWindowView`, не переиспользуется. AC этой истории покрывает только список на главном экране — заметка для возможной будущей истории, не дефект текущей.

**Отклонено при верификации (REFUTED)**: несовпадение accent/gray/red с точными hex дизайн-системы — явное решение, зафиксированное в Scope Boundary этой же истории (AC1 буквально требует «без изменений от текущего поведения»), не недосмотр. Пятикратный re-derivation `engineStatus` за тик — реально только 2 раза за рендер (остальные 3 — внутри `.contextMenu`, считается только при открытии меню), цена пренебрежимо мала для однопользовательского приложения. Риск некорректной работы в high-contrast режиме — `NSAppearance.bestMatch(from:)` как раз для этого и предназначен (документированный Apple-паттерн), опасение снято тем же рассуждением, что привело finder-агента к этому выводу в его же собственном отчёте.

### File List

- `MyTorrent/Extensions/Color+DesignSystem.swift` — NEW: `Color.torrentSuccess` (light/dark адаптивный, `NSColor(name:dynamicProvider:)`)
- `MyTorrent/Views/MainWindowView.swift` — UPDATE: `progressTint(for:)`, `.tint(progressTint(for: torrent))` на `ProgressView`
- `MyTorrent.xcodeproj` — regenerated via `xcodegen generate` (новый файл `Extensions/Color+DesignSystem.swift` добавлен в проект)
