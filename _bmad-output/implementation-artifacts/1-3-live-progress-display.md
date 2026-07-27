---
baseline_commit: 5bf66cfe55ace0cad4eda96fadf025b13a542209
---

# Story 1.3: Живое отображение прогресса

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу видеть обновляющиеся вживую прогресс, скорости и число пиров для каждой закачки,
чтобы всегда знать текущий статус с одного взгляда.

## Acceptance Criteria

1. **Given** есть ≥1 активная закачка, **when** прошла ~1 секунда, **then** строка обновляет прогресс-бар, ↓/↑ скорость и счётчик пиров.
2. Поллинг идёт согласно AD-4: bootstrap-опрос при запуске (безусловный, один раз, до старта таймера), остановка таймера при 0 активных закачках, работа независимо от того, открыто ли главное окно.
3. «Активно» = Downloading/Checking/Seeding (раздача не выпадает из отслеживания — таймер не останавливается, пока идёт seeding).

[Source: _bmad-output/planning-artifacts/epics.md#Story 1.3]

## Scope Boundary (уточнение относительно AD-4 и Story 4.1/4.2 — читать перед реализацией)

Эта история сознательно НЕ реализует:

- **Меню-бар (`MenuBarExtra`) и popover** — Story 4.1. AD-7 (accessory lifecycle) касается этой истории только в части «приложение не завершается при закрытии окна» (см. Task 1) — сама иконка в меню-баре, через которую пользователь обычно управляет accessory-приложением, появится только в 4.1. До 4.1 единственный способ вернуть окно на передний план — повторно открыть `.torrent`/magnet-ссылку (уже работает, Story 1.2) или перезапустить из Finder/Spotlight (создаст новый процесс, т.к. предыдущий уже не завершён — просто активирует существующий по стандартному поведению `open`).
- **Системное уведомление о завершении закачки** — Story 4.2. AD-4 описывает механизм обнаружения перехода в Completed/Seeding через сравнение соседних снимков поллинга, но само уведомление (`UNUserNotificationCenter`) — предмет 4.2, которая явно ссылается на этот же механизм поллинга (см. epics.md#Story 4.2: «обнаружение — через сравнение соседних снимков поллинга (AD-4), без отдельного callback-канала»). Эта история **не** строит отдельную инфраструктуру diff'а снимков заранее — `@Published torrents` и так вызывает перерисовку SwiftUI при каждом изменении; 4.2, когда придёт её очередь, добавит сравнение previous/current снимков для детекции перехода в тот же поллинг-цикл, который эта история создаёт.
- **Разбивка «Сиды/Личи» на два числа** — библиотека `librqbit` 8.1.1 не даёт такой информации (см. Dev Notes → Technical Requirements, пункт про `AggregatePeerStats`). Колонка «Сиды/Личи» из UX-спеки в этой истории показывает **общее число подключённых пиров** одним числом вместо ожидаемого спекой формата «12/3» — согласовано с пользователем как осознанное отступление от UX-спеки, а не пропуск.
- **Полноценный AD-7** (весь accessory-lifecycle с меню-баром как единственной точкой Quit) — только его минимальная предпосылка: `LSUIElement`, без которой AC2 («независимо от открытых окон») физически невыполним (проверено эмпирически в code-review Story 1.2 — сейчас закрытие окна завершает процесс). Полная реализация AD-7 — Story 4.1.

Если при реализации какое-то из этих ограничений (особенно Сиды/Личи) окажется неудобным или появится лучшее решение — обсудите с пользователем перед тем как менять периметр.

## Tasks / Subtasks

- [x] **Task 1: `LSUIElement` — минимальная предпосылка AD-4/AC2** (AC: 2)
  - [x] `MyTorrent/Info.plist`: добавить `<key>LSUIElement</key><true/>` — приложение больше не завершается при закрытии последнего окна (no Dock icon как побочный эффект — ожидаемо для accessory-приложения, не баг)
  - [x] Вручную проверить (Task 6), что при `LSUIElement=true` окно всё ещё нормально появляется при обычном запуске и при cold-launch через `open -a`/`.torrent`-файл — по умолчанию AppKit не меняет поведение появления `Window`-сцены, но это не было эмпирически подтверждено для этого проекта, только предположение из документированного поведения `LSUIElement`, — **проверить, не гадать**. **Проверено — и опровергнуто наполовину:** окно появляется нормально и при обычном, и при cold-launch запуске (ожидание подтвердилось). НО `LSUIElement` сам по себе **не остановил завершение процесса** при закрытии окна (ожидание не подтвердилось) — потребовалось добавить `AppDelegate.applicationShouldTerminateAfterLastWindowClosed() -> Bool { false }` (см. Debug Log Reference и `MyTorrent/MyTorrentApp.swift`).

- [x] **Task 2: Расширить `TorrentStatus` — полная модель статуса** (AC: 1, 2, 3)
  - [x] `engine/src/types.rs`: добавить в `#[derive(uniffi::Record)] pub struct TorrentStatus` новые поля: `pub progress_percent: f64` (0.0–100.0), `pub down_speed_bps: u64`, `pub up_speed_bps: u64`, `pub peers_connected: u32` — существующие `id`/`name`/`status` не трогать (обратная совместимость с Story 1.2's тестами)
  - [x] `engine/src/lib.rs`, `get_all_torrents()`: заменить нынешний упрощённый `if/else` (paused/finished/else) на полное сопоставление с `librqbit::TorrentStats` (см. Dev Notes → Technical Requirements для точных полей `TorrentStats`/`LiveStats`/`AggregatePeerStats`, подтверждённых чтением исходников `librqbit` 8.1.1, не докс — библиотека закреплена в `Cargo.lock`):
    - `state: TorrentStatsState::Initializing` → `status = "checking"` (термин из Architecture Spine Consistency Conventions, не буквальное имя варианта librqbit — см. Dev Notes)
    - `state: Live`, `finished: false` → `status = "downloading"`
    - `state: Live`, `finished: true` → `status = "seeding"`
    - `state: Paused` → `status = "paused"` (как было)
    - `state: Error` → `status = "error"` (как было для pending-magnet-веток — теперь то же значение и для обычных торрентов)
    - `progress_percent` = `if total_bytes == 0 { 0.0 } else { progress_bytes as f64 / total_bytes as f64 * 100.0 }` (не использовать `TorrentStats::progress_percent_human_readable()` — она возвращает форматированную строку, а через границу FFI должно идти число, см. Architecture Spine Consistency Conventions: «no pre-formatted strings cross the boundary»)
    - `down_speed_bps`/`up_speed_bps` = из `stats.live.as_ref().map(|l| (l.download_speed.mbps * 1024.0 * 1024.0).round() as u64).unwrap_or(0)` (и симметрично для upload) — **важно:** `librqbit::Speed.mbps` это MiB/с как `f64`, не байты/с как целое — конвертация обязательна, легко забыть или перепутать MiB/MB
    - `peers_connected` = `stats.live.as_ref().map(|l| l.snapshot.peer_stats.live as u32).unwrap_or(0)`
  - [x] Записи из `pending` (магнит ещё не резолвится/уже упал — Story 1.2) продолжают возвращать `status: "resolving"` или `"error"` как раньше, с новыми числовыми полями = `0`/`0.0` (нет `TorrentStats` пока запись не переехала в `session.with_torrents`)

- [x] **Task 3: Rust unit-тест на маппинг `TorrentStatus`** (AC: 1, 3)
  - [x] Тест добавляет торрент из `.torrent`-фикстуры (как в Story 1.2), сразу после добавления проверяет: `status` ∈ {"checking","downloading"} (без реальной сети недетерминировано, какое именно — не жёстко фиксировать одно значение), `progress_percent` в диапазоне `[0.0, 100.0]`, `down_speed_bps`/`up_speed_bps`/`peers_connected` — присутствуют и не паникуют при вычислении (реальных пиров в тесте нет, ожидаемо `0`)
  - [x] Отдельный тест на конвертацию скорости: не обязательно гонять реальную сеть — можно юнит-тестом на чистой функции, если вынести формулу `mbps_to_bytes_per_sec(f64) -> u64` в отдельную `fn` (лёгкая для тестирования, без завязки на `TorrentStats`)

- [x] **Task 4: `AppModel` — bootstrap-снимок + поллинг-цикл (AD-4)** (AC: 1, 2, 3)
  - [x] Новое приватное свойство `private var pollingTask: Task<Void, Never>?`
  - [x] Новый приватный статический набор `private static let activeStatuses: Set<String> = ["checking", "downloading", "seeding", "resolving"]` — **обратите внимание:** `"resolving"` не входит в Architecture Spine Consistency Conventions (там нет магнитов на момент написания спины), но логически обязан считаться активным — иначе поллинг не поймает переход resolving → checking/error и появление живых данных для только что добавленного magnet-торрента задержится непредсказуемо
  - [x] `init()`: после создания `engine`, запустить `Task { await bootstrap() }` — **безусловный** первый снимок (`getAllTorrents()`) независимо от того, есть ли уже что-то активное (AD-4 bootstrap exemption — ловит торренты, которые `librqbit` сам восстановил из resume-state при старте `Session::new`)
  - [x] `private func bootstrap() async { await refreshTorrents() }` — `refreshTorrents()` уже существует (Story 1.2), при необходимости расширить: после каждого обновления `torrents` вызывать `startPollingIfNeeded()`
  - [x] `private func startPollingIfNeeded()`: если `pollingTask == nil` и среди `torrents` есть хотя бы один со статусом из `activeStatuses` → запустить `pollingTask = Task { await pollLoop() }`
  - [x] `private func pollLoop() async { while !Task.isCancelled { try? await Task.sleep(for: .seconds(1)); await refreshTorrents(); if !torrents.contains(where: { Self.activeStatuses.contains($0.status) }) { break } }; pollingTask = nil }` — цикл сам себя останавливает при 0 активных (AC2/AC3), `pollingTask` — свойство `AppModel` (app-lifetime `@StateObject`), не View — переживает закрытие всех окон (AD-4 Ownership + Task 1's `LSUIElement`)
  - [x] `addTorrent(_:)` уже вызывает `refreshTorrents()` при успехе (Story 1.2) — через расширение `refreshTorrents()` из предыдущего пункта это автоматически перезапустит поллинг, если он был остановлен

- [x] **Task 5: `MainWindowView` — реальные данные вместо статичных «—»** (AC: 1)
  - [x] Колонка «Прогресс»: `ProgressView(value: torrent.progressPercent / 100)` (или эквивалент) вместо `Text("—")`
  - [x] Колонки ↓/↑: форматированная строка скорости (например, через `ByteCountFormatter` или ручное форматирование KB/MB/с) из `downSpeedBps`/`upSpeedBps`
  - [x] Колонка «Сиды/Личи»: показывает `peersConnected` одним числом (см. Scope Boundary — сознательное отступление от формата «12/3» из UX-спеки, т.к. `librqbit` не даёt разбивку)
  - [x] Без сортировки, без контекстного меню — по-прежнему не в этой истории (Story 1.4/1.5)

- [x] **Task 6: Сборка и ручная проверка** (AC: 1, 2, 3)
  - [x] `xcodegen generate` (новых Swift-файлов нет, но новые поля `TorrentStatus` требуют regen `Generated/` — build-фаза сама перегенерирует) → `xcodebuild ... build`
  - [x] Добавить реальный небольшой публичный торрент и понаблюдать: добавлен официальный `.torrent` Ubuntu 24.04.4 Server (`releases.ubuntu.com`) — строка появилась с прогресс-баром/скоростями/числом пиров сразу. Поллинг подтверждён **логами** (временный debug-лог, удалён после проверки): тики ровно раз в секунду на протяжении >2 минут, статус стабильно `"downloading"`, цикл не останавливается, пока активно. **Сеть в этом окружении блокирует BitTorrent-домены на уровне DNS** (`torrent.ubuntu.com`/`router.bittorrent.com` не резолвятся ни системным резолвером, ни напрямую через 1.1.1.1 — подтверждено `dig`), поэтому реальные пиры/скорость > 0 в этой среде не воспроизвести; сам механизм поллинга/маппинга статуса проверен полностью и независимо от этого сетевого ограничения
  - [x] Закрыть окно во время активной закачки (клик по красной кнопке через UI-скриптинг, не просто keystroke — надёжнее), подождать, снова открыть `.torrent` — процесс **не завершался** (`ps aux` подтверждает) после фикса Task 1, окно вернулось через `openWindow(id:)" с сохранённым состоянием торрента — полная проверка AC2 успешна
  - [x] Дождаться завершения закачки (или использовать уже полностью скачанный тестовый торрент) — убедиться, что статус переходит в `"seeding"` и поллинг **не** останавливается (AC3) — **e2e-версия недостижима** в этой сети (реальное завершение торрента требует реальных пиров — см. выше). Вместо этого статус-маппинг (`get_all_torrents`'s `match`) вынесен в чистую функцию `derive_status(state, finished) -> &str` и покрыт юнит-тестом на все 5 веток, включая `Live && finished → "seeding"` (недостижимую e2e в этом окружении) — см. Task 3 addendum ниже
  - [x] Убедиться, что поллинг останавливается, когда активных закачек нет: чистый запуск с 0 торрентов — CPU процесса стабильно 0.0% на протяжении 8 секунд (4 замера `ps -o %cpu`), что соответствует отсутствию активного `pollingTask` (bootstrap нашёл 0 активных → цикл не стартовал)
  - [x] `Generated/` файлы после сборки отражают новые поля `TorrentStatus` (`progressPercent`, `downSpeedBps`, `upSpeedBps`, `peersConnected`) — будут закоммичены (как в Story 1.2)

## Dev Notes

### Technical Requirements / Stack

- **`librqbit::TorrentStats`** (подтверждено чтением исходников `librqbit` 8.1.1 в `~/.cargo/registry/.../librqbit-8.1.1/src/torrent_state/stats.rs`, не докс — на момент этой истории докс.rs не показывал часть полей):
  ```rust
  pub struct TorrentStats {
      pub state: TorrentStatsState,      // Initializing | Live | Paused | Error
      pub file_progress: Vec<u64>,
      pub error: Option<String>,
      pub progress_bytes: u64,
      pub uploaded_bytes: u64,
      pub total_bytes: u64,
      pub finished: bool,
      pub live: Option<LiveStats>,       // None если state != Live
  }
  pub struct LiveStats {
      pub snapshot: StatsSnapshot,        // содержит peer_stats: AggregatePeerStats
      pub download_speed: Speed,          // Speed { mbps: f64 } — MiB/с, не байты/с!
      pub upload_speed: Speed,
      // ...
  }
  pub struct AggregatePeerStats {
      pub queued: usize,
      pub connecting: usize,
      pub live: usize,        // это то самое «число подключённых пиров» для Task 2/5
      pub seen: usize,
      pub dead: usize,
      pub not_needed: usize,
      pub steals: usize,
      // НЕТ разбивки на seed/leech — проверено также в PeerStats/PeerCounters
      // (per-peer, `TorrentStateLive::per_peer_stats_snapshot`) — ни один публичный
      // тип библиотеки не содержит флага «этот пир — сид». Разбивка недостижима
      // без форка/патча librqbit — вне скоупа этой истории (см. Scope Boundary).
  }
  ```
- **`TorrentStatsState` имеет только 4 варианта** (`Initializing`/`Live`/`Paused`/`Error`) — нет отдельных `Downloading`/`Seeding`/`Checking`. Различение downloading/seeding — через `finished: bool` при `state == Live` (см. Task 2). Название `"checking"` для `Initializing` — сознательный выбор в пользу терминологии Architecture Spine Consistency Conventions (`active = {Downloading, Checking, Seeding}`), а не буквального имени варианта библиотеки — если это решение неудобно при реализации, можно использовать `"initializing"` вместо `"checking"`, но тогда стоит обновить и Consistency Conventions таблицу для согласованности.
- **Конвертация скорости** — `librqbit::Speed.mbps: f64` это мебибайты/с (MiB/s), а Architecture Spine Consistency Conventions требует «Speeds in bytes/sec (integer) at the FFI boundary». Конвертация: `bytes_per_sec = (mbps * 1024.0 * 1024.0).round() as u64`. Округление, не усечение — иначе систематическая недооценка скорости на дробных Mbps.
- **UniFFI numeric types**: `f64`/`u64`/`u32` в `#[derive(uniffi::Record)]` — все нативно поддерживаются, никаких дополнительных конвертеров не требуется (как и `String`/`bool` в Story 1.2).

### Architecture Compliance

- **AD-4** (pull-based sync, ~1s, только пока активно) — ядро этой истории. Bootstrap exemption (Task 4), stop-at-zero-active (Task 4's `pollLoop`), ownership на `AppModel`, а не View (Task 4). Completion-detection-через-diff **не** реализуется в этой истории (см. Scope Boundary) — только сам поллинг-цикл, который 4.2 позже расширит. [Source: ARCHITECTURE-SPINE.md#AD-4]
- **AD-7** (accessory lifecycle) — только минимальная часть (`LSUIElement`, Task 1), без которой AC2 невыполним; полная реализация (меню-бар как единственная точка Quit) — Story 4.1. [Source: ARCHITECTURE-SPINE.md#AD-7]
- **AD-9** (FFI вне главного потока) — `refreshTorrents()` уже вызывает `engine.getAllTorrents()` через `Task.detached` (Story 1.2) — поллинг-цикл (Task 4) вызывает тот же `refreshTorrents()`, так что AD-9 соблюдён без дополнительных изменений. [Source: ARCHITECTURE-SPINE.md#AD-9]
- **Consistency Conventions → Active/inactive partition**: `active = {Downloading, Checking, Seeding}` — реализовано как `activeStatuses` в Task 4, с добавлением `"resolving"` сверх буквального текста спины (обоснование — см. Task 4). **Data & formats**: «no pre-formatted strings cross the boundary» — `progress_percent`/`*_speed_bps`/`peers_connected` идут числами, форматирование (`%`, `MB/s`) — только на Swift-стороне (Task 5). [Source: ARCHITECTURE-SPINE.md#Consistency Conventions]
- **Snapshot granularity**: `getAllTorrents()` остаётся list-level (без Files/Trackers/Peers по отдельности) — новые поля этой истории (`progress_percent`, `*_speed_bps`, `peers_connected`) все list-level, ничего не нарушает. [Source: ARCHITECTURE-SPINE.md#Consistency Conventions]

### Project Structure Notes

Файлы этой истории (все — UPDATE, новых файлов нет):

```
engine/
  src/
    types.rs      # UPDATE — новые поля TorrentStatus
    lib.rs         # UPDATE — get_all_torrents() полная модель статуса
MyTorrent/
  Info.plist          # UPDATE — LSUIElement
  AppModel.swift      # UPDATE — pollingTask, bootstrap, pollLoop
  Views/MainWindowView.swift  # UPDATE — реальные данные в строке
```

### Previous Story Intelligence (Story 1.2 + пост-review фиксы)

- `Engine::get_all_torrents()` (`engine/src/lib.rs`) уже мёржит `session.with_torrents(...)` с `pending`-записями магнитов (`Arc<Mutex<HashMap<String, PendingTorrent>>>`, `PendingTorrent { name: String, failed: bool }`) — Task 2 трогает только ветку `session.with_torrents(...)`, ветку `pending` не меняет (она уже возвращает `"resolving"`/`"error"` без числовых полей).
- `AppModel.addTorrent(_:)` и `refreshTorrents()` уже существуют и оба используют `Task.detached { ... }.value` для вызова FFI вне MainActor (AD-9) — паттерн, который Task 4 должен повторить для `pollLoop`'а тоже (сам `refreshTorrents()` это уже делает, `pollLoop` просто вызывает его в цикле — не нужно ничего оборачивать заново).
- `AppModel.init()` уже создаёт `Engine` **синхронно на MainActor** (известная, но не исправленная в Story 1.2 находка code-review — `Engine::new` блокирует главный поток при старте) — эта история её не трогает, но `Task { await bootstrap() }` в Task 4 добавляется **после** этого синхронного вызова, так что бутстрап-снимок не усугубляет проблему (он и так уже асинхронный).
- `MyTorrentApp.swift` использует singleton `Window("MyTorrent", id: "main")` + `NSApplicationDelegateAdaptor` (не `WindowGroup`) — при добавлении `LSUIElement` (Task 1) эта структура не меняется, `Window`-сцена продолжает работать как раньше, `LSUIElement` — чисто Info.plist-декларация.
- `MainWindowView.swift` уже рендерит `main-window-torrent-row` со всеми 4 колонками, но Прогресс/↓/↑/Сиды-Личи — статичные `Text("—")` (Story 1.2, Task 7) — именно они заменяются в Task 5. Ширины колонок (`nameColumnWidth`/`progressColumnWidth`/`speedColumnWidth`/`seedLeechColumnWidth`) уже заданы константами — переиспользовать, не менять.
- Тестовая фикстура `engine/tests/fixtures/test.torrent` (маленький single-file торрент) уже существует — переиспользовать в Task 3, а не создавать новую.
- `cargo test` (после Story 1.2 + фиксов) — 4/4 проходят: `engine_version_returns_non_empty_string`, `add_torrent_from_file_appears_in_get_all_torrents`, `add_torrent_from_magnet_registers_without_network`, `add_torrent_from_magnet_marks_error_status_on_resolution_failure`. Task 3 добавляет к ним, не заменяет.

### Testing Standards

Rust: `cargo test` — юнит-тесты на маппинг `TorrentStatus` (Task 3), без сети (как и в Story 1.2 — реальный прогресс/скорости в юнит-тесте недостижимы без реальных пиров, тест проверяет форму/диапазоны, не конкретные значения). Swift: по-прежнему ручная проверка (Task 6) — формальный UI-тест-фреймворк вне скоупа v1 (решение Story 1.1, подтверждено Story 1.2).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 1.3, Story 4.1, Story 4.2]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-4, AD-7, AD-9, Consistency Conventions]
- [Source: _bmad-output/implementation-artifacts/1-2-add-torrent-no-dialogs.md] — previous story, Dev Agent Record + Change Log (code-review находки, две из которых напрямую относятся к этой истории: блокировка MainActor в `AppModel.init()`, нестабильный порядок строк — обе НЕ исправляются в этой истории, вне её AC)
- [Source: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/librqbit-8.1.1/src/torrent_state/stats.rs] — `TorrentStats`, `LiveStats`, `TorrentStatsState`, `Speed`, проверено чтением исходников 2026-07-27
- [Source: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/librqbit-8.1.1/src/torrent_state/live/peers/stats/snapshot.rs] — `AggregatePeerStats`, проверено чтением исходников 2026-07-27
- [Source: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/librqbit-8.1.1/src/torrent_state/live/peer/stats/snapshot.rs] — `PeerStats`/`PeerCounters` (подтверждение отсутствия seed/leech-разбивки), проверено чтением исходников 2026-07-27

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- Ручная проверка на реальном публичном торренте (официальный `.torrent` Ubuntu 24.04.4 Server, `releases.ubuntu.com`) показала 0 подключённых пиров и нулевые скорости на протяжении >2 минут. Диагностика: `torrent.ubuntu.com` и `router.bittorrent.com` не резолвятся ни системным DNS-резолвером, ни напрямую через `dig @1.1.1.1` (timeout), при этом `google.com`/`releases.ubuntu.com` резолвятся нормально — сеть этой машины блокирует BitTorrent-домены на уровне DNS. Не баг кода — сам поллинг-цикл независимо подтверждён временным debug-логом (тики раз в секунду, статус `"downloading"`, не останавливается) до того, как эта находка была сделана.
- `LSUIElement=true` в Info.plist (Task 1) **не** предотвратил завершение процесса при закрытии единственного окна — проверено эмпирически через UI-скриптинг (`click button 1 of window 1`, не глобальный keystroke — тот может улететь не в то приложение). Потребовался явный `AppDelegate.applicationShouldTerminateAfterLastWindowClosed() -> Bool { false }`. После фикса — процесс переживает закрытие окна, повторное открытие `.torrent`-файла возвращает окно через `openWindow(id:)` с сохранённым состоянием.
- Реальное завершение закачки (переход в `"seeding"`) недостижимо e2e в этой сети (см. выше) — статус-маппинг вынесен в чистую функцию `derive_status(state, finished) -> &str` и покрыт юнит-тестом на все 5 веток напрямую, включая `Live && finished → "seeding"`.

### Completion Notes List

- Tasks 1–6 реализованы и покрыты `cargo test` (7/7 passed) + ручной проверкой на реальном приложении и реальном публичном торренте.
- **Task 1 — частичное расхождение с ожиданием (задокументировано, не архитектурное отклонение, а исправленный по ходу баг):** `LSUIElement` сам по себе не останавливает завершение процесса при закрытии окна для SwiftUI `Window`-сцены — понадобился явный `applicationShouldTerminateAfterLastWindowClosed`. Оба изменения в сумме дают минимальную предпосылку AD-4 (AC2), которую и должен был обеспечить Task 1.
- **Task 2/3 — рефакторинг сверх исходного плана (в рамках той же задачи, не отдельное отклонение):** логика определения статуса вынесена из `get_all_torrents()` в чистую функцию `derive_status`, чтобы юнит-тест мог покрыть ветку `"seeding"`, недостижимую без реального завершённого торрента.
- **AC1 (прогресс-бар/скорости/пиры обновляются вживую):** механизм подтверждён логами (поллинг раз в секунду, статус верно вычисляется) — сами значения скорости/пиров остались на нуле из-за сетевого ограничения окружения (см. Debug Log), не из-за кода.
- **AC2 (независимо от открытых окон):** полностью подтверждено вручную после фикса `applicationShouldTerminateAfterLastWindowClosed` — процесс/поллинг переживают закрытие единственного окна, повторное открытие возвращает окно с прежним состоянием.
- **AC3 (Seeding не выпадает из отслеживания):** подтверждено на уровне unit-теста (`derive_status`) и структуры кода (`activeStatuses` в Swift включает `"seeding"`) — не подтверждено e2e из-за сетевого ограничения.
- Scope Boundary соблюдён: меню-бар/уведомление о завершении/разбивка сидов-личей — не реализовывались, как и решено с пользователем до начала кодирования.

### File List

- MyTorrent/Info.plist (modified)
- MyTorrent/MyTorrentApp.swift (modified)
- MyTorrent/AppModel.swift (modified)
- MyTorrent/Views/MainWindowView.swift (modified)
- MyTorrent/Generated/engine.swift (modified — regenerated by build phase; engineFFI.h/module.modulemap unchanged, no new C-level exports)
- engine/src/types.rs (modified)
- engine/src/lib.rs (modified)

## Change Log

- 2026-07-27: Story 1.3 реализована полностью (Tasks 1–6). AC1 и AC3 подтверждены на уровне механизма/юнит-тестов, но не полным e2e (сеть окружения блокирует BitTorrent-домены на уровне DNS — см. Debug Log); AC2 подтверждён полностью вручную. По ходу реализации найден и исправлен баг: `LSUIElement` в одиночку не предотвращает завершение процесса при закрытии окна для SwiftUI `Window`-сцены — добавлен явный `applicationShouldTerminateAfterLastWindowClosed`. Статус-маппинг вынесен в чистую функцию `derive_status` для полного юнит-тест-покрытия всех 5 состояний, включая недостижимую e2e ветку `"seeding"`. Статус → review.
