---
baseline_commit: b5e5a7813cba5a5841fe160452386edbe080d8be
---

# Story 6.2: Список торрентов переживает перезапуск (session persistence)

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу, чтобы мой список торрентов переживал выход и повторный запуск приложения,
чтобы выход никогда не означал потерю закачек или повторное добавление всего вручную.

## Acceptance Criteria

1. **Given** в списке есть торренты (качающиеся, на паузе, раздающиеся), **when** выхожу и запускаю приложение снова, **then** все торренты снова в списке, в том же порядке (restore идёт с `preferred_id` — id стабильны, сортировка Story 1.2 по id сохраняет порядок), с тем же прогрессом.
2. **And** полная перепроверка хэшей при старте НЕ выполняется — `fastresume: true` (bitfield кусков персистится в `.bitv`-файлах, store сам выступает `BitVFactory`).
3. **And** пауза переживает перезапуск (`SerializedTorrent.is_paused` → restore с `paused: true`).
4. **And** per-torrent выходная папка сохраняется (`SerializedTorrent.output_folder` — включая подпапку от `resolve_output_folder`, Story 3.1).
5. **Given** реализация, **then** `Engine::new(download_dir: String, state_dir: String)`; сессия через `Session::new_with_opts` с `SessionOptions { persistence: Some(SessionPersistenceConfig::Json { folder: Some(state_dir.into()) }), fastresume: true, ..Default::default() }`.
6. **And** Swift (`AppModel.setUpEngine`) передаёт `~/Library/Application Support/MyTorrent/session`, создав каталог через `FileManager.createDirectory(withIntermediateDirectories: true)` до вызова `Engine(downloadDir:stateDir:)`.
7. **And** новый Rust-тест roundtrip: add в Engine с tempdir-persistence → пересоздать Engine с тем же `state_dir` → торрент восстановлен (дождаться restore); существующие тестовые конструкторы (`test_engine`/`test_engine_no_peer_sources`) остаются БЕЗ persistence и продолжают проходить.
8. **Given** диалог выхода (Story 6.1), **then** `quit_confirm.message` обновлён: ru «Закачки приостановятся и продолжатся при следующем запуске. Завершить приложение?», en "Downloads will pause and resume on next launch. Quit anyway?" — заголовок и кнопки не меняются (решение brainstorming-сессии 2026-07-30: диалог остаётся).
9. **And** комментарии, ссылающиеся на «session has no persistence» (`TorrentEngineStatus.blocksQuit`, `applicationShouldTerminate` в `MyTorrentApp.swift`), актуализированы; запись «Session persistence отсутствует» в `deferred-work.md` закрыта.

## Scope Boundary

- **Принятое ограничение (зафиксировать, не чинить):** magnet, не успевший отрезолвить метаданные к моменту выхода, не персистится — он живёт в client-side `pending`-карте (Story 1.2), а не в сессии librqbit; после перезапуска его нужно добавить заново. Персистить `pending` — отдельная машинерия ради суб-секундного окна, не делаем.
- Удаление торрента (`remove_torrent` → `session.delete`) чистит и persistence-store — это librqbit-механика, отдельного кода не требует; roundtrip-тест МОЖЕТ это проверить (remove → restart → не восстановился), но это не обязательный AC.
- Настройки экспорта/очистки session-каталога, UI для «забыть всё» — вне скоупа (YAGNI).

## Tasks / Subtasks

