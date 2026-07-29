---
baseline_commit: ce57ec117dcfd8925bc3139e91e44dc01b27d54e
---

# Story 5.1: Данные о размере, ETA и статусе в строке списка

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу видеть скачанный/общий размер, оставшееся время (ETA) и текстовый статус закачки прямо под её именем в списке,
чтобы понимать прогресс без открытия отдельного окна деталей.

## Acceptance Criteria

1. **Given** закачка в списке со статусом «Скачивается» (`downloading`), **when** отображается строка списка, **then** под именем показывается вторая строка вида «X ГБ / Y ГБ • Осталось N мин/ч • Скачивается».
2. **Given** та же закачка, **when** `down_speed_bps == 0` (временный застой) **or** `total_bytes == 0` (размер ещё неизвестен), **then** соответствующий фрагмент (ETA или размер) не показывается вовсе — не показывается «0» или прочерк вместо него.
3. **Given** закачка завершена (`status == "seeding"`), **then** вторая строка — «Y ГБ • Раздача», без ETA и без «X / Y» (раз завершено, X == Y).
4. **Given** закачка на паузе (`status == "paused"`), **then** вторая строка — «X ГБ / Y ГБ • На паузе», без ETA (неактивна — ETA бессмысленна).
5. **Given** закачка проверяется (`status == "checking"`), **then** вторая строка — «Проверка…», без размера/ETA.
6. **Given** закачка в состоянии ошибки (`status == "error"`), **then** вторая строка — «Ошибка», без размера/ETA.
7. **Given** закачка — magnet-ссылка, ещё не разрешённая (`status == "resolving"`, см. Scope Boundary), **then** вторая строка — «Определение…», без размера/ETA (размер неизвестен до резолва метаданных).
8. **Given** `TorrentStatus`/`TorrentDetail` (Rust/UniFFI), **then** оба содержат новые поля `total_bytes: u64` и `downloaded_bytes: u64`, проброшенные из уже читаемых в `torrent.stats()` значений `stats.total_bytes`/`stats.progress_bytes` — без нового engine-функционала, только проброс существующих значений через границу FFI.
9. **Given** размер закачки, **then** он форматируется через уже существующий `Formatting.size(_:)` — не дублирует `ByteCountFormatter`-логику третий раз.

## Scope Boundary

**Решено при создании истории:**

- **Обнаружен 6-й статус, не покрытый эпиком.** `epics.md`'s Story 5.1 AC изначально перечисляла только 5 статусов (downloading/seeding/paused/checking/error) — реальный движок (`engine/src/lib.rs`, `get_all_torrents`) также возвращает `"resolving"` для pending-магнетов, чей metadata ещё не разрешён (Story 1.2), и `AppModel.activeStatuses` уже учитывает его как активный. Решение, зафиксированное здесь как AC7: «Определение…»/«Resolving…», без размера (`total_bytes` для pending-записи всегда `0`, см. `get_all_torrents`'s `pending_torrents` ветку — метаданные ещё не распарсены) и без ETA. Цвет прогресс-бара для `resolving` — вне scope этой истории (Story 5.2), но пока не задан явно — обычный accent-цвет по умолчанию (тот же путь кода, что `downloading`/`checking`), см. Story 5.2.
- **Композиция подстроки — маленькие переиспользуемые ключи, не один шаблон на статус.** Вместо 5 отдельных полноформатных строк-предложений на язык (что дало бы 10+ ключей и дублирование "•"-паттерна форматирования), вторая строка собирается в Swift из 2-3 маленьких локализованных фрагментов (размер, ETA, статус-слово), склеенных `" • "` — тот же принцип, что уже применяется в `MenuBarView` (отдельные `.down_speed_label`/`.up_speed_label`/`.active_label` ключи + переиспользуемый `Formatting.speed`), а не как в `menu_bar.popover.active_count`, где число обязано жить внутри самого предложения (там альтернативы не было — здесь есть, значит используется более простой паттерн).
- **ETA — не engine-функциональность.** Rust не считает и не возвращает ETA — движок уже даёт `down_speed_bps` и (с этой историей) `total_bytes`/`downloaded_bytes`; ETA = `(total_bytes - downloaded_bytes) / down_speed_bps`, вычисляется полностью в Swift (`Formatting.eta`). Не заводить новый UniFFI-вызов/поле ради этого — так же, как ETA-подобные вычисления (агрегированная скорость в popover) уже делаются на Swift-стороне из уже существующих полей.
- **`DateComponentsFormatter` для ETA, не собственная логика форматирования длительности.** Уже установленный в проекте принцип — переиспользовать системные форматтеры вместо ручной строки (`ByteCountFormatter` для размера/скорости, теперь `DateComponentsFormatter` для длительности) — авто-локализуется под системную локаль без ручной RU/EN-ветки внутри самого форматтера (шаблон-обёртка «Осталось %@»/«%@ left» вокруг него всё равно локализуется через `.xcstrings`, т.к. порядок слов отличается между языками).
- **Цвет прогресс-бара — не в этой истории.** Это Story 5.2 (`epics.md#Story 5.2`) — эта история только добавляет данные под именем, не трогает `.tint(...)` прогресс-бара. Не смешивать оба изменения в одном коммите/PR ради атомарности (тот же принцип, что разделение Story 3.1/3.2 на save-folder и seed-duration).
- **Не в этой истории**: hover-тултип меню-бара (Story 5.3), кастомная иконка (Story 5.4) — отдельные истории того же эпика, не затрагиваются здесь.

