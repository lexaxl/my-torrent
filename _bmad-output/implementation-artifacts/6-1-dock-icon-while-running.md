---
baseline_commit: 26b78f331c6ab873f743eec9a6e9b0eda8da2eb5
---

# Story 6.1: Иконка в Dock во время работы

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу, чтобы приложение показывало свою иконку в Dock, пока оно запущено,
чтобы видеть, что оно работает, переключаться на него и открывать его окно так же, как у любого другого Mac-приложения.

## Acceptance Criteria

1. **Given** приложение запущено, **then** иконка-насос видна в Dock всё время работы (и в Cmd+Tab), включая состояние «все окна закрыты, закачки идут в фоне» — `LSUIElement` убран из `Info.plist` (статический вариант, решение brainstorming-сессии 2026-07-30; НЕ `setActivationPolicy` на лету).
2. **Given** все окна закрыты, приложение работает, **when** кликаю по иконке в Dock, **then** открывается и активируется главное окно.
3. **Given** закрываю все окна, **then** приложение продолжает работать, закачки не прерываются (ядро AD-7 сохраняется — `applicationShouldTerminateAfterLastWindowClosed` остаётся `false`).
4. **Given** есть ≥1 незавершённая закачка (downloading/checking/resolving — **не** seeding), **when** выхожу любым путём — Cmd+Q, ПКМ по Dock-иконке → «Завершить», кнопка Quit в попапе меню-бара, — **then** перед выходом показывается диалог подтверждения; «Отмена» оставляет приложение работать, подтверждение завершает его.
5. **Given** незавершённых закачек нет (пусто, всё на паузе или только раздачи), **when** выхожу любым из тех же путей, **then** приложение завершается сразу, без диалога.
6. **And** иконка меню-бара, тултип и popover не меняются — Dock и меню-бар сосуществуют.
7. **And** `ARCHITECTURE-SPINE.md` AD-7 переписан (accessory → regular app; «закрытие окон никогда не завершает процесс» и «не sandboxed» сохраняются), устаревший комментарий у Quit-кнопки в `MenuBarView` исправлен.
8. **And** строки диалога локализованы через существующий `Localizable.xcstrings`-паттерн (ru source + en).

## Scope Boundary

- **Session persistence — ЯВНО ВНЕ СКОУПА.** Подтверждено чтением вендоренного librqbit-8.1.1 (`session.rs`: `Session::new` → `SessionOptions::default()`, а `SessionOptions` — `#[derive(Default)]` → `persistence: None`, `fastresume: false`): список торрентов **не восстанавливается** между запусками приложения. Поэтому текст диалога подтверждения обязан говорить «прервать закачки», а НЕ обещать «продолжатся при следующем запуске» — это было бы ложью. Включение persistence — отдельное продуктовое решение (кандидат в `deferred-work.md`), не часть этой истории.
- Настройка «прятать из Dock» — не делаем (YAGNI, решение brainstorming-сессии: статический вариант A, а не динамический `setActivationPolicy`).
- Убирание/опционализация меню-бара — не делаем (решение той же сессии: сосуществуют, как Docker Desktop/Telegram).

## Tasks / Subtasks

- [x] **Task 1: Убрать `LSUIElement` из `Info.plist`** (AC: 1)
  - [x] Удалить пару `<key>LSUIElement</key><true/>` (строки 27-28). Новых файлов нет → `xcodegen generate` НЕ нужен (нужен только при добавлении файлов, Story 2.1).
  - [x] Собрать, запустить, эмпирически подтвердить: иконка-насос в Dock и Cmd+Tab; у приложения появилось стандартное SwiftUI-меню (App menu с Quit ⌘Q — приходит бесплатно от SwiftUI `Window`/`WindowGroup`-сцен).
- [x] **Task 2: Клик по Dock-иконке при закрытых окнах открывает главное окно** (AC: 2)
  - [x] СНАЧАЛА эмпирически проверить дефолт: у SwiftUI-приложений reopen часто работает из коробки (AppKit reopen-событие → SwiftUI восстанавливает `Window`-сцену). Если работает — кода не писать, зафиксировать в Dev Notes.
  - [x] Если НЕ работает: `applicationShouldHandleReopen(_:hasVisibleWindows:)` в `AppDelegate` — при `hasVisibleWindows == false` открыть главное окно через closure-паттерн `onOpenURLs` (см. Dev Notes → wiring pitfall) и вернуть `false`; иначе вернуть `true`.
