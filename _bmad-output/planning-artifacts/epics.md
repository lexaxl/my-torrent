---
stepsCompleted: [step-01-validate-prerequisites, requirements-confirmed, step-02-design-epics, step-03-create-stories]
inputDocuments:
  - design-artifacts/A-Product-Brief/01-product-brief.md
  - design-artifacts/E-Development/deliveries/DD-001-core-download-experience.yaml
  - design-artifacts/C-UX-Scenarios/01-alexs-daily-download/1.1-main-window/1.1-main-window.md
  - design-artifacts/C-UX-Scenarios/02-alex-inspects-a-torrent/2.2-torrent-detail/2.2-torrent-detail.md
  - design-artifacts/C-UX-Scenarios/03-alex-sets-defaults-once/3.1-settings/3.1-settings.md
  - design-artifacts/C-UX-Scenarios/04-alex-glances-at-the-menu-bar/4.1-menu-bar/4.1-menu-bar.md
  - _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md
---

# my-torrent - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for my-torrent, decomposing requirements from the WDS Product Brief + Design Delivery (in place of a formal PRD), the 4 UX Design page specifications, and the Architecture Spine.

## Requirements Inventory

### Functional Requirements

FR1: Opening a `.torrent` file (Finder double-click) or magnet link (browser) adds it to the downloads list immediately with no configuration dialog, using current global defaults.
FR2: Main window lists every active torrent with live progress bar, download speed, upload speed, seeder count, leecher count.
FR3: Right-clicking a torrent row shows a context menu: Pause/Resume, Remove, Show in Finder.
FR4: Clicking a torrent row opens a separate detail window with Files / Trackers / Peers tabs (read-only in v1, no per-file selection).
FR5: Settings window lets the user set a default save location and default seeding duration (off / 1-168 hours / indefinite); changes apply instantly, no save button.
FR6: Menu-bar icon is always present; clicking it opens a popover with aggregate download/upload speed and active-download count, plus quick actions to open the main window (always brought to front) or Settings.
FR7: A system notification fires when a download completes.
FR8: Main window toolbar has a search/filter field that filters the list by torrent name in real time.
FR9: Main window and menu-bar popover show an appropriate empty state when there are no active downloads; Torrent Detail's Peers tab shows an empty state when no peers are connected.

### NonFunctional Requirements

NFR1: Near-zero CPU/memory usage while idle (zero active transfers) — the primary, non-negotiable success metric.
NFR2: Runs on Apple Silicon (arm64) only, on the last 1-2 macOS versions — no Intel or older-OS support.
NFR3: Distributed as an unsigned build ($0 budget, no Apple Developer Program), with user-facing Gatekeeper-bypass instructions.
NFR4: All UI copy follows the Tone of Voice: quiet, plain, native-standard phrasing — no exclamation marks/emoji, silence as default (no confirmation toasts for routine actions).
NFR5: All Rust FFI calls execute off the main thread — blocking engine I/O must never freeze the SwiftUI main thread.
NFR6: The app runs as an accessory app (`LSUIElement`) — it does not quit when all windows are closed; the menu bar is the persistent presence.

### Additional Requirements (Architecture)

- Rust engine embedded in-process via UniFFI — no separate daemon/XPC process, no IPC/sockets (AD-1).
- BitTorrent protocol delegated to `librqbit` (tokio-based) — no from-scratch protocol implementation (AD-2).
- SwiftUI throughout, `MenuBarExtra` (macOS 13+) for the menu-bar layer — no AppKit (AD-3).
- Pull-based state sync: shell polls a UniFFI snapshot function ~1s, only while ≥1 torrent is active; one unconditional bootstrap poll on launch to discover auto-resumed torrents; timer owned by an app-lifetime object, not a View (AD-4).
- Split state ownership: Rust/librqbit owns torrent/session state; Swift owns settings via `UserDefaults`; settings flow one-directionally into Rust as call parameters (AD-5).
- Typed errors: every fallible UniFFI function returns `Result<T, EngineError>` → Swift `throws`; Rust panics are caught at the FFI boundary, never abort the process (AD-6).
- Not sandboxed — App Sandbox entitlements unnecessary for non-App-Store, unsigned distribution (AD-7).
- Mutation calls (`pause_torrent`/`resume_torrent`/`remove_torrent`) block until `librqbit`'s state machine reflects the change (AD-8).
- Build/release via GitHub Actions CI (macOS runner, arm64) triggered on tag → unsigned `.dmg` → GitHub Releases (AD-10).
- `.torrent` UTType file association and `magnet:` URL scheme registration required in `Info.plist`.
- Seed duration is cumulative app-open time, not wall-clock time — no background daemon exists to advance it while the app is closed.
- Torrent identity = BitTorrent infohash (lowercase hex, v1-canonical for hybrid torrents) everywhere.

