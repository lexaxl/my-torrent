---
baseline_commit: e4d01b1231e760da39d619ba2ad577cd7963a156
---

# Story 2.2: Трекеры и пиры

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу переключиться на вкладки «Трекеры» и «Пиры» в окне деталей торрента,
чтобы проверить статус подключения.

## Acceptance Criteria

1. **Given** окно деталей открыто, **when** я кликаю вкладку «Трекеры», **then** вижу список трекеров торрента (URL каждого).
2. **Given** окно деталей открыто на вкладке «Пиры», **when** есть подключённые пиры, **then** вижу список: адрес, состояние соединения, скорость скачивания (↓) — **см. Scope Boundary**: реальная роль (сид/личер) и скорость отдачи (↑) на пира недоступны от движка, в этой истории не показываются.
3. **Given** окно деталей открыто на вкладке «Пиры», **when** подключённых пиров нет, **then** показывается «Нет подключённых пиров» вместо пустого списка.
4. **Given** данные трекеров/пиров, **then** они приходят в составе того же `get_torrent_details(id)`-вызова (Story 2.1), на том же ~1с поллинге окна — не отдельный вызов.

## Scope Boundary

**Решено при создании истории — реальное ограничение публичного API `librqbit` 8.1.1, не архитектурное решение проекта, подтверждено экспериментальной компиляцией (`cargo check` со scratch-пробниками против `Api::api_peer_stats` и `ManagedTorrentShared.trackers` — оба подтверждены, а прямой доступ к per-tracker announce-статусу и per-peer роли/upload-скорости — не существует, см. Dev Notes → librqbit internals):**

