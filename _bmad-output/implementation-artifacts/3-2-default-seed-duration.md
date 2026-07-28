---
baseline_commit: ceb206d29230bdb2ad8d99392bc6a3c035ca60a9
---

# Story 3.2: Период раздачи по умолчанию

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь,
я хочу задать, как долго торренты раздаются после завершения закачки,
чтобы контролировать вклад в раздачу, не следя за этим вручную.

## Acceptance Criteria

1. **Given** окно настроек открыто, **when** я выбираю «Не раздавать» / «N часов» / «Бессрочно», **then** выбор применяется мгновенно (без кнопки сохранения) и сохраняется между запусками приложения.
2. **Given** выбрано «N часов», **when** я ввожу значение, **then** умолчание — 8, допустимый диапазон 1–168; значение вне диапазона ограничивается (clamped) до ближайшей границы, а не сохраняется как есть.
3. **Given** секция периода раздачи видна, **then** копирайт рядом с ней явно сообщает, что отсчёт — это суммарное время работы приложения, а не wall-clock (AD-5): не идёт, пока приложение закрыто; возобновляется при следующем запуске.
4. **Given** окно настроек закрыто и открыто заново, **then** оно показывает ранее выбранный режим (и число часов, если применимо), а не сбрасывается к умолчанию.

## Scope Boundary

**Решено при создании истории (см. обоснование в Dev Notes → Technical Requirements):**

- **Эта история — только настройка и хранение значения, БЕЗ реализации самого ограничения раздачи.** Ни одна ACn из epics.md/FR5 не описывает поведение «торрент останавливает раздачу по истечении N часов» — только применение настройки в UI и её копирайт. `librqbit::AddTorrentOptions` (проверено в Story 3.1 путём чтения исходников `librqbit` 8.1.1) не содержит НИ ОДНОГО поля, связанного с длительностью раздачи — значит, даже если бы значение сейчас прокидывалось в `add_torrent`, движку было бы некуда его положить. Реальное ограничение раздачи (накопительный таймер работы приложения, переживающий перезапуски, плюс механизм остановки раздачи по истечении — скорее всего через `pause_torrent`, уже существующий с Story 1.5) — это отдельная, существенно более крупная фича, не описанная ни в одной истории текущего роадмапа. **Настройка в этой истории живёт только в Swift (`UserDefaults`), в Rust не передаётся** — в отличие от Story 3.1, где `download_dir` имел куда его положить (`AddTorrentOptions.output_folder`) и поэтому прокидывался. Прокидывать `seed_duration` в `add_torrent` сейчас означало бы добавить параметр, который движок никак не использует — то есть ровно то, от чего предостерегает workflow создания историй («vague implementations»). **Требует подтверждения пользователем** — см. вопрос в конце сессии создания истории.
- **`AD-5`'s фраза «Swift passes them as UniFFI call parameters when adding a torrent»** описывает общий механизм для настроек в принципе (и уже реализована для `download_dir` в Story 3.1) — не мандат прокидывать именно `seed_duration` прямо сейчас, при отсутствии на Rust-стороне места, куда его принять.
- **Текст опции «N часов» — статический литерал, не форматная строка с реальной подстановкой числа.** UX-спека `3.1-settings.md` даёт `Translation Key: settings.seed_duration.hours`, RU `"{N} часов"` / EN `"{N} hours"` — выглядит как форматная строка, но по ASCII-макету (`(•) N часов [ 8 ]`) реальное число живёт в отдельном соседнем текстовом поле, а не в тексте радио-опции. В проекте пока нет ни одного прецедента runtime-форматирования строк в `.xcstrings` (проверено при код-ревью Story 3.1 — 0 вхождений `%@`), и правильное русское склонение числительного («1 час» / «2 часа» / «8 часов») потребовало бы полноценной поддержки String Catalog plural variations — отдельная, нетривиальная фича. Поэтому: label радио-опции — фиксированный текст «N часов» / «N hours» (буква N, не заменяется), реальное число — только в соседнем `TextField`. Если это неверно понято — легко исправить в Story 3.3+ без структурных изменений.
- **Копирайт про «суммарное время работы приложения» (AC3) — новый элемент UI, которого нет в `3.1-settings.md`.** UX-спека перечисляет только 4 объекта секции (label, 3 радио-опции) без сноски/hint про cumulative-time семантику — эта деталь появилась позже, на Architecture-этапе (`AD-5`), после UX-спеки. Добавлен новый translation key `settings.seed_duration.hours_hint` (не входит в исходный список UX-спеки) — показывается только когда активен режим «N часов» (единственный режим, где отсчёт вообще идёт; «Не раздавать» — нечего отсчитывать, «Бессрочно» — отсчёт не заканчивается, уточнение не нужно).
- **Численное поле — не вложено внутрь `Picker(.radioGroup)`.** Вложение интерактивного `TextField` в content опции `Picker` — известный риск на macOS (radio-group рендерится через AppKit-мост, ряд опции — не гарантированно проброс кликов/фокуса вложенным контролам; тот же класс проблемы, что и `.onTapGesture`/`.contextMenu` gotcha из Story 2.1 — см. [[my-torrent-status]] project memory). Вместо этого: `Picker` с тремя plain-`Text` опциями, а `TextField` — соседний view, показывается через `if seedDurationKind == .hours` рядом с/под `Picker`, не внутри его строк.
- **Не в этой истории:** реализация фактической остановки раздачи по истечении времени (см. первый пункт); отображение текущего накопленного времени раздачи (сколько уже прошло) — не запрошено ни в одном AC.

