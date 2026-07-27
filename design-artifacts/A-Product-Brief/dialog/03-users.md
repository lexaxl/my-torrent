# Step 3: User Definition

**Completed:** 2026-07-27
**Session:** 1

---

## Opening Question

**Agent asked:**
"Типичный сценарий использования: что за торренты обычно качают, одна закачка или несколько параллельно, оставляют раздавать надолго? Конкретное раздражение от текущих инструментов? Цель/ощущение от использования? Вторичные пользователи?"

**User's initial response:**
"Приложение для скачивания разных файлов параллельно. Оставлять раздавать на определенный период времени. Необходимо учесть такой параметр и задавать его в настройках. В текущих инструментах бесит загроможденный UI, лишние окна и уведомления, медленная работа, высокое потребление ресурсов и сложность настройки. Цель - "поставил и забыл" с возможностью перейти в настройки и настроить под себя."

---

## User Exploration

### Primary User Discovery

**Key exchanges:**

**Agent:** Что за торренты обычно качают — одна закачка или несколько параллельно?
**User:** Разные файлы, параллельно (несколько закачек одновременно).

**Agent:** Оставляют раздавать надолго после скачивания?
**User:** Раздавать на определённый период времени — это должно быть настраиваемым параметром (не бесконечно, не сразу выключать).

**Agent:** Что конкретно бесит в текущих клиентах?
**User:** Загромождённый UI, лишние окна и уведомления, медленная работа, высокое потребление ресурсов, сложность настройки.

---

## Reflection Checkpoint

**Agent's synthesis:**
"Основной пользователь — вы сами и похожие: скачивают параллельно разные файлы, хотят раздавать ограниченное настраиваемое время, бесит захламлённый UI/лишние окна-уведомления/тормоза/прожорливость/сложная настройка. Цель — 'поставил и забыл' по умолчанию, но с доступом к настройкам для тонкой подстройки (включая период раздачи). Один тип пользователя, без отдельного secondary-профиля."

**User response:**
- [x] Confirmed
- [ ] Corrected

**Corrections (if any):**
None — user explicitly confirmed single user type, no secondary profile.

---

## Primary User Definition

**Who they are:**
macOS users who value minimalism and simplicity (Alex + like-minded people) — not defined by technical skill level, but by taste/values around software.

**Their context:**
Downloading multiple different files in parallel via BitTorrent, running the client continuously in the background on a Mac.

**Their frustrations:**
- Cluttered/overloaded UI (qBittorrent-style)
- Extra windows and notifications getting in the way
- Slow/sluggish performance
- High resource consumption (CPU/memory) for something meant to run constantly
- Complex, unclear configuration (across various existing clients)

**What they're trying to achieve:**
"Set and forget" as the default experience — install, open a `.torrent`, it downloads and manages seeding without babysitting — while still being able to drop into settings to configure specifics when they want to (notably: configurable seeding duration per torrent/globally).

**How they currently solve this:**
qBittorrent (heavy, being deprecated on macOS), Transmission (light but inconvenient), other unnamed clients (confusing).

---

## Secondary Users (if applicable)

**User 2:** None identified — confirmed single user type for this project. May revisit post-launch if public/open-source feedback (per Vision) surfaces a distinct less-technical segment.
**User 3:** —

---

## User Scenarios Captured

**Scenario 1:** User downloads several unrelated torrents at once (e.g. a Linux ISO and some software), each configured to seed for a set period afterward, without needing to remember to manually stop seeding later.
**Scenario 2:** User leaves the app running in the background for days/weeks; it stays out of the way (no popup clutter, low CPU/memory) until they glance at the menu-bar icon or open the main window to check status.

---

**Documented in:** `wds-project-outline.yaml` → `users`