- [x] **Task 1: Rust — persistence в `Engine::new`** (AC: 2, 5)
  - [x] Импортировать `SessionPersistenceConfig` (проверить путь: `librqbit::SessionPersistenceConfig` — при несовпадении найти re-export через `cargo doc`/`grep " pub use" в vendored crate, НЕ гадать).
  - [x] `Engine::new(download_dir: String, state_dir: String)` → `Session::new_with_opts(download_dir.into(), SessionOptions { persistence: Some(Json { folder: Some(state_dir.into()) }), fastresume: true, ..Default::default() })`. Остальное тело конструктора без изменений.
  - [x] `disable_dht_persistence` НЕ трогать в проде (DHT-persistence — отдельный механизм, выключен только в тестах по причинам sandbox'а — см. комментарий `test_engine`).
- [x] **Task 2: Rust — roundtrip-тест** (AC: 7)
  - [x] Тест `persisted_torrent_survives_engine_recreation`: tempdir для download и state; добавить fixture-торрент (паттерн существующих тестов); дропнуть Engine/session; создать новый Engine с тем же state_dir; подождать restore (restore асинхронный внутри `Session::new_with_opts` — но `new_with_opts` возвращается после запуска restore-цикла; выяснить эмпирически, нужен ли retry-loop с таймаутом на `get_all_torrents`, — тесты проекта уже имеют паттерн ожидания, посмотреть `remove_torrent_removes_from_get_all_torrents`).
  - [x] Убедиться: 26/26 (25 старых + новый) зелёные. Старые тестовые конструкторы не трогать — они без persistence, их поведение не должно измениться.
- [x] **Task 3: Swift — state_dir** (AC: 6)
  - [x] В `AppModel.setUpEngine`: собрать URL `FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)` + `MyTorrent/session`, `createDirectory(withIntermediateDirectories: true)`, передать путь в `Engine(downloadDir:stateDir:)`. Ошибка создания каталога → тот же fail-path, что ошибка инициализации движка (существующая конвенция `setUpEngine`; см. Dev Notes про известный стартовый крэш).
  - [x] `./scripts/build-engine.sh` ОБЯЗАТЕЛЕН (конструктор UniFFI изменился — Swift-биндинги перегенерируются), затем `xcodebuild`. `xcodegen generate` НЕ нужен (новых файлов нет; `Generated/` уже в проекте).
- [x] **Task 4: Текст диалога + комментарии** (AC: 8, 9)
  - [x] `Localizable.xcstrings`: обновить ТОЛЬКО `quit_confirm.message` (ru+en). Вставка/правка — python json c сохранением порядка файла (файл НЕ отсортирован; сортирующая перезапись даёт 360-строчный дифф — грабли Story 6.1).
  - [x] Комментарии: `TorrentEngineStatus.blocksQuit` («session has no persistence... does not survive an app restart» — переписать: прерывание теперь = пауза до следующего запуска, но диалог остаётся осмысленным); `MyTorrentApp.applicationShouldTerminate` (та же ссылка). Заодно НЕ трогать `Alerts.swift`/кнопки.
  - [x] `deferred-work.md`: закрыть запись «Session persistence отсутствует» (зачеркнуть + «РЕАЛИЗОВАНО Story 6.2»); в записи про стартовый крэш `setUpEngine` дописать, что restore добавил приложению стартовой работы — если крэш вернётся, проверять и persistence-путь.
- [x] **Task 5: Сборка и ручная верификация** (AC: 1-4, 8)
  - [x] Чеклист: удалить leftover `~/Downloads/Torrents/test.txt`; добавить fixture → выйти (диалог! проверить НОВЫЙ текст) → «Завершить» → запустить → торрент в списке, прогресс на месте, статус «Скачивается»; поставить на паузу → выйти (тихо — пауза не блокирует выход, Story 6.1) → запустить → всё ещё на паузе; порядок строк при ≥2 торрентах сохраняется.
  - [x] Проверить содержимое `~/Library/Application Support/MyTorrent/session/` (json + `.torrent` + `.bitv`).
  - [x] После верификации: удалить fixture-торрент из списка ЧЕРЕЗ ПРИЛОЖЕНИЕ (remove чистит и store), затем leftover-файл с диска. Реальные торренты пользователя не трогать. Экранная автоматизация — по разрешению (действует разрешение сессии на AX-клики в окнах MyTorrent/Dock; при сомнении переспросить).

## Dev Notes

### Technical Requirements / Stack

- **Всё построено на фактах из vendored `librqbit-8.1.1` (проверено чтением исходников при создании истории, 2026-07-30):**
  - `SessionPersistenceConfig::Json { folder: Option<PathBuf> }` (`session.rs:367-372`); `None` → OS-специфичный каталог librqbit — НЕ использовать, передавать свой явно.
  - `SerializedTorrent` (`session_persistence/mod.rs:21-33`): info_hash, torrent_bytes, trackers, output_folder, only_files, is_paused.
  - Restore автоматический внутри `Session::new_with_opts` (`session.rs:698+`): `stream_all` → `into_add_torrent` c `preferred_id = Some(id)` (стабильные id) и `overwrite: true` (`mod.rs:61` — «файл уже существует» на restore-пути невозможен; наша add-path fixture-граблина не задета).
  - `fastresume: true` → store сам служит `BitVFactory` (`session.rs:551-556`), bitfield в `<infohash>.bitv`, метаданные в `<infohash>.torrent` + общий json (`json.rs:116-121`).
  - Незарезолвленный magnet восстанавливается как magnet (`Magnet::from_id20`), ЕСЛИ он успел попасть в сессию; наш client-side `pending` — не успевший — теряется (Scope Boundary).
- **`Engine::new` — единственное место создания прод-сессии** (`engine/src/lib.rs:214-216`, сейчас `Session::new`). Единственный Swift-вызов: `AppModel.swift:109` (`Task.detached { try Engine(downloadDir:) }`).
- **UniFFI**: смена сигнатуры конструктора требует перегенерации биндингов — `./scripts/build-engine.sh` (c `--target`-фиксом Story 4.3, второй прогон быстрый).

### Architecture Compliance

- **AD-4 буквально предусматривает эту фичу**: «one unconditional bootstrap poll on launch to discover auto-resumed torrents» — bootstrap-poll уже существует (Story 1.3); восстановленные торренты будут подхвачены им и запустят поллинг через `hasActiveTorrents` без единой правки Swift-поллинга. Если что-то не работает — чинить restore, не поллинг.
- **Story 4.2 (completion notification) устойчива к restore**: уведомление требует наблюдения того же id в не-seeding статусе ранее за сессию — восстановленный уже-seeding торрент НЕ дасть ложного уведомления; восстановленный качающийся, докачавшийся после запуска, — даст (корректно). Проверять не обязательно, но не сломать: не менять diffing-логику.
- AD-5 (split ownership): state_dir — параметр вызова из Swift в Rust, как download_dir (одно-направленный поток настроек) — консистентно.
- AD-1/AD-2/AD-6/AD-8/AD-9 — без изменений; новых FFI-функций нет, меняется только конструктор.

### Project Structure Notes

```
engine/src/lib.rs               # UPDATE — Engine::new(+state_dir), roundtrip-тест
MyTorrent/AppModel.swift        # UPDATE — setUpEngine: state_dir + createDirectory
MyTorrent/MyTorrentApp.swift    # UPDATE — только комментарий applicationShouldTerminate
MyTorrent/Models/TorrentEngineStatus.swift  # UPDATE — только комментарий blocksQuit
MyTorrent/Localizable.xcstrings # UPDATE — quit_confirm.message (ru/en)
MyTorrent/Generated/            # regenerated — build-engine.sh
_bmad-output/implementation-artifacts/deferred-work.md  # UPDATE
```

### Previous Story Intelligence (Story 6.1)

- Прод-путь и review-фиксы 6.1 свежие: `blocksQuit` на enum'е (любая новая классификация статусов — ТОЛЬКО как свойство enum'а с exhaustive switch), `Alerts.runWarning` (безопасная кнопка первой), `AppModel.activateApp()`. Эта история их не меняет — только два комментария.
- **Известный невоспроизведённый стартовый крэш** (`fatalError` в `setUpEngine` при неудачной инициализации движка, deferred-work): эта история добавляет стартовой работе restore-фазу — при верификации перезапусков следить за крэш-репортами; если крэш проявится с persistence — это шанс наконец его изолировать, зафиксировать всё в deferred-work.
- xcstrings НЕ отсортирован — править точечно, не пересериализовывать с сортировкой.
- Верификация без кликов где возможно: AppleEvent `quit` = ровно `applicationShouldTerminate`; содержимое окна — read-only AX (`entire contents of window`); AX-клики в MyTorrent/Dock — разрешены пользователем в этой сессии (Story 6.1), при новой сессии — переспросить.
- Fixture-граблина: leftover `~/Downloads/Torrents/test.txt` блокирует повторный ДОБАВЛЯЮЩИЙ add (`allow_overwrite=false`); к restore не относится (`overwrite: true`), но перед первым add — чистить.

### Testing Standards

- Rust: `cargo test` в `engine/` (25 существующих + новый roundtrip). Тестовые конструкторы без persistence не трогать.
- Swift: формального фреймворка нет — ручная верификация по чеклисту Task 5.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 6.2] — AC-источник
- [Source: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/librqbit-8.1.1/src/session.rs:367-378,535-585,698-725] — persistence config, factory, restore-цикл
- [Source: ~/.cargo/registry/.../librqbit-8.1.1/src/session_persistence/mod.rs:21-66] — SerializedTorrent, into_add_torrent (overwrite/paused/output_folder)
- [Source: ~/.cargo/registry/.../librqbit-8.1.1/src/session_persistence/json.rs:116-121] — раскладка файлов store
- [Source: engine/src/lib.rs:214-226] — текущий Engine::new; :338-346 — remove через session.delete; :693-720 — паттерн тестовых конструкторов
- [Source: MyTorrent/AppModel.swift:104-116] — setUpEngine (вызов конструктора, fatalError-конвенция)
- [Source: _bmad-output/implementation-artifacts/6-1-dock-icon-while-running.md] — диалог выхода, blocksQuit, review-фиксы
- [Source: _bmad-output/implementation-artifacts/deferred-work.md] — закрываемая запись + стартовый крэш

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (Claude Code)

### Debug Log References

- **Новая находка про AX-верификацию (важно для будущих историй): SwiftUI `.contextMenu` на строке списка НЕДОСТИЖИМ через Accessibility API в этом приложении** — строки в AX «расплющены» до статик-текстов прямо в scroll area (нет per-row группы), `perform action "AXShowMenu"` на статик-тексте не открывает доступного меню (ни на элементе, ни на уровне процесса). Любая будущая верификация контекстно-меню-действий — только реальным правым кликом руками пользователя, либо через engine-level тест того же вызова.
- Из-за этого два пункта чеклиста Task 5 выполнены иным способом, чем написано (отклонения зафиксированы честно): (1) персистентность паузы проверена НЕ через UI-паузу, а новым детерминированным Rust-тестом `paused_state_survives_engine_recreation` (UI-пауза зовёт ровно тот же `pause_torrent`); тест заодно доказал, что librqbit пишет `is_paused` в store в момент паузы, а не только при add. (2) Очистка после верификации — не через remove в приложении (то же контекстное меню), а удалением тестового `session/`-каталога целиком при выключенном приложении (создан этой же верификацией, содержал только фикстуру).
- Порядок строк при ≥2 торрентах не проверялся в UI (второго fixture-торрента нет; повторный add того же infohash невозможен) — стабильность id покрыта assert'ом `restored[0].id == id` в roundtrip-тесте плюс `preferred_id`-механикой librqbit (`session.rs:721`).
- AC2 (нет полной перепроверки хэшей): `.bitv`-файл в store подтверждён; восстановленный торрент появился в статусе «Скачивается» без видимой checking-фазы. Отдельного замера времени не делалось (фикстура крошечная — замер не был бы показателен).
- `Session::stop()` перед пересозданием Engine в тестах — прямой доступ к `engine.session` из тестового модуля; ретрай-цикл ожидания restore оставлен (restore-выполнение внутри `new_with_opts` — implementation detail librqbit, не контракт).

### Completion Notes List

- **AC1 — подтверждено на реальном приложении**: fixture добавлен → выход через диалог → повторный запуск → торрент снова в списке в статусе «Скачивается» (bootstrap-poll AD-4 подхватил, как и предсказывала архитектура). Плюс Rust-тест `persisted_torrent_survives_engine_recreation` (id стабилен).
- **AC2 — подтверждено**: `fastresume: true`, `.bitv` в store (см. Debug Log про глубину проверки).
- **AC3 — подтверждено** Rust-тестом `paused_state_survives_engine_recreation` (см. Debug Log про отклонение от UI-способа).
- **AC4 — покрыто механикой** `SerializedTorrent.output_folder` (restore в персистированную папку) + roundtrip-тест использует общий download_dir; отдельной UI-проверки смены папки не было.
- **AC5/AC6 — сделано**: `Engine::new(download_dir, state_dir)` + `Session::new_with_opts` (Json persistence + fastresume); Swift собирает `Application Support/MyTorrent/session`, создаёт каталог, ошибка → существующий fatalError-path. `./scripts/build-engine.sh` перегенерировал биндинги, BUILD SUCCEEDED.
- **AC7 — сделано**: 27/27 тестов зелёные (25 старых + 2 новых: roundtrip и paused-restore); новый хелпер `test_engine_with_persistence` зеркалит прод-опции + `disable_dht_persistence` как в остальных тестовых конструкторах.
- **AC8 — подтверждено на реальном диалоге**: новый текст «Закачки приостановятся и продолжатся при следующем запуске. Завершить приложение?» показан в живом диалоге (AX-чтение); дифф xcstrings — 2 строки (точечная правка, порядок файла сохранён).
- **AC9 — сделано**: комментарии `blocksQuit` и `applicationShouldTerminate` актуализированы; `deferred-work.md` — persistence-запись закрыта (исходный текст сохранён в `<details>`), крэш-запись дополнена persistence-путём для будущей диагностики.
- Тестовый session-store и leftover-файл фикстуры удалены; реальный store появится при первом обычном запуске приложения пользователем.

### File List

- `engine/src/lib.rs` — UPDATE: `Engine::new(+state_dir)` с Json persistence + fastresume; импорт `SessionPersistenceConfig`; хелпер `test_engine_with_persistence`; тесты `persisted_torrent_survives_engine_recreation`, `paused_state_survives_engine_recreation`
- `MyTorrent/AppModel.swift` — UPDATE: `setUpEngine` собирает/создаёт state_dir, вызывает `Engine(downloadDir:stateDir:)`
- `MyTorrent/MyTorrentApp.swift` — UPDATE: только комментарий в `applicationShouldTerminate`
- `MyTorrent/Models/TorrentEngineStatus.swift` — UPDATE: только комментарий `blocksQuit`
- `MyTorrent/Localizable.xcstrings` — UPDATE: `quit_confirm.message` (ru/en)
- `MyTorrent/Generated/` — regenerated (`./scripts/build-engine.sh`, новый конструктор UniFFI)
- `_bmad-output/implementation-artifacts/deferred-work.md` — UPDATE: persistence-запись закрыта, крэш-запись дополнена, добавлена запись про NFR1-регрессию

### Code Review (2026-07-30, `/code-review high` — 8 finder angles + verify)

Полный отчёт разобран с пользователем; решения по каждой находке — ниже.

- **CONFIRMED-high — повреждённый store делал приложение незапускаемым (crash-loop).** Одна нечитаемая запись в persistence-store'е (битый `session.json`) роняла весь `Session::new_with_opts` → `Engine::new` Err → Swift `fatalError` на каждом запуске, пока пользователь вручную не удалит `~/Library/Application Support/MyTorrent/session/`. **ИСПРАВЛЕНО:** новый `Engine::open_with_recovery` — при первой ошибке, ЕСЛИ store непустой (значит есть что портить), карантинит папку (`session` → `session.corrupt-<unix>`, `quarantine_store`) и переоткрывает пустую сессию один раз; пустой store / неудачный карантин / вторая ошибка — прежний Err-путь (это настоящий сбой движка). Новый тест `corrupt_store_is_quarantined_and_engine_recovers` проверяет ровно прод-путь (через `open_with_recovery` с битым `session.json`).
- **CONFIRMED-med — диалог выхода врал про `.resolving`-magnet'ы.** Незарезолвленный magnet живёт только в in-memory `pending` (не персистится), но `blocksQuit` включал `.resolving`, а новый текст обещал «продолжатся при следующем запуске». **ИСПРАВЛЕНО (решение пользователя из multiple-choice):** `.resolving` исключён из `blocksQuit` — незарезолвленный magnet больше не блокирует выход (суб-секундное окно, Scope Boundary уже принимает его потерю); как только он резолвится в `.downloading`/`.checking`, снова блокирует — честно. Комментарий на enum'е обновлён.
- **PLAUSIBLE product — NFR1 idle-регрессия от restore раздающих торрентов.** **ПРИНЯТО + задокументировано** (решение пользователя): восстановленный `seeding` держит приложение неidle с первой секунды каждого запуска. Прямое следствие фичи; запись добавлена в `deferred-work.md` (направления: «восстанавливать на паузе» / увязка с seed-duration Story 3.2).
- **REFUTED (окончательно, не переоткрывать):** fastresume + удалённые/перемещённые пользователем файлы данных — librqbit self-heals (re-hash перед доверием bitfield'у); «прод не зовёт `session.stop`, store устаревает» (write awaited внутри `add_torrent`, atomic tmp+rename); «DHT-persistence роняет sandboxed app» (app не sandboxed, AD-7); «`Session::new` прятал дефолты».
- **Cleanup — ПРИМЕНЕНО ВСЁ (решение пользователя):** (a) дедуп тестов — `Engine::from_session` (был скопирован 4×), `persistent_session_options` (прод↔тест), `test_persistent_options` (тест-вариант), `wait_until`-хелпер (5 рукописных 250×20ms-циклов); (b) деривация пути store'а перенесена в `AppSettings.sessionStateDirectory`; (c) `MyTorrent/session` разбит на два path-компонента, убран лишний `create: true` рядом с `createDirectory`; (d) тест paused-restore устойчивее — ждёт выхода из транзиентного `checking` перед проверкой resting-статуса (`== "paused"`), а не ломается на первом сэмпле «paused».

Итог: 28/28 Rust-тестов зелёные (было 27 + новый recovery-тест), `xcodebuild` BUILD SUCCEEDED.
