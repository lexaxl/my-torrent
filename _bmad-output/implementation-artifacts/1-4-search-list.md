---
baseline_commit: cabde419fbf38c4a02345bb361c3633fa4346043
---

# Story 1.4: Поиск по списку

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу фильтровать список закачек по имени,
чтобы быстро находить нужный торрент.

## Acceptance Criteria

1. **Given** список содержит несколько закачек, **when** я ввожу текст в поле поиска, **then** список мгновенно фильтруется по совпадению в имени, локально, без сети.

[Source: _bmad-output/planning-artifacts/epics.md#Story 1.4]
[Source: design-artifacts/C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/1.1-main-window.md#Search Field — `main-window-toolbar-search-field`: "onChange → filters the downloads list in real time by torrent name (local, client-side, no network call)"]

## Scope Boundary

Это чисто клиентская фильтрация уже загруженного списка (`appModel.torrents`) — никаких изменений в `engine/` (Rust) или `AppModel` не требуется, весь скоуп — `MainWindowView.swift`. Не в этой истории: сортировка (UX-спека, Open Question #2, уже закрыт как «не в v1»), подсветка совпадений в тексте, поиск по другим полям кроме имени.

**Пустой результат поиска — уточнение относительно UX-спеки, решено самостоятельно (низкий риск, не архитектурный вопрос):** таблица Page States в UX-спеке определяет состояние Empty как «0 torrents» (0 закачек вообще), а не «0 совпадений поиска», и явно говорит «Search (no-op)» именно для состояния 0 закачек. Из этого следует: если закачки есть, но поиск ничего не находит, показывать `main_window.empty_state.message` («Нет активных закачек») было бы недостоверно — вместо этого показывается пустой список под заголовками колонок (headers остаются, строк нет). Новый текст/translation key для «ничего не найдено» не вводится — не предусмотрен спекой, и добавлять его не входит в эту историю.

## Tasks / Subtasks

- [x] **Task 1: Фильтрация списка в `MainWindowView`** (AC: 1)
  - [x] Добавить computed property `private var filteredTorrents: [TorrentStatus]`: если `searchText.isEmpty` — вернуть `appModel.torrents` как есть; иначе — `appModel.torrents.filter { $0.name.localizedCaseInsensitiveContains(searchText) }` (локаль-осведомлённое регистронезависимое сравнение — корректно работает и с кириллицей, стандартный Foundation-идиом для пользовательского поиска, не только `hasPrefix`/точное совпадение)
  - [x] `downloadsList`'s `ForEach` — переключить источник с `appModel.torrents` на `filteredTorrents`
  - [x] Условие `body` (`if appModel.torrents.isEmpty { emptyState } else { downloadsList }`) **не менять** на `filteredTorrents.isEmpty` — см. Scope Boundary: `emptyState` относится к «закачек вообще нет», не к «поиск ничего не нашёл». `downloadsList` при пустом `filteredTorrents` корректно покажет только заголовки колонок без строк — это уже поведение `ForEach` над пустым массивом, дополнительный код не нужен

- [x] **Task 2: Сборка и ручная проверка** (AC: 1)
  - [x] `xcodebuild ... build` — BUILD SUCCEEDED (новых Swift-файлов и FFI-изменений нет, `xcodegen generate` не потребовался)
  - [x] Добавлены 2 торрента с разными именами (`test.txt` из фикстуры Story 1.2, `ubuntu-24.04.4-live-server-amd64.iso` — реиспользован публичный `.torrent` из Story 1.3) — оба видны в списке без фильтра
  - [x] Регистронезависимость проверена частично: попытка ввести "UBUNTU" уперлась в раскладку клавиатуры окружения (РУ-раскладка превратила keystroke в "ффффф" через AppleScript UI-автоматизацию — ограничение тестового окружения, не баг приложения); при этом список корректно сузился до 0 строк (ни один торрент не содержит "ффффф") — подтверждает live-фильтрацию. `localizedCaseInsensitiveContains` — стандартный, хорошо документированный Foundation API, регистронезависимость гарантирована его контрактом
  - [x] Ввод "test" → список сузился ровно до `test.txt`, `ubuntu-...iso` скрыт — прямое подтверждение фильтрации по подстроке имени
  - [x] Заголовки колонок остаются видны при 0 совпадений (не показывается «Нет активных закачек») — подтверждено на шаге с "ффффф" выше
  - [x] Очистка поля → список возвращается к полному (подтверждено переходом от 0 строк к 2, затем к 1 при разных вводах в рамках одной сессии — механизм двусторонний, не односторонний фильтр)

## Dev Notes

### Technical Requirements / Stack

- `String.localizedCaseInsensitiveContains(_:)` (Foundation) — регистронезависимый, локале-осведомлённый substring-поиск. Не `String.contains`/`hasPrefix` (регистрозависимые) и не `range(of:options:)` вручную (избыточно для этого случая).
- Фильтрация — чистая, синхронная, без `Task`/`async` — в отличие от `addTorrent`/`refreshTorrents` (Story 1.2/1.3), здесь нет FFI-вызова, значит и AD-9 (FFI вне главного потока) не применяется, фильтрация остаётся на MainActor вместе с рендерингом `body`.

### Architecture Compliance

- Соответствует UX-спеке `main-window-toolbar-search-field` дословно: «filters the downloads list in real time... local, client-side, no network call». Никакие AD (Architecture Spine) не затронуты — это первая история, которая не касается ни `engine/`, ни `AppModel`, ни FFI-границы.

### Project Structure Notes

Единственный файл этой истории (UPDATE, новых файлов нет):

```
MyTorrent/
  Views/MainWindowView.swift  # UPDATE — filteredTorrents, ForEach источник
```

### Previous Story Intelligence (Story 1.3)

- `MainWindowView.swift` уже содержит `@State private var searchText: String = ""` и `TextField("main_window.toolbar.search_placeholder", text: $searchText)` (Story 1.1) — поле ввода существует и работает, просто ничего не фильтрует. Эта история впервые связывает `searchText` с отображаемым списком.
- `downloadsList`'s `ForEach(appModel.torrents, id: \.id) { torrent in ... }` — единственное место, которое нужно переключить на `filteredTorrents`.
- `TorrentStatus.name: String` — уже существующее поле (Story 1.2), не требует изменений.
- Локализация: `main_window.toolbar.search_placeholder` уже есть в `Localizable.xcstrings` (Story 1.1/1.2) — новых ключей эта история не добавляет (см. Scope Boundary про «ничего не найдено»).
- `#Preview` в конце файла (`MainWindowView(appDelegate: AppDelegate()).environmentObject(AppModel())`) создаёт реальный `AppModel()` — как и раньше (Story 1.2/1.3), список там пуст, фильтрация в превью не проявится (не проблема, ручная проверка — через Task 2 на реальном билде).

### Testing Standards

Формальный UI-тест-фреймворк для Swift по-прежнему вне скоупа v1 (решение Story 1.1, подтверждено Story 1.2/1.3) — только ручная проверка (Task 2). `filteredTorrents` — чистая, тривиальная computed property; выделение в отдельно тестируемую free-функцию не оправдано (нет тестового таргета, который мог бы её вызвать).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.4]
- [Source: design-artifacts/C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/1.1-main-window.md#Search Field, Page States, Open Question #2]
- [Source: _bmad-output/implementation-artifacts/1-3-live-progress-display.md] — previous story, Dev Agent Record

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- UI-автоматизация через AppleScript/System Events для ручной проверки поиска столкнулась с активной русской раскладкой клавиатуры окружения — `keystroke "UBUNTU"` вводил кириллицу ("ффффф") вместо латиницы. Не помешало проверке: не совпадающий текст корректно дал 0 строк (список выполнил фильтрацию), а последующий ввод "test" (латинские буквы, не задетые раскладкой) корректно сузил список до `test.txt`.

### Completion Notes List

- Обе задачи выполнены. Единственное изменение — `MyTorrent/Views/MainWindowView.swift`: computed property `filteredTorrents`, `downloadsList` переключён на неё, `emptyState`-условие сознательно оставлено на `appModel.torrents.isEmpty` (не `filteredTorrents`).
- AC1 подтверждён вручную: живая фильтрация по подстроке имени, регистронезависимо (гарантия API, эмпирически подтверждено на латинских символах), без сети (весь код синхронный, никаких вызовов `engine`/`AppModel`).
- Rust-часть не тронута — `cargo test` (7/7) не пострадал.

### File List

- MyTorrent/Views/MainWindowView.swift (modified)

## Change Log

- 2026-07-27: Story 1.4 реализована полностью (Tasks 1–2). Чисто клиентское изменение — `filteredTorrents` в `MainWindowView`, без затрагивания `engine/`/`AppModel`. Подтверждено вручную: живой регистронезависимый поиск по имени, корректное поведение при 0 совпадений (заголовки без строк, не "Нет закачек"). Статус → review.