- [x] **Task 3: Подтверждение выхода при незавершённых закачках** (AC: 4, 5)
  - [x] Новое computed-свойство `AppModel.hasUnfinishedDownloads`: статусы `.downloading`/`.checking`/`.resolving` через `TorrentEngineStatus` (НЕ raw-строки — правило Story 5.1). Обязателен doc-комментарий, почему это ДРУГОЕ множество, чем `activeStatuses` (то включает `.seeding` для поллинга/меню-бара; здесь seeding не повод для диалога — иначе диалог показывался бы почти всегда).
  - [x] `AppDelegate.applicationShouldTerminate(_:)`: если незавершённых нет → `.terminateNow`; иначе `NSApp.activate(ignoringOtherApps: true)` (диалог не должен оказаться позади чужих окон), `NSAlert.runModal()`, по результату `.terminateNow`/`.terminateCancel`.
  - [x] Wiring: `AppDelegate` получает `var hasUnfinishedDownloads: (() -> Bool)?` (тот же closure-паттерн, что `onOpenURLs`); `nil` → вести себя как «нет закачек» (не блокировать выход). Устанавливать из ВСЕГДА живой вьюхи — см. Dev Notes → wiring pitfall.
  - [x] Проверить все три пути выхода: Cmd+Q, ПКМ по Dock → «Завершить», Quit в попапе меню-бара (`NSApp.terminate(nil)` — уже идёт через `applicationShouldTerminate`, код кнопки менять не надо).
- [x] **Task 4: Локализация диалога** (AC: 8)
  - [x] Ключи по устоявшемуся паттерну `section.element`: `quit_confirm.title`, `quit_confirm.message`, `quit_confirm.quit_button`, `quit_confirm.cancel_button` (ru source + en, как остальные 41 ключ). В `NSAlert` — через `String(localized:)`.
  - [x] Тон (UX-DR9, Calm Under Errors / Plain & Direct): title «Идут незавершённые закачки», message «Выход прервёт их. Завершить приложение?» (без обещания автопродолжения — см. Scope Boundary), кнопки «Завершить» / «Отмена». EN: "Downloads in progress" / "Quitting will interrupt them. Quit anyway?" / "Quit" / "Cancel".
- [x] **Task 5: Документация** (AC: 7)
  - [x] `ARCHITECTURE-SPINE.md` AD-7: переписать Rule — regular app (Dock-иконка во время работы), закрытие всех окон по-прежнему никогда не завершает процесс, выход — явный (Cmd+Q/Dock/меню-бар) с подтверждением при незавершённых закачках; «не sandboxed» без изменений. Обновить и Prevents-строку (она ссылается на «runs quietly in the background» — смягчить до «работает в фоне с видимой Dock-иконкой»).
  - [x] `MenuBarView.swift` строки 69-72: комментарий «единственный способ выйти» больше не верен — переписать (кнопка остаётся как удобный выход из попапа, но теперь есть и штатные пути).
  - [x] `deferred-work.md`: закрыть/отметить запись про Dock-иконку (строка ~41, «Deferred from: Story 5.4») как реализованную этой историей; добавить новую запись-кандидата «session persistence» (см. Scope Boundary).
- [x] **Task 6: Сборка и ручная верификация** (AC: все)
  - [x] `./scripts/build-engine.sh` НЕ нужен (Rust не трогаем) — только `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build`.
  - [x] Чеклист: Dock-иконка видна; закрыть все окна → процесс жив, иконка в Dock осталась; клик по Dock → главное окно открылось; добавить fixture-торрент (`engine/tests/fixtures/test.torrent`) → Cmd+Q → диалог; «Отмена» → работает дальше; убрать/дождаться закачку → Cmd+Q → тихий выход; Quit из меню-бар-попапа при закачке → тот же диалог.
  - [x] Перед добавлением fixture: удалить leftover `~/Downloads/Torrents/test.txt`, иначе add молча упадёт (`allow_overwrite = false` — грабли Story 5.1/5.2). Реальные торренты пользователя в `~/Downloads/Torrents/` (Queen, Ubuntu ISO) НЕ трогать.
  - [x] Экранную автоматизацию (клики/клавиши) — только спросив пользователя (правило Story 3.2/5.2). Cmd+Q слать только когда MyTorrent подтверждённо frontmost.

