# my-torrent

Нативный, минималистичный BitTorrent-клиент для macOS. Rust-движок ([librqbit](https://github.com/ikatson/rqbit)) под капотом, SwiftUI-оболочка сверху. Только Apple Silicon.

## Установка

1. Скачайте `MyTorrent.dmg` со страницы [Releases](https://github.com/lexaxl/my-torrent/releases).
2. Откройте `.dmg` и перетащите `MyTorrent.app` в `Applications`.

## Обход Gatekeeper (неподписанная сборка)

Приложение распространяется без подписи Apple Developer ID (проект без бюджета на платную подписку) — при первом запуске macOS покажет предупреждение о том, что разработчик не подтверждён.

**Как открыть:**

1. Попробуйте открыть `MyTorrent.app` как обычно — если увидите предупреждение, нажмите **«Отмена»**, а не «В корзину».
2. Откройте **System Settings → Privacy & Security**.
3. Прокрутите вниз до сообщения о заблокированном `MyTorrent` и нажмите **«Open Anyway»** («Открыть в любом случае»).
4. Подтвердите ещё раз в появившемся диалоге.

После этого приложение будет открываться нормально при последующих запусках.

<details>
<summary>Альтернатива через Terminal (для продвинутых пользователей)</summary>

```sh
xattr -d com.apple.quarantine /Applications/MyTorrent.app
```

</details>

## Сборка из исходников

Требуется Xcode, [XcodeGen](https://github.com/yonaskolb/XcodeGen) и Rust (с таргетом `aarch64-apple-darwin`).

```sh
xcodegen generate
xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build
```

## Презентации

Разбор того, как проект был сделан: архитектура (Rust-движок + SwiftUI-оболочка через UniFFI) и процесс разработки по BMAD-конвейеру.

- [Полная версия](https://lexaxl.github.io/my-torrent/presentations/pipeline-full.html) — 14 слайдов
- [Короткая версия](https://lexaxl.github.io/my-torrent/presentations/pipeline-short.html) — 11 слайдов

Обе версии с послайдовым оглавлением — на [странице презентаций](https://lexaxl.github.io/my-torrent/).
Слайды листаются стрелками / пробелом. Исходники — в [`docs/presentations/`](docs/presentations/).

## Харнес разработки

Проект целиком сделан через агентский харнес — набор версионируемых процедур, которые лежат в репозитории рядом с кодом, а не живут в переписке с моделью. Ниже — точный состав того, что использовалось.

### Состав и версии

[BMAD Method](https://github.com/bmad-code-org/BMAD-METHOD) **v6.10.0**, установлен 26 июля 2026. Пять модулей:

| Модуль | Версия | Источник | Скиллов | Роль в проекте |
|---|---|---|---|---|
| `core` | 6.10.0 | built-in | 13 | Общая база: конфиг, брейншторминг, ретроспективы, party-mode |
| `bmm` | 6.10.0 | built-in | 33 | Основной путь разработки: требования → архитектура → эпики → истории → реализация |
| `wds` | [v0.4.3](https://github.com/bmad-code-org/bmad-method-wds-expansion) | npm `bmad-wds`, sha `cc16f09` | 15 | Продуктовый и UX-трэк: бриф, trigger map, сценарии, дизайн-система |
| `tea` | [v1.19.1](https://github.com/bmad-code-org/bmad-method-test-architecture-enterprise) | npm `bmad-method-test-architecture-enterprise`, sha `74cf6e6` | 10 | Тест-архитектура — установлен, но почти не использовался (см. «Отступления») |
| `bmad-loop` | [v0.9.0](https://github.com/bmad-code-org/bmad-loop) | git, sha `7255174` | 3 | Автономный цикл разработки — не использовался |

Итого 74 скилла. В манифесте установки записаны пять целевых сред — `claude-code`, `codex`, `cursor`, `codewhale`, `opencode` — но на диске лежат четыре папки вывода (для `cursor` ничего не сгенерировано). Фактически вся работа шла в **Claude Code**.

**Что из этого лежит в репозитории, а что нет.** В git закоммичен только `_bmad/` (70 файлов): конфиги модулей, `module-help.csv` с картой фаз, данные WDS (гайды агентов, шаблоны документов, правила дизайн-системы), питон- и js-скрипты (`resolve_config.py`, `resolve_customization.py`, `memlog.py`, `wds-*.js`) и манифесты в `_bmad/_config/`. Сами тела скиллов — по 74 `SKILL.md` в `.claude/skills/`, `.agents/skills/` и `.codewhale/skills/`, плюс 74 команды в `.opencode/commands/` — **не коммитятся**: это генерируемый установщиком вывод под конкретную среду. Чтобы воспроизвести харнес, нужно переустановить BMAD той же версии — манифест `_bmad/_config/manifest.yaml` фиксирует версии и SHA всех внешних модулей именно для этого.

### Конфигурация

Настройки лежат в четырёх слоях, каждый следующий переопределяет предыдущий (скаляры затираются, таблицы сливаются вглубь):

| Файл | Кто владеет | В git |
|---|---|---|
| `_bmad/config.toml` | установщик, перезаписывается при переустановке | да |
| `_bmad/config.user.toml` | установщик, ответы конкретного пользователя | да |
| `_bmad/custom/config.toml` | человек, командные пины и кастомные агенты | да |
| `_bmad/custom/config.user.toml` | человек, личные пины | нет (gitignored) |

Эффективные значения после слияния слоёв — их печатает `_bmad/scripts/resolve_config.py --project-root . --key modules` (нужен Python 3.11+ ради `tomllib`):

```toml
[core]
project_name             = "my-torrent"
document_output_language = "Russian"     # все артефакты пишутся по-русски
output_folder            = "{project-root}/_bmad-output"

[modules.bmm]
planning_artifacts       = "{project-root}/_bmad-output/planning-artifacts"
implementation_artifacts = "{project-root}/_bmad-output/implementation-artifacts"
project_knowledge        = "{project-root}/_bmad-output/project-knowledge"  # переопределено

[modules.wds]
design_artifacts   = "{project-root}/design-artifacts"
project_type       = "digital_product"
design_system_mode = "none"              # своя дизайн-система не заводилась
methodology_version = "wds-v6"
product_languages  = ["en", "ru"]
```

Плюс личный слой: `user_name = "Alex"`, `communication_language = "Russian"` (диалог по-русски), `user_skill_level = "beginner"` — это влияет на то, насколько подробно скиллы объясняют свои шаги.

Единственное место, где значения отличаются от установочных, — `project_knowledge`. Установщик по умолчанию ставит его в `{project-root}/docs`, но эта же папка настроена корнем GitHub Pages: любой документ, записанный туда харнесом, молча оказался бы на публичном сайте. Поэтому он уведён в `_bmad-output/project-knowledge`, рядом с остальными артефактами.

Правка нужна в двух местах — у харнеса два параллельных механизма чтения настроек:

| Где | Кто читает | Переживает переустановку |
|---|---|---|
| `_bmad/custom/config.toml` | 5 скиллов из 74 — те, что идут через `resolve_config.py` | да, слой `custom/` установщик не трогает |
| `_bmad/bmm/config.yaml`, `_bmad/wds/config.yaml` | 60 скиллов из 74 — они читают YAML своего модуля напрямую | **нет**, файлы генерируются установщиком |

> После переустановки BMAD значение в двух `config.yaml` придётся вернуть руками — в самих файлах на этом месте стоит комментарий-напоминание.

### Агенты и роли

Каждая фаза активируется своей персоной — со своим чек-листом, форматом выхода и границей ответственности. Архитектор не пишет истории, разработчик не переписывает архитектуру.

| Скилл | Персона | Роль | Модуль |
|---|---|---|---|
| `bmad-agent-analyst` | Mary 📊 | Business Analyst | bmm |
| `bmad-agent-pm` | John 📋 | Product Manager | bmm |
| `bmad-agent-architect` | Winston 🏗️ | System Architect | bmm |
| `bmad-agent-ux-designer` | Sally 🎨 | UX Designer | bmm |
| `bmad-agent-dev` | Amelia 💻 | Senior Software Engineer | bmm |
| `bmad-agent-tech-writer` | Paige 📚 | Technical Writer | bmm |
| `bmad-tea` | Murat 🧪 | Master Test Architect | tea |
| `wds-agent-saga-analyst` | Saga 📚 | WDS Analyst — бриф и trigger map | wds |
| `wds-agent-freya-ux` | Freya 🎨 | WDS Designer — сценарии и спеки экранов | wds |
| `wds-agent-mimir-builder` | Mimir 🔨 | WDS Builder — тех-аудит и сборка | wds |

### Цикл одной истории

Один и тот же повторяющийся проход, 18 раз:

1. **`bmad-create-story`** — создаёт файл истории в `_bmad-output/implementation-artifacts/`. Фиксированная структура: `baseline_commit` во frontmatter, затем Story (в форме «как пользователь, я хочу…»), Acceptance Criteria в Given/When/Then, Scope Boundary (что в эту историю **не** входит), Tasks/Subtasks и блок Dev Notes — технические требования, соответствие архитектуре, заметки по структуре проекта, «Previous Story Intelligence» (что узнали в прошлой истории), стандарты тестирования и ссылки на источники.
2. **`bmad-dev-story`** — реализация. Заполняется Dev Agent Record: какая модель работала, ссылки на отладочный лог, Completion Notes и File List. Отклонения от плана записываются туда же — отчёт о том, что реально произошло, а не о том, что задумывалось.
3. **`/code-review high`** — независимый враждебный проход перед коммитом: 8 параллельных finder-агентов ищут проблемы каждый со своего угла, затем verify-проход проверяет каждого кандидата. Результаты вписываются в файл истории отдельной секцией с пометкой, что исправлено, а что осознанно оставлено.
4. **Коммит** — одна история, один коммит, вместе с обновлённым файлом истории и затронутыми документами.
5. **`bmad-retrospective`** на эпик — что сработало, зоны роста, action items.

Модели: истории 1.1–5.4 сделаны на Claude Sonnet 5, 6.1–6.2 — на Claude Fable 5, всё внутри Claude Code.

### Артефакты

```
_bmad/                              харнес: конфиги, данные модулей, скрипты
_bmad-output/
  planning-artifacts/
    epics.md                        15 FR + 6 NFR → 6 эпиков → 18 историй
    architecture/architecture-my-torrent-2026-07-27/
      ARCHITECTURE-SPINE.md         AD-1…AD-10, инварианты, стек, карта покрытия
      ARCHITECTURE-EXPLAINED.md     тот же материал в объяснительной форме
      reviews/                      три независимых ревью архитектуры:
                                    adversarial, rubric-walker, tech-verification
  implementation-artifacts/
    <18 файлов историй>.md
    deferred-work.md                отложенное с адресом: откуда пришло и почему
    epic-6-retro-2026-07-30.md      ретроспектива
  test-artifacts/                   пусто
  project-knowledge/                пусто (уведено сюда из docs/, см. «Конфигурация»)
design-artifacts/                   WDS-трэк, фазы A–E
  A-Product-Brief/                  бриф + полный лог диалога, из которого он вырос
  B-Trigger-Map/                    бизнес-цели → психология пользователя
  C-UX-Scenarios/                   4 сценария, спеки экранов, ASCII-скетчи
  D-Design-System/                  визуальные решения + HTML-прототип
  E-Development/                    передача в разработку: delivery + тест-сценарии
docs/                               только сайт: корень GitHub Pages
```

Ключевой документ — `ARCHITECTURE-SPINE.md`. Это не описание системы, а список инвариантов, которым код обязан подчиняться: 10 архитектурных решений от AD-1 (single-process embedded core) до AD-10 (CI-сборка, релиз без подписи). Истории ссылаются на них по номеру, и отступление от AD — повод переписать AD, а не тихо обойти его в коде.

### В цифрах

| Что | Сколько |
|---|---|
| Код | 4852 строки: Swift 3116 (12 файлов) + Rust 1736 (3 файла) |
| Артефакты процесса | 9012 строк в 60 файлах `.md` / `.yaml` |
| Соотношение | ≈1.9 строки документации на строку кода |
| Требования | 15 функциональных + 6 нефункциональных |
| Разбивка | 6 эпиков, 18 историй, 10 архитектурных решений |
| Тесты | 28 на стороне Rust, на стороне Swift — ноль |
| Сроки | 26–30 июля 2026, 22 коммита |

### Отступления от метода

Харнес адаптировали под масштаб задачи, а не исполняли ритуально. Каждое отступление — осознанное, и все они зафиксированы в самих артефактах:

- **PRD не писали.** Его роль сыграла связка «WDS Product Brief + Design Delivery»; это прямо записано в шапке `epics.md`, чтобы отличать адаптацию от забывчивости.
- **Тест-модуль почти не задействован.** `tea` установлен, но `test-artifacts/` пуст: тест-стратегия, трассировка требований и аудит NFR не велись. Тесты писались внутри историй, только на стороне Rust.
- **Код-ревью — не модульным скиллом.** Вместо `bmad-code-review` использовался `/code-review high` из Claude Code: он давал параллельный многоугловой проход, которого хотелось для каждой истории.
- **Статусы историй не доводились до `done`.** 15 файлов из 18 так и остались в `Status: review`, хотя работа по ним закончена и вошла в `v1.0.0`. Формальный переход статуса не соблюдался.
- **Отложенное фиксировалось отдельно.** `deferred-work.md` держит вещи, которые решили не делать сейчас, с указанием, откуда они пришли — например, невоспроизведённое падение при старте и регрессия по NFR1 из-за автоматического возобновления раздачи.
