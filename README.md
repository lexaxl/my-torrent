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

Слайды листаются стрелками / пробелом. Исходники — в [`docs/presentations/`](docs/presentations/).