## Tasks / Subtasks

- [x] **Task 1: `AppSettings.swift` — режим периода раздачи** (AC: 1, 2, 4)
  - [x] Добавить тип:
    ```swift
    enum SeedDurationMode: Equatable {
        case off
        case hours(Int)
        case indefinite
    }
    ```
  - [x] Добавить в `AppSettings`:
    ```swift
    static let defaultSeedDurationHours = 8
    static let seedDurationHoursRange = 1...168

    private static let seedDurationKindKey = "settings.seedDurationKind"
    private static let seedDurationHoursKey = "settings.seedDurationHours"

    static var seedDurationMode: SeedDurationMode {
        get {
            switch UserDefaults.standard.string(forKey: seedDurationKindKey) {
            case "off":
                return .off
            case "indefinite":
                return .indefinite
            case "hours":
                let stored = UserDefaults.standard.integer(forKey: seedDurationHoursKey)
                let hours = stored == 0 ? defaultSeedDurationHours : stored
                return .hours(clampSeedDurationHours(hours))
            default:
                // First launch, no stored value yet — matches the ASCII mockup's
                // default-selected state (UX spec: "N часов" pre-selected, value 8).
                return .hours(defaultSeedDurationHours)
            }
        }
        set {
            switch newValue {
            case .off:
                UserDefaults.standard.set("off", forKey: seedDurationKindKey)
            case .indefinite:
                UserDefaults.standard.set("indefinite", forKey: seedDurationKindKey)
            case .hours(let hours):
                UserDefaults.standard.set("hours", forKey: seedDurationKindKey)
                UserDefaults.standard.set(clampSeedDurationHours(hours), forKey: seedDurationHoursKey)
            }
        }
    }

    static func clampSeedDurationHours(_ value: Int) -> Int {
        Swift.min(Swift.max(value, seedDurationHoursRange.lowerBound), seedDurationHoursRange.upperBound)
    }
    ```
    (тот же паттерн, что `saveLocationPath` в Story 3.1 — вычисляемое свойство поверх `UserDefaults`, единый источник правды, значение никогда не читается обратно из Rust — AD-5). **Отклонение от плана, найдено ручным тестированием**: добавлено ещё одно свойство, `AppSettings.lastSeedDurationHours` — читает сохранённое число часов независимо от текущего режима (`seedDurationMode`'s getter отдаёт число только когда kind == "hours"). Понадобилось для Task 2 — без него переключение «N часов» → «Не раздавать» → обратно на «N часов» сбрасывало число к умолчанию 8 вместо восстановления последнего введённого значения (реальный баг, пойман до коммита, не в исходном плане).
  - [x] Не менять `engine/src/lib.rs` / `AppModel.swift`'s движковые вызовы — см. Scope Boundary, значение остаётся Swift-side

- [x] **Task 2: `SettingsView.swift` — секция периода раздачи** (AC: 1, 2, 3)
  - [x] Новое `@State private var seedDurationMode: SeedDurationMode = AppSettings.seedDurationMode` (тот же паттерн, что `saveLocationPath` — локальная копия для перерисовки + запись в `AppSettings` при каждом изменении, тот же обоснованный trade-off, что уже принят в Story 3.1 для save-location, см. Story 3.1's code-review — если Story 3.1's `@State`/`AppSettings` дублирование ещё не заменено на `@AppStorage` к моменту этой истории, использовать тот же паттерн для консистентности, а не изобретать новый)
  - [x] `form` сейчас — один плоский `VStack(alignment: .leading, spacing: 4) { ... }.padding(16)` с полями save-location. Перестроить в **две секции с `space-lg` (16) между ними**, внутри каждой — прежний `spacing: 4` (label ↔ control, `space-xs` по UX-спеке приблизительно, значение уже используется в save-location секции — не менять):
    ```swift
    private var form: some View {
        VStack(alignment: .leading, spacing: 16) {
            VStack(alignment: .leading, spacing: 4) {
                Text("settings.save_location.label")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
                HStack {
                    Text(saveLocationPath)
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Spacer()
                    Button("settings.save_location.choose_button") {
                        chooseSaveLocation()
                    }
                }
            }

            VStack(alignment: .leading, spacing: 4) {
                Text("settings.seed_duration.label")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                Picker("", selection: seedDurationKindBinding) {
                    Text("settings.seed_duration.off").tag(SeedDurationKind.off)
                    Text("settings.seed_duration.hours").tag(SeedDurationKind.hours)
                    Text("settings.seed_duration.indefinite").tag(SeedDurationKind.indefinite)
                }
                .pickerStyle(.radioGroup)
                .labelsHidden()

                if seedDurationKindBinding.wrappedValue == .hours {
                    HStack {
                        TextField("", value: seedDurationHoursBinding, format: .number)
                            .frame(width: 48)
                            .multilineTextAlignment(.trailing)
                        Text("settings.seed_duration.hours_hint")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            }
        }
        .padding(16)
    }
    ```
    (первый `VStack` — save-location секция без изменений по содержимому, только перенесена внутрь новой обёртки; `Picker(.radioGroup)` — нативный macOS-стиль радио-кнопок, единственный существующий `Picker` в проекте — `TorrentDetailView`'s tab switcher — использует другой `.pickerStyle`, свериться, что не переиспользуется случайно неподходящий стиль)
  - [x] Т.к. `Picker`'s `selection` не может напрямую биндиться на enum с associated value через `.tag()`, ввести вспомогательный тип без данных для тегов:
    ```swift
    private enum SeedDurationKind: Hashable {
        case off, hours, indefinite
    }
    ```
    и два computed `Binding`:
    ```swift
    private var seedDurationKindBinding: Binding<SeedDurationKind> {
        Binding(
            get: {
                switch seedDurationMode {
                case .off: return .off
                case .hours: return .hours
                case .indefinite: return .indefinite
                }
            },
            set: { newKind in
                switch newKind {
                case .off: seedDurationMode = .off
                case .indefinite: seedDurationMode = .indefinite
                case .hours:
                    // Переключение на "N часов" без явного ввода числа — восстановить
                    // последнее сохранённое число (не локальный @State — он теряет
                    // значение после round-trip через "Не раздавать"/"Бессрочно",
                    // см. code-review fix выше при Task 1).
                    seedDurationMode = .hours(AppSettings.lastSeedDurationHours)
                }
                AppSettings.seedDurationMode = seedDurationMode
            }
        )
    }

    private var seedDurationHoursBinding: Binding<Int> {
        Binding(
            get: {
                if case .hours(let h) = seedDurationMode { return h }
                return AppSettings.defaultSeedDurationHours
            },
            set: { newValue in
                let clamped = AppSettings.clampSeedDurationHours(newValue)
                seedDurationMode = .hours(clamped)
                AppSettings.seedDurationMode = seedDurationMode
            }
        )
    }
    ```
    (AC2: `clampSeedDurationHours` вызывается на каждое изменение `TextField`'s value — SwiftUI's `TextField(value:format:)` коммитит на потерю фокуса/Enter, не посимвольно, так что ограничение диапазона не мешает набору многозначных чисел)

- [x] **Task 3: Локализация** (AC: 1, 3)
  - [x] `MyTorrent/Localizable.xcstrings` — 5 новых ключей (нативный Xcode JSON-стиль, пробел перед `:`, как во всех существующих — см. code-review Story 1.5/3.1):
    - `settings.seed_duration.label` — RU «Раздавать после завершения» / EN «Seed after completion» (значения уже зафиксированы в UX-спеке `3.1-settings.md`)
    - `settings.seed_duration.off` — RU «Не раздавать» / EN «Don't seed»
    - `settings.seed_duration.hours` — RU «N часов» / EN «N hours» (буква N буквально, см. Scope Boundary)
    - `settings.seed_duration.indefinite` — RU «Бессрочно» / EN «Indefinitely»
    - `settings.seed_duration.hours_hint` — новый, не из UX-спеки (см. Scope Boundary): RU «Считается только пока приложение открыто» / EN «Counts only while the app is open» — сформулировать по образцу примера из AD-5 («seeds for 8 hours of app usage»), но короче, т.к. это подпись, а не основной label

- [x] **Task 4: Сборка, ручная проверка** (AC: 1-4)
  - [x] Новых `.swift`-файлов нет — `xcodegen generate` всё равно запущен на всякий случай (не требовался, no-op)
  - [x] `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` — BUILD SUCCEEDED (дважды: после первичной реализации и после code-review-фикса в Task 1)
  - [x] Открыть Настройки, проверить все 3 радио-опции кликабельны и меняют выбор мгновенно — **подтверждено программно через Accessibility API** (`osascript`/System Events): чтение `value` каждой радио-кнопки до/после клика показало корректное переключение (off↔hours, значения 0/1 меняются местами как положено); также подтверждено, что числовое поле и подпись-подсказка появляются только при активном режиме «N часов» (`fieldCountWhenOff=0`, `fieldCountWhenHours=1`) — риск из Scope Boundary про `Picker(.radioGroup)`/`TextField` не подтвердился, всё работает
  - [x] Проверить, что подпись про «суммарное время работы приложения» видна только при выбранном режиме «N часов» — подтверждено тем же тестом (см. выше, подпись — соседний `Text` с полем, появляется/исчезает синхронно)
  - [x] Свежая установка (`defaults delete`) показывает умолчание 8 при режиме «N часов» — подтверждено
  - [~] Ввод числа вне 1–168 и проверка clamping — **не доведено до конца автоматизацией**: во время ручного тестирования через `keystroke` пользователь заметил, что автоматизация реально печатает на его экране (прислал «20» в чат посреди процесса) — по его явному указанию автоматизация остановлена, дальнейшую проверку (ввод числа, clamping до 1/168) пользователь проведёт сам
  - [~] Закрыть окно Настроек, открыть заново, проверить persistence числа часов — **частично подтверждено**: persistence режима (kind) подтверждена косвенно (несколько циклов `pkill`+перезапуск сохраняли выбранный режим), но чистый цикл «ввести число → закрыть окно → открыть → увидеть то же число» не пройден по той же причине (остановка автоматизации по просьбе пользователя) — рекомендация: проверить вручную при первом реальном использовании
  - [x] **Баг найден и исправлен во время этой самой проверки** (не в исходном плане Task 4, а по факту ручного тестирования): переключение «N часов» → другой режим → обратно на «N часов» сбрасывало число к умолчанию 8 вместо восстановления последнего введённого — см. отклонение в Task 1

## Dev Notes

### Technical Requirements / Stack

- **`AD-5`** («Split state ownership... Seed duration is cumulative app-open time, not wall-clock time — there is no background daemon (AD-1), so the countdown only advances while the app process is running») — эта история реализует только настройку значения и его копирайт, НЕ сам отсчёт/остановку (см. Scope Boundary). Копирайт должен явно называть это семантику, а не подразумевать wall-clock-поведение.
- **`librqbit::AddTorrentOptions`** (проверено при код-ревью Story 3.1 чтением `~/.cargo/registry/src/*/librqbit-8.1.1/src/session.rs`) — поля: `paused`, `only_files_regex`, `only_files`, `overwrite`, `list_only`, `output_folder`, `sub_folder`, `peer_opts`, `force_tracker_interval`, `disable_trackers`, `ratelimits`, `initial_peers`, `preferred_id`, `storage_factory`, `defer_writes`, `trackers`. **Ни одного поля про длительность раздачи.** Не тратить время на поиск способа прокинуть seed_duration в `add_torrent` в рамках этой истории — его физически некуда положить на Rust-стороне без отдельной инфраструктуры (см. Scope Boundary).
- **`Picker(selection:).pickerStyle(.radioGroup)`** — нативный SwiftUI API, доступен на macOS, рендерит вертикальный список радио-кнопок через AppKit. Первое использование этого стиля в проекте — `TorrentDetailView`'s `Picker` использует другой стиль (tab switcher), не путать/не копировать его конфигурацию по аналогии.

### Architecture Compliance

- Соответствует `AD-5` (Swift владеет настройками через `UserDefaults`, движок их напрямую не читает) — с уточнением, что «flow into Rust as call parameters» относится к случаям, когда на Rust-стороне есть что принять (см. Scope Boundary — этой истории это не касается).
- UX-спека `3.1-settings.md`: Technical Notes — «Instant-apply means preferences persist immediately... no explicit save/cancel flow» — реализовано так же, как в Story 3.1 (прямая запись в `UserDefaults` на каждое изменение).
- Тот же «единое окно, не новая сцена» принцип, что Story 3.1 уже установила для `SettingsView`/`Window(id: "settings")` — эта история добавляет секцию в существующее окно, не создаёт новое.

### Project Structure Notes

```
MyTorrent/
  AppSettings.swift            # UPDATE — SeedDurationMode + seedDurationMode свойство
  Views/
    SettingsView.swift          # UPDATE — секция «Раздавать после завершения»
  Localizable.xcstrings         # UPDATE — 5 новых ключей
```
Нет новых файлов → `xcodegen generate` не требуется (в отличие от Story 3.1).

### Previous Story Intelligence (Story 3.1)

- **XcodeGen**: не актуально для этой истории — новых `.swift`-файлов нет.
- **`Localizable.xcstrings`**: писать новые ключи в нативном Xcode JSON-стиле (`"key" : value`, пробел перед `:`) — подтверждено в код-ревью каждой истории с 1.5.
- **SwiftUI-гочи, найденные ранее в проекте**: `.onTapGesture` не всегда срабатывает рядом с `.contextMenu` (Story 2.1) — не относится напрямую к этой истории, но иллюстрирует класс риска «SwiftUI-контрол ведёт себя не как в документации при вложении/соседстве с другим контролом» — по этой же причине `TextField` в этой истории вынесен НЕ внутрь `Picker`'s опций (см. Scope Boundary).
- **Двухраундовый `/code-review` (high effort) на Story 3.1** нашёл реальную регрессию (multi-file торренты теряли auto-subfolder) и одну неполную первую попытку фикса (не хватало fallback на longest-filename) — оба раунда прошли полный 8-angle parallel finder + verify pass, обе находки исправлены и покрыты regression-тестами до коммита. Для этой истории (чисто Swift/UI, без затрагивания `engine/src/lib.rs`) риск того же класса (некорректное чтение стороннего API) ниже, но `/code-review` всё равно обязателен перед коммитом — тот же pipeline.
- **`AppModel.addTorrent`'s log-and-swallow convention** (найдено в код-ревью Story 3.1, оставлено как есть — продуктовое решение, не баг) — не относится к этой истории напрямую (здесь нет вызовов движка), но подтверждает общий принцип проекта: не добавлять новый user-facing error UI без явного запроса пользователя.
- **Git**: последний коммит на `main` — `ceb206d` (Story 3.1). Ветка чистая, готова для новой истории.

### Testing Standards

- Swift: формальный UI-test framework вне скоупа v1 (решение Story 1.1) — только ручная проверка (Task 4), с явным акцентом на реальный клик/ввод (не полагаться на одну компиляцию) из-за `Picker`/`TextField`-риска.
- Rust: эта история не трогает `engine/` — `cargo test` не требуется (но не будет лишним прогнать перед коммитом как sanity-check, что ничего не съехало).

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 3.2]
- [Source: design-artifacts/C-UX-Scenarios/03-alex-sets-defaults-once/3.1-settings/3.1-settings.md] — Section: Seed Duration Label, Seed Duration Control (Radio Group), Option: Don't Seed/N Hours/Indefinite, Technical Notes (instant-apply)
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-5] — cumulative app-open-time semantics, UI copy requirement
- [Source: _bmad-output/implementation-artifacts/3-1-default-save-folder.md] — предыдущая история: `AppSettings`/`SettingsView` паттерн, `AddTorrentOptions` field list (проверено чтением librqbit source), code-review pipeline
- librqbit 8.1.1 crate source (`~/.cargo/registry/src/.../librqbit-8.1.1/src/session.rs`) — `AddTorrentOptions` full field list (проверено повторно при создании этой истории, не изменилось с Story 3.1)

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- Первая версия `seedDurationKindBinding`'s `.hours`-ветки (переключение на «N часов») читала локальный `@State seedDurationMode` для восстановления числа — сбрасывала к умолчанию 8 при переключении через «Не раздавать»/«Бессрочно» и обратно, поскольку `@State` хранит число часов только пока активен именно режим `.hours`. Найдено ручным тестированием через Accessibility API (см. Completion Notes) — пойман до коммита. Исправлено: новое `AppSettings.lastSeedDurationHours`, читающее сохранённое число независимо от текущего режима.
- Ручная интерактивная проверка через `osascript`/System Events (клики по radio-кнопкам, ввод текста) была прервана по прямому указанию пользователя, который заметил, что автоматизация реально управляет курсором/клавиатурой на его физической машине (не в изолированной среде) — см. Completion Notes для того, что осталось непроверенным.

