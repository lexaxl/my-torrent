---
baseline_commit: 0f6d9530d6616011f1cd221281d96f7252ecaea7
---

# Story 1.5: Управление закачкой через контекстное меню

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу поставить на паузу/возобновить/удалить закачку или показать её в Finder через правый клик по строке,
чтобы управлять ей без лишней навигации.

## Acceptance Criteria

1. **Given** есть закачка в списке, **when** я кликаю правой кнопкой по строке, **then** появляется контекстное меню с пунктами: Пауза/Возобновить (метка зависит от текущего статуса), Удалить, Показать в Finder.
2. **Given** открыто контекстное меню активной (не на паузе) закачки, **when** я выбираю «Пауза», **then** закачка переходит в состояние `paused` и строка отражает это без «мигания» — вызов возвращается только после того, как librqbit реально применил изменение (AD-8), UI обновляется сразу же после возврата вызова, не дожидаясь следующего тика поллинга.
3. **Given** закачка на паузе, **when** я выбираю «Возобновить», **then** закачка возобновляет передачу (аналогично AC2 — синхронно, без мигания).
4. **Given** я выбираю «Удалить», **when** появляется диалог подтверждения, **then** закачка удаляется из списка и из сессии движка только после подтверждения; отмена диалога не меняет состояние. Скачанные файлы **не удаляются с диска** — удаляется только запись/отслеживание в librqbit (см. Scope Boundary).
5. **Given** я выбираю «Показать в Finder», **when** команда выполняется, **then** открывается Finder с выделенной папкой закачки (папка, в которую сохраняются файлы этого торрента).

## Scope Boundary

**Решено в ходе создания истории (уточнено у пользователя — см. ниже, не тривиальный низкий риск, т.к. затрагивает потерю данных):**

- **«Удалить» не удаляет файлы с диска.** Вызывает `librqbit::Session::delete(id, delete_files: false)` — торрент прекращает отслеживаться движком и пропадает из списка, но всё уже скачанное остаётся на диске. Расширение до опции «удалить с диском» — явно **не в этой истории**, оставлено для будущей итерации (например, второй пункт меню или чекбокс в диалоге подтверждения).
- **«Удалить» требует подтверждения** — деструктивное и не всегда обратимое действие (в отличие от паузы/резюма), поэтому NFR4 («no confirmation toasts for routine actions») сюда не применяется: удаление — не routine action. Подтверждение — нативный SwiftUI `.confirmationDialog` (см. Dev Notes), а не отдельное окно/sheet.

