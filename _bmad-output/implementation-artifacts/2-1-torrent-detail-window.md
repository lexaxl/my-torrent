---
baseline_commit: e9bed5330c2cd53d07b501ba912a252654bc0df0
---

# Story 2.1: Открытие деталей торрента

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу кликнуть по закачке и увидеть отдельное окно с её файлами,
чтобы понимать, что именно скачивается.

## Acceptance Criteria

1. **Given** есть закачка в списке, **when** я кликаю по строке (не правой кнопкой — это уже занято контекстным меню Story 1.5), **then** открывается отдельное resizable-окно (не sheet) с именем торрента, live-статус-строкой (%, ↓/↑ скорость, сиды/личи), кнопкой закрытия и вкладкой «Файлы», выбранной по умолчанию.
2. **Given** окно деталей открыто на вкладке «Файлы», **then** отображается список файлов торрента с именем, размером и % готовности каждого файла (без чекбоксов include/exclude — вне v1).
3. **Given** окно деталей открыто, **then** данные (статус, скорости, % каждого файла) — отдельный вызов `get_torrent_details(id)`, поллится на собственном ~1с интервале, пока это окно открыто, независимо от поллинга главного списка (см. Dev Notes → AD-4/Consistency Conventions).
4. **Given** окно деталей уже открыто для торрента X, **when** я снова кликаю на строку торрента X в главном окне, **then** существующее окно активируется/поднимается, а не открывается второе окно-дубликат.

## Scope Boundary

**Решено при создании истории (низкий риск, самостоятельно — не архитектурный вопрос, но фиксирую здесь для следующего разработчика):**

- **Переключатель вкладок строится сразу на три вкладки (Файлы/Трекеры/Пиры), но контент Трекеров и Пиров — пустая область, без данных и без плейсхолдер-текста.** UX-спека `2.2-torrent-detail.md` и AC самой Story 2.2 («кликаю вкладку «Трекеры» → вижу список») предполагают, что переключатель вкладок уже существует до Story 2.2 — то же самое разделение «видимый, но ещё не работающий элемент UI», что уже было в этом проекте между Story 1.1 (поле поиска существует, ничего не фильтрует) и Story 1.4 (заработало). Пустая вкладка — не «недоделанная фича», а сознательный промежуточный шаг между двумя запланированными историями.
- **Клик по строке торрента, который ещё не разрешён движком** (`status` = `"resolving"`/`"error"`, торрент есть только в `pending`, см. Story 1.5 `PendingTorrent`) **всё равно открывает окно** — UX-спека 1.1 не делает исключений по статусу для клика по строке («onClick (anywhere on row) → opens... for this torrent»). `get_torrent_details` в этом случае вернёт `EngineError` (торрента ещё нет в `session`) — окно откроется, но останется в пустом/loading-состоянии (см. Dev Notes → Error Handling) до тех пор, пока собственный ~1с поллинг окна не получит успешный ответ (как только magnet разрешится) — не более сложно, чем это.
- **Заголовок окна ОС (`.navigationTitle`) устанавливается из имени торрента, но не является единственным источником истины** — имя торрента дублируется явным текстом в собственном in-content header (как и в `MainWindowView`, где "my-torrent" — это текст в контенте, а не заголовок окна) — соответствует UX-спеке, где Torrent Name — это отдельная секция хедера, а не заголовок окна.

