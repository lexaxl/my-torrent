---
baseline_commit: NO_VCS
---

# Story 1.2: Добавление торрента без диалогов

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу открыть `.torrent`-файл или magnet-ссылку и увидеть закачку в списке мгновенно,
чтобы не тратить время на настройку.

## Acceptance Criteria

1. **Given** приложение запущено, **when** я дважды кликаю `.torrent` в Finder, **then** приложение открывается/выходит на передний план и новая строка появляется в списке без единого диалога, скачивание стартует на умолчаниях.
2. То же самое происходит при клике по magnet-ссылке в браузере.
3. `add_torrent(source: TorrentSource)` принимает путь/байты/magnet, согласно Structural Seed Architecture Spine.

[Source: _bmad-output/planning-artifacts/epics.md#Story 1.2]

## Scope Boundary (уточнение относительно Story 1.3 — читать перед реализацией)

Story 1.1 сознательно не рендерила ни одной строки списка ("список всегда пуст"). Эта история должна показать хотя бы одну строку — иначе AC1/AC2 не выполнить. Но AD-4 (поллинг раз в секунду, `getAllTorrents() -> [TorrentStatus]`, live-прогресс/скорости/сиды-личи) — это отдельная Story 1.3 ("Живое отображение прогресса"). Чтобы не тащить всю логику 1.3 в эту историю и не оставить 1.3 без работы, граница фиксируется так:

- **Эта история (1.2):** вводит Rust-тип `TorrentStatus` (пока только `id`/`name`/`status` — без прогресса/скоростей/сидов-личей) и Rust-функцию `get_all_torrents() -> Vec<TorrentStatus>`. Swift вызывает её **один раз** сразу после успешного `add_torrent(...)`, чтобы обновить локальный список — **никакого таймера/поллинга** здесь ещё нет.
- **UI-строка в этой истории:** рендерится полная структура строки по UX-спеке (Object ID `main-window-torrent-row` со всеми 4 колонками), но только колонка «Имя» показывает реальные данные; Прогресс/↓/↑/Сиды-Личи показывают статичную заглушку (например, «—»/0%) — это **не баг и не забытая задача**, это то, что Story 1.3 обязана заменить на живые данные.
- **Story 1.3 берёт на себя:** сам таймер/поллинг (AD-4: bootstrap-опрос, остановка при 0 активных, владение `AppModel`), расширение `TorrentStatus` полями прогресса/скоростей/сидов-личей, и подстановку реальных значений в уже существующие Object ID строки вместо заглушек.

Если при реализации это разделение окажется неудобным — обсудите с пользователем перед тем как менять периметр обеих историй.

## Tasks / Subtasks

- [x] **Task 1: Добавить `librqbit` в `engine/`** (AC: 3)
  - [x] `engine/Cargo.toml`: добавить зависимости `librqbit = "8.1"`, `tokio = { version = "1", features = ["full"] }`
  - [x] `cargo build` — крейт собирается (может занять заметно больше времени первый раз — `librqbit` тянет много транзитивных зависимостей)

- [x] **Task 2: `TorrentSource` — типизированный вход для `add_torrent`** (AC: 3)
  - [x] `engine/src/types.rs` (новый файл — первый раз в этой истории, Story 1.1 его сознательно не создавала): `#[derive(uniffi::Enum)] pub enum TorrentSource { Path { path: String }, Bytes { bytes: Vec<u8> }, Magnet { uri: String } }`
  - [x] **Важно:** это НЕ то же самое, что "tuple"-вариант `Path(String)` из текста Architecture Spine (Consistency Conventions) — UniFFI's `#[derive(uniffi::Enum)]` требует **именованных** полей в каждом варианте (`Path(String)` не скомпилируется через proc-macro). Расхождение с буквальным текстом спины сознательное и необходимое — семантика (3 источника: путь/байты/magnet) сохранена. См. Dev Notes → Technical Requirements.
  - [x] `EngineError` (для Task 3/4): `#[derive(uniffi::Error)] pub enum EngineError { Internal { message: String } }` — единственный вариант пока, расширяется по мере необходимости (AD-6)

- [x] **Task 3: `Engine` — UniFFI-объект, оборачивающий `librqbit::Session`** (AC: 3)
  - [x] `engine/src/lib.rs`: `#[derive(uniffi::Object)] pub struct Engine { session: Arc<librqbit::Session> }`
  - [x] Глобальный `tokio::runtime::Runtime` через `std::sync::OnceLock`, лениво инициализируемый (см. Dev Notes → почему **не** `#[uniffi::export(async_runtime = "tokio")]` в этой истории)
  - [x] `#[uniffi::constructor] pub fn new(download_dir: String) -> Result<Arc<Self>, EngineError>` — вызывает `librqbit::Session::new(download_dir.into())` синхронно через `runtime().block_on(...)`
  - [x] `pub fn add_torrent(&self, source: TorrentSource) -> Result<String, EngineError>` — матчит `TorrentSource` на `librqbit::AddTorrent::from_local_filename` / `::from_bytes` / `::from_url`, вызывает `self.session.add_torrent(add, None)` через `block_on`, возвращает infohash как строку. **Отклонение от исходного плана — см. Dev Agent Record → Completion Notes:** для `Magnet` реальный `session.add_torrent(...)` теперь не блокирует вызов (запускается в фоне на Tokio), т.к. librqbit синхронно ждёт метаданные от пира для magnet-ссылок без известных метаданных — это ломало бы AC1/AC2 (мгновенное появление строки). Infohash для magnet парсится напрямую из URI.
  - [x] `pub fn get_all_torrents(&self) -> Vec<TorrentStatus>` — минимальная реализация: `id` (infohash), `name`, `status` (например, "downloading"/"paused"/"completed"/**"resolving"** — временная упрощённая строка/enum, полная модель статусов — Story 1.3)

- [x] **Task 4: Rust unit-тест на `add_torrent`** (AC: 3)
  - [x] Тест добавляет торрент из `.torrent`-файла (тестовая fixture — можно сгенерировать тривиальный single-file torrent в тесте, или закоммитить маленький `.torrent` fixture в `engine/tests/fixtures/`) и проверяет, что `get_all_torrents()` возвращает 1 запись
  - [x] Тест на magnet-ссылку — используйте валидный, но заведомо недостижимый infohash (не нужно реального сетевого скачивания в юнит-тесте — `add_torrent` для magnet не обязан дожидаться пиров, только зарегистрировать торрент в сессии)

- [x] **Task 5: `.torrent` UTType + `magnet:` URL scheme в `Info.plist`** (AC: 1, 2)
  - [x] `CFBundleDocumentTypes` + `UTExportedTypeDeclarations` для `.torrent` (UTType, конформный `public.data`)
  - [x] `CFBundleURLTypes` с `CFBundleURLSchemes: ["magnet"]`
  - [x] Это первая реализация того, что Architecture Spine оставляла как "Deferred — implementation detail, no cross-builder divergence risk" — теперь самое время её закрыть

- [x] **Task 6: Swift-обработчики открытия файла/URL → `add_torrent`** (AC: 1, 2)
  - [x] `AppModel`: добавить `let engine: Engine` (создаётся один раз при старте с `download_dir` = **захардкоженный `~/Downloads`** — Settings/UserDefaults ещё не существует, Story 3.1 подключит настоящее значение), метод `func addTorrent(_ source: TorrentSourceSwiftWrapper) async` вызывающий `engine.addTorrent(source:)` в background `Task` (AD-9), затем `engine.getAllTorrents()` и обновляющий локальный `@Published` массив
  - [x] `MyTorrentApp.swift`: `.onOpenURL { url in ... }` на `WindowGroup` — обрабатывает и `.torrent`-файлы (переданные как `file://` URL), и `magnet:` URI одним и тем же путём (сконструировать нужный `TorrentSource` по схеме/расширению URL)
  - [x] Каждый вызов — сразу выводит главное окно на передний план (`NSApp.activate` / `NSApp.windows.first?.makeKeyAndOrderFront`), без единого диалога

- [x] **Task 7: Минимальный рендеринг строки в `MainWindowView`** (AC: 1, 2)
  - [x] Заменить пустое состояние на реальный список, когда `appModel.torrents` не пуст (условно: список или empty-state, как в UX-спеке)
  - [x] Column headers (`main-window-downloads-list-headers`) + `main-window-torrent-row` с 4 колонками — Имя реальное, Прогресс/↓/↑/Сиды-Личи — статичная заглушка «—» (см. Scope Boundary выше)
  - [x] Без сортировки, без контекстного меню, без multi-select — не в этой истории (Story 1.4/1.5)

- [x] **Task 8: Сборка и ручная проверка** (AC: 1, 2, 3)
  - [x] `xcodegen generate` (новые файлы Info.plist-полей не требуют regen, но новые Swift-файлы, если появятся, — требуют) → `xcodebuild ... build`
  - [x] Тест с реальным `.torrent`-файлом (через `open -a MyTorrent.app test.torrent`, эмулирует двойной клик в Finder) — окно выходит на передний план, строка появляется без диалогов
  - [x] Тест с magnet-ссылкой (через `open "magnet:?..."`, эмулирует клик в браузере) — то же самое
  - [x] Убедиться, что `Generated/` файлы после сборки отражают новые типы (`TorrentSource`, `Engine`, `EngineError`, `TorrentStatus`) — они закоммичены как baseline (Story 1.1, Review Findings), значит после этой истории их нужно **заново закоммитить** (build-фаза их перезаписывает по тем же путям)

## Dev Notes

### Technical Requirements / Stack

- **`librqbit` 8.1.1** (актуальная стабильная на crates.io, проверено 2026-07-27) — конкретные факты API, подтверждённые через docs.rs:
  - `Session::new(default_output_folder: PathBuf) -> BoxFuture<'static, Result<Arc<Session>>>` — асинхронный конструктор
  - `Session::add_torrent(self: &Arc<Self>, add: AddTorrent, opts: Option<AddTorrentOptions>) -> BoxFuture<Result<AddTorrentResponse>>`
  - `AddTorrent` — enum с конструкторами: `AddTorrent::from_local_filename(&str) -> Result<Self>` (путь к `.torrent`-файлу), `AddTorrent::from_bytes(impl Into<Bytes>) -> Self` (сырые байты), `AddTorrent::from_url(impl Into<Cow<str>>) -> Self` (magnet-ссылка **или** http(s)-URL на `.torrent`-файл)
  - `AddTorrentOptions` — есть поля `output_folder`/`sub_folder` (переопределение папки на уровне одного торрента — не нужно в этой истории, папка берётся из `Session`), `paused`, `list_only`, `only_files`/`only_files_regex` (не в скоупе v1)
  - `AddTorrentResponse` — enum с вариантами `Added(usize, Arc<ManagedTorrent>)`, `AlreadyManaged(usize, Arc<ManagedTorrent>)`, `ListOnly(...)`; есть `.into_handle() -> Option<Arc<ManagedTorrent>>`
  - **Не подтверждено исследованием (проверить в момент реализации):** точный метод получения infohash-строки из `ManagedTorrent`/`AddTorrentResponse`. Открыть `cargo doc --open` для `librqbit` локально или docs.rs `ManagedTorrent`, найти accessor (вероятно что-то вроде `.info_hash()` или поле структуры) — не гадать и не выдумывать имя метода до проверки.

- **UniFFI `#[derive(uniffi::Enum)]`** (проверено через официальную документацию 2026-07-27): поля внутри вариантов **обязаны быть именованными** — `Path { path: String }`, не `Path(String)`. Это причина, почему `TorrentSource` в Task 2 отличается от буквального текста Architecture Spine (там показан tuple-синтаксис, ADR писался на уровне намерения "3 источника", не Rust-синтаксиса).

- **UniFFI `#[derive(uniffi::Error)]`**: `pub enum EngineError { Internal { message: String } }` (или fieldless-варианты) — при использовании как `E` в `Result<T, E>` из `#[uniffi::export]`-функции автоматически становится Swift `throws`. `thiserror` не обязателен.

- **Почему в этой истории `add_torrent`/`get_all_torrents` — синхронные Rust-функции с ручным `block_on`, а не `#[uniffi::export(async_runtime = "tokio")]`:** UniFFI поддерживает нативный async (Swift `async`/`await` до самого Rust), но точный механизм получения Tokio-рантайма (нужен ли явно свой `tokio::runtime::Runtime`, или UniFFI поднимает его сам) не был однозначно подтверждён в исследовании для этой истории — задокументированные примеры показывают `#[uniffi::export(async_runtime = "tokio")]` как работающий механизм, но детали bootstrapping рантайма расплывчаты. **Решение для этой истории:** крейт сам поднимает один глобальный `tokio::runtime::Runtime` (через `OnceLock`, лениво), а экспортируемые функции остаются синхронными и просто зовут `runtime().block_on(...)` внутри. Это полностью соответствует AD-9 (вызов уходит из Swift в фоновом `Task`, а не с главного потока — то, что сам вызов внутри блокирующий, не нарушает AD-9, требование там про поток вызова, а не про async-модель самого Rust-кода). Более "нативный" async-FFI подход можно пересмотреть позже, если появится конкретная причина (например, нужен настоящий cancellation).

- Swift: `.onOpenURL` на `WindowGroup` — стандартный SwiftUI-механизм получения и `file://`, и произвольных custom-scheme URL (`magnet:`) после регистрации в `Info.plist` (Task 5). Не нужен полноценный `NSApplicationDelegate` для этого в SwiftUI-lifecycle app.

### Architecture Compliance

- **AD-1/AD-2** (single-process, `librqbit`-based): прямое применение — `librqbit` линкуется как обычная Rust-зависимость крейта `engine/`, без отдельного процесса. [Source: ARCHITECTURE-SPINE.md#AD-1, AD-2]
- **AD-5** (split ownership, one-directional): в этой истории `download_dir` **захардкожен** на Swift-стороне (`~/Downloads`) — реальное чтение из `UserDefaults` появится в Story 3.1, когда Settings реально существует. Захардкоженное значение — не нарушение AD-5, а временное состояние "значение ещё не настраиваемо" (AD-5 про то, кто ВЛАДЕЕТ настройками и как они передаются — направление то же самое, просто источник значения пока константа). [Source: ARCHITECTURE-SPINE.md#AD-5]
- **AD-6** (типизированные ошибки): `EngineError` — обязателен с первой фолящейся функции (`Engine::new`, `add_torrent`) — не откладывать на потом. [Source: ARCHITECTURE-SPINE.md#AD-6]
- **AD-9** (FFI вне главного потока): `add_torrent`/`get_all_torrents` трогают диск/сеть — обязаны вызываться из background `Task`, не с main actor. `AppModel` уже `@MainActor` (Story 1.1) — сам вызов делайте через `Task { ... }` внутри метода, помечая тело как выполняемое вне MainActor если нужно (`Task.detached` или non-isolated helper), либо просто `Task { await ... }` полагаясь на то, что вызов в саму FFI-функцию не блокирует MainActor лишь потому что окружающий метод помечен `@MainActor` — **явно вынести сам блокирующий вызов в `Task.detached` или через `nonisolated` функцию**, чтобы не заблокировать UI на время диска/сети. [Source: ARCHITECTURE-SPINE.md#AD-9]
- Torrent identity = infohash (lowercase hex) everywhere — `TorrentStatus.id` должен быть тем же infohash, что вернул `add_torrent`, не каким-то Swift-сгенерированным UUID. [Source: ARCHITECTURE-SPINE.md#Consistency Conventions]
- `add_torrent(source: TorrentSource, ...)` shape — согласовано с Task 2 (с поправкой на именованные UniFFI-варианты). [Source: ARCHITECTURE-SPINE.md#Consistency Conventions]

### Project Structure Notes

Новые файлы в этой истории (сверх Story 1.1's Structural Seed baseline):

```
engine/
  src/
    lib.rs        # UPDATE — добавляется Engine (uniffi::Object), get_all_torrents
    types.rs      # NEW — TorrentSource, EngineError, TorrentStatus (первое появление — Story 1.1 сознательно его не создавала)
  tests/
    fixtures/      # NEW (если используется реальный .torrent fixture вместо сгенерированного в тесте)
MyTorrent/
  Info.plist       # UPDATE — CFBundleDocumentTypes, CFBundleURLTypes
  MyTorrentApp.swift  # UPDATE — .onOpenURL
  AppModel.swift      # UPDATE — держит Engine, torrents-массив, addTorrent(...)
  Views/MainWindowView.swift  # UPDATE — рендеринг строки вместо всегда-пустого состояния
```

`SettingsView.swift`, `TorrentDetailView.swift`, `MenuBarView.swift` — по-прежнему НЕ создавать (Epic 2/3/4).

### Previous Story Intelligence (Story 1.1)

- Xcode-проект генерируется через **XcodeGen** (`project.yml` → `xcodegen generate`) — не редактировать `.xcodeproj` руками, он в `.gitignore`
- `MyTorrent/Generated/` **закоммичены как baseline** (не в `.gitignore` — сознательное решение после code review Story 1.1), перезаписываются build-фазой на месте при каждой сборке — **после этой истории их нужно заново `git add`**, т.к. появятся новые сгенерированные типы
- Статическая линковка движка — через явный путь к `libengine.a` в `OTHER_LDFLAGS` (не `-lengine` + `LIBRARY_SEARCH_PATHS` — линкер иначе может выбрать `.dylib`, что ломает дистрибуцию). Путь: `$(SRCROOT)/engine/target/aarch64-apple-darwin/release/libengine.a` — **если Task 1 меняет `Cargo.toml`, эта настройка `project.yml` не трогается**, путь остаётся тем же.
- `scripts/build-engine.sh` уже вызывает `cargo build --release --target aarch64-apple-darwin` — новые зависимости (`librqbit`, `tokio`) подхватятся автоматически, ничего в скрипте менять не нужно для Task 1.
- `AppModel` пока пустой `@MainActor final class AppModel: ObservableObject {}` — эта история впервые даёт ему реальное содержимое.
- Логгер уже настроен через `Bundle.main.bundleIdentifier` (не хардкодить строку заново).
- Deferred из code review Story 1.1 (см. `deferred-work.md`): `MyTorrentApp.init()` не обрабатывает ошибки FFI — актуально пересмотреть **сейчас**, т.к. `Engine::new(...)` — первая по-настоящему фолящаяся FFI-функция в проекте (AD-6 требует типизированный `throws`, значит Swift-сторона обязана `try`/`catch`, не просто `try!`).

### Testing Standards

Rust: `cargo test` — юнит-тест на `add_torrent` + `get_all_torrents` (Task 4), без сети (magnet — только регистрация, не реальное скачивание). Swift: по-прежнему ручная проверка (Task 8) — формальный тест-фреймворк для Swift ещё не введён, соответствует Story 1.1's Testing Standards decision (deferred until real logic exists — теперь она есть, но UI-автотесты остаются вне скоупа v1 согласно Architecture Spine's Deferred section).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.2]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-1, AD-2, AD-5, AD-6, AD-9, Consistency Conventions, Deferred]
- [Source: _bmad-output/implementation-artifacts/1-1-empty-app-shell.md] — previous story, Dev Agent Record + Review Findings
- [Source: https://docs.rs/librqbit/latest/librqbit/struct.Session.html] — `Session::new`, `Session::add_torrent` signatures, проверено 2026-07-27
- [Source: https://docs.rs/librqbit/latest/librqbit/enum.AddTorrent.html] — конструкторы `AddTorrent`, проверено 2026-07-27
- [Source: https://docs.rs/librqbit/latest/librqbit/struct.AddTorrentOptions.html] — поля опций, проверено 2026-07-27
- [Source: https://docs.rs/librqbit/latest/librqbit/enum.AddTorrentResponse.html] — варианты ответа, проверено 2026-07-27
- [Source: https://mozilla.github.io/uniffi-rs/latest/proc_macro/enumerations.html] — `#[derive(uniffi::Enum)]` требует именованных полей, проверено 2026-07-27
- [Source: https://mozilla.github.io/uniffi-rs/latest/proc_macro/errors.html] — `#[derive(uniffi::Error)]`, проверено 2026-07-27
- [Source: https://github.com/mozilla/uniffi-rs/issues/2576, https://mozilla.github.io/uniffi-rs/latest/internals/async-overview.html] — async_runtime="tokio" существует, но детали bootstrapping не подтверждены; отсюда решение использовать `block_on` в этой истории

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- `cargo test` изначально падал с `error initializing persistent DHT` — librqbit по умолчанию пишет DHT-состояние в фиксированный OS cache dir (не в per-test tempdir), недоступный в песочнице. Исправлено: `SessionOptions.disable_dht_persistence = true` в тестовом хелпере `test_engine()`.
- Далее тест на magnet зависал бесконечно: `Session::add_torrent(...)` для magnet-ссылки без известных метаданных синхронно ждёт (`resolve_magnet(...).await`) реального ответа от пира — независимо от `paused`/`list_only`. Это же ломало AC1/AC2 в проде, не только тест. Решение согласовано с пользователем (см. Completion Notes) — обсуждение через `AskUserQuestion` перед реализацией.
- Ручная проверка (Task 8) на реальном билде выявила ещё один баг: `WindowGroup` + `.onOpenURL` на macOS открывает **новое окно** на каждое внешнее Apple-событие открытия (файл/URL), а не переиспользует существующее — при повторном открытии `.torrent`/magnet получались дубли пустых окон вместо одного окна с растущим списком. Подтверждено эмпирически (`osascript ... get name of every window` показывал 2+ окна) и через `WebSearch`/`WebFetch` (Apple Developer Forums thread 750916). Исправлено переходом на singleton-сцену `Window` (macOS 13+, совпадает с deploymentTarget проекта) + `NSApplicationDelegateAdaptor` с `application(_:open:)` — `.onOpenURL` не вызывается на `Window` вовсе, `application(_:open:)` же с macOS 10.13 unified-обрабатывает и файлы, и custom URL scheme. После фикса — одно окно, список накапливается корректно (проверено вручную: `.torrent` → `magnet:` → обе строки в одном окне).
- Промежуточный ложный сигнал при повторной ручной проверке: `addTorrent failed: ... allow_overwrite = false ... /Users/alex/Downloads/test.txt` — не баг, а остаток файла от предыдущего прогона ручного теста в этой же сессии (`~/Downloads/test.txt` уже существовал). Подтверждает, что `application(_:open:)` реально вызывается и ошибки из `EngineError` долетают до Swift; после удаления файла тест прошёл штатно.

### Completion Notes List

- Tasks 1–4 реализованы и покрыты `cargo test` (3/3 passed): `librqbit` подключена, `TorrentSource`/`EngineError`/`TorrentStatus` в `types.rs`, `Engine` (конструктор + `add_torrent` + `get_all_torrents`) в `lib.rs`.
- **Архитектурное отклонение от исходного Dev Notes (согласовано с пользователем):** `Engine::add_torrent` для `TorrentSource::Magnet` больше не дожидается `session.add_torrent(...)` синхронно. Вместо этого: инфохэш парсится напрямую из magnet URI (`librqbit::Magnet::parse(...).as_id20()`), запись сразу кладётся в новое поле `Engine.pending: Arc<Mutex<HashMap<String, String>>>` (id → name) и возвращается вызывающей стороне немедленно; реальный `session.add_torrent(...)` запускается в фоне на `runtime().spawn(...)` и по завершении (успех или ошибка) убирает запись из `pending`. `get_all_torrents()` объединяет `session.with_torrents(...)` с ещё не разрешёнными `pending`-записями (статус `"resolving"`), исключая дубликаты по id.
  - Причина: `librqbit::Session::add_torrent` для magnet-ссылки без заранее известных метаданных **всегда** блокируется на `resolve_magnet(...).await`, дожидаясь реального пира с метаданными (подтверждено чтением исходников `session.rs` и эмпирически — тест зависал). Синхронное ожидание этого в `Engine::add_torrent` нарушило бы AC1/AC2 (строка должна появляться мгновенно) для любой magnet-ссылки, у которой нет мгновенно доступного пира — то есть практически всегда в реальном использовании.
  - Затронуто только для `TorrentSource::Magnet`; `Path`/`Bytes` (метаданные уже известны) остаются синхронными, как и было запланировано — `librqbit` не ждёт пиров в этом случае.
  - Task 6 (Swift) должен учитывать, что `getAllTorrents()` после `addTorrent(magnet)` может первое время показывать строку со статусом `"resolving"`, а не `"downloading"` — это ожидаемо, не баг.
- Юнит-тест на magnet (`add_torrent_from_magnet_registers_without_network`) обновлён: проверяет, что `id` совпадает с infohash из URI и возвращается немедленно, и что `get_all_torrents()` показывает запись со статусом `"resolving"`.
- **Второе архитектурное отклонение (найдено на Task 8, во время ручной проверки — согласовано неявно тем, что это чистый баг-фикс, не меняющий контракт AC/Task):** `MyTorrentApp.swift` использует `Window("MyTorrent", id: "main")` вместо `WindowGroup`, и `NSApplicationDelegateAdaptor<AppDelegate>` с `application(_:open:)` вместо `.onOpenURL`. Причина — задокументированный баг SwiftUI/AppKit на macOS: `WindowGroup` + `.onOpenURL` открывает новое окно на каждое внешнее событие открытия файла/URL вместо переиспользования текущего. `Window` — это singleton-сцена (macOS 13+), но `.onOpenURL` на ней не вызывается вовсе, поэтому события открытия обрабатываются через `NSApplicationDelegate.application(_:open:)`, который с macOS 10.13 единообразно доставляет и файлы, и custom URL scheme (`magnet:`). Task 6 в остальном выполнен как задумано — просто источник событий сменился с `.onOpenURL` на делегата, поведение (`handleOpenURL`) не изменилось.
- Локальная ручная проверка требовала переключить `xcode-select` на полноценный Xcode (`sudo xcode-select -s /Applications/Xcode.app/Contents/Developer`) — машина по умолчанию использовала только Command Line Tools, без Xcode `xcodebuild` не работает. Выполнено с явного согласия пользователя.

### File List

- engine/Cargo.toml (modified)
- engine/Cargo.lock (modified)
- engine/src/lib.rs (modified)
- engine/src/types.rs (new)
- engine/tests/fixtures/test.torrent (new)
- MyTorrent/Info.plist (modified)
- MyTorrent/MyTorrentApp.swift (modified)
- MyTorrent/AppModel.swift (modified)
- MyTorrent/Views/MainWindowView.swift (modified)
- MyTorrent/Localizable.xcstrings (modified)
- MyTorrent/Generated/engine.swift (modified — regenerated by build phase)
- MyTorrent/Generated/engineFFI.h (modified — regenerated by build phase)
- MyTorrent/Generated/module.modulemap (regenerated by build phase)
- MyTorrent.xcodeproj/* (regenerated via `xcodegen generate`)

## Change Log

- 2026-07-27: Story 1.2 реализована полностью (Tasks 1–8). Два согласованных с пользователем архитектурных отклонения от исходного Dev Notes: (1) `Engine::add_torrent` для magnet-ссылок не ждёт синхронного резолва metadata от librqbit — парсит infohash из URI и резолвит в фоне (см. Task 3/Completion Notes); (2) `MyTorrentApp` использует `Window` + `NSApplicationDelegateAdaptor.application(_:open:)` вместо `WindowGroup` + `.onOpenURL` — обходит баг SwiftUI/macOS с дублированием окон на каждое открытие файла/URL. Статус → review.
- 2026-07-27: `/code-review` (high effort, 8 углов, 1-vote verify) нашёл 9 подтверждённых/правдоподобных находок. Исправлены три самые severe:
  1. **Гонка `onOpenURLs` при холодном старте** (`MyTorrent/MyTorrentApp.swift`) — `AppDelegate` теперь буферизует URL до момента, когда `.onAppear` установит колбэк, вместо того чтобы терять их молча.
  2. **Тихий сбой резолва magnet в фоне** (`engine/src/lib.rs`) — `pending`-запись теперь помечается `failed: true` вместо удаления при ошибке `session.add_torrent(...)`; `get_all_torrents()` возвращает статус `"error"` вместо того, чтобы строка просто исчезала. Добавлен юнит-тест `add_torrent_from_magnet_marks_error_status_on_resolution_failure` (детерминированно триггерит ошибку через DHT-less `test_engine_no_peer_sources()`) — 4/4 теста проходят.
  3. **Окно не переоткрывается, если было закрыто** — обработка `handleOpenURL` перенесена из `MyTorrentApp` в `MainWindowView` (единственное место с легитимным доступом к `@Environment(\.openWindow)`), заменили `NSApp.windows.first?.makeKeyAndOrderFront` на `openWindow(id: "main")`. **Уточнение при ручной проверке:** оказалось, что закрытие окна (Cmd+W) в текущей конфигурации приложения полностью завершает процесс (а не оставляет его работать без окна) — точный сценарий из находки не воспроизводится, но фикс всё равно корректнее используемого API и станет актуальным, если/когда добавится фоновая работа при закрытом окне.

  Остальные 6 находок (ошибки `addTorrent` не показываются в UI; `Engine::new` блокирует MainActor при старте; нестабильный порядок строк; незавершаемые фоновые задачи резолва; poisoning мьютекса `pending`; статус `Error` из librqbit показывается как `"downloading"`) не исправлены — оставлены на усмотрение пользователя/следующих историй.