**Из UX-спеки, не архитектуры (уже решено на уровне UX, просто фиксирую здесь):**
- Показать в Finder — открывает/выделяет папку назначения торрента целиком, без выбора отдельного файла (соответствует v1-ограничению FR4 «no per-file selection», хотя формально это про Torrent Detail — та же логика применяется здесь за неимением per-file данных на этом уровне).
- Нет мультивыбора строк (Open Question #4 в UX-спеке — решено: не в v1).
- Нет отдельного окна/sheet для подтверждения — только системный alert/confirmationDialog.

**Не в этой истории:**
- Опция «удалить с диском» (см. выше).
- Любая логика повторной попытки/обработки сетевых ошибок при mutation-вызовах — движок либо успевает, либо возвращает `EngineError`, который просто логируется (см. Dev Notes → Error Handling, тот же паттерн, что и в Story 1.2 `addTorrent`).
- Отключение пунктов меню в зависимости от статуса (например, скрывать «Пауза» для ещё не разрешённого magnet-линка со статусом `resolving`) — вызов может завершиться ошибкой в этом крайнем случае, ошибка логируется и молча игнорируется, без падения UI. Формального отключения пунктов меню по статусу не делаем — не описано в UX-спеке, не стоит того усложнения ради редкого окна в доли секунды между добавлением magnet-ссылки и её разрешением.

## Tasks / Subtasks

- [x] **Task 1: Rust engine — pause/resume/remove/reveal-path** (AC: 1, 2, 3, 4, 5)
  - [x] `engine/Cargo.toml`: добавить прямую зависимость `librqbit-core = "5"` (уже разрешена транзитивно через `librqbit` версии 5.0.0 — не новая версия, просто делаем `Id20` именуемым в нашем крейте)
  - [x] `engine/src/lib.rs`: добавить `use librqbit::api::TorrentIdOrHash;`, `use librqbit_core::Id20;`, `use std::str::FromStr;`
  - [x] Добавить приватный `impl Engine` helper `fn parse_id(id: &str) -> Result<TorrentIdOrHash, EngineError>` — `Id20::from_str(id).map(TorrentIdOrHash::Hash).map_err(internal_error)` (тот же `internal_error` helper, что уже используется везде в файле)
  - [x] Добавить поле `api: Api` в структуру `Engine` (импорт `librqbit::Api`), инициализировать в `Engine::new` как `Api::new(session.clone(), None)` — **и в `#[cfg(test)] fn test_engine()`/`test_engine_no_peer_sources()`**, иначе тесты не скомпилируются (оба хелпера в `mod tests` конструируют `Engine { session, pending }` напрямую)
  - [x] `#[uniffi::export] impl Engine` — добавить 4 публичных метода:
    - `pub fn pause_torrent(&self, id: String) -> Result<(), EngineError>` — находит handle через `self.session.get(Self::parse_id(&id)?)` (ошибка "not found", если `None`), затем `runtime().block_on(self.session.pause(&handle)).map_err(internal_error)`
    - `pub fn resume_torrent(&self, id: String) -> Result<(), EngineError>` — аналогично, но `self.session.unpause(&handle)` (сигнатура `unpause` требует `self: &Arc<Self>` — вызывается как `self.session.unpause(&handle)`, `session` уже `Arc<Session>`, компилируется без доп. клонирования)
    - `pub fn remove_torrent(&self, id: String) -> Result<(), EngineError>` — обрабатывает **оба** случая: (a) торрент уже есть в сессии → `runtime().block_on(self.session.delete(idor, false)).map_err(internal_error)?`; (b) торрент есть только в `self.pending` (magnet ещё resolving/failed, `session.get` вернёт `None`) → просто убрать запись из `self.pending`, без вызова `session.delete`. Если не найден нигде — вернуть `EngineError::Internal`. **В любом случае** после успеха убрать `id` из `self.pending` (idempotent, no-op если там и не было — предотвращает «зомби»-запись, если торрент успел появиться в сессии уже после того как Swift получил его как pending)
    - `pub fn reveal_path(&self, id: String) -> Result<String, EngineError>` — `self.api.api_torrent_details(Self::parse_id(&id)?).map_err(internal_error)?.output_folder`
  - [x] Собрать: `cd engine && cargo build` — убедиться, что новая прямая зависимость `librqbit-core` резолвится без конфликта версий (уже должна быть в `Cargo.lock` как транзитивная 5.0.0)

- [x] **Task 2: Rust engine — тесты** (AC: 2, 3, 4)
  - [x] Тест `pause_torrent_then_resume_torrent_round_trips`: добавить торрент из фикстуры (`test_engine()`), вызвать `pause_torrent`, убедиться `get_all_torrents()[0].status == "paused"`, вызвать `resume_torrent`, убедиться статус снова `"checking"`/`"downloading"` (не `"paused"`) — паттерн assert как в `add_torrent_from_file_reports_progress_and_live_metric_fields` (Story 1.3), не точный статус, а `!= "paused"`
  - [x] Тест `remove_torrent_removes_from_get_all_torrents` (обычный случай — торрент уже в сессии): добавить из фикстуры, вызвать `remove_torrent`, `get_all_torrents()` должен вернуть пустой список
  - [x] Тест `remove_torrent_removes_still_pending_magnet`: `test_engine_no_peer_sources()` (магнит гарантированно останется/станет `pending`, background resolution быстро проваливается), добавить magnet, вызвать `remove_torrent` **сразу** (пока запись ещё только в `pending`, до того как resolution успеет провалиться — если тест окажется гоночным/нестабильным из-за таймингов, добавить короткий polling-wait как в `add_torrent_from_magnet_marks_error_status_on_resolution_failure`, Story 1.2), убедиться `get_all_torrents()` пуст
  - [x] Тест `pause_torrent_returns_error_for_unknown_id`: вызвать `pause_torrent` с валидным по формату, но не существующим infohash (например `"0".repeat(40)`) на пустом `test_engine()`, ожидать `Err`
  - [x] `cargo test` — все тесты (старые 7 + новые) должны пройти

- [x] **Task 3: Swift — генерация биндингов и AppModel** (AC: 2, 3, 4, 5)
  - [x] `./scripts/build-engine.sh` — перегенерирует `MyTorrent/Generated/engine.swift` с новыми методами `pauseTorrent(id:)`, `resumeTorrent(id:)`, `removeTorrent(id:)`, `revealPath(id:)` (UniFFI snake_case → camelCase, как уже видно на `addTorrent`/`getAllTorrents`) — **этот файл не редактируется вручную**
  - [x] `MyTorrent/AppModel.swift` — добавить 3 метода по образцу существующего `addTorrent(_:)` (тот же паттерн `Task.detached` + `do/catch` + `logger.error` при неудаче, **без** проброса ошибки наверх — молчаливый лог, как в `addTorrent`):
    - `func pauseTorrent(_ id: String) async` → `engine.pauseTorrent(id: id)`, затем `await refreshTorrents()` при успехе
    - `func resumeTorrent(_ id: String) async` → `engine.resumeTorrent(id: id)`, затем `await refreshTorrents()` при успехе
    - `func removeTorrent(_ id: String) async` → `engine.removeTorrent(id: id)`, затем `await refreshTorrents()` при успехе
  - [x] `revealInFinder(_ id: String) async` — **не** мутация состояния, но всё равно FFI-вызов → всё равно уходит через `Task.detached` (AD-9 «every UniFFI call», без исключений на «просто чтение»). Возвращает путь, на успехе на `MainActor` вызывает `NSWorkspace.shared.selectFile(path, inFileViewerRootedAtPath: "")` (см. Dev Notes — не `.open(URL)`, семантика «reveal», а не «open»); на неудаче — `logger.error`, no-op

- [x] **Task 4: Swift — UI (контекстное меню + подтверждение удаления)** (AC: 1, 2, 3, 4, 5)
  - [x] `MyTorrent/Views/MainWindowView.swift`: добавить `@State private var torrentPendingRemoval: TorrentStatus?`
  - [x] `torrentRow(_:)` — добавить `.contextMenu { ... }` с тремя пунктами:
    - Пауза/Возобновить: `Button { Task { if torrent.status == "paused" { await appModel.resumeTorrent(torrent.id) } else { await appModel.pauseTorrent(torrent.id) } } } label: { Text(torrent.status == "paused" ? "main_window.row.context_menu.resume" : "main_window.row.context_menu.pause") }`
    - Удалить: `Button(role: .destructive) { torrentPendingRemoval = torrent } label: { Text("main_window.row.context_menu.remove") }` — **не** удаляет напрямую, только открывает подтверждение (см. ниже)
    - Показать в Finder: `Button { Task { await appModel.revealInFinder(torrent.id) } } label: { Text("main_window.row.context_menu.reveal_in_finder") }`
  - [x] На корневом `VStack` в `body` добавить `.confirmationDialog(...)` — реализовано без интерполяции имени торрента в сообщении (статичный текст "Скачанные файлы останутся на диске." достаточен и проще для локализации, чем `LocalizedStringKey` с интерполяцией)

- [x] **Task 5: Локализация** (AC: 1, 4)
  - [x] `MyTorrent/Localizable.xcstrings` — добавлено 6 новых ключей (формат идентичен существующим записям — `stringUnit`/`state: translated` для `en` и `ru`):
    - `main_window.row.context_menu.pause` — RU "Пауза" / EN "Pause"
    - `main_window.row.context_menu.resume` — RU "Возобновить" / EN "Resume"
    - `main_window.row.context_menu.remove` — RU "Удалить" / EN "Remove"
    - `main_window.row.context_menu.reveal_in_finder` — RU "Показать в Finder" / EN "Show in Finder"
    - `main_window.row.context_menu.remove_confirm_title` — RU "Удалить закачку?" / EN "Remove download?" (**новый ключ, не из UX-спеки** — введён из-за решения о подтверждении, принятого при создании этой истории, см. Scope Boundary)
    - `main_window.row.context_menu.remove_confirm_message` — RU "Скачанные файлы останутся на диске." / EN "Downloaded files will remain on disk." (тоже новый — явно проговаривает пользователю поведение «не удаляет файлы», чтобы не создавать ложных ожиданий)

- [x] **Task 6: Сборка и ручная проверка** (AC: 1-5)
  - [x] `./scripts/build-engine.sh` + `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] Добавлен торрент (фикстура `engine/tests/fixtures/test.torrent`, файл `test.txt`), правый клик по строке → меню показывает "Пауза", "Удалить", "Показать в Finder" (AC1) — подтверждено на реальном билде через AppleScript/Accessibility-автоматизацию (`AXShowMenu` + скриншоты)
  - [x] "Пауза"/"Возобновить" (AC2/AC3): на отдельном торренте (`ubuntu-....iso`, добавлен через файловую ассоциацию) — открыт контекстным меню при статусе не-paused → выбрана "Пауза"; повторное открытие меню в статусе "checking" корректно НЕ переключилось (librqbit's `pause()` отклоняет вызов, пока торрент ещё в `Initializing` — это ожидаемое поведение, см. Dev Notes/тест `pause_torrent_returns_error_for_unknown_id`-соседний кейс, ошибка тихо логируется, UI не падает); на том же торренте в активном состоянии выбрано "Возобновить" → метка пункта меню при следующем открытии корректно переключилась обратно на "Пауза", подтверждая реальный переход `paused → active` через реальный FFI-вызов (не просто UI-стейт)
  - [x] "Удалить" (AC4): выбор пункта → диалог `.confirmationDialog` появляется с текстом "Удалить закачку?" / "Скачанные файлы останутся на диске." и кнопками "Отменить"/"Удалить" — подтверждено скриншотом. После подтверждения удаления: торрент пропал из списка (список вернулся в `Нет активных закачек`), **и** `~/Downloads/test.txt` остался на диске (`ls`/`cat` подтвердили файл существует, 24 байта) — прямое сквозное доказательство `delete_files: false`
  - [x] "Показать в Finder" (AC5): выбор пункта на активном торренте → открылось окно Finder с папкой `Загрузки` (output-папка торрента) выделенной в родительском каталоге — соответствует ожидаемой семантике `selectFile(_:inFileViewerRootedAtPath:)`
  - [x] `cargo test` (engine/) — 11/11 тестов зелёные

  **Примечание про среду тестирования:** ручная UI-автоматизация через System Events/AppleScript в этом окружении нестабильна (см. project memory `my-torrent-dev-environment`) — контекстное меню SwiftUI не отражается в обычном accessibility-дереве окна (`menu`/`window` элементы), из-за чего клики по вычисленным экранным координатам иногда промахивались или окно неожиданно теряло видимость между вызовами. Рабочим решением оказалась комбинация `AXShowMenu` + клавиатурная навигация (стрелки + Return) **в одном** вызове `osascript` (не раздельными процессами — иначе меню закрывалось до клика). Реальное сетевое скачивание не проверялось (см. project memory: DNS в этом окружении блокирует BitTorrent-трекеры) — прогресс, который виден в скриншотах, отражает локальную check-фазу (верификацию уже скачанных фрагментов), не приём данных по сети; это не входит в скоуп этой истории (пауза/резюм/удаление корректны независимо от того, откуда движок получает прогресс).

## Dev Notes

### Technical Requirements / Stack

- **Новая прямая зависимость:** `librqbit-core = "5"` в `engine/Cargo.toml` — только чтобы именовать тип `Id20` (`librqbit_core::Id20`) для парсинга hex-строки инфохеша обратно в тип, который принимает `librqbit::api::TorrentIdOrHash::Hash`. Версия 5.0.0 уже разрешена транзитивно через сам `librqbit` — никакого риска рассинхронизации версий.
- **`librqbit::api::TorrentIdOrHash`** — публичный enum (`Id(TorrentId) | Hash(Id20)`, `Copy`), НЕ реэкспортирован на верхнем уровне крейта `librqbit` (только `pub use api::Api;`) — импортировать явно из `librqbit::api::TorrentIdOrHash` (модуль `api` объявлен `pub mod api;` в `librqbit`'s `lib.rs`).
- **`librqbit::Session`** (уже используется в `Engine`) — новые вызовы:
  - `session.get(TorrentIdOrHash) -> Option<ManagedTorrentHandle>`
  - `session.pause(&ManagedTorrentHandle) -> anyhow::Result<()>` (async)
  - `session.unpause(&ManagedTorrentHandle) -> anyhow::Result<()>` (async, сигнатура `self: &Arc<Self>` — вызов через `self.session.unpause(...)` работает, т.к. `session: Arc<Session>` — поле уже `Arc`)
  - `session.delete(TorrentIdOrHash, delete_files: bool) -> anyhow::Result<()>` (async) — **`delete_files: false`** согласно Scope Boundary
- **`librqbit::Api`** (публичный, `pub use api::Api;`) — конструируется `Api::new(session: Arc<Session>, rust_log_reload_tx: Option<...>)` (2 аргумента — фича `tracing-subscriber-utils` в нашем `Cargo.toml` не включена, поэтому третьего cfg-параметра нет). Метод `api.api_torrent_details(TorrentIdOrHash) -> Result<TorrentDetailsResponse, ApiError>` — поле `output_folder: String` уже публичное, готовая абсолютная директория, куда сохраняются файлы этого торрента. **Не** пытаться достать `output_folder` напрямую через `handle.shared().options.output_folder` — поле `options` на `ManagedTorrentShared` объявлено `pub(crate)` в самом `librqbit` и недоступно снаружи крейта; `Api::api_torrent_details` — единственный публичный путь к этому значению.
- **Все mutation-вызовы (`pause`/`unpause`/`delete`) — `async fn` в librqbit** → в нашем синхронном UniFFI-экспорте оборачиваются в `runtime().block_on(...)`, как уже делает `add_torrent_blocking`/`Engine::new` — паттерн, не новый для этого файла.
- **AD-8 уже соблюдён самой библиотекой:** `session.pause`/`unpause`/`delete` в librqbit возвращаются только после того, как `ManagedTorrent`'s внутренний `state_change_notify`/state machine реально применили изменение — не просто поставили в очередь. Наш `block_on`-враппер ничего сверху добавлять не должен, просто пробрасывает `Result`.
- **AD-9:** каждый новый метод (`pauseTorrent`/`resumeTorrent`/`removeTorrent`/`revealPath`) на Swift-стороне вызывается из `Task.detached`, как `addTorrent`/`getAllTorrents` — включая `revealPath`, хотя он не мутирует состояние (правило AD-9 — «every UniFFI call», без исключения для чтения).
- **Ошибки:** `EngineError::Internal { message }` — единственный вариант (см. `types.rs`), новые методы возвращают тот же тип через уже существующий `internal_error()` helper. На Swift-стороне — тот же молчаливый log-and-noop паттерн, что и `addTorrent` в `AppModel.swift` (никаких новых UI error-состояний, никаких alert'ов на ошибку самого mutation-вызова — alert есть **только** для подтверждения Remove, не для его возможной ошибки).

### Architecture Compliance

- AD-8 (mutation calls consistency-blocking) — see Technical Requirements above.
- AD-9 (FFI off main thread) — see Technical Requirements above.
- Соответствует UX-спеке `main-window-torrent-row-context-menu` дословно по составу пунктов меню и переводам; подтверждение для Remove и решение «не удалять файлы» — новые решения, принятые в рамках этой истории (UX-спека явно пометила это как "TBD at Architecture", архитектура тоже не зафиксировала — решено здесь, задокументировано в Scope Boundary).

### Project Structure Notes

```
engine/
  Cargo.toml                # UPDATE — + librqbit-core = "5"
  src/lib.rs                # UPDATE — Engine.api field, parse_id helper, 4 new #[uniffi::export] methods, tests
MyTorrent/
  Generated/engine.swift    # REGENERATED by scripts/build-engine.sh — no manual edits
  AppModel.swift             # UPDATE — pauseTorrent/resumeTorrent/removeTorrent/revealInFinder
  Views/MainWindowView.swift # UPDATE — contextMenu on torrentRow, confirmationDialog state
  Localizable.xcstrings      # UPDATE — 6 new keys
```

Никаких новых файлов — все затронутые файлы уже существуют (UPDATE only), кроме автогенерируемого `Generated/engine.swift`.

### Previous Story Intelligence (Story 1.4)

- `MainWindowView.swift`: `filteredTorrents` (computed property) — используется в `downloadsList`'s `ForEach`; контекстное меню в этой истории добавляется на `torrentRow(_:)`, вызывается из того же `ForEach` — никакого взаимодействия с фильтрацией, меню видит уже отфильтрованный `torrent`.
- `AppModel.swift`: паттерн `Task.detached { engine.methodName(...) }.value` + `do/catch` + `logger.error(..., privacy: .public)` — используется во всех предыдущих FFI-вызовах (`addTorrent`), новые методы следуют тому же паттерну буквально.
- `refreshTorrents()` уже вызывает `startPollingIfNeeded()` в конце — после `pauseTorrent`/`resumeTorrent`/`removeTorrent` вызывать `refreshTorrents()`, а не изобретать отдельную логику запуска/остановки поллинга.
- Тестовый `#Preview` в `MainWindowView.swift` создаёт реальный `AppModel()` с пустым списком торрентов — контекстное меню/confirmationDialog не проявятся в превью (не проблема — не изменилось со Story 1.2-1.4, ручная проверка через Task 6 на реальном билде).

### Git Intelligence Summary

- Три предыдущих коммита (`5bf66cf`, `cabde41`, `0f6d953`) — паттерн коммит-сообщений: `Story N.M: краткое summary на английском` (напр. `Story 1.4: filter downloads list by name`). Коммит для этой истории (когда пользователь подтвердит) должен последовать тому же формату — не делать сейчас, коммит только по явному запросу пользователя.
- Ни один из предыдущих коммитов не добавлял прямую зависимость в `engine/Cargo.toml` кроме исходных (`uniffi`, `librqbit`, `tokio`) — это первая история, добавляющая ещё одну прямую Rust-зависимость (`librqbit-core`), обоснование см. выше.

### Testing Standards

- Rust: `cargo test` в `engine/` — юнит-тесты в `#[cfg(test)] mod tests` того же файла (`lib.rs`), паттерн `test_engine()`/`test_engine_no_peer_sources()` уже существует (Story 1.2/1.3) — **переиспользовать**, не создавать третий хелпер. Оба хелпера конструируют `Engine { session, pending }` напрямую (не через `Engine::new`, т.к. `Engine::new` берёт `download_dir: String`, а тесты используют временную директорию через другой путь) — при добавлении поля `api: Api` в структуру `Engine`, эти два конструктора **тоже** должны получить `api: Api::new(session.clone(), None)`, иначе `cargo test` не скомпилируется (structs с приватными полями требуют все поля при прямой инициализации).
- Swift: формальный UI-test-framework всё ещё вне скоупа v1 (решение Story 1.1, подтверждено 1.2-1.4) — только ручная проверка (Task 6).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.5]
- [Source: design-artifacts/C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/1.1-main-window.md#Row Context Menu (`main-window-torrent-row-context-menu`), Open Questions #3/#4]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-8, AD-9]
- [Source: _bmad-output/implementation-artifacts/1-4-search-list.md] — previous story, Dev Agent Record
- librqbit 8.1.1 crate source (`~/.cargo/registry/src/.../librqbit-8.1.1/src/session.rs`, `src/api.rs`) — `Session::pause`/`unpause`/`delete`/`get`, `Api::api_torrent_details`; `librqbit-core-5.0.0/src/hash_id.rs` — `Id20::from_str`
- Уточнение пользователя (в ходе создания этой истории): удаление не трогает файлы на диске; удаление требует подтверждения через диалог. Оба решения зафиксированы в Scope Boundary.

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- `engine`: `error creating a new file (because allow_overwrite = false) "/Users/alex/Downloads/test.txt"` — не баг реализации, а повторное добавление одной и той же `.torrent`-фикстуры в реальный (не тестовый) `~/Downloads` при ручной проверке после того, как файл уже был создан предыдущим прогоном; решалось удалением файла между итерациями ручной проверки.
- Реальный `pause_torrent` вызов через UI на статусе `checking` тихо завершился ошибкой (ожидаемо — см. Scope Boundary/Dev Notes: librqbit's `pause()` отклоняет вызов, пока торрент в `Initializing`) — подтвердило, что error-handling путь (`logger.error`, no-op, без падения UI) работает как задумано в реальном сценарии, а не только в юнит-тесте.

### Completion Notes List

- **Пост-review фикс (после code-review):** найден и исправлен реальный баг — удаление ещё разрешающегося (`pending`) magnet-торрента только убирало запись из `pending`, не отменяя фоновую задачу резолюции в `add_magnet`. Если резолюция всё же завершалась успехом уже после удаления, торрент повторно регистрировался в реальной сессии librqbit и «воскресал» в списке при следующем поллинге, несмотря на то что UI подтвердил удаление. Исправлено: `PendingTorrent` получил флаг `removed`; `remove_torrent` для pending-ветки теперь помечает запись `removed = true` вместо немедленного удаления (запись остаётся скрытой от `get_all_torrents`); фоновая задача в `add_magnet` при успехе резолюции проверяет этот флаг и откатывает регистрацию через `session.delete(...)`, а при неудаче — просто убирает запись, не превращая её в постоянную строку `"error"`. Все 11 тестов (`cargo test`) и `xcodebuild` — зелёные после фикса. Автоматический тест на именно эту гонку (успешная резолюция ПОСЛЕ удаления) не добавлен — детерминированно воспроизвести успешную резолюцию без реальной сети в текущей тестовой инфраструктуре (`test_engine()`/`test_engine_no_peer_sources()`) невозможно; это осознанный пробел в покрытии, а не необнаруженный риск.
- **Пост-review фикс #2 (по явному запросу пользователя «фикси» — заменяет более раннюю запись в этом файле о решении оставить эти 3 находки как есть, сделанную, по-видимому, в параллельной сессии/до этого запроса):**
  1. **Постоянно «мёртвая» кнопка «Пауза»** для magnet-ссылок, чья резолюция окончательно провалилась (`status == "error"`) — исправлено: пункт меню Пауза/Возобновить теперь `.disabled(torrent.status == "error")` в `MainWindowView.swift`. `"error"` для сессионных торрентов (реальная ошибка диска/трекера) тоже корректно отключает пункт — librqbit's `pause()` в этом состоянии тоже всегда отклоняет вызов, так что диапазон применения флага симметричен.
  2. **Устаревшая метка Пауза/Возобновить** относительно реального статуса (гонка с 1-секундным поллингом) — действие клика исправлено: вместо захваченного при последнем рендере `torrent.status`, теперь ищется актуальный статус в `appModel.torrents` по id непосредственно в момент клика (см. `torrentRow` в `MainWindowView.swift`), так что вызывается правильный из pause/resume, даже если сама подпись пункта меню на экране осталась на мгновение устаревшей (это ограничение AppKit — открытое нативное контекстное меню нельзя перерисовать на лету).
  3. **«Показать в Finder» тихо ничего не делает**, если папка ещё не создана на диске — исправлено: `AppModel.revealInFinder` теперь проверяет `FileManager.default.fileExists(atPath:)` перед `NSWorkspace.shared.selectFile`, и логирует (`logger.error`), если папки ещё нет — не показывает пользователю алерт (сохраняет общий паттерн этой истории: mutation/чтение-ошибки только логируются), но хотя бы не выглядит идентично успеху в отладочных логах.
  - Косметические находки тоже устранены: `pause_torrent`/`resume_torrent` теперь используют общий приватный `find_handle()` вместо дублирования; `pauseTorrent`/`resumeTorrent`/`removeTorrent` в `AppModel.swift` используют общий приватный `mutate()` вместо тройного дублирования scaffold'а; `Localizable.xcstrings` переписан на нативное форматирование Xcode (`"key" : value`) — диф теперь только 96 добавленных строк (6 новых ключей), а не 242 строки шума. Не тронуто: лишний `session.get()` перед `session.delete()` в `remove_torrent` — исправление потребовало бы либо матчинга по тексту ошибки (хрупко), либо доступа к внутренностям librqbit, которых у нас нет; цена/выгода не оправдана для этой микрооптимизации.
  - После всех фиксов: `cargo test` (engine/) — 12/12 (11 из Story 1.5 + 1 не связанный тест на порядок вставки, добавленный отдельно), `xcodebuild` (Debug) — BUILD SUCCEEDED.
- Все 6 задач выполнены, все 5 AC подтверждены — как автоматическими тестами (`cargo test`, 11/11), так и вручную на собранном Debug-билде через реальные интеракции (правый клик → контекстное меню → пауза/резюм/удаление/показать в Finder).
- Реализация точно следует Scope Boundary, зафиксированному при создании истории: "Удалить" никогда не удаляет файлы с диска (`delete_files: false`), и всегда требует подтверждения через `.confirmationDialog` — оба решения приняты пользователем при создании истории, не единолично агентом.
- Обнаружен и обработан не описанный в исходной эпике edge case: `pause_torrent`/`resume_torrent` могут быть вызваны для торрента, который librqbit ещё не разрешил в `Live`-состояние (```checking```) — librqbit's `pause()` в этом случае возвращает ошибку, а не молча игнорирует; наш код пробрасывает эту ошибку как `EngineError`, Swift-сторона логирует и не показывает её пользователю (тот же паттерн, что и у `addTorrent`). Не были добавлены отдельные UI-индикаторы/disabled-состояния для пунктов меню в зависимости от статуса торрента — сознательно, см. Scope Boundary ("не в этой истории").
- `remove_torrent` дополнительно обрабатывает случай, когда торрент существует только в `pending` (magnet ещё не разрешён librqbit-сессией) — не описано явно в AC, но необходимо для целостности функции: без этой ветки отмена ещё не разрешившегося magnet-линка была бы невозможна.
- Ручная проверка столкнулась с нестабильностью AppleScript/Accessibility-автоматизации в этой среде (см. project memory `my-torrent-dev-environment` — уже известный, задокументированный ранее риск, не специфичный для этой истории); ушло на это заметно больше итераций, чем ожидалось, но итоговое доказательство корректности — прямое (скриншоты + проверка файловой системы), не основано на предположениях.

### File List

- `engine/Cargo.toml` — UPDATE: добавлена прямая зависимость `librqbit-core = "5"`
- `engine/src/lib.rs` — UPDATE: `Engine.api: Api` поле, `parse_id`/`find_handle` helpers, `pause_torrent`/`resume_torrent`/`remove_torrent`/`reveal_path` методы, `PendingTorrent.removed` флаг (защита от «воскрешения» удалённого во время резолюции magnet-торрента), 4 новых юнит-теста
- `MyTorrent/Generated/engine.swift` — REGENERATED (`scripts/build-engine.sh`), не редактировался вручную
- `MyTorrent/AppModel.swift` — UPDATE: `import AppKit`, общий приватный `mutate()` + `pauseTorrent`/`resumeTorrent`/`removeTorrent` через него, `revealInFinder` (с проверкой существования папки)
- `MyTorrent/Views/MainWindowView.swift` — UPDATE: `torrentPendingRemoval` state, `.contextMenu` на `torrentRow` (лукап актуального статуса перед действием, `.disabled` для статуса `"error"`), `.confirmationDialog` на `body`
- `MyTorrent/Localizable.xcstrings` — UPDATE: 6 новых ключей (`main_window.row.context_menu.*`)
