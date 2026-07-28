---
baseline_commit: 755e91a25f963bdec08f6c2b53968306c4b00027
---

# Story 4.3: Сборка и релиз

Status: review

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

Как пользователь (в роли релиз-менеджера собственного проекта),
я хочу, чтобы git-тэг автоматически собирал и публиковал релиз,
чтобы выпуск новой версии не требовал ручных шагов.

## Acceptance Criteria

1. **Given** запушен git-тэг (`v*`, например `v1.0.0`), **when** срабатывает GitHub Actions, **then** собирается `.app` (Release-конфигурация, Apple Silicon/arm64, per `project.yml`/AD-10).
2. **Then** `.app` упаковывается в неподписанный `.dmg`.
3. **Then** `.dmg` публикуется как asset GitHub Release, привязанного к этому тэгу.
4. **Given** пользователь скачал `.dmg` с GitHub Releases, **then** в README есть понятная инструкция по обходу Gatekeeper для неподписанной сборки (per Product Brief: «System Settings → allow anyway», это явно зафиксированный в проекте метод, не изобретённый заново).

## Scope Boundary

**Решено при создании истории:**

- **CI-раннер — `macos-14` (Apple Silicon, arm64), не `macos-latest`.** `AD-10` требует «GitHub Actions (macOS runner, arm64)»; `project.yml` уже фиксирует `ARCHS: arm64` / `ONLY_ACTIVE_ARCH: YES` и Apple-Silicon-only таргет (`aarch64-apple-darwin` — см. `scripts/build-engine.sh`). Явный `macos-14` (не `-latest`) — воспроизводимость: GitHub периодически меняет, на что указывает `-latest`, а этот проект уже пинит конкретные версии в других местах (Rust target, Xcode). Использовать именно `macos-14`, а не более старую версию — GitHub-хостед раннеры этой версии уже arm64-native (Apple Silicon), что и нужно для нативной сборки без кросс-компиляции.
- **`xcodegen generate` обязателен в workflow** — `.xcodeproj` в `.gitignore` (комментарий в файле: «Xcode project regenerated from project.yml»), значит CI, как и локальная разработка, должен сгенерировать его перед `xcodebuild`. `xcodegen` не предустановлен на GitHub-раннерах — установка через `brew install xcodegen` (Homebrew предустановлен на macOS-раннерах).
- **Rust toolchain через `dtolnay/rust-toolchain@stable`** (не полагаться на то, что предустановлено на раннере) — с явным `target: aarch64-apple-darwin`, тот же target, что `scripts/build-engine.sh` уже жёстко фиксирует. Именно этот target — нативный для arm64-раннера, `rustup target add` не обязателен (это host-target), но использовать `dtolnay/rust-toolchain` для детерминированной версии тулчейна — не полагаться на предустановленный Rust, который может меняться между обновлениями образа раннера.
- **`MyTorrent/Generated/*` уже закоммичены как baseline** (см. комментарий в `.gitignore`) — `xcodebuild`'s pre-build script (`scripts/build-engine.sh`) перезапишет их на месте при сборке, ничего специального для CI не требуется на этот счёт — свежий `checkout` уже содержит валидные (хоть и устаревшие) файлы, так что XcodeGen не «не найдёт» источники.
- **`.dmg` включает symlink на `/Applications`** (не просто голый `.app` внутри диск-образа) — стандартный macOS drag-to-install UX, дешёвая полировка (одна доп. команда `ln -s /Applications`), консистентно с общим уровнем внимания к UX в этом проекте (см. предыдущие истории).
- **Подпись — уже ad-hoc через существующий `project.yml`** (`CODE_SIGN_IDENTITY: "-"`, `ENABLE_HARDENED_RUNTIME: NO`) — Release-конфигурация автоматически использует те же настройки таргета, что Debug; никакого отдельного шага `codesign` в workflow не требуется сверх того, что `xcodebuild` уже делает согласно `project.yml`.
- **Публикация через `softprops/action-gh-release@v2`** — широко используемый, поддерживаемый GitHub Action для создания Release + прикрепления assets по тэгу; требует `permissions: contents: write` в workflow (иначе `GITHUB_TOKEN` по умолчанию read-only и Action не сможет создать Release/аплоадить asset).
- **Тэг-паттерн — `v*`** (например `v1.0.0`, `v0.1.0`) — стандартная конвенция для этого типа релизного workflow; в epics.md AC не фиксирует конкретный паттерн буквально («запушен git-тэг»), так что `v*` — разумное, общепринятое решение, не «vague implementation» (единственная разумная альтернатива — вообще без префикса `v`, что менее стандартно и хуже читается в UI GitHub Releases).
- **README.md — новый файл**, в репозитории на момент создания истории такого файла ещё не было (проверено `ls`). Помимо AC4's инструкции по Gatekeeper — минимальное описание проекта и (кратко) инструкция сборки из исходников (`xcodegen generate` + `xcodebuild`, как локальная разработка), не разрастаться в полноценную документацию — не запрошено ни одним AC.
- **Реальная сквозная проверка (push тэга → GitHub Actions собирает → создаётся настоящий публичный Release) НЕ выполняется автоматически в рамках этой истории** — пуш тэга создаёт видимый внешний артефакт (публичный GitHub Release в репозитории пользователя) и запускает биллингуемую CI-инфраструктуру; это классическое «hard-to-reverse, visible to others» действие. Явное подтверждение пользователя запрашивается отдельно, до пуша тэга, после того как сам YAML/README реализованы и прошли ручную проверку синтаксиса/логики без реального запуска.

