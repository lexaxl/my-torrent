---
baseline_commit: 44d878cbcc4810bd94930e8e8aa514fe0a3acda1
---

# Story 4.2: Уведомление о завершении

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу получать системное уведомление, когда закачка завершается, даже если все окна закрыты,
чтобы знать об этом, не проверяя вручную.

## Acceptance Criteria

1. **Given** приложение работает в фоне (все окна закрыты), **when** закачка переходит в статус «завершено», **then** приходит системное macOS-уведомление с именем закачки.
2. Обнаружение перехода — через сравнение соседних снимков поллинга (AD-4: diff предыдущего снимка `AppModel.torrents` с текущим по каждому id), **без** отдельного callback-канала/нового FFI-вызова.
3. **Given** первый снимок после старта приложения (bootstrap-снимок AD-4, включая торренты, авто-восстановленные `librqbit` из своей сессии), **then** уведомление НЕ отправляется ни для одного торрента, уже находящегося в статусе «раздача» на этом первом снимке — только для реальных переходов, замеченных между двумя последовательными снимками.
4. **Given** пользователь запускает приложение впервые, **then** приложение запрашивает разрешение системы на показ уведомлений (`UNUserNotificationCenter`) один раз при старте; отказ не создаёт вылетов/ошибок — уведомления просто не показываются (без own error UI, тот же принцип, что и остальной проект).

## Scope Boundary

**Важное расхождение архитектуры с реальным движком, обнаруженное при создании истории:**

- **В движке (`engine/src/lib.rs`, `derive_status`) нет отдельного статуса «завершено» / «Completed-not-seeding».** Architecture Spine (`AD-4` Consistency Conventions) описывает `inactive = {Paused, Error, Completed-not-seeding}`, подразумевая, что «завершено, но не раздаётся» — отдельное состояние. По факту `derive_status` (строки 127-133) маппит `TorrentStatsState::Live if finished` **напрямую в `"seeding"`** — нет промежуточного «completed»/«finished-not-seeding» статуса вообще (подтверждено чтением исходников и юнит-тестом `derive_status_covers_every_state`). Это согласуется с тем, что фактическое ограничение раздачи по времени (Story 3.2, seed-duration enforcement) ещё не реализовано — движку буквально некуда положить «доскачал, но раздачу остановил» состояние, пока эта фича не появится отдельной историей. **Вывод: единственный наблюдаемый сигнал «завершения» сегодня — переход статуса торрента В `"seeding"` из любого НЕ-`"seeding"` статуса** (`"downloading"`, `"checking"`, `"resolving"`). Это и есть трактовка AC1/AC2 в данной истории — «завершено» ⇔ «стало раздавать» на текущей стадии проекта. Если в будущей истории появится настоящее «завершено-не-раздаёт» состояние — этот диффинг нужно будет расширить, не заменяя логику.
- **`resolving` статус (Story 1.2, магнит ещё не резолвится) НЕ может напрямую перейти в `seeding`** — реальный путь всегда `resolving → checking/downloading → seeding`, так что диффинг «предыдущий ≠ seeding, текущий = seeding» покрывает это естественно, без специальной обработки `resolving`.
- **Пограничный случай, оставленный как есть, а не переусложнённый**: если пользователь ставит на паузу уже полностью раздающийся торрент, затем возобновляет его, и он ненадолго проходит через `"checking"` перед повторным входом в `"seeding"` — уведомление придёт повторно (диффинг основан на переходе, а не на «первый раз в жизни торрента»). Ни один AC не запрещает этого явно, и это соответствует буквальной формулировке AD-4 («diffing consecutive snapshots... transition»), так что переусложнять специальным «уже раздавал когда-то» флагом не нужно.
- **Тестовое ограничение этой машины**: DNS на этой машине блокирует BitTorrent-домены (`torrent.ubuntu.com`, `router.bittorrent.com` — подтверждено в предыдущих историях, см. [[my-torrent-dev-environment]] project memory) — **реальное сквозное завершение закачки (получение пиров → докачка → переход в seeding) нельзя проверить вручную на этой машине**. Живая проверка ограничена: (а) запрос разрешения на уведомления при первом запуске (не требует сети), (б) корректность diff-логики через чтение кода/логирования, (в) явная рекомендация пользователю проверить реальное срабатывание на закачке, которая может фактически подключиться к пирам.
- **Не в этой истории**: настройка «включить/выключить уведомления» (не запрошено ни в одном AC, добавлять toggle без запроса — vague-implementation risk); уведомление о других переходах статуса (ошибка, пауза и т.д. — только «завершено» упомянуто в epics.md/FR7).

