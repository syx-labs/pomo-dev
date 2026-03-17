<p align="center">
  <img src="apps/pomodoro/src-tauri/icons/128x128@2x.png" width="128" height="128" alt="Pomo logo">
</p>

<h1 align="center">Pomo</h1>

<p align="center">
  <strong>A beautiful, open-source Pomodoro timer for macOS, Windows, and Linux.</strong>
</p>

<p align="center">
  <a href="https://github.com/syx-labs/pomo-dev/releases/latest">
    <img src="https://img.shields.io/github/v/release/syx-labs/pomo-dev?style=flat-square" alt="Latest Release">
  </a>
  <a href="https://github.com/syx-labs/pomo-dev/releases/latest">
    <img src="https://img.shields.io/github/downloads/syx-labs/pomo-dev/total?style=flat-square" alt="Downloads">
  </a>
  <a href="https://github.com/syx-labs/pomo-dev/blob/main/LICENSE">
    <img src="https://img.shields.io/github/license/syx-labs/pomo-dev?style=flat-square" alt="License">
  </a>
</p>

---

## Features

- **Pomodoro Timer** — Work/break cycles with customizable durations (25/5/15 min defaults)
- **Task Manager** — Create, prioritize, and track tasks with estimated pomodoros
- **Analytics** — Daily/weekly stats, heatmap, streaks, and time-of-day insights
- **Ambient Sounds** — Rain, cafe, white noise, nature, and lofi with mixable presets
- **AI Assistant** — Daily briefings, session debriefs, and productivity coaching
- **Integrations** — Slack, Discord, and webhook notifications for session events
- **System Tray** — Start/pause/skip from the tray without opening the window
- **Auto-Update** — Seamless in-app updates when a new version is released
- **Cross-Platform** — Native performance on macOS (ARM + Intel), Windows, and Linux

## Download

Get the latest release for your platform:

| Platform              | Download                                                            |
| --------------------- | ------------------------------------------------------------------- |
| macOS (Apple Silicon) | [`.dmg`](https://github.com/syx-labs/pomo-dev/releases/latest)      |
| macOS (Intel)         | [`.dmg`](https://github.com/syx-labs/pomo-dev/releases/latest)      |
| Windows               | [`.exe`](https://github.com/syx-labs/pomo-dev/releases/latest)      |
| Linux (Debian/Ubuntu) | [`.deb`](https://github.com/syx-labs/pomo-dev/releases/latest)      |
| Linux (AppImage)      | [`.AppImage`](https://github.com/syx-labs/pomo-dev/releases/latest) |

> **macOS note:** The app is not notarized yet. On first launch, right-click the app and choose **Open**, then click **Open** again in the dialog.

## Tech Stack

| Layer     | Technology                                                          |
| --------- | ------------------------------------------------------------------- |
| Framework | [Tauri v2](https://v2.tauri.app/)                                   |
| Frontend  | Vue 3, TypeScript, Pinia, vue-router                                |
| Backend   | Rust, SQLite (rusqlite), rodio (audio)                              |
| Build     | [Vite+](https://viteplus.dev/) monorepo with Vite, Rolldown, Oxlint |
| CI/CD     | GitHub Actions — cross-platform builds with auto-update             |

## Development

### Prerequisites

- Node.js >= 22.12.0
- Rust toolchain (stable)
- [Vite+ CLI](https://viteplus.dev/) (`vp`)

### Getting Started

```bash
# Install dependencies
vp install

# Run the Pomodoro app in dev mode (Vite on :1420 + Rust backend)
vp run pomodoro#tauri dev

# Run all checks (format + lint + typecheck + tests)
vp run ready
```

### Project Structure

```
apps/
  pomodoro/              # Tauri v2 desktop app
    src/                 # Vue 3 frontend
      views/             # Timer, Tasks, Analytics, Settings
      stores/            # Pinia stores (timer, tasks, settings, sound)
      components/        # UI components
      lib/tauri.ts       # IPC bridge to Rust backend
    src-tauri/           # Rust backend
      src/main.rs        # App setup, plugins, system tray
      src/commands.rs    # IPC command handlers
      src/db.rs          # SQLite schema + queries
      src/timer.rs       # Pomodoro state machine
      src/audio.rs       # Ambient sound engine
      src/ai.rs          # AI assistant
  website/               # Landing page (Vite + TypeScript)
packages/
  utils/                 # Shared TypeScript utilities
```

### Useful Commands

```bash
vp run pomodoro#tauri dev     # Dev mode with hot reload
vp run pomodoro#tauri build   # Production build
vp check                      # Format + lint + typecheck
vp check --fix                # Auto-fix lint/format issues
vp run test -r                # Run all tests
vp run build -r               # Build all packages
```

## Releasing

Releases are fully automated via GitHub Actions.

**Option A — Tag push:**

```bash
node scripts/bump-version.mjs 1.0.0
git add -A && git commit -m "chore(release): v1.0.0"
git tag v1.0.0
git push && git push --tags
```

**Option B — Manual trigger:**
Go to Actions > Release > Run workflow > enter version (e.g. `1.0.0`)

Both methods build for all platforms, generate a changelog from conventional commits, and publish a GitHub Release with all artifacts.

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feat/amazing-feature`)
3. Use [conventional commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`, etc.)
4. Run `vp run ready` before pushing
5. Open a pull request

## License

[MIT](LICENSE)