## Tasks / Subtasks

- [x] **Task 1: `.github/workflows/release.yml` — новый файл** (AC: 1, 2, 3)
  - [x] Структура:
    ```yaml
    name: Release

    on:
      push:
        tags:
          - "v*"

    permissions:
      contents: write

    jobs:
      build-and-release:
        runs-on: macos-14
        steps:
          - name: Checkout
            uses: actions/checkout@v4

          - name: Install Rust toolchain
            uses: dtolnay/rust-toolchain@stable
            with:
              targets: aarch64-apple-darwin

          - name: Install XcodeGen
            run: brew install xcodegen

          - name: Generate Xcode project
            run: xcodegen generate

          - name: Build (Release)
            run: |
              xcodebuild \
                -project MyTorrent.xcodeproj \
                -scheme MyTorrent \
                -configuration Release \
                -derivedDataPath build \
                build

          - name: Package .dmg
            run: |
              APP_PATH="build/Build/Products/Release/MyTorrent.app"
              STAGING_DIR="dmg-staging"
              mkdir -p "$STAGING_DIR"
              cp -R "$APP_PATH" "$STAGING_DIR/"
              ln -s /Applications "$STAGING_DIR/Applications"
              hdiutil create -volname "MyTorrent" -srcfolder "$STAGING_DIR" -ov -format UDZO MyTorrent.dmg

          - name: Publish GitHub Release
            uses: softprops/action-gh-release@v2
            with:
              files: MyTorrent.dmg
    ```
    (`build/Build/Products/Release/` — стандартный путь при явном `-derivedDataPath build`, не зависит от глобального DerivedData раннера)
  - [x] Не добавлять шаг `codesign`/`notarize` — см. Scope Boundary, `project.yml`'s существующие настройки таргета уже покрывают ad-hoc подпись

- [x] **Task 2: `README.md` — новый файл** (AC: 4)
  - [x] Минимальная структура:
    - Заголовок, короткое описание проекта (native macOS torrent client, Rust engine, Apple Silicon only) — 2-3 предложения, не переписывать весь Product Brief
    - Секция «Установка» — скачать `.dmg` с GitHub Releases, перетащить `MyTorrent.app` в `Applications`
    - Секция «Обход Gatekeeper (неподписанная сборка)» — **основной метод**: при первом запуске macOS покажет предупреждение «нельзя открыть, разработчик не подтверждён» → System Settings → Privacy & Security → прокрутить вниз → «Open Anyway» рядом с упоминанием MyTorrent → подтвердить ещё раз в диалоге. Отметить, что это тот же метод, что зафиксирован в Product Brief проекта, не альтернатива на усмотрение разработчика истории
    - Секция «Сборка из исходников» (кратко) — `xcodegen generate` → `xcodebuild -project MyTorrent.xcodeproj -scheme MyTorrent -configuration Debug build` (тот же процесс, что уже используется локально во всех предыдущих историях)
  - [x] Не расписывать архитектуру/roadmap — не запрошено ни одним AC, это README для конечного пользователя/контрибьютора, не design-документ