## Tasks / Subtasks

- [x] **Task 1: `engine/src/types.rs` — новые поля** (AC: 8)
  - [x] Добавить в `TorrentStatus` (после `peers_connected`, до derive-макроса, точное имя поля — `total_bytes`/`downloaded_bytes`, не `size_bytes`/`progress_bytes` — чтобы не путать с одноимёнными, но иначе устроенными полями `TorrentFile.size_bytes`/`.progress_percent`, у которых `progress_percent` — это `f64`-процент, а не байты):
    ```rust
    pub struct TorrentStatus {
        pub id: String,
        pub name: String,
        pub status: String,
        pub progress_percent: f64,
        pub down_speed_bps: u64,
        pub up_speed_bps: u64,
        pub peers_connected: u32,
        pub total_bytes: u64,
        pub downloaded_bytes: u64,
    }
    ```
  - [x] То же самое в `TorrentDetail` (для единообразия между list-level и detail-level snapshot'ами — TorrentDetailView может в будущем захотеть те же поля, хотя эта история не трогает `TorrentDetailView.swift`)
  - [x] **Не трогать** `summarize_stats`'s сигнатуру/tuple (в `lib.rs`) — оба места её вызова (`get_all_torrents`, `get_torrent_details`) уже держат в скоупе локальную переменную `stats: TorrentStats`, откуda `stats.total_bytes`/`stats.progress_bytes` читаются напрямую при сборке `TorrentStatus`/`TorrentDetail` — раздувать 5-элементный tuple до 7 элементов не нужно

- [x] **Task 2: `engine/src/lib.rs` — проброс полей в 3 местах конструирования** (AC: 8)
  - [x] `get_all_torrents`, ветка реальных session-торрентов (~строка 257) — добавить `total_bytes: stats.total_bytes, downloaded_bytes: stats.progress_bytes` в literal `TorrentStatus { ... }`
  - [x] `get_all_torrents`, ветка pending/resolving-магнетов (~строка 284) — добавить `total_bytes: 0, downloaded_bytes: 0` (метаданные ещё не распарсены, размер объективно неизвестен — см. Scope Boundary AC7)
  - [x] `get_torrent_details` (~строка 456) — добавить `total_bytes: stats.total_bytes, downloaded_bytes: stats.progress_bytes` в literal `Ok(TorrentDetail { ... })`
  - [x] `cargo test` в `engine/` — 25/25 прошли, без изменений в существующих тестах. Также усилен `get_torrent_details_matches_get_all_torrents_summary_fields` тремя новыми ассертами (`total_bytes`/`downloaded_bytes` parity + `total_bytes >= downloaded_bytes`) — тест существовал именно для отлова расхождений между двумя call site, новые поля читаются не через `summarize_stats`, так что стоило явно покрыть
  - [x] После правок Rust — `./scripts/build-engine.sh` пересобирает UniFFI-байндинги; новые поля появятся в `MyTorrent/Generated/engine.swift` автоматически как `totalBytes`/`downloadedBytes`

- [x] **Task 3: `Formatting.swift` — новый `eta(_:)` helper** (AC: 1, 2)
  - [x] Добавить рядом с существующими `speed`/`size`:
    ```swift
    private static let etaFormatter: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day, .hour, .minute]
        formatter.maximumUnitCount = 1
        formatter.unitsStyle = .abbreviated
        return formatter
    }()

    // nil когда ETA не вычислима/не осмысленна (см. вызывающий код AC2): нет скорости,
    // нечего осталось скачивать, или формула даёт интервал < 1 минуты — не показывать
    // "Осталось 0 мин", просто опустить фрагмент целиком.
    static func eta(remainingBytes: UInt64, downSpeedBps: UInt64) -> String? {
        guard downSpeedBps > 0, remainingBytes > 0 else { return nil }
        let seconds = Double(remainingBytes) / Double(downSpeedBps)
        guard seconds >= 60 else { return nil }
        return etaFormatter.string(from: seconds)
    }
    ```
  - [x] `DateComponentsFormatter` авто-локализуется под системную локаль (`.abbreviated` даёт «4 мин»/«4 min» без ручной RU/EN-ветки) — тот же принцип, что уже применяет `ByteCountFormatter` в этом файле

- [x] **Task 4: `MainWindowView.swift` — вторая строка под именем** (AC: 1-7, 9)
  - [x] Новый computed helper (по образцу существующих `filteredTorrents`) для текста подстроки:
    ```swift
    private func subtitle(for torrent: TorrentStatus) -> String {
        switch torrent.status {
        case "downloading":
            var parts = [String]()
            if torrent.totalBytes > 0 {
                parts.append("\(Formatting.size(torrent.downloadedBytes)) / \(Formatting.size(torrent.totalBytes))")
            }
            // Guarded, not a bare subtraction — UInt64 underflow traps in Swift, and
            // downloadedBytes reaching/momentarily exceeding totalBytes right at 100%
            // is exactly the edge this guards against.
            let remaining = torrent.totalBytes > torrent.downloadedBytes
                ? torrent.totalBytes - torrent.downloadedBytes
                : 0
            if let eta = Formatting.eta(remainingBytes: remaining, downSpeedBps: torrent.downSpeedBps) {
                parts.append(String(format: String(localized: "main_window.row.subtitle.eta"), eta))
            }
            parts.append(String(localized: "main_window.row.subtitle.status.downloading"))
            return parts.joined(separator: " • ")
        case "seeding":
            return "\(Formatting.size(torrent.totalBytes)) • \(String(localized: "main_window.row.subtitle.status.seeding"))"
        case "paused":
            let sizePart = torrent.totalBytes > 0
                ? "\(Formatting.size(torrent.downloadedBytes)) / \(Formatting.size(torrent.totalBytes)) • "
                : ""
            return "\(sizePart)\(String(localized: "main_window.row.subtitle.status.paused"))"
        case "checking":
            return String(localized: "main_window.row.subtitle.status.checking")
        case "resolving":
            return String(localized: "main_window.row.subtitle.status.resolving")
        default: // "error" и любой нераспознанный будущий статус — безопасный fallback на "Ошибка"
            return String(localized: "main_window.row.subtitle.status.error")
        }
    }
    ```
    (`default:` покрывает и `"error"`, и любой статус, которого движок ещё не возвращает — избегает `switch` non-exhaustive ошибки на `String` без явного перечисления всех веток; ловит будущий 7-й статус деградацией на «Ошибка» вместо падения/пустой строки)
  - [x] В `torrentRow(_:)` — под `Text(torrent.name)` добавить вторую строку в том же `VStack(alignment: .leading, spacing: 2)`, оборачивающем имя:
    ```swift
    VStack(alignment: .leading, spacing: 2) {
        Text(torrent.name)
            .lineLimit(1)
            .truncationMode(.middle)
        Text(subtitle(for: torrent))
            .font(.caption)
            .foregroundStyle(.secondary)
            .lineLimit(1)
    }
    .frame(width: nameColumnWidth, alignment: .leading)
    ```
    (текущий код — просто `Text(torrent.name).frame(width: nameColumnWidth, ...)` без обёртки `VStack` — обязательно завернуть, не добавлять вторую `Text` как sibling вне колонки, иначе она не выровняется под первой строкой)
  - [x] **Строка теперь двухстрочная** — проверить, не ломает ли это `.padding(.vertical, 12)` визуально (строка станет выше); не гнаться за пиксель-перфектом без реальной сборки, но не оставлять видимого обрезания текста

- [x] **Task 5: Локализация — новые ключи в `Localizable.xcstrings`** (AC: 1, 3-7)
  - [x] `sourceLanguage` файла — `"ru"` (см. существующий файл) — RU-значение обязательно, EN — вторичный `localizations.en`
  - [x] Новые dot-path ключи (простые `stringUnit`, без `variations.plural` — здесь нет счётного числа, в отличие от `menu_bar.popover.active_count`):
    - `main_window.row.subtitle.eta` — RU `"Осталось %@"` / EN `"%@ left"`
    - `main_window.row.subtitle.status.downloading` — RU `"Скачивается"` / EN `"Downloading"`
    - `main_window.row.subtitle.status.seeding` — RU `"Раздача"` / EN `"Seeding"`
    - `main_window.row.subtitle.status.paused` — RU `"На паузе"` / EN `"Paused"`
    - `main_window.row.subtitle.status.checking` — RU `"Проверка…"` / EN `"Checking…"`
    - `main_window.row.subtitle.status.error` — RU `"Ошибка"` / EN `"Error"`
    - `main_window.row.subtitle.status.resolving` — RU `"Определение…"` / EN `"Resolving…"`
  - [x] Формат JSON-записи — как у существующего `main_window.list.header.name` (см. Dev Notes → References для точного примера), не как у `menu_bar.popover.active_count` (это не plural-ключ)

- [x] **Task 6: Сборка и ручная проверка** (AC: 1-9)
  - [x] `cargo test` в `engine/` — 25/25 прошли, без упавших
  - [x] `./scripts/build-engine.sh` → `xcodegen generate` → `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] Ручная проверка — с явного разрешения пользователя: перезапущено ранее запущенное (с 28 июля) debug-приложение, добавлен тестовый `.torrent` (`engine/tests/fixtures/test.torrent`), скриншотами подтверждена вторая строка «0 КБ / 0 КБ • Скачивается» (downloading, ETA скрыт — `down_speed_bps == 0`, реальных пиров нет — DNS блокирует BitTorrent-домены в этом окружении, см. project memory). Seeding/paused/checking/error/resolving — не воспроизведены живьём (нет реальных пиров/трекеров в этом окружении), проверены чтением кода (`subtitle(for:)` — чистый детерминированный `switch` по строке статуса, без side effects). С разрешения пользователя тестовый торрент удалён через контекстное меню (AXShowMenu + подтверждение), список возвращён в пустое состояние — среда пользователя не осталась захламлена тестовыми данными

## Dev Notes

### Technical Requirements / Stack

- **`AD-9`** (FFI calls off main thread) — не затронуто напрямую: эта история не добавляет новых UniFFI-вызовов, только новые поля на уже существующих возвращаемых структурах (`get_all_torrents`/`get_torrent_details`), которые уже вызываются из `Task.detached` в `AppModel`.
- **`AD-6`** (typed errors) — не затронуто: новые поля не fallible, не проходят через `Result`.
- **Consistency Conventions → Naming (Rust ↔ Swift)**: `total_bytes`/`downloaded_bytes` (snake_case в Rust) автоматически становятся `totalBytes`/`downloadedBytes` (camelCase) в `MyTorrent/Generated/engine.swift` после `uniffi-bindgen` — **не** хардкодить/переименовывать вручную.
- **Consistency Conventions → Data & formats**: «Speeds in bytes/sec... The shell formats to `MB/s` etc. for display only — no pre-formatted strings cross the boundary». Эта история следует тому же принципу для размера/ETA: `total_bytes`/`downloaded_bytes` — сырые `u64`, всё форматирование (`Formatting.size`/`Formatting.eta`) — на Swift-стороне.
- **`Formatting.swift`** (`MyTorrent/Formatting.swift`) — существующий файл, shared между `MainWindowView` и `TorrentDetailView` с Story 2.1. `size(_:)` уже готов к переиспользованию как есть; `speed(_:)` — не нужен этой истории напрямую (ETA использует свой отдельный форматтер), но остаётся в том же файле по тому же паттерну.
- **`DateComponentsFormatter`** — стандартный Foundation-класс, доступен на всех целевых macOS-версиях проекта (13+), новых зависимостей не требует.

### Architecture Compliance

- `AD-4` (poll-based sync) — соблюдено: новые поля идут через уже существующий ~1с snapshot-поллинг (`getAllTorrents`), отдельного вызова/таймера не заводится.
- `AD-5` (split state ownership) — не применимо: это read-only отображаемые данные торрента (Rust владеет), не настройки.
- Deferred-раздел Architecture Spine («Row hover visual treatment; menu-bar icon design») **не** покрывает эту историю — та запись про Story 5.3/5.4, не про 5.1/5.2.

### Project Structure Notes

```
engine/
  src/
    types.rs                      # UPDATE — total_bytes/downloaded_bytes на TorrentStatus и TorrentDetail
    lib.rs                        # UPDATE — 3 места конструирования (get_all_torrents×2, get_torrent_details×1)
MyTorrent/
  Formatting.swift                # UPDATE — новый eta(remainingBytes:downSpeedBps:)
  Views/
    MainWindowView.swift          # UPDATE — subtitle(for:) helper, вторая строка в torrentRow
  Localizable.xcstrings           # UPDATE — 7 новых ключей (простые stringUnit, без plural)
  Generated/engine.swift          # regenerated build output — не редактировать руками
```
Никаких новых файлов — `xcodegen generate` нужен по привычке (safety-запуск, см. project memory про XcodeGen-готчу), но строго не обязателен, т.к. новых `.swift`-файлов в проекте не появляется.

### Testing Standards

- Rust: `cargo test` в `engine/` — sanity-check, что 3 места конструирования компилируются и существующие тесты (assert-диапазоны `progress_percent` и т.п.) не сломаны новыми полями. Формального теста конкретно на `total_bytes`/`downloaded_bytes` не требуется (это прямой проброс уже протестированных полей `TorrentStats`, не новая вычисляемая логика) — если хочется покрыть, самый дешёвый вариант — assert `total_bytes >= downloaded_bytes` в уже существующем snapshot-тесте.
- Swift: формального UI-test framework в проекте нет (решение Story 1.1) — только ручная проверка (Task 6). `Formatting.eta` достаточно прост (тонкая обёртка над `DateComponentsFormatter`), чтобы не требовать отдельного unit-теста, но при желании дешёво проверить руками: `nil` при `downSpeedBps == 0`, непустая строка при валидных значениях.

### Previous Story Intelligence (Story 4.3 / Epic 4 code-review passes)

- **`/code-review high` (8-angle parallel + verify) обязателен** перед коммитом — тот же pipeline, что все предыдущие 12 историй.
- **Safety-урок про `osascript`/Accessibility-автоматизацию** (Story 3.2/4.1): реальные клики на физическом экране пользователя — предпочесть скриншоты, спросить перед любой автоматизацией кликов.
- **Централизовать повторяющиеся raw-string литералы** — если статус-строки (`"downloading"`, `"seeding"` и т.д.) в `subtitle(for:)`'s `switch` начинают повторяться в третьем месте (например, если Story 5.2's цветовая логика тоже свитчит по тем же строкам) — это ожидаемо и нормально для *этой* истории (ещё вторая точка использования, не третья), но если появится третье место — вынести в общий `enum`/статические константы, не оставлять третью копию raw-строк (установленный паттерн проекта, см. Epic 4 ретроспектива в project memory).
- **Git**: последний коммит на `main` — `ce57ec1` (Epic 5 добавлен в `epics.md`). Рабочая ветка чистая, готова для новой истории.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 5.1]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-4] — poll-based sync, single snapshot function
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#Consistency Conventions] — naming (snake_case↔camelCase), data & formats (raw values cross FFI, formatting is shell-side)
- [Source: engine/src/types.rs] — существующий `TorrentStatus`/`TorrentDetail` shape
- [Source: engine/src/lib.rs] — `summarize_stats`, `get_all_torrents` (2 construction sites), `get_torrent_details` (1 construction site), все читают `TorrentStats.total_bytes`/`.progress_bytes` уже сегодня для `progress_percent`
- [Source: MyTorrent/Formatting.swift] — существующий `size(_:)`/`speed(_:)`, паттерн для нового `eta(_:)`
- [Source: MyTorrent/Views/MainWindowView.swift] — `torrentRow(_:)`, текущая однострочная структура колонки имени
- [Source: MyTorrent/AppModel.swift] — `activeStatuses` включает `"resolving"`, подтверждает реальность 6-го статуса
- [Source: MyTorrent/Localizable.xcstrings] — существующие ключи `main_window.*`, формат `stringUnit` vs `variations.plural`
- [Source: _bmad-output/implementation-artifacts/4-1-menu-bar-popover.md] — паттерн маленьких переиспользуемых локализованных фрагментов вместо одного шаблона-предложения

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- Пришлось перезапустить уже запущенный (с 28 июля) debug-инстанс `MyTorrent.app`, чтобы подтянуть новый билд — сделано с явного разрешения пользователя (`kill` старого PID, `open` нового `.app` из DerivedData).
- Ручная проверка через Accessibility API: `perform action "AXShowMenu"` на статик-тексте имени торрента открыл то же контекстное SwiftUI `.contextMenu`, что и реальный правый клик — использовано вместо попытки навести курсор и кликнуть правой кнопкой физически. Последующие попытки `click menu item "Удалить" of menu 1 ...` возвращали ошибки резолва референса в AppleScript, но системные логи (`log show`, `perform action for menu item` / `sendAction`) и повторный скриншот подтвердили, что клик по «Удалить» и подтверждение диалога реально прошли — список вернулся в пустое состояние.

### Completion Notes List

- Все 6 задач выполнены, все 9 AC покрыты. Rust: `total_bytes`/`downloaded_bytes` добавлены в `TorrentStatus`/`TorrentDetail`, проброшены в 3 местах конструирования из уже читаемых `stats.total_bytes`/`stats.progress_bytes` — без нового engine-функционала. `cargo test`: 25/25 (существующий тест `get_torrent_details_matches_get_all_torrents_summary_fields` усилен тремя новыми ассертами).
- Swift: новый `Formatting.eta(remainingBytes:downSpeedBps:)` (guard на `downSpeedBps > 0`/`remainingBytes > 0`/`seconds >= 60`, никогда не показывает «0 мин»). Новый `MainWindowView.subtitle(for:)` — детерминированный `switch` по `torrent.status`, вторая строка под именем в `torrentRow(_:)` (обёрнута в `VStack`, не sibling вне колонки).
- **Обнаружен и закрыт пробел эпика**: движок также возвращает статус `"resolving"` (pending-магниты, Story 1.2) — не упомянут в исходном `epics.md`'s Story 5.1 AC. Зафиксирован как AC7 при создании истории и реализован (`"Определение…"`, без размера/ETA).
- **Гарантирована защита от UInt64-underflow**: `remaining = totalBytes > downloadedBytes ? totalBytes - downloadedBytes : 0` вместо голого вычитания — Swift-трап при `downloadedBytes` временно `>= totalBytes` на самой границе 100% иначе уронил бы приложение.
- Локализация: 7 новых dot-path ключей (`main_window.row.subtitle.*`) в `Localizable.xcstrings`, простые `stringUnit` (RU+EN), без `variations.plural` — здесь нет счётного числа.
- Сборка: `cargo test` (25/25) → `./scripts/build-engine.sh` → `xcodegen generate` → `xcodebuild` — BUILD SUCCEEDED.
- **Живая проверка (AC1, AC2)**: реальный тестовый торрент (`engine/tests/fixtures/test.torrent`) в статусе downloading показал вторую строку «0 КБ / 0 КБ • Скачивается» (ETA скрыт, `down_speed_bps == 0` — нет реальных пиров в этом окружении, DNS блокирует BitTorrent-трекеры, см. project memory). **Не проверено живьём** (нет реальных пиров/сети для воспроизведения): seeding (AC3), paused (AC4), checking (AC5), error (AC6), resolving (AC7) — покрыты только чтением кода, логика `subtitle(for:)` — чистая функция без побочных эффектов, риск низкий. С разрешения пользователя тестовый торрент удалён после проверки, среда пользователя не осталась захламлена.
- Изменений в `TorrentDetailView.swift` нет — `TorrentDetail` получил новые поля для единообразия с `TorrentStatus`, но эта история не добавляет их отображение в окне деталей (не требовалось AC).

### Code Review (`/code-review high`)

8 параллельных finder-агентов (line-by-line, removed-behavior, cross-file tracer, reuse, simplification, efficiency, altitude, CLAUDE.md conventions) + 8 отдельных verify-проходов на кандидатов → 5 находок прошли верификацию, все исправлены тем же сеансом:

- **Исправлено (CONFIRMED):** ветка `"seeding"` в `subtitle(for:)` показывала размер без проверки `totalBytes > 0` (в отличие от `"downloading"`/`"paused"`) — торрент с `finished == true` и `total_bytes == 0` (реальное состояние по контракту librqbit, подтверждено чтением `chunk_tracker.rs`/`stats.rs` в `~/.cargo/registry/.../librqbit-8.1.1`) показал бы «Zero KB • Раздача»; добавлен `guard torrent.totalBytes > 0` с тем же fallback-текстом, что у downloading/paused.
- **Исправлено (CONFIRMED):** `total_bytes`/`downloaded_bytes` читались напрямую из `stats` в 3 местах `lib.rs`, минуя `summarize_stats` — функцию, чей doc-комментарий обещает, что общие поля «can't silently drift apart». `summarize_stats` расширена с 5- до 7-элементного tuple (добавлены `stats.total_bytes`/`stats.progress_bytes`), оба call site (`get_all_torrents`, `get_torrent_details`) переведены на чтение через неё — теперь оба поля действительно проходят через единственную anti-drift точку, а не читаются independently.
- **Исправлено (PLAUSIBLE):** `subtitle(for:)` свитчила по сырой строке `torrent.status` с `default`→«Ошибка» — третье независимое место в Swift-коде (после `AppModel.activeStatuses`/`seedingStatus` и `MainWindowView`'s equality-проверок в контекстном меню), хардкодящее словарь статусов движка. Добавлен новый файл `MyTorrent/Models/TorrentEngineStatus.swift` (`enum TorrentEngineStatus: String, Hashable` + `TorrentStatus.engineStatus` computed property, единственный `.error`-fallback через `?? .error`) — используется теперь во всех трёх местах (`AppModel.activeStatuses`/`hasActiveTorrents`/`activeTorrents`/`detectCompletionsAndNotify`, `MainWindowView.subtitle(for:)`, контекстное меню pause/resume/error-disable), закрывая риск из `ARCHITECTURE-SPINE.md`'s Deferred-заметки про будущий статус «Completed-not-seeding».
- **Исправлено (PLAUSIBLE, severity minor):** пара «скачано / всего» и её guard `totalBytes > 0` дублировались почти дословно между ветками `"downloading"` и `"paused"` — вынесены в новый приватный `sizePairFragment(_:) -> String?`, используемый обеими ветками.
- **Исправлено (PLAUSIBLE, severity minor):** `String(localized:)` для ETA-шаблона и всех 6 статус-строк вызывался заново на каждый вызов `subtitle(for:)` (не кэшировался, в отличие от `Formatting`'s `byteCountFormatter`/`etaFormatter`) — вынесены в 7 новых `private static let subtitle*` констант.
- **Отклонено при верификации (REFUTED), фикс не применялся:** подозрение, что ветка `"downloading"` может показать «105 МБ / 100 МБ» (недоскачано > всего) — доказано структурно невозможным: пока `derive_status` возвращает `"downloading"`, `progress_bytes < total_bytes` гарантированно (оба значения — из одного и того же snapshot `TorrentStats`, а `finished` флаг и байтовые счётчики связаны инвариантом `needed_bytes > 0 ⟺ !finished`). Также отклонена гипотеза, что `subtitle(for:)` дублирует `TorrentDetailView`'s summary-строку — они показывают непересекающиеся данные для разных целей (список уже показывает %/скорости/пиры отдельными колонками, детали — нет).

Билд после фиксов: `cargo test` (25/25) → `./scripts/build-engine.sh` → `xcodegen generate` (новый файл `Models/TorrentEngineStatus.swift`) → `xcodebuild` — BUILD SUCCEEDED. Живая повторная проверка на реальном приложении не проводилась для этого раунда фиксов — все пять правок либо чисто внутренний рефакторинг с сохранением поведения (enum, static-кэш, Rust tuple), либо guard на edge case (`seeding`+`totalBytes==0`), не воспроизводимый в этом окружении без реального роя пиров.

### File List

- `engine/src/types.rs` — UPDATE: `total_bytes`/`downloaded_bytes` на `TorrentStatus` и `TorrentDetail`
- `engine/src/lib.rs` — UPDATE: 3 места конструирования (`get_all_torrents` × 2, `get_torrent_details` × 1) + усилен тест `get_torrent_details_matches_get_all_torrents_summary_fields`; code review: `summarize_stats` расширена до 7-элементного tuple, оба call site переведены на неё вместо прямого чтения `stats.total_bytes`/`stats.progress_bytes`
- `MyTorrent/Formatting.swift` — UPDATE: новый `eta(remainingBytes:downSpeedBps:)` + `etaFormatter` (`DateComponentsFormatter`)
- `MyTorrent/Views/MainWindowView.swift` — UPDATE: `subtitle(for:)`, вторая строка в `torrentRow(_:)` (имя обёрнуто в `VStack`); code review: `sizePairFragment(_:)` helper, `seeding`-ветка получила `totalBytes > 0` guard, статус-свитч и context-menu проверки переведены на `torrent.engineStatus`, локализованные строки вынесены в `static let`
- `MyTorrent/Models/TorrentEngineStatus.swift` — NEW (code review): `enum TorrentEngineStatus` + `TorrentStatus.engineStatus`, первый файл в ранее пустой `Models/` директории из Structural Seed архитектуры
- `MyTorrent/AppModel.swift` — UPDATE (code review): `activeStatuses`/`hasActiveTorrents`/`activeTorrents`/`detectCompletionsAndNotify` переведены с raw-string на `TorrentEngineStatus`, убран `seedingStatus`
- `MyTorrent/Localizable.xcstrings` — UPDATE: 7 новых ключей (`main_window.row.subtitle.eta`, `.status.downloading`, `.status.seeding`, `.status.paused`, `.status.checking`, `.status.error`, `.status.resolving`)
- `MyTorrent/Generated/engine.swift` — regenerated build output (`./scripts/build-engine.sh`), не редактировался руками
- `MyTorrent.xcodeproj` — regenerated via `xcodegen generate` (новый файл `Models/TorrentEngineStatus.swift` добавлен в проект)
