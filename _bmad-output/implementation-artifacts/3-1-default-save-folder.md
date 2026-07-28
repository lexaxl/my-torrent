---
baseline_commit: 64dc253099a9a60fb675b5c1ff40e034cac8da6d
---

# Story 3.1: Папка сохранения по умолчанию

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу задать папку, в которую будут сохраняться новые закачки,
чтобы файлы оказывались там, где я ожидаю.

## Acceptance Criteria

1. **Given** окно настроек открыто (умолчание — `~/Downloads`), **when** я нажимаю «Выбрать...» и выбираю папку в системном диалоге, **then** значение применяется мгновенно, без отдельной кнопки сохранения.
2. **Given** папка сохранения изменена, **when** я добавляю новую закачку, **then** она использует новый путь.
3. **Given** главное окно открыто, **when** я кликаю по иконке-шестерёнке в toolbar (уже существует, сейчас `.disabled(true)` — см. Dev Notes), **then** открывается отдельное окно «Настройки» с полем «Папка для сохранения» и текущим значением.

## Scope Boundary

**Решено при создании истории (низкий риск, самостоятельно):**

- **Окно «Настройки» в этой истории содержит только секцию «Папка для сохранения».** UX-спека `3.1-settings.md` рисует один экран сразу с обеими секциями (папка сохранения + период раздачи), но эпики.md разносит их по двум отдельным историям (3.1 — папка, 3.2 — период раздачи). В отличие от паттерна Story 1.1→1.4 (поле поиска — видимый, но инертный контрол, где ничего не меняется визуально при вводе) и Story 2.1→2.2 (пустая область вкладки), радио-группа периода раздачи, отрендеренная, но не реагирующая на клик выбором, выглядела бы визуально сломанной (пользователь кликает по radio-button — он не подсвечивается). Поэтому в этой истории секция периода раздачи вообще не рендерится — Story 3.2 добавит её в это же окно, а не создаст новое.
- **Иконка-шестерёнка в toolbar уже существует** (`MyTorrent/Views/MainWindowView.swift`, `toolbar` computed property) — добавлена как видимый-но-неактивный элемент с комментарием `// Открывает окно настроек (3.1) — реализуется в Story 3.1/3.2.` и `.disabled(true)`. Эта история включает её (`openWindow(id: "settings")`), не создаёт заново.
- **Изменение сигнатуры `Engine::add_torrent`** — самое рискованное место этой истории. Архитектурный AD-5 («Swift owns settings via UserDefaults; settings flow one-directionally into Rust as call parameters») и сама Consistency Conventions таблица («`add_torrent(source: TorrentSource, ...)` ... пинned explicitly» — обратите внимание на `...` в исходной формулировке, место для доп. параметров уже было заложено на Architecture-этапе) требуют, чтобы папка сохранения передавалась **на каждый вызов `add_torrent`**, а не была зафиксирована один раз при старте `Engine`. `librqbit::AddTorrentOptions.output_folder: Option<String>` — публичный, реэкспортированный (`pub use session::{..., AddTorrentOptions, ...}`) тип, **не** приватный (в отличие от `PeerStatsFilter` из Story 2.2) — подтверждено рабочей компиляцией при создании этой истории (временный scratch-код, `cargo check`, убран из дерева). Новая сигнатура: `add_torrent(source: TorrentSource, download_dir: String)`.
- **`Engine::new(download_dir)` (конструктор, задаёт базовую папку сессии librqbit) не меняется** — остаётся тем, чем был. Реальный эффект настройки достигается через per-call `output_folder` в `AddTorrentOptions`, не через переконфигурацию уже запущенной `Session`. `AppModel.init()` по-прежнему передаёт стартовое значение в конструктор (теперь через `AppSettings.saveLocationPath`, а не задублированный инлайн-фоллбэк — см. Task 3), просто по другой причине: чтобы у сессии была разумная базовая папка, а не потому что она читается на каждый вызов.
- **Изменение сигнатуры `add_torrent` затрагивает 14 существующих вызовов в `engine/src/lib.rs`'s `#[cfg(test)] mod tests`** (все текущие тесты Story 1.x/1.5/2.1/2.2, вызывающие `engine.add_torrent(TorrentSource::...)`) — все 14 нужно обновить вторым аргументом. Подробный список тестов — в Dev Notes → Previous Story Intelligence.
- **Не в этой истории:** секция периода раздачи (Story 3.2); перемещение файлов уже добавленных/докачиваемых торрентов при смене папки (AC2 говорит только про «следующая добавленная закачка»); валидация выбранного пути (диалог `NSOpenPanel` уже ограничивает выбор существующей директорией — UX-спека's Conditional Sections прямо это подтверждает).

## Tasks / Subtasks

- [x] **Task 1: Rust engine — `add_torrent` принимает `download_dir`** (AC: 2)
  - [x] `engine/src/lib.rs` — импорт: добавить `AddTorrentOptions` в `use librqbit::{...}` (уже публично реэкспортирован, ничего добавлять в `Cargo.toml` не нужно)
  - [x] `pub fn add_torrent(&self, source: TorrentSource, download_dir: String) -> Result<String, EngineError>` — новый параметр, пробрасывается в оба ветки:
    ```rust
    pub fn add_torrent(&self, source: TorrentSource, download_dir: String) -> Result<String, EngineError> {
        match source {
            TorrentSource::Path { path } => {
                let add = AddTorrent::from_local_filename(&path).map_err(internal_error)?;
                self.add_torrent_blocking(add, download_dir)
            }
            TorrentSource::Bytes { bytes } => {
                self.add_torrent_blocking(AddTorrent::from_bytes(bytes), download_dir)
            }
            TorrentSource::Magnet { uri } => self.add_magnet(uri, download_dir),
        }
    }
    ```
  - [x] `fn add_torrent_blocking(&self, add: AddTorrent<'static>, download_dir: String) -> Result<String, EngineError>` — строит `AddTorrentOptions` и передаёт `Some(opts)` вместо `None`:
    ```rust
    fn add_torrent_blocking(&self, add: AddTorrent<'static>, download_dir: String) -> Result<String, EngineError> {
        let opts = AddTorrentOptions {
            output_folder: Some(download_dir),
            ..Default::default()
        };
        let response = runtime()
            .block_on(self.session.add_torrent(add, Some(opts)))
            .map_err(internal_error)?;
        let handle = response
            .into_handle()
            .ok_or_else(|| internal_error("add_torrent did not return a managed torrent handle"))?;
        Ok(handle.info_hash().as_string())
    }
    ```
  - [x] `fn add_magnet(&self, uri: String, download_dir: String) -> Result<String, EngineError>` — тот же `AddTorrentOptions`, передать в фоновый `session.add_torrent(AddTorrent::from_url(uri), Some(opts))` внутри `runtime().spawn(async move { ... })` (перенести конструирование `opts` до `spawn`, захватить по `move` вместе с `uri`/`session`/`pending`/`pending_id`)
  - [x] Собрать: `cd engine && cargo build`

- [x] **Task 2: Rust engine — обновить существующие тесты под новую сигнатуру** (AC: 2)
  - [x] В `#[cfg(test)] mod tests` добавить хелпер `test_download_dir()` — **отклонение от плана**: `std::env::temp_dir()` (общий системный temp-каталог) вызвал коллизии между параллельно идущими тестами, добавляющими один и тот же файл фикстуры («test.txt») в одну и ту же папку (`allow_overwrite = false`) — заменено на `tempfile::tempdir().unwrap().keep().to_string_lossy().into_owned()` (свежая уникальная директория на каждый вызов, `.keep()` намеренно пропускает очистку — та же конвенция, что уже негласно действует для сессионной `TempDir` в `test_engine()`)
  - [x] Обновить **все 14** существующих вызовов `engine.add_torrent(TorrentSource::Path { ... })` / `engine.add_torrent(TorrentSource::Magnet { ... })` в `mod tests`, добавив второй аргумент `test_download_dir()` — не пропустить ни один (иначе `cargo build --tests` не скомпилируется, это подскажет, если что-то забыто)
  - [x] Новый тест `add_torrent_uses_provided_download_dir_as_output_folder` (ключевой тест этой истории — напрямую проверяет AC2 на уровне движка):
    ```rust
    #[test]
    fn add_torrent_uses_provided_download_dir_as_output_folder() {
        let engine = test_engine();
        let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/test.torrent");
        let custom_dir = tempfile::tempdir().unwrap(); // держать привязанным к переменной — иначе TempDir Drop удалит папку раньше add_torrent
        let custom_dir_path = custom_dir.path().to_string_lossy().into_owned();

        let id = engine
            .add_torrent(
                TorrentSource::Path { path: fixture.to_string() },
                custom_dir_path.clone(),
            )
            .expect("add_torrent should succeed for a valid .torrent file");

        let output_folder = engine
            .reveal_path(id)
            .expect("reveal_path should succeed for a known torrent");
        assert_eq!(output_folder, custom_dir_path);
    }
    ```
    (`reveal_path` уже существует с Story 1.5 — `self.api.api_torrent_details(idor)...output_folder`, ничего в нём менять не нужно, он уже отражает то, что было передано в `AddTorrentOptions.output_folder`)
  - [x] `cargo test` — все тесты зелёные (22 существующих Story 1.x–2.2, обновлённые под новую сигнатуру, + 1 новый = 23/23)

- [x] **Task 3: Swift — `AppSettings` (единый источник правды для настроек)** (AC: 1, 2)
  - [x] Новый файл `MyTorrent/AppSettings.swift`:
    ```swift
    import Foundation

    // Story 3.1: Swift owns settings via UserDefaults; they flow one-directionally
    // into the Rust engine as call parameters (AD-5) — never read back out of Rust.
    enum AppSettings {
        private static let saveLocationKey = "settings.saveLocationPath"

        static var saveLocationPath: String {
            get { UserDefaults.standard.string(forKey: saveLocationKey) ?? defaultSaveLocationPath }
            set { UserDefaults.standard.set(newValue, forKey: saveLocationKey) }
        }

        static var defaultSaveLocationPath: String {
            FileManager.default
                .urls(for: .downloadsDirectory, in: .userDomainMask)
                .first?.path
                ?? NSString(string: "~/Downloads").expandingTildeInPath
        }
    }
    ```
    (`defaultSaveLocationPath` — то же вычисление, что раньше было заинлайнено в `AppModel.init()`; вынесено сюда, чтобы не дублировать между `AppModel` и `SettingsView`'s начальным значением NSOpenPanel)

- [x] **Task 4: Swift — окно настроек** (AC: 1, 3)
  - [x] Новый файл `MyTorrent/Views/SettingsView.swift`:
    - `@Environment(\.dismiss) private var dismiss`, `@State private var saveLocationPath: String = AppSettings.saveLocationPath`
    - Хедер: `Text("settings.header.title")` + close-button (`Image(systemName: "xmark")`, `.buttonStyle(.borderless)`, `onClick → dismiss()`) — тот же паттерн, что `TorrentDetailView`'s хедер
    - Секция «Папка для сохранения»: `Text("settings.save_location.label")` (подпись) + `HStack { Text(saveLocationPath) [truncationMode: .middle]; Spacer(); Button("settings.save_location.choose_button") { chooseSaveLocation() } }`
    - `private func chooseSaveLocation()`:
      ```swift
      private func chooseSaveLocation() {
          let panel = NSOpenPanel()
          panel.canChooseDirectories = true
          panel.canChooseFiles = false
          panel.canCreateDirectories = true
          panel.allowsMultipleSelection = false
          panel.directoryURL = URL(fileURLWithPath: saveLocationPath)

          guard panel.runModal() == .OK, let url = panel.url else { return }
          saveLocationPath = url.path
          AppSettings.saveLocationPath = url.path
      }
      ```
      (не sandboxed — AD-7 — `NSOpenPanel` работает без дополнительных entitlements; применяется мгновенно при выборе, без кнопки сохранения — AC1, UX-спека Technical Notes)
    - `import AppKit` (для `NSOpenPanel`, тот же импорт, что уже есть в `MainWindowView.swift`)
    - `.frame(minWidth: 420, minHeight: 160)` на корневой `VStack`
  - [x] `MyTorrent/MyTorrentApp.swift` — новая сцена: `Window("settings.header.title", id: "settings") { SettingsView() }` (singleton-окно, как `Window("MyTorrent", id: "main")` — не `WindowGroup(for:)`, потому что настройки не привязаны к конкретной сущности; заголовок окна ОС берётся из того же локализационного ключа, что и заголовок в контенте — не противоречит паттерну Story 2.1's Scope Boundary про заголовок-не-единственный-источник-истины, т.к. здесь это один и тот же текст, а не дублирование разных источников)
  - [x] `MyTorrent/Views/MainWindowView.swift` — toolbar: убрать `.disabled(true)` у кнопки-шестерёнки, заменить пустое тело `Button { }` на `Button { openWindow(id: "settings") }` (используя уже существующий `@Environment(\.openWindow) private var openWindow`, объявленный в этом файле для `handleOpenURL`), убрать комментарий-заглушку `// Открывает окно настроек (3.1) — реализуется в Story 3.1/3.2.`

- [x] **Task 5: Swift — `AppModel` использует `AppSettings` вместо инлайн-фоллбэка** (AC: 2)
  - [x] `MyTorrent/AppModel.swift`, `init()` — заменить инлайн-вычисление `downloadDir` на `AppSettings.saveLocationPath`:
    ```swift
    init() {
        Task { await setUpEngine(downloadDir: AppSettings.saveLocationPath) }
    }
    ```
    (комментарий `// Hardcoded until Settings exists (Story 3.1)...` удалить — Settings теперь существует)
  - [x] `addTorrent(_ source: TorrentSource)` — читать текущее значение настройки **на MainActor**, до `Task.detached` (тот же принцип, что уже применяется к `source`/`engine` — синхронные значения захватываются до ухода в фон), и передать вторым аргументом:
    ```swift
    func addTorrent(_ source: TorrentSource) async {
        guard let engine else {
            logger.error("addTorrent called before engine finished initializing")
            return
        }
        let downloadDir = AppSettings.saveLocationPath

        let result: Result<String, EngineError> = await Task.detached {
            do {
                return .success(try engine.addTorrent(source: source, downloadDir: downloadDir))
            } catch let error as EngineError {
                return .failure(error)
            } catch {
                return .failure(.Internal(message: "\(error)"))
            }
        }.value
        ...
    }
    ```

- [x] **Task 6: Локализация** (AC: 1, 3)
  - [x] `MyTorrent/Localizable.xcstrings` — 3 новых ключа (тот же JSON-стиль с пробелом перед `:`, что и все существующие):
    - `settings.header.title` — RU "Настройки" / EN "Settings"
    - `settings.save_location.label` — RU "Папка для сохранения" / EN "Save location"
    - `settings.save_location.choose_button` — RU "Выбрать..." / EN "Choose..."

- [x] **Task 7: Генерация биндингов, сборка, ручная проверка** (AC: 1-3)
  - [x] `./scripts/build-engine.sh` — перегенерирует `Generated/engine.swift` с новой сигнатурой `addTorrent(source:downloadDir:)`
  - [x] `xcodegen generate` (обязательно — 2 новых `.swift`-файла, `AppSettings.swift` и `Views/SettingsView.swift`, см. известный XcodeGen-гочу из Story 2.1) + `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] Открыть приложение, кликнуть по шестерёнке в toolbar — **подтверждено скриншотом**: открывается окно «Настройки» (заголовок ОС и заголовок в контенте оба «Настройки»), показывает `/Users/alex/Downloads` — умолчание, соответствует AC3
  - [~] Нажать «Выбрать...» — **частично подтверждено**: кнопка открывает нативный `NSOpenPanel` («Открыть», заскриншочено), доказывая, что обработчик подключён правильно. Довести до конца (выбрать другую папку через сайдбар панели и увидеть, что поле в окне настроек обновилось) не удалось — AppleScript-клики по строкам сайдбара `NSOpenPanel` не попадали в цель в этой среде (сторонний процесс `SingularityApp` перехватывал фокус в моменты клика — не связано с кодом этой истории). Код обработчика (`chooseSaveLocation()`) синхронно присваивает `saveLocationPath` и пишет в `AppSettings` сразу после `panel.runModal() == .OK` — простая, невысокого риска логика; рекомендация пользователю — вручную довести этот шаг до конца при первом реальном использовании
  - [x] Добавить новый торрент после смены папки — **это ядро AC2, подтверждено не вручную, а детерминированным Rust-тестом** `add_torrent_uses_provided_download_dir_as_output_folder` (см. Task 2), который проверяет именно то, что `output_folder` фактически применяется к добавленному торренту — надёжнее ручной проверки в этой среде
  - [x] `cargo test` (engine/) — все тесты зелёные (23/23: 22 существующих Story 1.x–2.2, обновлённых под новую сигнатуру, + 1 новый)

## Dev Notes

### Technical Requirements / Stack

- **`AddTorrentOptions.output_folder: Option<String>`** — подтверждено рабочей компиляцией при создании этой истории. В отличие от `PeerStatsFilter`/`PeerStatsSnapshot` (Story 2.2), этот тип **публично реэкспортирован** из `librqbit` (`pub use session::{AddTorrent, AddTorrentOptions, AddTorrentResponse, ...}` в `librqbit`'s `lib.rs`) — можно именовать напрямую, конструировать литералом `AddTorrentOptions { output_folder: Some(...), ..Default::default() }` (тип деривит `Default`).
- **`AD-5`** («Split state ownership... settings flow one-directionally into Rust as call parameters») — это первая история, которая реально это реализует. `Engine` ничего не хранит и не помнит про настройки между вызовами; каждый `add_torrent` получает актуальное значение папки от Swift-стороны на момент вызова.
- **Существующая сессионная `TempDir` в тестах** (`test_engine()`/`test_engine_no_peer_sources()`) уже удаляется сразу после возврата функции (объект `TempDir` роняется, `Drop` чистит директорию) — и тесты годами проходили, потому что ни один из них не ждёт реальной записи файла на диск в этой sandboxed-среде без сети (см. project memory `my-torrent-dev-environment` — DNS блокирует BitTorrent-домены). Новый параметр `download_dir` в тестах следует тому же допущению — `std::env::temp_dir()` как значение-заглушка, без проверки, что запись туда реально произошла (кроме нового теста Task 2, который проверяет **метаданные** `output_folder`, а не факт записи).

### Architecture Compliance

- Соответствует Consistency Conventions дословно (`add_torrent(source: TorrentSource, ...)` — доп. параметр был предусмотрен изначально).
- **AD-9** (FFI off main thread) — `addTorrent` в `AppModel` уже идёт через `Task.detached`; новый параметр `downloadDir` — обычная `String`, читается синхронно на `@MainActor` до входа в detached-таск, тот же паттерн, что уже применяется к `source`.
- UX-спека `3.1-settings.md`: Technical Notes — «instant-apply... persist immediately... no explicit save/cancel flow» — реализовано через прямую запись в `UserDefaults` при выборе папки, без отдельной кнопки «Сохранить».

### Project Structure Notes

```
engine/
  src/lib.rs                  # UPDATE — add_torrent/add_torrent_blocking/add_magnet получают download_dir,
                               #          14 тестовых вызовов обновлены, 1 новый тест
MyTorrent/
  AppSettings.swift            # NEW — единый источник правды для UserDefaults-настроек
  AppModel.swift                # UPDATE — init()/addTorrent используют AppSettings вместо инлайн-фоллбэка
  MyTorrentApp.swift            # UPDATE — новая Window("settings", id: "settings") сцена
  Views/
    SettingsView.swift          # NEW — окно настроек, секция «Папка для сохранения»
    MainWindowView.swift        # UPDATE — шестерёнка в toolbar включена (openWindow(id: "settings"))
  Localizable.xcstrings         # UPDATE — 3 новых ключа
```

### Previous Story Intelligence (Story 2.2 + вся история проекта)

- **XcodeGen**: добавление новых `.swift`-файлов (`AppSettings.swift`, `Views/SettingsView.swift`) требует `xcodegen generate` перед `xcodebuild`, иначе `cannot find 'X' in scope` — подтверждено в Story 2.1, повторялось в каждой истории с новыми файлами с тех пор.
- **`Localizable.xcstrings`**: писать новые ключи в нативном Xcode JSON-стиле (`"key" : value`, пробел перед `:`) — не голым `json.dump`/иначе раздувает диф (Story 1.5 code-review находка, подтверждена в каждой последующей истории).
- **Полный список из 14 существующих вызовов `add_torrent`, которые сломает смена сигнатуры** (все в `engine/src/lib.rs`, `#[cfg(test)] mod tests`, по состоянию на коммит `64dc253`): `add_torrent_from_file_appears_in_get_all_torrents`, `add_torrent_from_magnet_registers_without_network`, `add_torrent_from_magnet_marks_error_status_on_resolution_failure`, `add_torrent_from_file_reports_progress_and_live_metric_fields`, `pause_torrent_then_resume_torrent_round_trips`, `remove_torrent_removes_from_get_all_torrents`, `remove_torrent_removes_still_pending_magnet`, `get_all_torrents_preserves_insertion_order_for_pending_magnets` (3 вызова в цикле), `get_torrent_details_returns_files_with_sizes`, `get_torrent_details_matches_get_all_torrents_summary_fields`, `get_torrent_details_returns_trackers_from_torrent_file`, `get_torrent_details_returns_empty_peers_when_no_real_peers`, `get_torrent_details_does_not_panic_on_repeated_calls_with_no_peers`, `get_torrent_details_returns_empty_peers_without_error_when_paused`. Не полагаться только на память об этом списке — `cargo build --tests` немедленно укажет на любой пропущенный вызов ошибкой компиляции, это самопроверяющийся риск.
- **Найдено в code-review Story 2.2** (для общего контекста, не относится напрямую к этой истории): `get_torrent_details` теперь парсит id один раз и переиспользует, вместо повторного парсинга — если эта история трогает соседний код в `get_torrent_details`, не возвращать повторный парсинг обратно.
- `AppModel.engine` — `Engine?`, не `let Engine` (Story 1.2 follow-up) — не актуально для новых правок этой истории (`AppSettings` не трогает `engine`), но релевантно для `addTorrent`, который уже правильно это учитывает.

### Testing Standards

- Rust: `cargo test` в `engine/`, паттерн `test_engine()` уже существует. Новый тест на `output_folder` — прямая проверка AC2 на уровне движка, не полагаться только на ручную проверку в UI.
- Swift: формальный UI-test framework вне скоупа v1 (решение Story 1.1) — только ручная проверка (Task 7).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.1]
- [Source: design-artifacts/C-UX-Scenarios/03-alex-sets-defaults-once/3.1-settings/3.1-settings.md] — Section: Save Location Field, Save Location Picker Button, Technical Notes (instant-apply)
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-5, AD-7, AD-9, Consistency Conventions (`add_torrent` parameter shape)]
- [Source: _bmad-output/implementation-artifacts/2-2-trackers-and-peers.md] — предыдущая история: паттерн подтверждения технических решений живой компиляцией (`cargo check` со scratch-кодом) до фиксации в Dev Notes
- librqbit 8.1.1 crate source (`~/.cargo/registry/src/.../librqbit-8.1.1/src/session.rs`) — `AddTorrentOptions { output_folder: Option<String>, ... }` (публично реэкспортирован), `Session::add_torrent(add, opts: Option<AddTorrentOptions>)`

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- `test_download_dir()` изначально возвращал общий `std::env::temp_dir()` — вызвало коллизии между параллельными тестами, добавляющими один и тот же файл фикстуры («test.txt») в одну и ту же директорию (`allow_overwrite = false` в librqbit). Исправлено на уникальную директорию через `tempfile::tempdir().unwrap().keep()` на каждый вызов.
- Ручная проверка выбора папки через `NSOpenPanel`: сайдбар-клики через AppleScript/Accessibility не попадали в цель в этой среде — сторонний процесс `SingularityApp` периодически перехватывал фокус в момент клика (не связано с кодом этой истории). Открытие панели по кнопке подтверждено скриншотом; сам выбор папки и обновление поля — не доведено до конца автоматизацией, но логика обработчика тривиальна и низкого риска.

### Completion Notes List

- Все 7 задач выполнены. AC2 (следующая закачка использует новый путь) — самый важный и рискованный критерий этой истории — подтверждён детерминированным Rust-тестом `add_torrent_uses_provided_download_dir_as_output_folder`, а не только ручной проверкой (которая в этой среде ограничена).
- AC3 (окно настроек открывается, показывает текущее значение) подтверждено скриншотом.
- AC1 (мгновенное применение при выборе папки) подтверждено на уровне кода (синхронное присваивание `@State` + запись в `AppSettings`/`UserDefaults` сразу после `NSOpenPanel.runModal()`) и косвенно — открытие панели по кнопке подтверждено скриншотом — но не доведено до конца интерактивно из-за стороннего процесса, перехватывающего фокус в этой среде.
- **Рекомендация пользователю**: при первом реальном использовании вручную открыть Настройки, выбрать другую папку через «Выбрать...» и убедиться, что путь в поле обновился сразу же, без перезапуска приложения.
- Сигнатура `Engine::add_torrent` изменилась (добавлен `download_dir`) — все 14 существующих вызовов в тестах обновлены, регрессий нет (23/23 тестов зелёные).

### File List

- `engine/src/lib.rs` — UPDATE: `add_torrent`/`add_torrent_blocking`/`add_magnet` принимают `download_dir: String`, строят `AddTorrentOptions { output_folder: Some(download_dir), .. }`; `test_download_dir()` хелпер; все 14 существующих тестовых вызовов обновлены; 1 новый тест
- `MyTorrent/AppSettings.swift` — NEW: единый источник правды для `UserDefaults`-настроек (`saveLocationPath`, `defaultSaveLocationPath`)
- `MyTorrent/AppModel.swift` — UPDATE: `init()`/`addTorrent` используют `AppSettings.saveLocationPath` вместо инлайн-фоллбэка
- `MyTorrent/MyTorrentApp.swift` — UPDATE: новая сцена `Window("settings.header.title", id: "settings")`
- `MyTorrent/Views/SettingsView.swift` — NEW: окно настроек, секция «Папка для сохранения», `NSOpenPanel`
- `MyTorrent/Views/MainWindowView.swift` — UPDATE: шестерёнка в toolbar включена (`openWindow(id: "settings")`, `.disabled(true)` убран)
- `MyTorrent/Generated/engine.swift`, `engineFFI.h`, `module.modulemap` — REGENERATED
- `MyTorrent/Localizable.xcstrings` — UPDATE: 3 новых ключа (`settings.header.title`, `settings.save_location.label`, `settings.save_location.choose_button`)
- `MyTorrent.xcodeproj` — REGENERATED (`xcodegen generate`)