## Dev Notes

### Technical Requirements / Stack

- Чистый AppKit/SwiftUI, стабильные API (`NSApplicationDelegate`-хуки, `NSAlert`) — web-research новых версий не требуется. Rust/`engine/`/FFI не затрагиваются вообще.
- **Wiring pitfall (главная техническая ловушка истории):** closure'ы `AppDelegate` должны работать, когда главное окно ЗАКРЫТО. `onOpenURLs` сейчас ставится в `MainWindowView.onAppear` — для reopen/terminate это ненадёжное место (окно может быть закрыто, а `@Environment(\.openWindow)`, захваченный из демонтированной вьюхи, — недокументированная территория). Всегда живая вьюха в этом приложении — **label `MenuBarExtra`** (рендерится постоянно, в отличие от `MenuBarView`-попапа, живущего только пока открыт). Ставить closures в `.onAppear` label'а (в `MyTorrentApp.swift`), где `@Environment(\.openWindow)` и `appModel` оба доступны. Для открытия окна использовать существующий `AppModel.activateAndOpenWindow(id: WindowID.main, openWindow:)` — не дублировать `NSApp.activate` + `openWindow` руками (правило «второе появление → извлекай» уже отработало в Story 4.x).
- `applicationShouldTerminate` вызывается и при logout/shutdown macOS — модальный `NSAlert` там стандартен для приложений с несохранённой работой, отдельной обработки не делать.
- `NSAlert`: `alertStyle = .warning`, первая кнопка (`addButton`) — «Завершить» (default), вторая — «Отмена» (Esc). `runModal()` синхронен — это ок, `applicationShouldTerminate` приходит на главном потоке.

### Architecture Compliance

- **AD-7 — ЭТА ИСТОРИЯ ЕГО ПЕРЕСМАТРИВАЕТ** (утверждено пользователем на brainstorming-сессии 2026-07-30). Ядро сохраняется: закрытие окон не завершает процесс (`applicationShouldTerminateAfterLastWindowClosed` → `false` НЕ трогать — см. его комментарий: `LSUIElement` сам по себе никогда и не был причиной выживания процесса, Story 1.3 Task 6), not sandboxed. Меняется: accessory → regular. Spine обновить в Task 5 — код и архитектурный документ не должны разъехаться.
- AD-3 (SwiftUI throughout) — `NSAlert`/делегатные хуки не AppKit-вью в интерфейсе, а системные диалоги/lifecycle: тот же прецедент, что `NSOpenPanel` в `SettingsView` (Story 3.1) и `NSApp.activate` в `AppModel`. Не нарушение.
- AD-4 (поллинг) — не затрагивается: поллинг живёт в `AppModel` (app-lifetime), от окон и активационной политики не зависит.
- AD-8/AD-9 — не применимы (мутаций движка и новых FFI-вызовов нет; `hasUnfinishedDownloads` читает уже опрошенный `torrents`-снапшот на главном акторе).

### Project Structure Notes

```
MyTorrent/
  Info.plist                    # UPDATE — удалить LSUIElement
  MyTorrentApp.swift            # UPDATE — AppDelegate: applicationShouldTerminate (+ applicationShouldHandleReopen, если дефолт не работает); wiring в label MenuBarExtra
  AppModel.swift                # UPDATE — hasUnfinishedDownloads рядом с hasActiveTorrents/activeStatuses
  Views/MenuBarView.swift       # UPDATE — только комментарий у Quit-кнопки
  Localizable.xcstrings         # UPDATE — 4 ключа quit_confirm.*
_bmad-output/planning-artifacts/architecture/.../ARCHITECTURE-SPINE.md  # UPDATE — AD-7
_bmad-output/implementation-artifacts/deferred-work.md                  # UPDATE — закрыть Dock-запись, добавить persistence-кандидата
```
Новых `.swift`-файлов нет → `xcodegen generate` не требуется.