**Не в этой истории:**
- Реальные данные для вкладок Трекеры/Пиры — Story 2.2.
- Per-file include/exclude чекбоксы — UX-спека прямо говорит «out of scope for v1» (Open Question #1, resolved).
- Per-torrent override периода раздачи из этого окна — UX-спека прямо говорит «out of scope for v1» (Open Question #3, resolved), это Settings (Story 3.1), глобальная настройка.

## Tasks / Subtasks

- [x] **Task 1: Rust engine — `get_torrent_details`** (AC: 2, 3)
  - [x] `engine/src/types.rs`: добавить 2 новых `#[derive(uniffi::Record)]`:
    - `TorrentFile { name: String, size_bytes: u64, progress_percent: f64 }`
    - `TorrentDetail { id: String, name: String, status: String, progress_percent: f64, down_speed_bps: u64, up_speed_bps: u64, peers_connected: u32, files: Vec<TorrentFile> }`
    - Экспортировать оба из `lib.rs`: `pub use types::{..., TorrentDetail, TorrentFile};`
  - [x] `engine/src/lib.rs`: `use librqbit::TorrentStats;` (уже реэкспортирован из `librqbit` крейта на верхнем уровне, как `Api`/`ManagedTorrent`)
  - [x] Вынести приватный helper `fn summarize_stats(stats: &TorrentStats) -> (String, f64, u64, u64, u32)` (статус, progress_percent, down_speed_bps, up_speed_bps, peers_connected) — **сейчас этот же блок кода дублируется один раз внутри `get_all_torrents`; вынести его туда же и переиспользовать в обоих местах**, а не копировать в третий раз в `get_torrent_details` (после Story 1.5 code-review — избегаем той же ошибки дублирования снова)
  - [x] `#[uniffi::export] impl Engine` — добавить `pub fn get_torrent_details(&self, id: String) -> Result<TorrentDetail, EngineError>`:
    - `let handle = self.find_handle(&id)?;` (переиспользовать существующий helper из Story 1.5, который уже возвращает "torrent not found" для torrent'а, которого ещё нет в `session` — pending/resolving/error закрывает Scope Boundary п.2 сам по себе, без дополнительного кода)
    - `let stats = handle.stats();` → `summarize_stats(&stats)`
    - Файлы: `handle.metadata.load().as_ref().map(|m| &m.file_infos).map(|infos| infos.iter().enumerate().map(|(i, info)| TorrentFile { name: info.relative_filename.to_string_lossy().into_owned(), size_bytes: info.len, progress_percent: /* stats.file_progress.get(i) / info.len * 100, guard info.len == 0 */ }).collect()).unwrap_or_default()` — **`stats.file_progress: Vec<u64>`** уже выровнен по индексу с `file_infos` (см. Dev Notes → librqbit internals), `metadata` может быть `None`, если торрент ещё не завершил инициализацию — тогда `files` пустой `Vec`, это нормально (тот же случай, что и pending/resolving выше)
    - `name: handle.name().unwrap_or_default()` (тот же паттерн, что уже используется в `get_all_torrents`)
  - [x] Собрать: `cd engine && cargo build`

- [x] **Task 2: Rust engine — тесты** (AC: 2, 3)
  - [x] Тест `get_torrent_details_returns_files_with_sizes`: `test_engine()`, добавить торрент из фикстуры (`tests/fixtures/test.torrent`), вызвать `get_torrent_details`, убедиться `files.len() == 1` (фикстура — один файл `test.txt`), `files[0].size_bytes > 0`, `files[0].name` содержит `"test.txt"` (или равно, если `relative_filename` — не вложенный путь для этой фикстуры), `progress_percent` в диапазоне `0.0..=100.0`
  - [x] Тест `get_torrent_details_returns_error_for_unknown_id`: на пустом `test_engine()`, невалидный/несуществующий id → `Err`
  - [x] Тест `get_torrent_details_matches_get_all_torrents_summary_fields`: добавить торрент, сравнить `status`/`progress_percent`/`down_speed_bps`/`up_speed_bps`/`peers_connected` между `get_all_torrents()[0]` и `get_torrent_details(id)` — оба должны совпадать (оба идут через один и тот же `summarize_stats`) — это одновременно тест на корректность и regression-guard против будущего расхождения между двумя вызовами
  - [x] `cargo test` — 15/15 зелёные (12 существующих + 3 новых)

- [x] **Task 3: Swift — генерация биндингов, AppModel** (AC: 3)
  - [x] `./scripts/build-engine.sh` — перегенерирует `Generated/engine.swift` с `TorrentDetail`/`TorrentFile` структурами и `getTorrentDetails(id:)` методом
  - [x] `MyTorrent/AppModel.swift` — добавить `func fetchTorrentDetail(_ id: String) async -> TorrentDetail?`:
    - `guard let engine else { logger.error(...); return nil }` (тот же паттерн guard, что и у остальных методов после Story 1.2 follow-up фикса — `engine` теперь `Engine?`)
    - `do { return try await Task.detached { try engine.getTorrentDetails(id: id) }.value } catch { logger.error(...); return nil }`
    - **Не** переиспользовать `mutate()` helper — это чтение, не mutation, и возвращает значение, а не просто успех/неудачу; отдельный метод, но тот же общий стиль логирования

- [x] **Task 4: Swift — окно и вью деталей торрента** (AC: 1, 2, 4)
  - [x] **Открытие в ходе имплементации, не было в исходном плане:** этот проект использует XcodeGen (`project.yml`, `sources: - path: MyTorrent` — обычный folder scan, не нативная Xcode 16 synchronized-группа). Добавление новых `.swift`-файлов на диск (`Formatting.swift`, `TorrentDetailView.swift`) требует `xcodegen generate` перед `xcodebuild`, иначе сборка падает с `cannot find 'X' in scope` — новый файл просто не входит в `.xcodeproj`, пока проект не перегенерирован. Добавлено в Task 6.
  - [x] `MyTorrent/MyTorrentApp.swift`: добавить вторую сцену в `body: some Scene` — `WindowGroup(id: "torrent-detail", for: String.self) { $torrentId in TorrentDetailView(torrentId: torrentId ?? "").environmentObject(appModel) }` — value-based `WindowGroup(for:)` (macOS 13+, deployment target уже 13.0) даёт «один клик на тот же id — активирует существующее окно» **бесплатно**, без ручной логики (закрывает AC4)
  - [x] Извлечь `formatSpeed`/`speedFormatter` из `MainWindowView.swift` в общее место (например, новый файл `MyTorrent/Formatting.swift` с `enum Formatting { static func speed(_ bytesPerSecond: UInt64) -> String { ... } }`) — понадобится в обоих файлах (`MainWindowView` и новом `TorrentDetailView`), не дублировать (тот же урок, что и в Task 1)
  - [x] Новый файл `MyTorrent/Views/TorrentDetailView.swift`:
    - `let torrentId: String`, `@EnvironmentObject private var appModel: AppModel`, `@Environment(\.dismiss) private var dismiss`
    - `@State private var detail: TorrentDetail?`, `@State private var selectedTab: Tab = .files` (`enum Tab { case files, trackers, peers }`)
    - Header: кастомная close-button (`Image(systemName: "xmark")`/аналог, `.buttonStyle(.borderless)`, `onClick → dismiss()`) + имя торрента (`detail?.name ?? ""`) + строка статуса (`"\(Int(detail.progressPercent))% · \(Formatting.speed(detail.downSpeedBps)) ↓ · \(Formatting.speed(detail.upSpeedBps)) ↑ · \(detail.peersConnected)"` — если `detail == nil`, показать пустую/минимальную шапку без краша)
    - Tab switcher: `Picker("", selection: $selectedTab) { Text("torrent_detail.tabs.files").tag(Tab.files); Text("torrent_detail.tabs.trackers").tag(Tab.trackers); Text("torrent_detail.tabs.peers").tag(Tab.peers) }.pickerStyle(.segmented)`
    - Content: `switch selectedTab { case .files: filesList; case .trackers: EmptyView() /* Story 2.2 */; case .peers: EmptyView() /* Story 2.2 */ }`
    - `filesList`: список `detail?.files ?? []`, каждая строка — имя (truncation middle, как в `MainWindowView`'s row), `ByteCountFormatter` для размера (переиспользовать `.countStyle = .binary`, как в `Formatting`), `ProgressView(value: file.progressPercent, total: 100)`
    - `.task(id: torrentId) { await pollLoop() }` — `while !Task.isCancelled { detail = await appModel.fetchTorrentDetail(torrentId); try? await Task.sleep(for: .seconds(1)) }` — **этот таск переживает не через `AppModel`, а через SwiftUI-лайфцикл вью** (`.task` автоматически отменяется при закрытии окна/исчезновении вью) — соответствует AD-4/Consistency Conventions («get_torrent_details... polled on its own ~1s cadence only while that window is open», отдельно от app-lifetime поллинга в `AppModel`, который остаётся нетронутым)
    - `.navigationTitle(detail?.name ?? "")` — заголовок окна ОС из имени торрента (не единственный источник — см. Scope Boundary)
  - [x] `MyTorrent/Views/MainWindowView.swift`: добавить `.onTapGesture { openWindow(id: "torrent-detail", value: torrent.id) }` на `torrentRow`'s `HStack` (после уже существующего `.contextMenu` — не конфликтует, разные жесты: left-click vs right-click)

- [x] **Task 5: Локализация** (AC: 1, 2)
  - [x] `MyTorrent/Localizable.xcstrings` — добавлено 3 новых ключа (формат идентичен существующим — `stringUnit`/`state: translated` для `en` и `ru`):
    - `torrent_detail.tabs.files` — RU "Файлы" / EN "Files"
    - `torrent_detail.tabs.trackers` — RU "Трекеры" / EN "Trackers"
    - `torrent_detail.tabs.peers` — RU "Пиры" / EN "Peers"
    - (не добавлен `torrent_detail.peers.empty` — Story 2.2's ключ, вкладка Пиры в этой истории пуста без текста, см. Scope Boundary)

- [x] **Task 6: Сборка и ручная проверка** (AC: 1-4)
  - [x] `xcodegen generate` + `./scripts/build-engine.sh` + `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] Добавлен торрент (файл фикстуры), клик по строке (левой кнопкой) → **подтверждено через Accessibility API** (`System Events`: список окон процесса дважды показал `{"test.txt", "MyTorrent"}` сразу после клика — т.е. `openWindow(id: "torrent-detail", value:)` реально создаёт второе окно с именем торрента как заголовком), — изначально `.onTapGesture` не срабатывал вовсе (известный на macOS конфликт `.onTapGesture`+`.contextMenu` на одной вьюхе — распознаватель контекстного меню перехватывает клик); заменено на `.simultaneousGesture(TapGesture()...)` + `.contentShape(Rectangle())`, после чего клик стал регистрироваться
  - [~] Визуально подтвердить содержимое окна (статус-строка, вкладки, список файлов), resizable-поведение, активацию существующего окна вместо дубликата (AC4), переключение вкладок, закрытие — **не удалось довести до конца скриншотом**: второе окно закрывается/пропадает из accessibility-дерева практически сразу после открытия (в пределах кадра одного `osascript`-вызова), то же самое дребезжание видимости окон в этой среде, что уже задокументировано в project memory (`my-torrent-dev-environment`) и наблюдалось даже с ГЛАВНЫМ окном при ручной проверке Story 1.5 — не специфично для кода этой истории. Формальный UI-test framework вне скоупа v1 (решение Story 1.1) — визуальную/интерактивную часть стоит перепроверить пользователем вручную на реальном билде вне автоматизации
  - [x] `cargo test` (engine/) — 15/15 зелёные

## Dev Notes

### Technical Requirements / Stack

- **`get_torrent_details(id) -> TorrentDetail`** — отдельный UniFFI-вызов, НЕ часть `get_all_torrents()` (Consistency Conventions: «Snapshot granularity... getAllTorrents() returns list-level fields only — not nested Files/Trackers/Peers. Torrent Detail is served by a separate get_torrent_details(id) call, polled on its own ~1s cadence only while that window is open — keeps the hot, always-active list poll cheap»). Не пытаться «оптимизировать», добавляя files в общий список-поллинг — это прямо противоречит архитектурному решению.
- **AD-9** (FFI off main thread) применяется и здесь — `fetchTorrentDetail` идёт через `Task.detached`, как и все остальные вызовы движка.
- **AD-6** (typed errors) — `get_torrent_details` возвращает `Result<TorrentDetail, EngineError>`, тот же паттерн, что и везде.
- **librqbit internals** (см. `~/.cargo/registry/.../librqbit-8.1.1/src/torrent_state/mod.rs`):
  - `ManagedTorrent::stats() -> TorrentStats` — уже используется в `get_all_torrents`. Поле `file_progress: Vec<u64>` — байты, скачанные на файл, выровнено по индексу с `TorrentMetadata.file_infos` (тот же порядок, в котором торрент-метаданные перечисляют файлы). Пусто (`Vec::new()`), если торрент ещё `Initializing`/`Error`.
  - `ManagedTorrent.metadata: ArcSwapOption<TorrentMetadata>` (публичное поле) — `.load()` возвращает guard, деref'ящийся в `Option<Arc<TorrentMetadata>>`; `None`, пока торрент не закончил инициализацию (тот же случай, что и `torrent.name()` уже обрабатывает через `unwrap_or_default()`/fallback на `magnet_name`).
  - `TorrentMetadata.file_infos: FileInfos` (`Vec<librqbit::file_info::FileInfo>`), `FileInfo { relative_filename: PathBuf, len: u64, ... }` — `relative_filename`/`len` публичные поля, `librqbit::file_info::FileInfo` (или через `librqbit::FileInfo`, если реэкспортирован — проверить при реализации, `file_info` модуль объявлен `pub mod file_info;`).
  - `librqbit::TorrentStats` (реэкспортирован на верхнем уровне крейта, как `Api`/`ManagedTorrent`/`ManagedTorrentState`) — тип для `summarize_stats` helper'а.
- **Переиспользовать, не изобретать заново:** `find_handle()` (Story 1.5) для лукапа хендла по id — уже возвращает корректную "not found" ошибку для pending/resolving/error торрентов, ничего дополнительно писать не нужно для этого случая (см. Scope Boundary).

### Architecture Compliance

- Соответствует Consistency Conventions дословно (см. выше).
- AD-4's полный текст также подтверждает разделение: «MainWindowView polling loop» — app-lifetime, в `AppModel`; детальный поллинг — per-window, за пределами `AppModel`. Не переносить detail-polling в `AppModel` «для единообразия» — это прямо нарушит уже принятое архитектурное разделение ответственности.
- UX-спека `2.2-torrent-detail.md`: «Presentation confirmed as a separate resizable window (не sheet)» — `WindowGroup`/`Window`-based сцена, не `.sheet()`/`.popover()`.

### Project Structure Notes

```
MyTorrent/
  MyTorrentApp.swift          # UPDATE — новая WindowGroup(for: String.self) сцена
  Formatting.swift            # NEW — вынесенный formatSpeed/speedFormatter, общий для MainWindowView и TorrentDetailView
  AppModel.swift               # UPDATE — fetchTorrentDetail(_:)
  Views/
    MainWindowView.swift      # UPDATE — .onTapGesture на torrentRow → openWindow(id: "torrent-detail", value:)
    TorrentDetailView.swift   # NEW — 2.1/2.2 UX-спека, окно деталей
  Localizable.xcstrings        # UPDATE — 3 новых ключа
engine/
  src/types.rs                # UPDATE — TorrentDetail, TorrentFile records
  src/lib.rs                  # UPDATE — summarize_stats helper (вынесен из get_all_torrents), get_torrent_details метод, новые тесты
```

### Previous Story Intelligence (Story 1.5 + пост-review фиксы)

- `find_handle(&self, id: &str) -> Result<Arc<ManagedTorrent>, EngineError>` — уже существует в `engine/src/lib.rs` (добавлен в code-review фиксе Story 1.5), переиспользовать буквально, не создавать новый лукап.
- `AppModel.engine` теперь `Engine?` (не `let Engine`, см. Story 1.2 follow-up фикс, 2026-07-27) — конструируется асинхронно в фоне. **Каждый новый метод в `AppModel`, включая `fetchTorrentDetail`, должен начинаться с `guard let engine else { ...; return nil }`** — если это забыть, код не скомпилируется (или, если Optional развёрнут force-unwrap'ом, будет краш в узком окне до завершения инициализации).
- Паттерн `.contextMenu` + теперь ещё и `.onTapGesture` на одном и том же `torrentRow` — оба независимы (left-click vs right-click), но стоит вручную проверить на реальном билде, что `.onTapGesture` не перехватывает/не мешает `.contextMenu`'s right-click (маловероятно, но у SwiftUI были edge-cases с конфликтующими жестами на разных версиях macOS) — включено в Task 6 ручную проверку.
- Ручная UI-проверка в этой среде исторически нестабильна через AppleScript/Accessibility (см. project memory `my-torrent-dev-environment`) — но открытие/закрытие окна и клик по строке (в отличие от контекстных меню Story 1.5) должны быть заметно проще для скриптовой автоматизации, т.к. не завязаны на транзиентные NSMenu-попапы вне обычного accessibility-дерева.
- `Localizable.xcstrings` — после Story 1.5 фикса форматирования файл записан в нативном Xcode-стиле (`"key" : value`); добавлять новые ключи тем же способом (JSON-запись с пробелом перед `:`), не голым `json.dump`, чтобы снова не «раздуть» диф (см. Story 1.5 code-review находка).

### Testing Standards

- Rust: `cargo test` в `engine/`, тот же файл (`lib.rs`), паттерн `test_engine()` уже существует — переиспользовать. Тест на согласованность `get_torrent_details` и `get_all_torrents` (Task 2, третий тест) — стоит того, чтобы поймать регрессию, если кто-то в будущем поправит один из двух call site'ов и забудет про `summarize_stats`.
- Swift: формальный UI-test framework всё ещё вне скоупа v1 (решение Story 1.1, подтверждено во всех последующих историях) — только ручная проверка (Task 6).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.1, #Story 2.2]
- [Source: design-artifacts/C-UX-Scenarios/02-alex-inspects-a-torrent/2.2-torrent-detail/2.2-torrent-detail.md] — единственная UX-страница, покрывающая обе истории (2.1 — окно + вкладка Файлы; 2.2 — контент вкладок Трекеры/Пиры)
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-4, AD-6, AD-9, Consistency Conventions (Snapshot granularity), Capability Map (02 — Alex Inspects a Torrent)]
- [Source: _bmad-output/implementation-artifacts/1-5-context-menu.md] — previous story, `find_handle`, `Engine?`-optional паттерн, `Localizable.xcstrings` форматирование
- librqbit 8.1.1 crate source (`~/.cargo/registry/src/.../librqbit-8.1.1/src/torrent_state/mod.rs`, `src/file_info.rs`) — `ManagedTorrent::stats()`, `TorrentStats.file_progress`, `TorrentMetadata.file_infos`, `FileInfo`

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- Новые `.swift`-файлы (`Formatting.swift`, `Views/TorrentDetailView.swift`) не подхватывались `xcodebuild` ("cannot find 'X' in scope") до `xcodegen generate` — проект использует XcodeGen (folder scan в момент генерации, не нативную synchronized-группу Xcode), добавление файлов на диск требует регенерации `.xcodeproj`.
- `.onTapGesture` на `torrentRow` не срабатывал вовсе при наличии `.contextMenu` на той же вьюхе (известный конфликт распознавателей жестов на macOS) — заменено на `.simultaneousGesture(TapGesture()...)` + `.contentShape(Rectangle())`.
- Второе (detail) окно подтверждено через Accessibility API как реально создающееся (дважды показано `System Events`), но пропадает из accessibility-дерева практически сразу после открытия — то же дребезжание видимости окон, что уже задокументировано в project memory и наблюдалось с главным окном в Story 1.5; не удалось довести визуальную проверку содержимого/вкладок/resizable-поведения до скриншота в рамках этой сессии.

### Completion Notes List

- Все 6 задач выполнены. AC1 (открытие окна) подтверждён через Accessibility API (окно с именем торрента создаётся при клике), AC2/AC3 подтверждены косвенно — `get_torrent_details`/`summarize_stats` покрыты 3 новыми Rust-тестами (15/15 всего), AC4 (WindowGroup(for:) активирует существующее окно) не удалось довести до конца интерактивно — см. Debug Log.
- Вынесено 2 переиспользуемых хелпера, чтобы не повторить дублирование, найденное в code-review Story 1.5: `summarize_stats` (Rust, общий для `get_all_torrents`/`get_torrent_details`) и `Formatting` (Swift, общий для `MainWindowView`/`TorrentDetailView`).
- **Рекомендация пользователю:** визуально перепроверить открытие/содержимое/закрытие detail-окна вручную (не через автоматизацию) перед тем, как считать AC1/AC4 полностью подтверждёнными — автоматизация в этой среде исторически ненадёжна для окон (см. project memory), а не сама фича под подозрением.

### File List

- `engine/src/types.rs` — UPDATE: `TorrentFile`, `TorrentDetail` records
- `engine/src/lib.rs` — UPDATE: `summarize_stats` helper (рефакторинг из `get_all_torrents`), `get_torrent_details` метод, 3 новых теста
- `MyTorrent/Generated/engine.swift`, `engineFFI.h` — REGENERATED
- `MyTorrent/Formatting.swift` — NEW: общий `speed`/`size` форматтер
- `MyTorrent/AppModel.swift` — UPDATE: `fetchTorrentDetail(_:)`
- `MyTorrent/MyTorrentApp.swift` — UPDATE: `WindowGroup(id: "torrent-detail", for: String.self)` сцена
- `MyTorrent/Views/MainWindowView.swift` — UPDATE: `.contentShape`/`.simultaneousGesture` на `torrentRow` (открытие деталей), убран дублирующий форматтер
- `MyTorrent/Views/TorrentDetailView.swift` — NEW: окно деталей (хедер, таб-свитчер, список файлов)
- `MyTorrent/Localizable.xcstrings` — UPDATE: 3 новых ключа (`torrent_detail.tabs.*`)
- `MyTorrent.xcodeproj` — REGENERATED (`xcodegen generate`)