### UX Design Requirements

UX-DR1: Main window layout — toolbar (title, search field, settings icon) + downloads list (5 columns: Имя/Name, Прогресс/Progress, ↓, ↑, Сиды/Личи) + empty state ("Нет активных закачек" / "No active downloads"). [`1.1-main-window.md`]
UX-DR2: **List Row** component — shared across main-window torrent rows and Torrent Detail's Files/Trackers/Peers rows: `space-sm` vertical / `space-md` horizontal padding, hairline `color-border` divider (absent on last row), hover background. [`D-Design-System/00-design-system.md`]
UX-DR3: **Icon Button** component — shared across settings-gear and detail-close-button: 26×26px hit target, `border-radius: 6px`, transparent default, hover background. [`D-Design-System/00-design-system.md`]
UX-DR4: Torrent Detail layout — header (close button, torrent name, live status summary) + Files/Trackers/Peers segmented tab switcher + per-tab content list; presented as a separate resizable window (not a sheet). [`2.2-torrent-detail.md`]
UX-DR5: Settings layout — save-location field + "Выбрать…"/"Choose…" picker button; seed-duration radio group (Не раздавать/Don't seed, N часов/N hours with 1-168 numeric input default 8, Бессрочно/Indefinitely); presented as a separate window; every change applies instantly. [`3.1-settings.md`]
UX-DR6: Menu-bar layer — status-bar icon (idle/active visual states TBD) + click-triggered popover (not hover) showing aggregate ↓/↑ speed + active count (or empty state), with "Открыть окно"/"Open Window" and "Настройки"/"Settings" quick actions. [`4.1-menu-bar.md`]
UX-DR7: Color tokens — `color-accent` #4C5FD5 (light) / #7C8CF0 (dark), `color-success` #1F9254/#3FB873, `color-warning` #B7791F/#D2A245, plus bg/surface/border/text tokens; full light + dark theme support required. [`D-Design-System/00-design-system.md`]
UX-DR8: Spacing scale (4px-based grid: `space-3xs`=2px … `space-3xl`=64px) and Type scale (9px–24px, `text-md`=13px matching macOS system font size) — apply throughout, never raw pixel values. [`D-Design-System/00-design-system.md`]
UX-DR9: All UI copy (RU/EN) follows the Tone of Voice attributes (Quiet & Unobtrusive, Plain & Direct, Native/System-standard, Calm Under Errors) — see NFR4. [`01-product-brief.md`]

### FR Coverage Map

FR1: Epic 1 - zero-friction add flow (main window)
FR2: Epic 1 - live downloads list
FR3: Epic 1 - row context menu (pause/resume/remove/reveal)
FR8: Epic 1 - toolbar search/filter
FR9 (main window empty state): Epic 1
FR4: Epic 2 - torrent detail window, Files/Trackers/Peers tabs
FR9 (peers empty state): Epic 2
FR5: Epic 3 - settings window (save location, seed duration)
FR6: Epic 4 - menu-bar popover
FR7: Epic 4 - completion notification
FR9 (popover empty state): Epic 4
NFR1-NFR6: cross-cutting, established in Epic 1 (engine/shell foundation per Architecture Spine AD-1 through AD-10), relied upon by all later epics

## Epic List

### Epic 1: Добавление, просмотр и управление закачками
Пользователь может открыть `.torrent`-файл или magnet-ссылку и увидеть закачку в списке мгновенно, без диалогов; видит live-прогресс/скорости/сиды-личей для каждой закачки; может найти закачку через поиск; может поставить на паузу/возобновить/удалить/показать в Finder через контекстное меню строки. Это ядро продукта и фундамент — здесь закладывается Rust-движок (librqbit), UniFFI-мост, SwiftUI-оболочка и `AppModel` (app-lifetime поллинг) по Architecture Spine.
**FRs covered:** FR1, FR2, FR3, FR8, FR9 (главное окно, пустое состояние)

### Epic 2: Просмотр деталей торрента
Пользователь кликает по закачке и видит отдельное окно с вкладками Файлы/Трекеры/Пиры — полная прозрачность без усложнения главного экрана.
**FRs covered:** FR4, FR9 (вкладка «Пиры», пустое состояние)

### Epic 3: Настройка умолчаний
Пользователь один раз задаёт папку сохранения по умолчанию и период раздачи (не раздавать / N часов 1-168 / бессрочно) — применяется мгновенно, без кнопки сохранения.
**FRs covered:** FR5

### Epic 4: Меню-бар, уведомления и релиз
Пользователь получает беглый доступ к статусу закачек через иконку в меню-баре (без открытия главного окна) и системное уведомление по завершении. Здесь же — сборка и публикация неподписанной `.dmg` через GitHub Actions CI (не отдельная пользовательская ценность, поэтому не отдельный эпик).
**FRs covered:** FR6, FR7, FR9 (popover, пустое состояние)

---

## Epic 1: Добавление, просмотр и управление закачками

Пользователь может открыть `.torrent`-файл или magnet-ссылку и увидеть закачку в списке мгновенно, без диалогов; видит live-прогресс/скорости/сиды-личей для каждой закачки; может найти закачку через поиск; может поставить на паузу/возобновить/удалить/показать в Finder через контекстное меню строки.

### Story 1.1: Пустой каркас приложения

As a user,
I want to open the app and see an empty downloads list,
So that I know it's running correctly.

**Acceptance Criteria:**

**Given** приложение впервые собрано и запущено
**When** нет активных торрентов
**Then** главное окно показывает toolbar + сообщение «Нет активных закачек»
**And** Rust-движок (крейт `engine/`) собирается и линкуется через UniFFI в SwiftUI-приложение без ошибок
**And** `AppModel` существует как app-lifetime объект (заготовка под Story 1.3)

### Story 1.2: Добавление торрента без диалогов

As a user,
I want to open a `.torrent` file or magnet link and see it appear in the list instantly,
So that I don't waste time on setup.

**Acceptance Criteria:**

**Given** приложение запущено
**When** я дважды кликаю `.torrent` в Finder
**Then** приложение открывается/выходит на передний план и новая строка появляется в списке без единого диалога, скачивание стартует на умолчаниях
**And** то же самое происходит при клике по magnet-ссылке в браузере
**And** `add_torrent(source: TorrentSource)` принимает путь/байты/magnet, согласно Structural Seed Architecture Spine

### Story 1.3: Живое отображение прогресса

As a user,
I want to see progress, speeds, and seed/leech counts update live for each download,
So that I always know the current status at a glance.

**Acceptance Criteria:**

**Given** есть ≥1 активная закачка
**When** прошла ~1 секунда
**Then** строка обновляет прогресс-бар, ↓/↑ скорость, сиды/личи
**And** поллинг идёт согласно AD-4: bootstrap-опрос при запуске, остановка при 0 активных, работа независимо от открытых окон
**And** «активно» = Downloading/Checking/Seeding (раздача не выпадает из отслеживания)

### Story 1.4: Поиск по списку

As a user,
I want to filter the downloads list by name,
So that I can quickly find a specific torrent.

**Acceptance Criteria:**

**Given** список содержит несколько закачек
**When** я ввожу текст в поле поиска
**Then** список мгновенно фильтруется по совпадению в имени (локально, без сети)

### Story 1.5: Управление закачкой через контекстное меню

As a user,
I want to pause/resume/remove a download or reveal it in Finder via right-click,
So that I can control it without extra navigation.

**Acceptance Criteria:**

**Given** есть закачка в списке
**When** я кликаю правой кнопкой по строке
**Then** появляется меню: Пауза/Возобновить, Удалить, Показать в Finder
**And** каждая команда возвращается только когда состояние в движке реально применено (AD-8) — без «мигания» в UI

## Epic 2: Просмотр деталей торрента

Пользователь кликает по закачке и видит отдельное окно с вкладками Файлы/Трекеры/Пиры — полная прозрачность без усложнения главного экрана.

### Story 2.1: Открытие деталей торрента

As a user,
I want to click a download and see a separate window with its files,
So that I understand what's being downloaded.

**Acceptance Criteria:**

**Given** есть закачка в списке
**When** я кликаю по строке
**Then** открывается отдельное resizable-окно (не sheet) с именем, live-статус-строкой (%, скорости, сиды/личи), закрытием и вкладкой «Файлы» по умолчанию
**And** вкладка «Файлы» показывает список файлов торрента с размером и % готовности каждого (без чекбоксов include/exclude — вне v1)
**And** получение деталей — отдельный `get_torrent_details(id)`-вызов, не часть общего списочного поллинга

### Story 2.2: Трекеры и пиры

As a user,
I want to switch to the Trackers and Peers tabs,
So that I can check connection status.

**Acceptance Criteria:**

**Given** окно деталей открыто
**When** я кликаю вкладку «Трекеры»
**Then** вижу список трекеров со статусом (подключён/не отвечает)
**When** я кликаю вкладку «Пиры»
**And** есть подключённые пиры
**Then** вижу список: адрес, ↓/↑ скорость, роль (сид/личер)
**And** если пиров нет — показывается «Нет подключённых пиров» вместо пустого списка

## Epic 3: Настройка умолчаний

Пользователь один раз задаёт папку сохранения по умолчанию и период раздачи (не раздавать / N часов 1-168 / бессрочно) — применяется мгновенно, без кнопки сохранения.

### Story 3.1: Папка сохранения по умолчанию

As a user,
I want to set the folder new downloads are saved to,
So that files land where I expect them.

**Acceptance Criteria:**

**Given** окно настроек открыто (умолчание `~/Downloads`)
**When** я нажимаю «Выбрать...» и выбираю папку в системном диалоге
**Then** значение применяется мгновенно, без отдельной кнопки сохранения
**And** следующая добавленная закачка использует новый путь

### Story 3.2: Период раздачи по умолчанию

As a user,
I want to set how long torrents seed after completion,
So that I control how long I contribute back without babysitting it.

**Acceptance Criteria:**

**Given** окно настроек открыто
**When** я выбираю «N часов» и ввожу значение 1-168
**Then** применяется мгновенно (умолчание — 8)
**And** это суммарное время работы приложения, не wall-clock (AD-5) — копирайт в UI должен явно это отражать
**And** значение вне диапазона 1-168 отклоняется/ограничивается

## Epic 4: Меню-бар, уведомления и релиз

Пользователь получает беглый доступ к статусу закачек через иконку в меню-баре (без открытия главного окна) и системное уведомление по завершении. Здесь же — сборка и публикация неподписанной `.dmg` через GitHub Actions CI.

### Story 4.1: Иконка в меню-баре и popover

As a user,
I want to click the menu-bar icon and see aggregate stats,
So that I can check status without opening the main window.

**Acceptance Criteria:**

**Given** приложение запущено
**When** я кликаю по иконке
**Then** открывается popover со ↓/↑ скоростью и числом активных закачек (или «Нет активных закачек», если их нет)
**And** popover содержит действия «Открыть окно» (всегда выводит главное окно на передний план, не toggle) и «Настройки»
**And** взаимодействие — клик, не hover

### Story 4.2: Уведомление о завершении

As a user,
I want a system notification when a download completes, even with all windows closed,
So that I know without having to check manually.

**Acceptance Criteria:**

**Given** приложение работает в фоне (все окна закрыты)
**When** закачка переходит в статус «завершено»
**Then** приходит системное уведомление с именем файла
**And** обнаружение — через сравнение соседних снимков поллинга (AD-4), без отдельного callback-канала

### Story 4.3: Сборка и релиз

As a user (acting as release manager of their own project),
I want a git tag to automatically build and publish a release,
So that shipping a new version doesn't require manual steps.

**Acceptance Criteria:**

**Given** запушен git-тэг
**When** срабатывает GitHub Actions
**Then** собирается `.app`, упаковывается в неподписанный `.dmg` и публикуется в GitHub Releases
**And** README содержит инструкцию по обходу Gatekeeper для неподписанной сборки