### Previous Story Intelligence

- **Story 5.4:** Dock/Finder-иконка для многократно пересобираемого DerivedData-пути может показываться из устойчивого стале-кэша Finder — если Dock показывает НЕ насос, сначала скопировать свежий `.app` на новый путь (например `~/Desktop`) и проверить там, прежде чем подозревать код. Иконка запущенного приложения в Dock обычно берётся из бандла напрямую и должна быть корректной — но помнить про этот конфаунд.
- **Story 5.4:** в label `MenuBarExtra` — только `Color(nsColor:)`-обёртки, не голые SwiftUI semantic-цвета. Эта история label не меняет (только вешает `.onAppear`-wiring), но правило действует, если что-то придётся трогать.
- **Story 4.1/5.4:** меню-бар этой машины переполнен — «не вижу иконку меню-бара» не значит «сломано»; проверять baseline-видимость или AX-запрос (Story 5.3). К Dock-иконке не относится (Dock не переполняется), но помнить при проверке AC6.
- **Story 3.2/5.2 (безопасность):** экранная автоматизация идёт на реальном экране пользователя — спрашивать перед кликами/клавишами; `screencapture` меню-бара ненадёжен, Dock скриншотится нормально.
- **Story 5.1:** любые сравнения `torrent.status` — только через `TorrentEngineStatus`/`.engineStatus`, никогда raw-строки.

### Testing Standards

- Swift: формального UI-test-фреймворка нет (решение Story 1.1) — ручная/эмпирическая проверка по чеклисту Task 6.
- Rust: `engine/` не трогаем — `cargo test` не обязателен (но и не сломается: изменений нет).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 6.1] — AC-источник
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-7] — пересматриваемое решение
- [Source: MyTorrent/Info.plist:27-28] — удаляемый ключ
- [Source: MyTorrent/MyTorrentApp.swift:48-53] — `applicationShouldTerminateAfterLastWindowClosed`, НЕ трогать
- [Source: MyTorrent/AppModel.swift:37-56] — `activeStatuses`/`hasActiveTorrents`/`activeTorrents`, образец для `hasUnfinishedDownloads`
- [Source: MyTorrent/AppModel.swift:83-89] — `activateAndOpenWindow`, переиспользовать для reopen
- [Source: MyTorrent/Views/MenuBarView.swift:69-75] — Quit-кнопка + устаревший комментарий
- [Source: ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/librqbit-8.1.1/src/session.rs:381-397,487-489] — `SessionOptions` `#[derive(Default)]` → `persistence: None` — обоснование текста диалога
- [Source: _bmad-output/implementation-artifacts/deferred-work.md:41] — исходная отложенная запись про Dock-иконку

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (Claude Code)

### Debug Log References

- **Task 2 разрешился лучшим исходом: `applicationShouldHandleReopen` НЕ понадобился.** Эмпирически подтверждено на реальной сборке: после закрытия главного окна (процесс жив, окон ноль) клик по Dock-иконке сам открыл главное окно с сохранившимся содержимым списка — дефолтный SwiftUI-reopen для `Window`-сцены работает из коробки. Кода для AC2 нет вообще; wiring из label `MenuBarExtra` в итоге нужен только для `hasUnfinishedDownloads`-closure.
- **Верификация почти вся выполнена без синтетических кликов**: Dock-иконка — read-only AX-запрос к процессу Dock; добавление торрента — `open test.torrent` (file-open flow); выход — AppleEvent `quit` (идёт ровно через `applicationShouldTerminate`, тот же funnel, что Cmd+Q и `NSApp.terminate`). Синтетические клики (кнопки диалога, close-кнопка окна, Dock-иконка) — только после явного разрешения пользователя (правило Story 3.2/5.2 соблюдено).
- **Первый прогон Cancel-теста прервался неожиданным исчезновением процесса — оказалось не регрессией этой истории**: свежие крэш-репорты 15:31:11/15:31:14 показали `fatalError` в `AppModel.setUpEngine` (инициализация движка) на **старте** двух перезапусков подряд — идентичный стек уже есть в крэшах 13:54/14:09/14:11 того же дня, ДО изменений Story 6.1. Направленное воспроизведение (мгновенный перезапуск пустой сессии; перезапуск через ~2с после выхода с активной закачкой) не удалось — оба прошли чисто. Задокументировано в `deferred-work.md` как существующий невоспроизведённый стартовый крэш, актуальность которого выросла с появлением Dock-иконки (цикл «выход → тут же клик по иконке» станет обычным).
- Локализация: первая вставка ключей в `Localizable.xcstrings` пересортировала весь файл (он не был отсортирован) — дифф 360 строк; откачено и переделано минимально (64 строки вставки, порядок файла сохранён, `quit_confirm.*` рядом с `menu_bar.popover.quit`).
- SourceKit-диагностика «Cannot find type ...» во время правок — шум одиночного парса без индекса проекта; `xcodebuild` собрал без ошибок и предупреждений по изменённым файлам.