- [x] **Task 3: Ручная проверка (без реального пуша тэга)** (AC: 1-4)
  - [x] Валидировать YAML-синтаксис `release.yml` (например, `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"` или аналогичный линтер, если доступен `actionlint`/`yamllint`)
  - [x] Локально прогнать шаги 3-5 workflow вручную в этом же репозитории (тот же `xcodegen generate` + `xcodebuild -configuration Release ...` + `hdiutil create` с тем же путём `build/Build/Products/Release/MyTorrent.app`), чтобы убедиться, что Release-конфигурация действительно собирается и `.dmg` реально создаётся и монтируется корректно (`hdiutil attach` на созданный `.dmg`, проверить, что `MyTorrent.app` и symlink `Applications` видны) — это то, что можно проверить локально без реальной GitHub Actions инфраструктуры
  - [x] Проверить корректность README вручную (открыть в Preview/просмотрщике markdown)
  - [ ] **НЕ пушить реальный git-тэг без явного отдельного подтверждения пользователя** — см. Scope Boundary; после того как локальная проверка Task 3 пройдена, явно спросить пользователя, прежде чем создавать и пушить первый релизный тэг (например `v0.1.0`)

## Dev Notes

### Technical Requirements / Stack

- **`AD-10`** («GitHub Actions (macOS runner), triggered on git tag, builds the app and packages an unsigned `.dmg`, published to GitHub Releases. No code signing, no notarization, no App Store») — реализуется этой историей целиком.
- **`project.yml`** — уже фиксирует `ARCHS: arm64`, `ONLY_ACTIVE_ARCH: YES`, `deploymentTarget.macOS: "13.0"`, `CODE_SIGN_IDENTITY: "-"`, `ENABLE_HARDENED_RUNTIME: NO` — все нужные для unsigned Apple-Silicon-only сборки настройки уже на месте, эта история их не меняет, только добавляет CI, который их использует.
- **`scripts/build-engine.sh`** — уже вызывается как `preBuildScripts` в `project.yml`, сработает автоматически при `xcodebuild` в CI так же, как локально; жёстко фиксирует `RUST_TARGET="aarch64-apple-darwin"` — совпадает с тем, что должен установить `dtolnay/rust-toolchain`'s `targets:` параметр.
- **`engine/Cargo.toml`**: `crate-type = ["lib", "staticlib", "cdylib"]` — и `.a` (для линковки в `.app` через `OTHER_LDFLAGS`), и `.dylib` (для `uniffi-bindgen` генерации Swift-биндингов) собираются одной командой `cargo build --release --target aarch64-apple-darwin`, ничего дополнительно настраивать в CI для этого не нужно.
- **Product Brief** (`design-artifacts/A-Product-Brief/01-product-brief.md`, строка 152): «Distribute as an unsigned build with user-facing instructions for bypassing Gatekeeper (System Settings → allow anyway). **Fixed** (deliberate choice, not just default)» — README's Gatekeeper-секция должна использовать именно этот метод, а не альтернативный (например, `xattr -d com.apple.quarantine`), хотя последний можно упомянуть как fallback для продвинутых пользователей.

### Architecture Compliance

- `AD-10` — полностью реализуется этой историей (единственной, которая его покрывает — «Capability → Architecture Map» в `ARCHITECTURE-SPINE.md` явно связывает «Build & release» с `AD-10` и файлом `.github/workflows/release.yml`, путь, зафиксированный в Structural Seed архитектуры заранее).
- Никакие другие AD не затрагиваются — эта история не трогает `engine/`/`MyTorrent/` исходники вообще, только добавляет CI-конфигурацию и документацию.

### Project Structure Notes

```
.github/
  workflows/
    release.yml    # NEW — CI: build + package unsigned .dmg + publish on tag (AD-10)
README.md           # NEW — install instructions, Gatekeeper bypass, build-from-source
```
Оба пути совпадают с тем, что уже зафиксировано в Structural Seed архитектуры (`ARCHITECTURE-SPINE.md`) — `.github/workflows/release.yml` был предусмотрен заранее, ничего не отклоняется от плана.

### Previous Story Intelligence (Story 4.2)

- **`/code-review high` перед коммитом обязателен** — тот же pipeline, что все истории с 3.1. Для этой истории код-ревью особенно ценен на предмет самого YAML (шаги в правильном порядке, пути реально совпадают между шагами сборки/паковки, `permissions` корректны для `softprops/action-gh-release`).
- **Story 4.2 нашла реальное расхождение между Architecture Spine и реальным движком** (`Completed-not-seeding` статус не существует) и исправила документ по итогам код-ревью — та же дисциплина («если что-то в архитектурном документе не соответствует реальности, поправить документ, а не молча обойти») применима и здесь, если при реализации всплывёт похожее несоответствие.
- **Git**: последний коммит на `main` — `755e91a` (Story 4.2). Ветка чистая, готова для новой истории.

### Testing Standards

