---
baseline_commit: NO_VCS
---

# Story 1.1: Пустой каркас приложения

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу открыть приложение и увидеть пустой список закачек,
чтобы убедиться, что оно запустилось корректно.

## Acceptance Criteria

1. **Given** приложение впервые собрано и запущено, **when** нет активных торрентов, **then** главное окно показывает toolbar + сообщение «Нет активных закачек».
2. Rust-движок (крейт `engine/`) собирается и линкуется через UniFFI в SwiftUI-приложение без ошибок — подтверждается реальным вызовом хотя бы одной функции через границу FFI (не только успешной компиляцией).
3. `AppModel` существует как app-lifetime объект (заготовка под Story 1.3 — сам поллинг ещё не реализуется в этой истории).

[Source: _bmad-output/planning-artifacts/epics.md#Story 1.1]

## Tasks / Subtasks

- [x] **Task 1: Инициализировать репозиторий и структуру проекта** (AC: 2)
  - [x] `git init` в корне `my-torrent/`
  - [x] `.gitignore`: `target/`, `DerivedData/`, `*.xcuserstate`, `.build/`, `xcuserdata/`
  - [x] Создать структуру каталогов точно по Structural Seed (см. Dev Notes → Project Structure)

- [x] **Task 2: Rust-крейт `engine/`** (AC: 2)
  - [x] `cargo new --lib engine`
  - [x] `engine/Cargo.toml`: `crate-type = ["lib", "staticlib", "cdylib"]`; зависимость `uniffi = { version = "0.32", features = ["cli"] }`
  - [x] `engine/src/lib.rs`: `uniffi::setup_scaffolding!();` в начале файла + один тривиальный экспортируемый метод для проверки моста — `#[uniffi::export] pub fn engine_version() -> String`
  - [x] `cargo build` — крейт собирается без ошибок
  - [x] Убедиться, что билд идёт под `aarch64-apple-darwin` (Apple Silicon — единственная целевая архитектура, NFR2)

- [x] **Task 3: Сгенерировать Swift-байндинги и подключить к Xcode** (AC: 2)
  - [x] Сгенерировать Swift-биндинги из `engine` через `uniffi-bindgen` (proc-macro `[[bin]]`-таргет `uniffi-bindgen`, `cargo run --bin uniffi-bindgen -- generate --library target/release/libengine.dylib --language swift --out-dir MyTorrent/Generated`)
  - [x] macOS-only, единственная архитектура arm64 — линковка `libengine.a` напрямую через Xcode Build Phase (Run Script + `OTHER_LDFLAGS`/`LIBRARY_SEARCH_PATHS`), без XCFramework/lipo
  - [x] Сгенерированные файлы в `MyTorrent/Generated/` — регенерируются в build-фазе (`scripts/build-engine.sh`), добавлены в `.gitignore`
  - [x] Module map переименован в `module.modulemap` (делает build-скрипт после каждой генерации)

- [x] **Task 4: Xcode-проект `MyTorrent`** (AC: 1, 2)
  - [x] Xcode-проект сгенерирован через **XcodeGen** (`project.yml` → `xcodegen generate`) — решение принято совместно с пользователем, т.к. среда без GUI-мастера Xcode (см. Completion Notes); macOS App, SwiftUI App lifecycle, deployment target macOS 13.0, arm64 only
  - [x] `MyTorrentApp.swift`: `WindowGroup` с главным окном; **`LSUIElement` НЕ включён** — см. Dev Notes → Architecture Compliance (AD-7 вводится в Story 4.1)
  - [x] Подключены сгенерированные Swift-биндинги, `engineVersion()` вызывается в `init()` и логируется через `os.Logger` — подтверждено в системном логе (`engineVersion() = 0.1.0`)

- [x] **Task 5: `AppModel.swift` — заготовка** (AC: 3)
  - [x] `@MainActor final class AppModel: ObservableObject` — без свойств/таймера (поллинг — Story 1.3)
  - [x] Создаётся один раз в `MyTorrentApp` (`@StateObject`), прокидывается через `.environmentObject(appModel)` в `MainWindowView`

- [x] **Task 6: `MainWindowView.swift` — пустое состояние** (AC: 1)
  - [x] Toolbar: заголовок «my-torrent», поле поиска (плейсхолдер «Поиск»/«Search»), кнопка настроек (иконка `gearshape`, no-op) — визуально по UX-DR1
  - [x] Пустое состояние: по центру «Нет активных закачек» (RU) / «No active downloads» (EN)
  - [x] Строки/список/column headers не рендерятся в этой истории
  - [x] Локализация через `Localizable.xcstrings` (String Catalog), ключи `main_window.toolbar.search_placeholder` и `main_window.empty_state.message`, RU (source) + EN

- [x] **Task 7: Сборка и ручная проверка** (AC: 1, 2, 3)
  - [x] Собрано и запущено на Apple Silicon Mac (`xcodebuild ... build` → **BUILD SUCCEEDED**)
  - [x] Окно открывается, показывает toolbar + «Нет активных закачек», без диалогов/ошибок (подтверждено скриншотом)
  - [x] В системном логе подтверждён реальный FFI-вызов: `MyTorrent: (MyTorrent.debug.dylib) [com.alex.mytorrent:startup] engine linked, engineVersion() = 0.1.0`

### Review Findings

_Триаж 3 параллельных ревьюеров (Blind Hunter, Edge Case Hunter, Acceptance Auditor). Severity назначена после чтения кода/бинарника, не только по diff-хуйку — пункты 1-2 подтверждены напрямую через `otool`/`nm` и осмотр сгенерированного `project.pbxproj`._

- [x] [Review][Patch] Rust-движок линкуется динамически по абсолютному пути сборочной машины, не статически — сборка не переживёт перенос/дистрибуцию [project.yml:23, engine/Cargo.toml:7] — исправлено: `OTHER_LDFLAGS` теперь указывает на явный путь к `libengine.a`, подтверждено через `otool -L`/`nm` (символы `T`, никакой зависимости от `libengine.dylib`)
- [x] [Review][Patch] Сгенерированные UniFFI-биндинги не попадают в Xcode-таргет при первой генерации на чистом клоне (Generated/ в .gitignore, XcodeGen сканирует файлы на диске в момент `generate`) [project.yml:16-17, .gitignore:15] — исправлено: три файла `MyTorrent/Generated/` больше не игнорируются, закоммичены как baseline (build-фаза продолжает перезаписывать их на месте)
- [x] [Review][Patch] `cargo build` без `--target aarch64-apple-darwin` — сейчас совпадает с arm64 только по умолчанию хост-машины [scripts/build-engine.sh:14] — исправлено: явный `--target aarch64-apple-darwin` в build-скрипте
- [x] [Review][Patch] `uniffi-bindgen` пересобирается в debug-профиле внутри в остальном release-скрипта — лишняя двойная компиляция [scripts/build-engine.sh:16] — исправлено: добавлен `--release`
- [x] [Review][Patch] Logger subsystem `com.alex.mytorrent` не совпадает с `PRODUCT_BUNDLE_IDENTIFIER` `com.alex.mytorrent.MyTorrent` — фильтрация логов по bundle id не сработает [MyTorrent/MyTorrentApp.swift:4] — исправлено: subsystem теперь читается из `Bundle.main.bundleIdentifier`, устраняя дублирование источника истины, а не просто выравнивая строку
- [x] [Review][Patch] Кнопка настроек без `.disabled` — выглядит как сломанная, а не как нереализованная [MyTorrent/Views/MainWindowView.swift:29-34] — исправлено: добавлен `.disabled(true)`
- [x] [Review][Patch] Текст пустого состояния без явного шрифтового токена — совпадает с `text-md` только случайно (дефолт SwiftUI) [MyTorrent/Views/MainWindowView.swift:33-36] — исправлено: явный `.font(.system(size: 13))`
- [x] [Review][Defer] `MyTorrentApp.init()` вызывает `engineVersion()` без обработки ошибок — некритично для тривиальной инфallible-функции сейчас, пересмотреть в Story 1.2 когда появятся настоящие фолящиеся вызовы (AD-6) [MyTorrent/MyTorrentApp.swift:9] — deferred, low-stakes for a single infallible smoke-test call today
- [x] [Review][Defer] Debug-конфигурация Xcode всегда линкует Release-сборку Rust; Run Script Phase не объявляет `inputFiles` для инкрементальных пересборок [project.yml:23,30-37] — deferred, DX/perf nicety, not a correctness bug at current scale
- [x] [Review][Defer] Нет `rust-toolchain.toml`/pin MSRV [engine/Cargo.toml] — deferred, revisit at Story 4.3 CI setup when reproducibility across machines starts to matter
- [x] [Review][Defer] Нет CI-воркфлоу — deferred, explicitly in scope of Story 4.3 (Сборка и релиз), not this story

**Отклонено как шум/ложные срабатывания (совпадает со спекой или неверно распознано):**
- `ENABLE_HARDENED_RUNTIME: NO` + `CODE_SIGN_IDENTITY: "-"` — соответствует AD-10 (неподписанная дистрибуция) буквально
- `ARCHS: arm64` без Intel — соответствует NFR2 буквально
- Заголовок "my-torrent" в toolbar не через String Catalog — название продукта, не переводимая строка по UX-спеке
- `AppModel` полностью пустой — это и есть требование AC3 этой истории
- Нет entitlements/sandbox — соответствует AD-7 буквально (unsandboxed, unsigned)
- `searchText` ни на что не влияет — явно описано в Dev Notes как ожидаемое для 1.1 (список всегда пуст, реальная фильтрация — Story 1.4)
- Только en/ru локали — соответствует фактическому двуязычному скоупу продукта
- `engine/Cargo.lock` «отсутствует в диффе» (Acceptance Auditor) — ложное срабатывание: файл реально существует и застейджен в git, просто не был включён в текст диффа, переданный ревьюерам (сознательно исключён как generated lock-файл)

## Dev Notes

### Architecture Compliance

- **AD-1** (single-process embedded core): `engine/` линкуется как библиотека прямо в `.app`, никакого демона/сокета. [Source: ARCHITECTURE-SPINE.md#AD-1]
- **AD-3** (SwiftUI only): весь UI на SwiftUI, без AppKit. [Source: ARCHITECTURE-SPINE.md#AD-3]
- **AD-6** (типизированные ошибки): в этой истории функций, которые могут падать, ещё нет (`engine_version()` — тривиальная), но если добавляете что-то фолящееся — сразу `Result<T, EngineError>`, не строки. [Source: ARCHITECTURE-SPINE.md#AD-6]
- **AD-9** (FFI вне главного потока): даже разовый вызов `engine_version()` в Task 4 — тривиален и быстр, поэтому синхронный вызов на старте допустим для этой истории; но как только в Story 1.2/1.3 появятся вызовы, трогающие диск/сеть (`add_torrent`, поллинг) — они обязаны уходить в background `Task`. Заложите `AppModel` как `@MainActor`-класс уже сейчас, чтобы не переделывать в 1.3. [Source: ARCHITECTURE-SPINE.md#AD-9]
- **AD-7 — важное отступление на этой истории:** Architecture Spine предписывает accessory-lifecycle (`LSUIElement`) с самого начала, потому что меню-бар — единственная постоянная точка входа. Но `MenuBarExtra` появляется только в Story 4.1 (Epic 4). Если включить `LSUIElement=true` уже в 1.1, закрытие единственного окна оставит процесс работающим вообще без какого-либо UI (ни Dock-иконки, ни меню-бара) — пользователь не сможет ни открыть, ни закрыть приложение иначе как через Activity Monitor. **Решение для 1.1:** оставить обычный (не accessory) lifecycle с Dock-иконкой, приложение завершается при закрытии окна как обычное macOS-приложение. Переключить на `LSUIElement=true` в Story 4.1, когда появится `MenuBarExtra` — тогда AD-7 будет соблюдена полностью и без дыр в UX. Это не нарушение архитектуры, а последовательность её введения по мере готовности зависимых частей. [Source: ARCHITECTURE-SPINE.md#AD-7; логика — "история должна оставлять систему рабочей end-to-end"]

### Technical Requirements / Stack

- Rust: latest stable toolchain; target `aarch64-apple-darwin` only (Apple Silicon, NFR2 — не Intel).
- `uniffi` crate: **0.32.0** — актуальная стабильная версия на момент написания истории (проверено через docs.rs). Использовать proc-macro подход (`#[uniffi::export]` + `uniffi::setup_scaffolding!()`), НЕ `.udl`-файлы — проще для одного разработчика, `build.rs` не нужен.
- Swift 6 / Xcode latest stable / macOS 13 (Ventura)+ deployment target (нужен для `MenuBarExtra` в будущих историях — выставить сразу, чтобы не менять deployment target позже).
- `librqbit` в эту историю **не входит** — крейт `engine/` в 1.1 полностью пустой (только `engine_version()` для проверки моста). Реальная интеграция с `librqbit` начинается в Story 1.2 (`add_torrent`) и 1.3 (`get_all_torrents`/снапшоты).
- Не создавайте `engine/src/types.rs` в этой истории (в Structural Seed он есть, но домейн-типов — `TorrentStatus`, `EngineError` и т.д. — пока нет, это Story 1.2/1.3).

### Project Structure Notes

Точно по Structural Seed из Architecture Spine — создать в 1.1 только то, что реально нужно для пустого каркаса:

```
my-torrent/
  engine/
    src/
      lib.rs              # uniffi::setup_scaffolding!(); + engine_version()
    Cargo.toml
  MyTorrent/
    MyTorrentApp.swift     # WindowGroup, создаёт AppModel (НЕ accessory lifecycle — см. выше)
    AppModel.swift          # пустой ObservableObject-стаб
    Views/
      MainWindowView.swift  # toolbar + empty state
    Generated/               # build output от uniffi-bindgen, не редактировать руками
  .gitignore
```

`types.rs`, `Models/`, `TorrentDetailView.swift`, `SettingsView.swift`, `MenuBarView.swift`, `.github/workflows/release.yml` — из Structural Seed, но **не эта история** (Epics 2/3/4 и Story 4.3). Не создавайте их заранее пустыми — заводите ровно тогда, когда до них дойдёт своя история.

[Source: ARCHITECTURE-SPINE.md#Structural Seed]

### UX Compliance (полная спека — 1.1-main-window.md)

- Toolbar и список — один визуальный блок без зазора (hairline-разделитель), как в Finder/Mail — токен `space-zero` на границе. [Source: 1.1-main-window.md#Spacing]
- Отступы: `space-md` (toolbar padding, list horizontal), `space-sm` (row vertical — неприменимо в 1.1, строк ещё нет), `space-xl` (empty-state padding). [Source: D-Design-System/00-design-system.md#Spacing Scale]
- Типографика: `text-md` для empty-state сообщения (13px, соответствует `NSFont.systemFontSize`). [Source: 1.1-main-window.md#Typography]
- Точный hover/selection-стиль строк — открытый вопрос, неприменимо в 1.1 (строк нет).
- Не реализуйте контекстное меню строки, поиск-фильтрацию, колонки списка — это отдельные Object ID и отдельные истории (1.2–1.5). В 1.1 они существуют только как визуальные заглушки toolbar (поле поиска, кнопка настроек), без логики.

### Testing Standards

Architecture Spine явно относит стратегию тестирования к "Deferred" (низкий приоритет для соло-хобби-проекта на этом этапе) — формального фреймворка/покрытия не предписано. Для Story 1.1 достаточно ручной проверки по Task 7 (запуск, визуальная проверка empty state, проверка реального FFI-вызова). Автоматизированные тесты (XCTest/`cargo test`) имеет смысл вводить начиная с Story 1.2, когда появится первая настоящая логика (`add_torrent`).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.1] — формулировка story и AC
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-1, AD-3, AD-6, AD-7, AD-9, Stack, Structural Seed]
- [Source: design-artifacts/C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/1.1-main-window.md] — полная спека главного окна
- [Source: design-artifacts/D-Design-System/00-design-system.md#Spacing Scale, Type Scale]
- [Source: https://docs.rs/crate/uniffi/latest/source/Cargo.toml] — актуальная версия uniffi (0.32.0), проверено 2026-07-27
- [Source: https://mozilla.github.io/uniffi-rs/latest/swift/xcode.html] — интеграция UniFFI-биндингов в Xcode (module.modulemap, build phases)

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (claude-sonnet-5)

### Debug Log References

- `cargo test` (engine): 1 passed — `engine_version_returns_non_empty_string` (red→green подтверждён вручную: тест сначала падал на "cannot find function `engine_version`", затем прошёл после добавления `#[uniffi::export]`)
- `cargo build --release` (engine, aarch64-apple-darwin): успешно, `libengine.a` + `libengine.dylib` в `engine/target/release/`
- `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug -arch arm64 build`: **BUILD SUCCEEDED**
- `log show --predicate 'subsystem == "com.alex.mytorrent"' --info`: `2026-07-27 12:33:49 ... MyTorrent: (MyTorrent.debug.dylib) [com.alex.mytorrent:startup] engine linked, engineVersion() = 0.1.0` — подтверждает реальный вызов через границу FFI (AC2)
- Визуальная проверка: скриншот запущенного окна — toolbar «my-torrent» + «Поиск» + шестерёнка + «Нет активных закачек» (AC1)

### Completion Notes List

- **Отклонение от Dev Notes/черновика задач, согласовано с пользователем до реализации:** Xcode-проект создан не вручную через GUI-мастер (среда без него — только Xcode.app CLI + Command Line Tools), а сгенерирован через **XcodeGen** (`project.yml` → `xcodegen generate`, установлен через `brew install xcodegen`). Даёт настоящий `.xcodeproj`, полностью совместимый с обычным открытием в Xcode GUI впоследствии; `project.yml` — источник истины, сам `.xcodeproj` в `.gitignore` (регенерируется командой `xcodegen generate`).
- Потребовалось однократно принять лицензию Xcode (`sudo DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild -license`) — сделано пользователем интерактивно в Terminal.app, т.к. `sudo` требует реального TTY.
- UniFFI-биндинги генерируются `[[bin]]`-таргетом `uniffi-bindgen` внутри самого крейта `engine` (стандартный proc-macro подход, `build.rs` не нужен) — вызывается из `scripts/build-engine.sh`, который также собирает `engine` в release и переименовывает `engineFFI.modulemap` → `module.modulemap`. Скрипт подключён как Run Script Build Phase (`preBuildScripts` в `project.yml`) — компиляция Rust и генерация биндингов происходит автоматически при каждой сборке в Xcode, ничего вручную регенерировать не нужно.
- `SWIFT_VERSION` явно закреплён на `"5"` (Swift 5 language mode) — по рекомендации Architecture Spine (§Deferred: "UniFFI's Swift 6 support is partial ... target Swift 5 language mode if friction appears"), закреплено сразу, а не как реакция на будущие ошибки strict-concurrency.
- `Localizable.xcstrings` создан вручную (не через Xcode GUI) с ключами и RU/EN-значениями строго по UX-спеке `1.1-main-window.md`.
- Оба процесса тестового запуска (`open`/`open -n`) корректно завершены после проверки (`kill`) — в системе не осталось висящих инстансов приложения.
- Файл истории не содержит невыполненных задач; все 3 AC выполнены и подтверждены доказательствами выше (не только предположением "должно работать").

### File List

- `.gitignore` (new)
- `project.yml` (new)
- `scripts/build-engine.sh` (new)
- `engine/Cargo.toml` (new)
- `engine/Cargo.lock` (new)
- `engine/uniffi-bindgen.rs` (new)
- `engine/src/lib.rs` (new)
- `MyTorrent/MyTorrentApp.swift` (new)
- `MyTorrent/AppModel.swift` (new)
- `MyTorrent/Views/MainWindowView.swift` (new)
- `MyTorrent/Info.plist` (new)
- `MyTorrent/Localizable.xcstrings` (new)
- `MyTorrent/Generated/engine.swift`, `engineFFI.h`, `module.modulemap` (new — committed as baseline after code review Patch #2; overwritten in place by the build phase on every build, see `.gitignore` comment)
- `MyTorrent.xcodeproj/` (generated by XcodeGen from `project.yml`, gitignored)

## Change Log

- 2026-07-27 — Story 1.1 реализована полностью (все 7 задач, все 3 AC подтверждены сборкой/логом/скриншотом). Ключевое решение по ходу реализации: Xcode-проект генерируется через XcodeGen вместо ручного GUI-создания (согласовано с пользователем).
- 2026-07-27 — Code review (Blind Hunter + Edge Case Hunter + Acceptance Auditor): 7 patch-находок применены (статическая линковка движка вместо динамической по абсолютному пути; UniFFI-биндинги закоммичены как baseline вместо gitignore, чтобы чистый клон реально собирался; явный `--target aarch64-apple-darwin`; `--release` для uniffi-bindgen; logger subsystem через `Bundle.main.bundleIdentifier`; `.disabled` на кнопке настроек; явный шрифтовый токен на empty-state), 4 отложены в `deferred-work.md`, 8 отклонены как шум/ложные срабатывания. Регрессия (`cargo test`, чистая пересборка Xcode, повторный запуск с проверкой лога) подтверждена после патчей. Статус → `done`.