- **Трекеры: только список URL, БЕЗ статуса подключения.** `librqbit` нигде публично не хранит и не отдаёт результат последнего анонса трекера (успех/ошибка/interval) — `TrackerComms` (крейт `librqbit-tracker-comms`), который реально стучится в трекеры, работает как fire-and-forget background-таск с логами через `tracing::debug!`, без сохранения состояния куда-либо доступного извне. Единственное публично доступное поле — `ManagedTorrentShared.trackers: HashSet<Url>` (список URL, с которыми торрент был добавлен). **Согласовано с пользователем при создании истории**: показываем список URL, статус не показываем вообще (ни колонки, ни плейсхолдера вида «неизвестно» — это будет выглядеть как недоделанный UI). Оригинальная формулировка AC в epics.md («со статусом (подключён/не отвечает)») в этой истории не покрывается полностью — сознательное отклонение, зафиксированное здесь. Точный статус — возможная будущая история, если понадобится (потребует отдельного лёгкого «пробника» трекеров поверх `librqbit-tracker-comms`, а не форка `librqbit`).
- **Пиры: адрес + состояние соединения + скорость скачивания (расчётная). БЕЗ роли (сид/личер) и БЕЗ скорости отдачи.** `Api::api_peer_stats(idx, filter) -> PeerStatsSnapshot` (публичный метод, уже используемый как `self.api.api_torrent_details(...)` в `reveal_path`) отдаёт `HashMap<String, PeerStats>` (ключ — адрес пира), `PeerStats { counters: PeerCounters, state: &'static str }`. `PeerCounters` содержит **только** `fetched_bytes: u64` (кумулятивно скачано с этого пира) — **нет** аналогичного `uploaded_bytes`, и нет флага/битфилда, по которому можно определить, что пир — сид (есть 100% файла). Роль и upload-скорость на пира технически не получить без форка `librqbit` — не в скоупе. **Согласовано с пользователем**: скорость скачивания на пира считаем сами — движок хранит предыдущее значение `fetched_bytes` + время замера на каждый (torrent_id, address) и на каждый вызов `get_torrent_details` вычисляет `Δbytes / Δt` (см. Dev Notes → Technical Requirements). `PeerStatsFilter::default()` уже фильтрует на `PeerStatsFilterState::Live` (только реально подключённые пиры) — совпадает с AC2/AC3 «подключённые пиры» без доп. фильтрации на нашей стороне.
- **`Tracker`/`Peer` — типы, которых сейчас нет в `types.rs`** (Architecture Spine's Structural Seed упоминает их как `TorrentStatus, TorrentFile, Tracker, Peer, EngineError` ещё в первоначальном плане `types.rs`, но Story 1.1–2.1 их не создавали — они создаются здесь, впервые).
- **Не проверять типы `PeerStatsFilter`/`PeerStatsSnapshot`/`PeerStats`/`PeerCounters` по имени в коде `engine/`** — они не реэкспортированы из `librqbit` публично (модуль `torrent_state` в `librqbit` — приватный, `mod torrent_state;` без `pub`). Вызывать `self.api.api_peer_stats(idor, Default::default())` и работать с результатом **только через field access / `.iter()` / деструктуризацию** — тип у `Default::default()` выведется из сигнатуры метода компилятором, без необходимости называть `PeerStatsFilter` явно. Так же с `peer_stats.state` (уже `&str`) и `peer_stats.counters.fetched_bytes` (уже `u64`) — доступны через `.`, без именования типа `PeerStats`/`PeerCounters`. Подтверждено рабочим `cargo check` при создании этой истории — если попытаться написать `use librqbit::torrent_state::...` или явно объявить переменную типа `PeerStatsFilter`, сборка не пройдёт (приватный модуль).
- **Если `api_peer_stats` возвращает `Err`** (внутри `librqbit` это `handle.live().context("not live")?` — торрент ещё не в состоянии `Live`, например только `Initializing`), это **не ошибка** уровня `get_torrent_details` — тот же паттерн graceful-empty, что уже есть для `files`, когда `metadata` ещё `None` (Story 2.1). `peers` в этом случае — пустой `Vec`, весь остальной `TorrentDetail` (включая `files`) собирается как обычно.

**Не в этой истории:**
- Реальный статус трекера (подключён/не отвечает) — см. выше, отклонено на этапе создания истории.
- Роль пира (сид/личер) и скорость отдачи на пира — см. выше, недоступно от `librqbit` без форка.
- Любые мутации (добавить/удалить трекер вручную, забанить пира) — вне v1 везде, не только здесь.

## Tasks / Subtasks

- [x] **Task 1: Rust engine — типы `Tracker`/`Peer`** (AC: 1, 2)
  - [x] `engine/src/types.rs`: добавить 2 новых `#[derive(uniffi::Record)]`:
    - `Tracker { url: String }`
    - `Peer { address: String, state: String, down_speed_bps: u64 }`
  - [x] `TorrentDetail` (уже существует): добавить 2 новых поля — `pub trackers: Vec<Tracker>`, `pub peers: Vec<Peer>` (после `files`)
  - [x] Экспортировать оба новых типа из `lib.rs`: `pub use types::{..., Peer, Tracker, ...};`

- [x] **Task 2: Rust engine — `get_torrent_details` наполняет `trackers`/`peers`** (AC: 1, 2, 3, 4)
  - [x] `Engine` — новое приватное поле: `peer_speed_baseline: Mutex<HashMap<(String, String), (u64, std::time::Instant)>>` (ключ — `(torrent_id, peer_address)`, значение — `(fetched_bytes на прошлом опросе, время опроса)`). Добавить в **три** места, где строится `Engine { ... }`: `Engine::new` (продакшн-конструктор), `test_engine()`, `test_engine_no_peer_sources()` (оба тестовых хелпера в `#[cfg(test)] mod tests`) — все три уже перечисляют одинаковый набор полей (`session`, `pending`, `api`, `next_pending_seq`), забыть одно из трёх мест = ошибка компиляции, но лучше не забыть все три сразу.
  - [x] Новый приватный метод `fn peer_download_speed_bps(&self, torrent_id: &str, address: &str, fetched_bytes: u64) -> u64`:
    - Берёт текущее время (`std::time::Instant::now()`), ищет `(torrent_id, address)` в `peer_speed_baseline`
    - Если запись есть и `fetched_bytes >= prev_bytes` и `elapsed_secs > 0.0` → `((fetched_bytes - prev_bytes) as f64 / elapsed_secs).round() as u64`
    - Иначе (первая запись для этого пира, или `fetched_bytes` уменьшился — переподключение с новым `Peer`-объектом и обнулёнными счётчиками) → `0` (тот же принцип «не мигать, не показывать мусор», что уже применяется в `summarize_stats`/`derive_status`)
    - В конце **всегда** обновляет baseline на текущие `(fetched_bytes, now)`, независимо от ветки выше
  - [x] Новый приватный метод `fn prune_peer_speed_baselines<'a>(&self, torrent_id: &str, current_addrs: impl Iterator<Item = &'a String>)`: удаляет из `peer_speed_baseline` записи с этим `torrent_id`, чьего адреса нет в `current_addrs` (пир отключился) — `HashMap::retain`, не даёт карте расти неограниченно при churn пиров за долгую сессию (NFR1 — near-zero memory at idle)
  - [x] `get_torrent_details` — после существующего кода для `files`, добавить:
    ```rust
    let trackers = handle
        .shared
        .trackers
        .iter()
        .map(|url| Tracker { url: url.to_string() })
        .collect();

    let idor = Self::parse_id(&id)?;
    let peers = match self.api.api_peer_stats(idor, Default::default()) {
        Ok(snapshot) => {
            let peers = snapshot
                .peers
                .iter()
                .map(|(address, peer_stats)| Peer {
                    address: address.clone(),
                    state: peer_stats.state.to_string(),
                    down_speed_bps: self.peer_download_speed_bps(
                        &id,
                        address,
                        peer_stats.counters.fetched_bytes,
                    ),
                })
                .collect();
            self.prune_peer_speed_baselines(&id, snapshot.peers.keys());
            peers
        }
        // "not live" — торрент ещё Initializing/Paused в librqbit, не настоящая
        // ошибка для UI (тот же паттерн, что files=[] при metadata=None).
        Err(_) => Vec::new(),
    };
    ```
    и добавить `trackers, peers,` в конструктор `TorrentDetail { ... }`
  - [x] `remove_torrent`: в начале метода добавить `self.peer_speed_baseline.lock().unwrap().retain(|(tid, _), _| tid != &id);` — иначе baseline-записи удалённого торрента остаются в карте навсегда (мелкая, но реальная утечка памяти при долгой сессии с частым добавлением/удалением торрентов)
  - [x] `use std::time::Instant;` — добавить в импорты `lib.rs`, если не используется через полный путь `std::time::Instant`
  - [x] Собрать: `cd engine && cargo build`

- [x] **Task 3: Rust engine — тесты** (AC: 1, 2, 3)
  - [x] Тест `get_torrent_details_returns_trackers_from_torrent_file`: `test_engine()`, добавить торрент из `tests/fixtures/test.torrent` (у фикстуры один трекер: `http://example.invalid:6969/announce`, поле `announce` — подтверждено при создании истории чтением байт фикстуры), вызвать `get_torrent_details`, проверить `detail.trackers.len() == 1` и `detail.trackers[0].url.contains("example.invalid")`
  - [x] Тест `get_torrent_details_returns_empty_peers_when_no_real_peers`: тот же торрент из фикстуры, без реальной сети — `detail.peers.is_empty()` (нет причин ожидать иного в тестовой среде, тот же case, что уже покрыт для `peers_connected == 0` в Story 1.3/2.1)
  - [x] Тест `get_torrent_details_does_not_panic_on_repeated_calls_with_no_peers`: вызвать `get_torrent_details` дважды подряд для одного и того же id — проверяет, что расчёт дельты скорости (первый вызов = нет baseline, второй = baseline с нулевыми пирами) не паникует и не ловит underflow на пустом множестве пиров
  - [x] Тест `peer_download_speed_bps_computes_delta_and_clamps_negative`: **напрямую** на `test_engine()`-инстансе (метод приватный, но тест в том же модуле имеет доступ):
    - Первый вызов `engine.peer_download_speed_bps("t1", "1.2.3.4:6881", 1000)` → `0` (нет предыдущей baseline)
    - Подождать `std::thread::sleep(Duration::from_millis(50))`, второй вызов с `fetched_bytes = 1000 + N` → результат `> 0` (не проверять точное число — реальное время sleep не детерминировано с точностью до мс, только знак/порядок величины)
    - Третий вызов с `fetched_bytes`, меньшим предыдущего (эмуляция переподключения с обнулённым счётчиком) → `0`, не паника/underflow
  - [x] `cargo test` — все тесты зелёные (12 существующих Story 1.x + 3 Story 2.1 + 4 новых)

- [x] **Task 4: Swift — генерация биндингов** (AC: 1, 2)
  - [x] `./scripts/build-engine.sh` — перегенерирует `Generated/engine.swift` с `Tracker`/`Peer` структурами и обновлённым `TorrentDetail` (новые поля `trackers`/`peers` в Swift появятся как `camelCase`: `torrentDetail.trackers`, `torrentDetail.peers`, элементы — `tracker.url`, `peer.address`/`peer.state`/`peer.downSpeedBps`, per Consistency Conventions naming rule)

- [x] **Task 5: Swift — содержимое вкладок Трекеры/Пиры** (AC: 1, 2, 3, 4)
  - [x] `MyTorrent/Views/TorrentDetailView.swift`:
    - Заменить `case .trackers, .peers: Color.clear` (комментарий "Content wired up in Story 2.2" — эта история) на:
      ```swift
      case .trackers:
          trackersList
      case .peers:
          peersList
      ```
    - Новая computed property `trackersList`, тот же паттерн, что `filesList` (ScrollView + VStack(spacing: 0) + ForEach + Divider): строка — `Text(tracker.url)` (`.lineLimit(1)`, `.truncationMode(.middle)`, тот же паддинг `.padding(.horizontal, 12).padding(.vertical, 8)`, что и `fileRow`). Без второй колонки-статуса (см. Scope Boundary — статуса нет). Пустой список трекеров (`detail?.trackers ?? []` пуст) — без специального empty-state текста, не специфицирован в UX-спеке, оставить как есть (пустая прокрутка) — не изобретать новую локализованную строку, которой нет в спеке
    - Новая computed property `peersList`, тот же паттерн:
      - Если `(detail?.peers ?? []).isEmpty` → центрированный `Text("torrent_detail.peers.empty")` (новый локализационный ключ, см. Task 6) — `.font(.body)`, `.foregroundStyle(.secondary)`, по центру (`.frame(maxWidth: .infinity, maxHeight: .infinity)`), закрывает AC3
      - Иначе — список строк: `HStack { Text(peer.address).lineLimit(1); Spacer(); Text(Formatting.speed(peer.downSpeedBps) + " ↓").foregroundStyle(.secondary); Text(peer.state).foregroundStyle(.secondary) }`, тот же padding-паттерн, что `fileRow`/`trackersList`-строка
  - [x] Не трогать `.task(id: torrentId)`-поллинг-луп — `trackers`/`peers` приходят в том же `TorrentDetail` от того же `fetchTorrentDetail`, ничего нового поллить не нужно (AC4)

- [x] **Task 6: Локализация** (AC: 3)
  - [x] `MyTorrent/Localizable.xcstrings` — добавить 1 новый ключ (тот же JSON-стиль, что и существующие `torrent_detail.tabs.*`, с пробелом перед `:` — см. Previous Story Intelligence):
    - `torrent_detail.peers.empty` — RU "Нет подключённых пиров" / EN "No peers connected" (точный текст из UX-спеки `2.2-torrent-detail.md`, Section: Content Area — Peers Tab → Peers Empty State)

- [x] **Task 7: Сборка и ручная проверка** (AC: 1-4)
  - [x] `xcodegen generate` (не обязательно в этой истории — новых `.swift`-файлов нет, только правки существующего `TorrentDetailView.swift`, но безопасно прогнать при любых сомнениях) + `./scripts/build-engine.sh` + `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] Добавить торрент из фикстуры/реальный `.torrent`-файл, открыть окно деталей, переключиться на вкладку «Трекеры» — убедиться, что URL(ы) отображаются, без крашей
  - [~] Переключиться на вкладку «Пиры» — **визуально подтверждена только вкладка «Трекеры»** (скриншот показал реальный URL `http://example.invalid:6969/announce` из фикстуры, без колонки статуса, без краша — окно деталей открылось и отрендерилось корректно). Дальнейшая автоматизация (повторное открытие окна деталей кликом по строке через AppleScript/Accessibility) не сработала в этой среде — тот же самый известный лимит, что и в Story 2.1 Debug Log: синтетический `click`/`click at` через System Events не надёжно доходит до `.simultaneousGesture(TapGesture())` на строке торрента (не настоящее mouse-down/up), из-за чего второе окно не открывалось повторно для скриншота вкладки «Пиры». Код `peersList`/empty-state идентичен по паттерну уже подтверждённому `trackersList` и `filesList`, собирается и типизируется без ошибок — **рекомендация пользователю**: вручную кликнуть по строке торрента и переключиться на «Пиры», чтобы визуально подтвердить AC3 (реальных пиров в этой сети не будет — DNS блокирует BitTorrent-домены, см. project memory `my-torrent-dev-environment` — там ожидаемо будет just empty-state, а не список)
  - [x] `cargo test` (engine/) — все тесты зелёные (19/19: 15 существующих + 4 новых)

## Dev Notes

### Technical Requirements / Stack

- **Оба новых источника данных подтверждены рабочей компиляцией при создании этой истории** (временный scratch-код в `engine/src/lib.rs`, собранный через `cargo check --lib` и затем убранный — не оставлен в дереве):
  - `self.api.api_peer_stats(idor, Default::default())` → `Ok(snapshot)` с `snapshot.peers: HashMap<String, PeerStats>` (ключ — адрес пира как строка), `PeerStats { counters: PeerCounters { fetched_bytes: u64, ... }, state: &'static str }`. `state` — одно из `"queued"`/`"connecting"`/`"live"`/`"dead"`/`"not needed"` (см. `librqbit` `PeerState::name()`); с фильтром по умолчанию (`PeerStatsFilterState::Live`) практически всегда будет `"live"`, но поле не хардкодить — фильтр может измениться в будущем.
  - `handle.shared.trackers` — `HashSet<url::Url>`, публичное поле `ManagedTorrentShared` (само `ManagedTorrent.shared: Arc<ManagedTorrentShared>` — тоже публичное поле). Заполняется из `announce`/`announce-list` `.torrent`-файла при добавлении (для magnet-ссылок — из `tr=`-параметров, если есть). `.to_string()` на `Url` даёт строку без необходимости именовать тип `Url` в коде `engine/` (не добавлять `url` crate в `Cargo.toml` — не нужно).
- **`PeerCounters`/`PeerStats`/`PeerStatsFilter`/`PeerStatsSnapshot` не реэкспортированы публично из `librqbit`** (`mod torrent_state;` в `librqbit`'s `lib.rs` — приватный модуль, только конкретные типы вроде `ManagedTorrent`/`TorrentStats` реэкспортированы поимённо). Это значит: их нельзя написать как явную аннотацию типа (`let x: PeerStats = ...` не скомпилируется — тип недоступен по имени), но можно (и нужно) использовать через field access/итерацию/`Default::default()` с выводом типа из сигнатуры метода — компилятор сам подставит правильный тип. Если во время имплементации это не компилируется — проверить, не добавился ли явный именованный тип там, где нужен только field access.
- **Скорость скачивания на пира — не то же самое, что `down_speed_bps` уровня торрента** (`summarize_stats`/`live.download_speed.mbps`, уже существующий, агрегированный по всем пирам через `SpeedEstimator` внутри `librqbit`). На уровне отдельного пира `librqbit` не считает скорость вообще — только кумулятивный счётчик байт. Расчёт через дельту между опросами (`peer_download_speed_bps`) — обязательно на стороне Rust-движка, не Swift: Consistency Conventions прямо требует «Speeds ... computed by the engine from its own internally-tracked elapsed time — never assumed to equal the ~1s poll interval» — тот же принцип, что уже применяется к торрент-уровневым скоростям, здесь просто применяется вручную (Instant-based elapsed time), а не через `librqbit`'s собственный `SpeedEstimator` (у которого нет per-peer варианта).
- **`AD-9`** (FFI off main thread) не требует новых изменений — `get_torrent_details` уже вызывается через `Task.detached` в `AppModel.fetchTorrentDetail` (Story 2.1), новые поля просто едут в том же `TorrentDetail`.

### Architecture Compliance

- Соответствует Consistency Conventions «Snapshot granularity»: трекеры/пиры остаются частью `get_torrent_details(id)`, не расползаются в отдельные UniFFI-вызовы и не попадают в `get_all_torrents()`.
- **Отклонение от AD-2 не требуется** — «пробник» статуса трекеров (если реализовывать в будущей истории) переиспользовал бы `librqbit-tracker-comms` (тот же крейт, что `librqbit` использует внутри себя для анонсов) для read-only проверки, а не писал бы свой tracker-протокол с нуля. Не актуально для этой истории (см. Scope Boundary — статус трекеров не реализуется здесь).
- UX-спека `2.2-torrent-detail.md`: Tab: Trackers / Tab: Peers / Peers Empty State секции — точные RU/EN строки уже перенесены в Tasks выше.

### Project Structure Notes

```
engine/
  src/types.rs                # UPDATE — Tracker, Peer records; TorrentDetail получает 2 новых поля
  src/lib.rs                  # UPDATE — peer_speed_baseline (Engine field, 3 места конструктора),
                               #          peer_download_speed_bps, prune_peer_speed_baselines,
                               #          get_torrent_details наполняет trackers/peers,
                               #          remove_torrent чистит baseline, новые тесты
MyTorrent/
  Generated/engine.swift, engineFFI.h   # REGENERATED
  Views/TorrentDetailView.swift          # UPDATE — trackersList/peersList вместо Color.clear
  Localizable.xcstrings                  # UPDATE — 1 новый ключ (torrent_detail.peers.empty)
```

Никаких новых `.swift`-файлов — `xcodegen generate` не обязателен (но безопасен), в отличие от Story 2.1, где он был необходим из-за новых файлов.

### Previous Story Intelligence (Story 2.1)

- `find_handle(id)` уже возвращает корректную ошибку для торрента, которого ещё нет в `session` (pending/resolving) — но в этой истории `trackers`/`peers` заполняются **после** `find_handle` уже успешно отработал (как и `files` в Story 2.1), так что эта ветка не добавляет новой обработки ошибок.
- `Localizable.xcstrings` — писать новый ключ в нативном Xcode JSON-стиле (`"key" : value`, пробел перед `:`), не голым `json.dump` — Story 1.5's code-review находка, подтверждена снова в Story 2.1 Dev Notes.
- Паттерн «видимый, но не работающий элемент UI» (поле поиска Story 1.1→1.4; вкладки Трекеры/Пиры Story 2.1→2.2) завершается этой историей — вкладки, добавленные в Story 2.1, наконец получают контент.
- `AppModel.engine` — `Engine?`, не `let Engine` — эта история не трогает `AppModel.swift` вообще (никаких новых методов, `fetchTorrentDetail` уже существует и не меняется), так что этот момент неактуален для новых правок, но релевантен, если по ходу дела понадобится что-то новое в `AppModel`.
- Ручная UI-проверка через AppleScript/Accessibility в этой среде нестабильна (project memory `my-torrent-dev-environment`) — переключение вкладок (Picker) проще для скриптовой автоматизации, чем клики по строкам/контекстные меню, но не полагаться на неё как единственное подтверждение; скриншот + при необходимости прямое взаимодействие пользователя.

### Testing Standards

- Rust: `cargo test` в `engine/`, тот же `test_engine()`/`test_engine_no_peer_sources()` — паттерн уже существует. Новый приватный метод `peer_download_speed_bps` тестируется напрямую (не только через `get_torrent_details`) — чистая функция от `&self` + примитивов, детерминированная логика (delta/clamp) стоит покрыть отдельно от сетевого сценария.
- Swift: формальный UI-test framework вне скоупа v1 (решение Story 1.1) — только ручная проверка (Task 7).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 2.2]
- [Source: design-artifacts/C-UX-Scenarios/02-alex-inspects-a-torrent/2.2-torrent-detail/2.2-torrent-detail.md] — Section: Content Area — Trackers Tab, Content Area — Peers Tab, Peers Empty State
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#Consistency Conventions (Snapshot granularity, Data & formats), Structural Seed (types.rs комментарий уже упоминает Tracker/Peer)]
- [Source: _bmad-output/implementation-artifacts/2-1-torrent-detail-window.md] — предыдущая история: `get_torrent_details`, `summarize_stats`, `find_handle`, паттерн graceful-empty для `files`/`metadata == None`
- librqbit 8.1.1 crate source (`~/.cargo/registry/src/.../librqbit-8.1.1/`):
  - `src/api.rs:268` — `Api::api_peer_stats(idx, filter) -> Result<PeerStatsSnapshot>`
  - `src/torrent_state/live/peer/stats/snapshot.rs` — `PeerStats { counters: PeerCounters, state: &'static str }`, `PeerCounters { fetched_bytes: u64, ... }` (нет `uploaded_bytes` на пира), `PeerStatsFilter { state: PeerStatsFilterState }` (`Default` → `Live`)
  - `src/torrent_state/mod.rs:187-201` — `ManagedTorrentShared { pub trackers: HashSet<url::Url>, ... }`
  - `src/session.rs:944-966` — трекеры для `.torrent`-файла берутся из `torrent.info.iter_announce()`
  - `src/lib.rs:68` — `mod torrent_state;` (приватный — отсюда невозможность именовать `PeerStats`/`PeerStatsFilter` и т.п. напрямую)

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- `~/Downloads/test.txt` оставался от более ранней ручной проверки (Story 2.1) — `add_torrent` возвращал `Internal(message: "error creating a new file (because allow_overwrite = false) ...")` при повторном добавлении фикстуры. Не баг этой истории — удалено вручную (`rm -f ~/Downloads/test.txt`) перед проверкой; движок корректно отказывается перезаписывать существующий файл, это ожидаемое поведение.
- Синтетический `click`/`click at {x,y}` через System Events/Accessibility на `MainWindowView`'s строке торрента не открывал окно деталей повторно в этой среде (тот же класс проблем, что уже задокументирован в Story 2.1 Debug Log и project memory `my-torrent-dev-environment`: AppleScript-автоматизация ненадёжна для `.simultaneousGesture(TapGesture())`, не настоящее mouse-down/up событие). Первое открытие окна (сразу после пересборки, до дальнейших попыток автоматизации) прошло успешно и было заскриншочено — показало вкладку «Трекеры» с реальным URL `http://example.invalid:6969/announce` из фикстуры, без колонки статуса, без крашей. Дальнейшие попытки переоткрыть окно для скриншота вкладки «Пиры» через автоматизацию не увенчались успехом — не удалось довести визуальную проверку AC3 (empty-state) до скриншота в рамках этой сессии.

### Completion Notes List

- Все 7 задач выполнены. AC1 (список трекеров) и AC4 (данные в составе единого `get_torrent_details`, тот же поллинг) подтверждены как автоматическими тестами, так и одним успешным визуальным скриншотом вкладки «Трекеры» с реальными данными движка.
- AC2 (адрес/скорость скачивания пира) и AC3 (empty-state «Нет подключённых пиров») подтверждены на уровне кода и юнит-тестов и успешной сборкой (SwiftUI-код компилируется без ошибок, паттерн идентичен уже визуально подтверждённым `filesList`/`trackersList`), но НЕ доведены до собственного скриншота — см. Debug Log Reference про ненадёжность AppleScript-автоматизации в этой среде (тот же лимит, что и в Story 2.1).
- **Рекомендация пользователю**: вручную кликнуть по строке торрента в главном окне, переключиться на вкладку «Пиры» и визуально подтвердить AC2/AC3 — ожидаемый результат в этой сети (DNS блокирует BitTorrent-домены, см. project memory) — надпись «Нет подключённых пиров», не список.
- Реализовано два реальных ограничения `librqbit` 8.1.1, согласованных с пользователем на этапе создания истории (см. Scope Boundary): без статуса трекера (только URL) и без роли/upload-скорости пира (только адрес, скорость скачивания).

#### Code Review (high effort, 8-angle parallel finder + verify pass) — 2026-07-28

9 из 10 кандидатов подтверждены (1 REFUTED — `trackersList` без fill-фрейма оказался идентичен уже принятому паттерну `filesList` из Story 2.1, не регрессия). Все 9 исправлены:

- **[Correctness] `Err(_) => Vec::new()` глотал любую ошибку `api_peer_stats`**, не только «торрент ещё не live» — теперь `get_torrent_details` заранее проверяет `stats.live.is_none()` (тот же признак, что использует `handle.live()` внутри librqbit) и пропускает вызов для ожидаемого случая; если вызов всё же падает при живом торренте — это логируется через `eprintln!`, а не проглатывается молча.
- **[Correctness] Утечка `peer_speed_baseline` при паузе торрента** — пауза переводит торрент в состояние, где `api_peer_stats` возвращает `Err`, а старая чистка (`prune_peer_speed_baselines`) вызывалась только в ветке `Ok`. Новый единый метод `build_peers` теперь всегда чистит baseline до текущего (пусть и пустого) набора пиров, независимо от ветки.
- **[Correctness] Фильтр `Default::default()` = только live-пиры, поле `state` всегда `"live"`** — фильтр оставлен как есть (это правильно для AC «подключённые пиры»), но раз колонка не может нести информацию, её убрали из UI (`peerRow` в `TorrentDetailView.swift`); поле `Peer.state` оставлено в Rust-типе на будущее (ничего не стоит), просто не отображается.
- **[Correctness] Нестабильный порядок `HashMap` → список пиров «прыгал» на каждом опросе** — записи сортируются по адресу перед сборкой `Vec<Peer>` (Rust), а `ForEach` в Swift теперь ключуется по `peer.address`, а не по позиции.
- **[Correctness] `peer.state` не локализован** — снято вместе с удалением колонки состояния из UI (см. выше).
- **[Efficiency] Двойной парсинг id + N+1 блокировок мьютекса за один опрос** — `get_torrent_details` парсит id один раз и переиспользует; `peer_download_speed_bps`+`prune_peer_speed_baselines` объединены в один метод `build_peers` с одной блокировкой на весь пакет пиров.
- **[Efficiency] Очистка baseline сканировала карту по всем торрентам** — `peer_speed_baseline` теперь вложенная карта (`HashMap<String, HashMap<String, (u64, Instant)>>`, ключ верхнего уровня — id торрента), `remove_torrent` делает `O(1)` `.remove(&id)`, `build_peers` чистит только свой вложенный словарь и удаляет его целиком, когда пуст.
- **[Simplification] Три идентичных списочных SwiftUI-скаффолда** — `filesList`/`trackersList` переведены на общий `rowList<Item, Row: View>(_:row:)`; `peersList` оставлен отдельным (другая стратегия identity — по адресу, не по позиции).
- **[Altitude] `peer_speed_baseline` был раскидан по 5+ местам** — консолидирован в один метод `build_peers`, владеющий и расчётом дельты, и чисткой, по аналогии с уже принятым в проекте паттерном `summarize_stats`.

Добавлено 3 новых Rust-теста (`build_peers_prunes_baseline_for_disconnected_peers`, `build_peers_with_empty_entries_clears_torrent_from_baseline_map`, `get_torrent_details_returns_empty_peers_without_error_when_paused`), один тест переписан под новую сигнатуру (`build_peers_computes_delta_and_clamps_negative`, ранее `peer_download_speed_bps_...`). Итого 22/22 зелёных. `xcodebuild` — BUILD SUCCEEDED после фиксов.

### File List

- `engine/src/types.rs` — UPDATE: `Tracker`, `Peer` records; `TorrentDetail` получил поля `trackers`/`peers`
- `engine/src/lib.rs` — UPDATE: `peer_speed_baseline` (вложенная карта, поле `Engine`), единый метод `build_peers` (замена `peer_download_speed_bps`+`prune_peer_speed_baselines`), `get_torrent_details` наполняет `trackers`/`peers` с проверкой `stats.live`/логированием неожиданных ошибок/сортировкой по адресу, `remove_torrent` чистит baseline за `O(1)`, 7 тестов (4 новых для Story 2.2 + 3 из code-review фикса, один переписан)
- `MyTorrent/Generated/engine.swift`, `engineFFI.h`, `module.modulemap` — REGENERATED
- `MyTorrent/Views/TorrentDetailView.swift` — UPDATE: `trackersList`/`peersList`/`trackerRow`/`peerRow` вместо `Color.clear`-заглушек; общий `rowList` helper; `peersList` ключуется по адресу, без колонки `state`
- `MyTorrent/Localizable.xcstrings` — UPDATE: новый ключ `torrent_detail.peers.empty`
- `MyTorrent.xcodeproj` — REGENERATED (`xcodegen generate`)