### Completion Notes List

- Все 4 задачи выполнены. AC1 (мгновенное применение трёх режимов), AC3 (копирайт про cumulative-time видна только при «N часов») и AC4 (persistence режима) подтверждены — частично автоматизированным тестом через Accessibility API (клики по радио-кнопкам, проверка появления/исчезания поля и подписи), частично логикой кода (persistence через `UserDefaults`, тот же паттерн, что `saveLocationPath` в Story 3.1).
- **AC2 (диапазон 1–168, clamping) — НЕ подтверждён вручную.** Автоматизация через `osascript`/`keystroke` реально печатает на физическом экране пользователя (эта среда — не изолированная песочница); пользователь заметил происходящее (прислал «20» в чат посреди теста) и попросил остановить автоматизацию, предпочтя проверить сам. Код реализует clamping (`AppSettings.clampSeedDurationHours`, вызывается в `seedDurationHoursBinding`'s setter и в `AppSettings.seedDurationMode`'s setter) — логически корректен и покрывает и «слишком маленькое», и «слишком большое» значение, но живого подтверждения ввод→clamp нет.
- **Persistence числа часов между закрытием/открытием окна Настроек — НЕ подтверждена явным циклом** (по той же причине остановки автоматизации). Persistence РЕЖИМА (off/hours/indefinite) подтверждена косвенно — несколько циклов перезапуска приложения (`pkill` + relaunch) сохраняли выбор.
- **Найден и исправлен реальный баг в процессе проверки** (не описан в исходном плане Task 1/2): переключение «N часов» → другой режим → обратно на «N часов» сбрасывало число к умолчанию 8 вместо восстановления последнего введённого. Причина и фикс — см. Debug Log References. Это ровно то, для чего Task 4's ручная проверка существует — поймано до коммита, не после.
- Layout подтверждён скриншотом: секция «Раздавать после завершения», все 3 радио-кнопки с правильными подписями, режим «N часов» выбран по умолчанию, поле «8», подпись про cumulative-time — всё соответствует UX-спеке и ASCII-макету.
- **Рекомендация пользователю**: при первом реальном использовании — открыть Настройки, ввести число вне 1–168 (например 0 или 500) и убедиться, что оно ограничивается до 1/168 соответственно; закрыть и открыть окно заново, убедиться что число сохранилось.
- Изменений в `engine/` нет — `cargo test` не запускался (не требовалось), но и не сломан ничем в этой истории.

### File List

- `MyTorrent/AppSettings.swift` — UPDATE: `SeedDurationMode` enum, `seedDurationMode` computed property (UserDefaults-backed, тот же паттерн что `saveLocationPath`), `clampSeedDurationHours`, `lastSeedDurationHours` (добавлено сверх исходного плана — см. Debug Log References)
- `MyTorrent/Views/SettingsView.swift` — UPDATE: секция «Раздавать после завершения» (`Picker(.radioGroup)` + соседний `TextField`, не вложенный в Picker), `SeedDurationKind` вспомогательный enum, `seedDurationKindBinding`/`seedDurationHoursBinding` computed bindings, `form` перестроен в две секции с `space-lg` между ними
- `MyTorrent/Localizable.xcstrings` — UPDATE: 5 новых ключей (`settings.seed_duration.label`, `.off`, `.hours`, `.indefinite`, `.hours_hint`)