## Tasks / Subtasks

- [x] **Task 1: `AppModel.swift` — запрос разрешения на уведомления** (AC: 4)
  - [x] `import UserNotifications`
  - [x] В `setUpEngine(downloadDir:)` (после успешной инициализации `engine`, до или после bootstrap-снимка — порядок не критичен, т.к. это асинхронный fire-and-forget запрос) добавить:
    ```swift
    Task {
        do {
            let granted = try await UNUserNotificationCenter.current()
                .requestAuthorization(options: [.alert, .sound])
            logger.info("notification authorization granted: \(granted, privacy: .public)")
        } catch {
            logger.error("notification authorization request failed: \(String(describing: error), privacy: .public)")
        }
    }
    ```
    (не блокирует `setUpEngine`'s остальной ход — запрос идёт параллельно с bootstrap-снимком; результат только логируется, никакого user-facing UI/алерта об отказе — тот же no-user-facing-error-UI принцип, что уже задокументирован для `AppModel.addTorrent`/`mutate` в Story 3.1)

- [x] **Task 2: `AppModel.swift` — diff предыдущего/текущего снимка и отправка уведомления** (AC: 1, 2, 3)
  - [x] Новое приватное свойство:
    ```swift
    // nil до первого снимка — отличает "снимков ещё не было" (bootstrap, AC3: не
    // уведомлять о том, что уже раздавалось на момент старта) от "снимок был, но
    // пуст". Ключ — id торрента (infohash), значение — его status на прошлом снимке.
    private var previousStatusByID: [String: String]?
    ```
  - [x] В `refreshTorrents()`, **до** переприсваивания `torrents` новому значению (нужен доступ к старому `torrents` для сравнения) добавить diff-шаг:
    ```swift
    private func refreshTorrents() async {
        guard let engine else { return }
        let newTorrents = await Task.detached { engine.getAllTorrents() }.value
        detectCompletionsAndNotify(newTorrents: newTorrents)
        torrents = newTorrents
        startPollingIfNeeded()
    }

    // AD-4: "the shell detects a torrent's transition into Completed/Seeding by
    // diffing consecutive snapshots... No separate 'on complete' callback exists."
    // Движок не имеет отдельного "completed"-статуса (см. Scope Boundary) — переход
    // НЕ-seeding → seeding это и есть наблюдаемый сигнал завершения.
    private func detectCompletionsAndNotify(newTorrents: [TorrentStatus]) {
        defer {
            // Заменяем целиком (не мёржим) — естественно вычищает id удалённых
            // торрентов, тот же паттерн, что уже применяется на Rust-стороне для
            // per-peer baseline map (см. build_peers_prunes_baseline_for_disconnected_peers).
            previousStatusByID = Dictionary(uniqueKeysWithValues: newTorrents.map { ($0.id, $0.status) })
        }
        // Bootstrap-снимок (AD-4): ещё не с чем сравнивать — не уведомлять о том,
        // что уже раздавалось на момент запуска приложения (AC3).
        guard let previousStatusByID else { return }

        for torrent in newTorrents where torrent.status == "seeding" {
            if previousStatusByID[torrent.id] != "seeding" {
                postCompletionNotification(torrentName: torrent.name)
            }
        }
    }

    private func postCompletionNotification(torrentName: String) {
        let content = UNMutableNotificationContent()
        content.title = String(localized: "notification.download_complete.title")
        content.body = torrentName
        content.sound = .default
        let request = UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: nil)
        UNUserNotificationCenter.current().add(request) { error in
            if let error {
                logger.error("failed to post completion notification: \(String(describing: error), privacy: .public)")
            }
        }
    }
    ```
    (`trigger: nil` — показать немедленно, не по расписанию; `identifier: UUID().uuidString` — каждое уведомление уникально, повторное завершение того же торрента после паузы/резюме [см. Scope Boundary] не будет молча подавлено системой из-за одинакового identifier)
  - [x] Не менять сигнатуру/поведение существующих вызовов `refreshTorrents()` — вызывается из тех же мест (`setUpEngine`, `addTorrent`, `mutate`, `pollLoop`), diff просто добавляется внутрь

- [x] **Task 3: Локализация** (AC: 1)
  - [x] `MyTorrent/Localizable.xcstrings` — новый ключ:
    - `notification.download_complete.title` — RU «Закачка завершена» / EN «Download Complete»
  - [x] Имя торрента (`content.body`) — не локализуется (это данные, а не UI-текст), тот же принцип, что имена файлов/торрентов нигде в проекте не проходят через `.xcstrings`

- [x] **Task 4: Сборка, ручная проверка** (AC: 1-4)
  - [x] Новых `.swift`-файлов нет → `xcodegen generate` не обязателен, но прогнан на всякий случай (no-op)
  - [x] `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED
  - [x] Запустить приложение, подтвердить, что при первом запуске появляется системный запрос на разрешение уведомлений — **подтверждено скриншотом**: реальный системный диалог «Уведомления от «MyTorrent» — Уведомления могут содержать предупреждения, звуки и наклейки на значки приложений» появился при запуске
  - [~] Проверить логи на предмет `"notification authorization granted: true/false"` — **не подтверждено через `log show`**: `Logger.info` не персистентно (та же ситуация, что с существующим `"engine linked..."` info-логом в Story 1.1/1.3 — не появляется в `log show` без доп. флагов); процесс жив без крашей (`pgrep`), это лишь подтверждает, что путь кода выполнился без исключения — не более
  - [~] **Реальное завершение закачки → появление уведомления с именем — НЕ проверено на этой машине** (DNS блокирует BitTorrent-трекеры/пиров, см. Scope Boundary и [[my-torrent-dev-environment]]) — диалог разрешения на клик пользователю оставлен намеренно (реальное согласие пользователя на системном уровне — не то, что стоит кликать программно); рекомендация пользователю: нажать «Разрешить» в появившемся диалоге и проверить реальное срабатывание на закачке, которая может подключиться к пирам
  - [x] `cargo test` в `engine/` — 25/25 прошли (sanity-check, не требовалось по сути истории)

## Dev Notes

### Technical Requirements / Stack

- **`AD-4`** («Completion detection: the shell detects a torrent's transition into Completed/Seeding by diffing consecutive snapshots... fires the local `UNUserNotificationCenter` notification on that transition. No separate 'on complete' callback exists») — реализуется целиком в `AppModel.refreshTorrents()`/новом `detectCompletionsAndNotify`, единственном месте, где `torrents` обновляется.
- **`derive_status` (`engine/src/lib.rs:127-133`)** — движок не имеет отдельного статуса «завершено» (см. Scope Boundary); единственный наблюдаемый сигнал — переход в `"seeding"`.
- **`UNUserNotificationCenter`** — нативный Apple-фреймворк (`import UserNotifications`), локальные уведомления не требуют серверной инфраструктуры/push-токенов. Для несэндбоксированного/unsigned-приложения (`AD-10`, `CODE_SIGN_IDENTITY: "-"`) локальные уведомления в целом работают, если приложение — настоящий `.app`-бандл с валидным `CFBundleIdentifier` (уже есть, `com.alex.mytorrent.MyTorrent`) — никаких дополнительных `Info.plist`-ключей для локальных (не push) уведомлений не требуется.
- **`requestAuthorization` асинхронный и может показать системный диалог** — если пользователь уже отвечал раньше (grant/deny), повторный вызов не показывает диалог повторно, просто возвращает сохранённое решение — безопасно вызывать при каждом запуске приложения (не нужно flag в `UserDefaults`, чтобы «спросить один раз»).

### Architecture Compliance

- `AD-4` — полностью соблюдено: никакого нового FFI-вызова, никакого нового таймера/poll-cadence, diff происходит на существующем поллинг-цикле (`refreshTorrents`, вызываемом `pollLoop`/`addTorrent`/`mutate`/bootstrap).
- `AD-9` (FFI calls off main thread) — не затрагивается: эта история не добавляет новых FFI-вызовов, только читает уже полученный `newTorrents` (уже смаршален на `@MainActor` тем же путём, что раньше).
- Существующий no-user-facing-error-UI принцип (см. `AppModel.addTorrent`/`mutate`, задокументировано в Story 3.1) — соблюдён: отказ в разрешении на уведомления и ошибка `UNUserNotificationCenter.add` только логируются, никакого алерта/UI.

### Project Structure Notes

```
MyTorrent/
  AppModel.swift            # UPDATE — import UserNotifications, previousStatusByID,
                             #          detectCompletionsAndNotify, postCompletionNotification,
                             #          requestAuthorization в setUpEngine
  Localizable.xcstrings     # UPDATE — 1 новый ключ (notification.download_complete.title)
```
Нет новых файлов → `xcodegen generate` не требуется.

### Previous Story Intelligence (Story 4.1)

- **`/code-review high` перед коммитом обязателен** — тот же pipeline (8 параллельных finder-агентов + верификация), что использовался для всех историй с 3.1.
- **Не полагаться на предположения о поведении Apple-фреймворков без эмпирической проверки**, если возможно — Story 4.1's код-ревью изначально заподозрил, что `String(format: String(localized:), count)` не резолвит plural-формы правильно, но эмпирическая проверка (собранный тестовый бинарник против реально скомпилированного `.stringsdict`) показала, что предположение было ложным. Для этой истории: если есть сомнение в поведении `UNUserNotificationCenter`/`UNUserNotificationCenter.current().add` в несэндбоксированном/unsigned контексте — по возможности проверить эмпирически (запустить и понаблюдать системный диалог разрешения), а не полагаться только на документацию.
- **Safety-урок про `osascript`-автоматизацию** (Story 3.2/4.1) — для этой истории ручная UI-проверка ограничена системным диалогом разрешения уведомлений; кликать по самому системному диалогу разрешения программно не нужно/не стоит — пользователь либо уже дал разрешение раньше (тогда диалог не появится повторно), либо появится и потребует его собственного клика (системный диалог, не часть UI приложения).
- **Git**: последний коммит на `main` — `44d878c` (Story 4.1: иконка в меню-баре и popover). Ветка чистая (после коммита/пуша Story 4.1), готова для новой истории.

### Testing Standards

- Swift: формального UI-test framework в проекте нет (решение Story 1.1) — только ручная проверка (Task 4), с явным признанием, что часть проверки (реальное завершение закачки) невозможна на этой машине из-за DNS-блокировки BitTorrent-доменов.
- Rust: эта история не трогает `engine/` — `cargo test` не обязателен, но не будет лишним прогнать как sanity-check.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 4.2]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-4] — completion detection via diffing, no separate callback
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#Consistency Conventions] — active/inactive partition table (описывает "Completed-not-seeding", которого фактически нет в движке — см. Scope Boundary)
- [Source: engine/src/lib.rs#derive_status] (строки 127-133) — движок маппит `Live if finished` напрямую в `"seeding"`, без промежуточного статуса; проверено также юнит-тестом `derive_status_covers_every_state`
- [Source: MyTorrent/AppModel.swift] — существующий `refreshTorrents()`/`torrents`/поллинг-инфраструктура, переиспользуемая этой историей
- [Source: _bmad-output/implementation-artifacts/4-1-menu-bar-popover.md] — предыдущая история: код-ревью pipeline, урок про эмпирическую проверку вместо предположений об Apple-фреймворках
- project memory [[my-torrent-dev-environment]] — DNS блокирует BitTorrent-домены на этой машине, ограничивает живую проверку

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- Системный диалог запроса разрешения на уведомления появился при первом запуске приложения после этого изменения (подтверждено скриншотом: «Уведомления от «MyTorrent»...»). Диалог оставлен непринятым намеренно — это реальное согласие пользователя на системном уровне, кликать по нему программно не стал (см. Scope Boundary/project memory про риск автоматизации на реальном экране пользователя).
- `Logger.info("notification authorization granted: ...")` не появился в `log show` — ожидаемо, info-уровень логов не персистентен в этой среде без доп. флагов (та же картина наблюдалась для существующего `"engine linked..."` info-лога в более ранних историях) — не является признаком проблемы.

### Completion Notes List

- Все 4 задачи выполнены. `AppModel.swift`: `import UserNotifications`, запрос `requestAuthorization` в `setUpEngine` (fire-and-forget, без своего user-facing UI при отказе), новое `previousStatusByID: [String: String]?` и `detectCompletionsAndNotify`/`postCompletionNotification`, вызываемые из `refreshTorrents()` до переприсваивания `torrents`.
- **AC4 (запрос разрешения) подтверждён живым тестом** — реальный системный диалог появился.
- **AC1/AC2/AC3 (обнаружение перехода в «завершено» через diff, без уведомления на bootstrap-снимке) — подтверждены только чтением кода**, не живым тестом: `previousStatusByID` стартует `nil`, `detectCompletionsAndNotify` явно выходит из функции до диффинга, если `previousStatusByID == nil` (bootstrap), и только после этого заполняет его — гарантирует, что первый снимок никогда не порождает уведомлений. Диффинг сравнивает `previousStatusByID[id] != "seeding"` против нового статуса `"seeding"` — реальное сквозное завершение закачки (получение пиров → докачка → переход в seeding) не проверено вживую из-за DNS-блокировки BitTorrent-доменов на этой машине (см. Scope Boundary) — **рекомендация пользователю**: добавить реальный торрент, который может подключиться к пирам (например, в другой сети/VPN), дождаться завершения и убедиться, что уведомление с именем торрента появляется один раз.
- **Важная находка при создании истории, подтвердившаяся в реализации**: движок (`derive_status`, `engine/src/lib.rs`) не имеет отдельного статуса «завершено» — переход в `"seeding"` из любого другого статуса является единственным наблюдаемым сигналом завершения на данном этапе проекта (см. Scope Boundary).
- Изменений в `engine/` нет — `cargo test`: 25/25 прошли (sanity-check, не требовалось по сути истории).

### Code Review (`/code-review high`)

8 параллельных агентов-искателей (line-by-line, removed-behavior, cross-file tracer, reuse, simplification, efficiency, altitude, CLAUDE.md conventions) + верификация → 9 находок, все обработаны:

- **Исправлено (7):** диффинг больше не считает «завершением» первое же наблюдение id, которого раньше не было в `previousStatusByID` (закрывает дублирование уведомления при повторном добавлении того же infohash после удаления, и ложное срабатывание при добавлении уже полностью скачанного контента) — теперь требуется `if let previous = ..., previous != seedingStatus`; `refreshTorrents()` перестроен в FIFO-цепочку через новое `refreshChain: Task<Void, Never>?` — устраняет гонку при перекрывающихся вызовах (poll + пользовательское действие), которая могла откатить baseline к устаревшему снимку (независимо подтверждено 3 агентами); `postCompletionNotification` теперь строит identifier из id торрента (`"completion.\(torrentID)"`) вместо случайного `UUID()`; посылка уведомления переведена на `try await ... .add(request)` (do/catch), как везде в файле, вместо completion-handler closure; строка `"seeding"` вынесена в `private static let seedingStatus`; имя торрента в `content.body` обрезается до 200 символов; `ARCHITECTURE-SPINE.md`'s Consistency Conventions обновлена — убрано упоминание несуществующего статуса `Completed-not-seeding`, добавлена запись в Deferred с кросс-ссылкой на эту историю.
- **Осознанно не менялось, но задокументировано (2):** гонка «завершение закачки в узком окне между запросом и ответом на системный диалог разрешения» — новый комментарий объясняет, почему очередь/replay пропущенных уведомлений была бы избыточной инженерией для редкой гонки в hobby-приложении; отсутствие батчинга при одновременном завершении нескольких закачек в один poll-тик — новый комментарий объясняет, что это принятый UX-компромисс, не баг корректности.

Билд после фиксов: `xcodegen generate` + `xcodebuild` — BUILD SUCCEEDED (в т.ч. подтвердил, что async-throwing `UNUserNotificationCenter.add(_:)` компилируется на macOS 13+ target). `cargo test`: 25/25. Приложение перезапущено, живо (без крашей, подтверждено `pgrep`+отсутствием новых crash-репортов).

### File List

- `MyTorrent/AppModel.swift` — UPDATE: `import UserNotifications`; запрос `requestAuthorization` в `setUpEngine`; новое `previousStatusByID`; `refreshChain`-based сериализация `refreshTorrents()`/`performRefresh()`; `detectCompletionsAndNotify`/`postCompletionNotification` (с фиксами код-ревью выше); новый `seedingStatus` static-константа
- `MyTorrent/Localizable.xcstrings` — UPDATE: новый ключ `notification.download_complete.title`
- `_bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md` — UPDATE (код-ревью): Consistency Conventions строка про active/inactive partition исправлена, новая запись в Deferred про `Completed-not-seeding`