- Формального теста для GitHub Actions workflow в проекте нет и не заводится этой историей — верификация Task 3 (локальный прогон эквивалентных команд + YAML-валидация) — тот же уровень строгости, что установлен для остальных историй проекта (ручная проверка, без формального test framework).
- `cargo test`/Swift-тесты не затрагиваются этой историей (никаких изменений в `engine/`/`MyTorrent/` исходниках) — прогонять не обязательно, но не помешает как sanity-check, раз меняется общий release-pipeline.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Story 4.3]
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#AD-10] — CI-built, unsigned release distribution
- [Source: _bmad-output/planning-artifacts/architecture/architecture-my-torrent-2026-07-27/ARCHITECTURE-SPINE.md#Structural Seed] — `.github/workflows/release.yml` предусмотрен заранее
- [Source: design-artifacts/A-Product-Brief/01-product-brief.md] — $0 budget/unsigned distribution constraint (строка 19), Gatekeeper bypass method «System Settings → allow anyway» (строка 152, зафиксировано как deliberate choice)
- [Source: project.yml] — существующие build-настройки (arm64, ad-hoc signing, deployment target), переиспользуемые CI без изменений
- [Source: scripts/build-engine.sh] — существующий Rust-build-скрипт, вызываемый автоматически `xcodebuild`'s preBuildScripts
- [Source: engine/Cargo.toml] — `crate-type` подтверждает, что один `cargo build` даёт оба нужных артефакта (`.a` + `.dylib`)
- [Source: _bmad-output/implementation-artifacts/4-2-completion-notification.md] — предыдущая история: код-ревью pipeline, дисциплина исправления архитектурных документов при найденном расхождении

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (Claude Code)

### Debug Log References

- `python3 -c "import yaml; yaml.safe_load(...)"` изначально показал ключ `on:` как булево `True` (классическая YAML 1.1 особенность PyYAML — не баг GitHub Actions, но валидный повод устранить неоднозначность) — исправлено на `"on":` (закавычено), после чего парсится как строковый ключ `'on'` корректно.
- `actionlint` (установлен через `brew install actionlint` для этой проверки) — 0 замечаний по `.github/workflows/release.yml` (версии Actions валидны, встроенные `run`-скрипты прошли shellcheck).
- Локальный прогон эквивалента шагов 3-6 workflow (`xcodegen generate` → `xcodebuild -configuration Release -derivedDataPath build build` → сборка `.dmg` через `hdiutil create` с той же структурой staging-папки) — BUILD SUCCEEDED, `.dmg` создан (~8.6 МБ), примонтирован (`hdiutil attach`) и подтверждена корректная структура: `MyTorrent.app` + symlink `Applications → /Applications`. Тестовые артефакты (`build/`, `dmg-staging/`, `MyTorrent.dmg`) удалены после проверки; добавлены в `.gitignore`, чтобы больше не могли случайно попасть в коммит.

### Completion Notes List

- Задачи 1-3 выполнены полностью. Новый `.github/workflows/release.yml` (триггер по тэгу `v*`, сборка на `macos-14`, `xcodegen generate` + Rust toolchain через `dtolnay/rust-toolchain`, Release-сборка, упаковка `.dmg` со symlink на `/Applications`, публикация через `softprops/action-gh-release@v2`). Новый `README.md` с инструкцией по обходу Gatekeeper (метод «System Settings → Open Anyway», как зафиксировано в Product Brief) и краткой инструкцией сборки из исходников.
- **AC1/AC2 (сборка `.app` в Release, упаковка в `.dmg`) подтверждены локальным прогоном эквивалентных команд** — не через реальный GitHub Actions запуск (см. Scope Boundary), но с теми же самыми командами/путями, что в workflow-файле.
- **AC3 (публикация как asset GitHub Release) — НЕ проверено вживую.** Требует реального пуша тэга, который создаёт видимый публичный артефакт (GitHub Release) в репозитории пользователя — это осознанно не сделано автоматически, см. Scope Boundary. `softprops/action-gh-release@v2` — широко используемый, актуально поддерживаемый Action, его собственное поведение (создание Release + аплоад файлов из `files:`) не тестировалось повторно вручную, это на его собственной стороне.
- **AC4 (инструкция по Gatekeeper в README) подтверждена вручную** — README прочитан, структура и содержание корректны, метод совпадает с зафиксированным в Product Brief.
- **Финальный подпункт Task 3** («не пушить реальный тэг без явного подтверждения пользователя») — оставлен несделанным намеренно до отдельного разговора с пользователем после завершения этой истории; сам факт того, что тэг НЕ был запушен в рамках этой сессии, уже удовлетворяет требованию.
- `cargo test`: 25/25 прошли (sanity-check, `engine/` не менялся).

### Code Review (`/code-review high`)

8 параллельных агентов-искателей (line-by-line, removed-behavior, cross-file tracer, reuse, simplification, efficiency, altitude, CLAUDE.md conventions) + верификация → 10 находок, все обработаны:

- **Исправлено (8):** версия приложения теперь реально берётся из тэга — `Info.plist`'s `CFBundleShortVersionString`/`CFBundleVersion` заменены на `$(MARKETING_VERSION)`/`$(CURRENT_PROJECT_VERSION)`, `project.yml` задаёт дефолты (1.0/1) для локальных сборок, новый шаг workflow «Determine version» вычисляет версию из тэга (или `0.0.0-dev` для `workflow_dispatch`) и передаёт её как command-line override в `xcodebuild` — **проверено сквозным тестом** (реальная Release-сборка с оверрайдом дала `.app` с версией `1.2.3`/`42` вместо дефолтной, упаковка через `package-dmg.sh` подтвердила версию и внутри `.dmg`); добавлен `workflow_dispatch` триггер с publish-шагом, огороженным `if: github.event_name == 'push'`, — можно безопасно прогнать реальный CI-раннер без публикации настоящего Release; версия Xcode теперь запинена (`maxim-lobanov/setup-xcode@v1`, `16.2`) вместо дефолта раннера; **найден и исправлен реальный, измеримый баг** в уже существующем `scripts/build-engine.sh` — движок компилировался дважды (второй вызов `cargo run --bin uniffi-bindgen` не передавал `--target`, из-за чего Cargo использовал отдельный, нетронутый первым билдом каталог) — исправлено добавлением `--target aarch64-apple-darwin`; **проверено эмпирически**: после `cargo clean` первый билд занял 48.99с, второй (ранее дублирующий) — теперь 0.73с; `brew install xcodegen` получил `HOMEBREW_NO_AUTO_UPDATE=1` + вывод версии в лог; добавлен `actions/cache@v4` для Rust-зависимостей (keyed на `Cargo.lock`); упаковка `.dmg` вынесена в новый `scripts/package-dmg.sh` (по аналогии с `build-engine.sh`) — заодно устранило дублирование имени приложения/тома/dmg-файла (все теперь выводятся из одного `basename` аргумента).
- **Осознанно не менялось, задокументировано (2):** дублирование `xcodebuild`-команды между README и workflow — README намеренно оставлен как прямая команда, тот же паттерн, что использован во всех предыдущих историях проекта для локальной разработки, оборачивать в скрипт ради этой находки означало бы менять устоявшуюся конвенцию; arm64-target захардкожен в 3 местах (`project.yml`, `build-engine.sh`, `release.yml`) — добавлен комментарий-кросс-ссылка для discoverability через `grep`, но не унифицировано — единый механизм конфигурации через YAML/shell/project.yml был бы избыточной инженерией для $0-budget single-target hobby-проекта.

Билд после фиксов: `xcodegen generate` + `xcodebuild -configuration Debug` — BUILD SUCCEEDED (локальный dev-цикл не сломан, `Info.plist` резолвится в `1.0`/`1` по умолчанию — проверено `PlistBuddy`). `actionlint` + `python3 -c "import yaml..."` — чисто на финальной версии `release.yml`. `cargo test`: 25/25. Приложение перезапущено, живо.

### File List

- `.github/workflows/release.yml` — NEW: CI-workflow, триггер по тэгу `v*` + `workflow_dispatch` (AD-10), с фиксами код-ревью (версия из тэга, Xcode pin, Rust cache, вызов `package-dmg.sh`)
- `README.md` — NEW: описание проекта, инструкция установки/Gatekeeper-обхода, сборка из исходников
- `scripts/package-dmg.sh` — NEW (код-ревью): упаковка `.dmg`, вынесена из inline-YAML, единый источник имени приложения/тома
- `scripts/build-engine.sh` — UPDATE (код-ревью): фикс двойной компиляции движка (`--target` добавлен в `uniffi-bindgen` вызов)
- `MyTorrent/Info.plist` — UPDATE (код-ревью): `CFBundleShortVersionString`/`CFBundleVersion` → `$(MARKETING_VERSION)`/`$(CURRENT_PROJECT_VERSION)`
- `project.yml` — UPDATE (код-ревью): дефолтные `MARKETING_VERSION`/`CURRENT_PROJECT_VERSION` для локальных сборок
- `.gitignore` — UPDATE: добавлены `/build/`, `/dmg-staging/`, `*.dmg` (локальные артефакты проверки Task 3 / CI build output)