### Completion Notes List

- **AC1 — подтверждено**: `LSUIElement` удалён из `Info.plist` (и подтверждённо отсутствует в собранном бандле через `plutil`); иконка MyTorrent присутствует в Dock во время работы (read-only AX-запрос к Dock), включая состояние «окон нет, закачка идёт».
- **AC2 — подтверждено, без кода**: дефолтный SwiftUI-reopen открывает главное окно по клику в Dock (см. Debug Log).
- **AC3 — подтверждено**: закрытие главного окна кнопкой close → процесс жив, торрент в списке при последующем reopen.
- **AC4 — подтверждено оба исхода**: с закачкой в статусе «Скачивается» AppleEvent `quit` показал диалог с корректными ru-строками (приложение НЕ вышло, `runModal` блокирует); «Отмена» → приложение работает, торрент на месте; «Завершить» → процесс завершился. Все три пользовательских пути (Cmd+Q / Dock → Завершить / Quit в попапе) идут через единственный хук `applicationShouldTerminate` — верифицированный AppleEvent-путь и есть этот хук.
- **AC5 — подтверждено для случая «пусто»**: AppleEvent `quit` при пустом списке → тихий выход без диалога. Случаи «всё на паузе»/«только раздачи» отдельно не прогонялись эмпирически (нужен докачанный/паузированный торрент) — покрыты определением множества `unfinishedStatuses` (`.checking/.downloading/.resolving`), не включающего `.paused`/`.seeding`.
- **AC6 — подтверждено**: после всех изменений menu-bar item жив и отдаёт свой тултип через AX `help`-атрибут (техника Story 5.3); код label/попапа не менялся (только `.onAppear`-wiring и комментарий).
- **AC7 — сделано**: AD-7 переписан (regular app; ядро — «закрытие окон не завершает процесс», not sandboxed — сохранено, помечен revised 2026-07-30 Story 6.1), обновлены обе перекрёстные ссылки на accessory-lifecycle (AD-4 Ownership-note, Structural Seed) и комментарий Quit-кнопки в `MenuBarView`.
- **AC8 — сделано**: 4 ключа `quit_confirm.*` (ru+en) в `Localizable.xcstrings`, читаются через `String(localized:)`; ru-строки подтверждены на реальном диалоге.
- Тестов нет по конвенции проекта: Swift-тест-таргет отсутствует (решение Story 1.1), Rust не тронут (`cargo test` прогнан для полноты: 25/25 зелёные).
- **`/code-review high` (8 finder-углов → 25 кандидатов → дедуп до 11 → верификация): 9 находок (5 CONFIRMED, 4 PLAUSIBLE); все 5 CONFIRMED исправлены той же сессией по подтверждению пользователя:**
  - **Порядок кнопок диалога**: «Завершить» была default-кнопкой (Return) диалога защиты от потери данных — теперь «Отмена» добавляется первой (default/rightmost), «Завершить» второй; маппинг `runModal` инвертирован так, что terminate требует точно `.alertSecondButtonReturn`, любой другой ответ — безопасный `.terminateCancel`. Подтверждено на реальной сборке: AX-атрибут `AXDefaultButton` = «Отмена», клик «Завершить» завершает приложение.
  - **Классификация статусов переехала на enum**: оба set-литерала (`activeStatuses`, `unfinishedStatuses`) из `AppModel` удалены; вместо них — `TorrentEngineStatus.isActive`/`.blocksQuit`, exhaustive `switch` без `default` (новый case енума теперь ломает сборку в этих двух switch'ах, пока не будет явно классифицирован — та гарантия, ради которой enum создавался в Story 5.1; set-литерал её молча обходил).
  - **Устаревший комментарий** «an LSUIElement accessory app» у `activateAndOpenWindow` переписан под regular-app реальность.
  - **Новый `MyTorrent/Alerts.swift`** (`Alerts.runWarning(title:message:buttons:)`) — общий дом для warning-`NSAlert`-boilerplate'а (вторая копия появлялась в `applicationShouldTerminate`; первая — `SettingsView.showNotWritableAlert`, переведена на хелпер). Doc-комментарий хелпера фиксирует правило «безопасная кнопка — первой» для destructive-подтверждений.
  - **Новый `AppModel.activateApp()`** — единственный дом для `NSApp.activate(ignoringOtherApps:)`-идиомы (вторая inline-копия появлялась в `applicationShouldTerminate`; API deprecated на macOS 14+ — миграция теперь в одном месте).
- **PLAUSIBLE-находки, сознательно НЕ исправленные (решение пользователя «исправлять подтверждённые»), задокументированы для будущего**: (1) wiring quit-предиката через `.onAppear` label'а MenuBarExtra может молча не сработать при переполненном меню-баре (fail-open → фича отключена; глубокий фикс — AppDelegate владеет ссылкой на AppModel напрямую); (2) нет re-entrancy-guard'а вокруг `runModal` (повторный quit из меню-бара во время диалога → вложенная модальная сессия); (3) windowless-quit-алерт теоретически может оказаться позади frontmost-приложения (modal-panel level обычно спасает); (4) closure пересекает @MainActor-изоляцию — сломается при переходе на Swift 6 strict concurrency (вместе с ровно таким же существующим `onOpenURLs`).

### File List

- `MyTorrent/Info.plist` — UPDATE: удалён `LSUIElement`
- `MyTorrent/AppModel.swift` — UPDATE: `hasUnfinishedDownloads`; после ревью — set-литералы заменены на `engineStatus.isActive`/`.blocksQuit`, новый `activateApp()`-хелпер, актуализирован комментарий `activateAndOpenWindow`
- `MyTorrent/MyTorrentApp.swift` — UPDATE: `AppDelegate.hasUnfinishedDownloads`-closure + `applicationShouldTerminate` (подтверждение через `Alerts.runWarning`, «Отмена» — default); `.onAppear`-wiring на label `MenuBarExtra`; актуализирован комментарий `applicationShouldTerminateAfterLastWindowClosed`
- `MyTorrent/Models/TorrentEngineStatus.swift` — UPDATE (ревью): `isActive`/`blocksQuit` exhaustive-switch-свойства
- `MyTorrent/Alerts.swift` — NEW (ревью): общий `Alerts.runWarning`-хелпер
- `MyTorrent/Views/SettingsView.swift` — UPDATE (ревью): `showNotWritableAlert` переведён на `Alerts.runWarning`
- `MyTorrent/Views/MenuBarView.swift` — UPDATE: только комментарий у Quit-кнопки
- `MyTorrent/Localizable.xcstrings` — UPDATE: 4 ключа `quit_confirm.*` (ru/en)
- `MyTorrent.xcodeproj` — regenerated via `xcodegen generate` (новый `Alerts.swift`; сам `.xcodeproj` не коммитится)
- `_bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md` — UPDATE: AD-7 переписан, 2 перекрёстные ссылки обновлены
- `_bmad-output/implementation-artifacts/deferred-work.md` — UPDATE: Dock-запись закрыта; добавлены кандидаты «session persistence» и «стартовый fatalError-крэш setUpEngine»
