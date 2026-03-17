# Pomodoro App

Tauri v2 desktop app — Pomodoro timer + task manager with local SQLite storage.

## Stack

- **Frontend:** Vue 3 + TypeScript, vue-router, Pinia (state management)
- **Backend:** Rust (Tauri v2), SQLite via rusqlite (bundled), uuid v4, chrono, rodio (audio)
- **Tauri Plugins:** notification, global-shortcut, autostart

## Commands

```bash
cargo tauri dev           # Run in dev mode (Vite on :1420 + Rust backend)
cargo tauri build         # Build production binary
vue-tsc --noEmit          # Type-check Vue frontend only
vp build                  # Build Vue frontend only (without Tauri)
```

## Structure

```
src/                      # Vue frontend
├── views/                # Route views (Timer, Tasks, Stats, Settings)
├── components/           # UI components (timer/, tasks/, layout/, stats/)
├── stores/               # Pinia stores (timer, tasks, settings)
├── composables/          # Vue composables (useTasks, useStats, useSettings)
├── lib/tauri.ts          # IPC bridge — all Rust command invocations + types
├── router.ts             # Vue Router config
└── styles/variables.css  # CSS design tokens (zinc/emerald dark theme)

src-tauri/                # Rust backend
├── src/main.rs           # Tauri setup, plugin registration, system tray, background timer thread
├── src/commands.rs       # 18 IPC command handlers
├── src/db.rs             # SQLite connection + inline schema migrations
├── src/models.rs         # Data structs (Task, PomodoroSession, TimerSettings)
├── src/timer.rs          # Timer state machine (PomodoroTimer)
├── capabilities/         # Tauri v2 permission capabilities
└── tauri.conf.json       # Window config (900x700), app ID: com.pomodoro.app
```

## Routes

| Path        | View         | Purpose                                             |
| ----------- | ------------ | --------------------------------------------------- |
| `/`         | TimerView    | Pomodoro timer with ring, controls, cycle indicator |
| `/tasks`    | TasksView    | Task list with CRUD, filters, priority              |
| `/stats`    | StatsView    | Daily/weekly statistics, project breakdown          |
| `/settings` | SettingsView | Timer durations, sound, autostart config            |

## SQLite Schema (inline in db.rs — no migration files)

```sql
-- tasks: id(TEXT PK), title, description, status(DEFAULT 'todo'), priority(DEFAULT 0),
--   project, due_date, estimated_pomos(DEFAULT 0), sort_order(DEFAULT 0), created_at, updated_at
-- pomodoro_sessions: id(TEXT PK), task_id(FK→tasks ON DELETE SET NULL), session_type,
--   duration_secs, started_at, ended_at, completed(DEFAULT 0), notes
-- settings: key(TEXT PK), value(TEXT)
-- Indexes: idx_sessions_task, idx_sessions_started, idx_tasks_status
```

All timestamps are **RFC3339 strings**. Stats commands expect **YYYY-MM-DD** date format.

## IPC Commands (commands.rs)

**Timer:** `start_timer`, `pause_timer`, `resume_timer`, `skip_timer`, `reset_timer`, `get_timer_state` — all return `TimerState`

**Tasks:** `create_task(input)`, `update_task(input)`, `delete_task(id)`, `list_tasks(status?, project?)`, `reorder_tasks(taskIds)`

**Sessions:** `start_pomodoro_for_task(taskId)`, `get_sessions_for_task(taskId)`

**Settings:** `get_setting(key)`, `set_setting(key, value)`, `get_all_settings()`

**Stats:** `get_daily_stats(date)`, `get_weekly_stats(startDate)`

## Key Patterns

- **IPC:** Frontend calls Rust via `invoke()` from `@tauri-apps/api/core`. All commands defined in `src/lib/tauri.ts`, implemented in `src-tauri/src/commands.rs`.
- **State:** Pinia stores wrap IPC calls. Components use stores/composables, never invoke Tauri directly.
- **Window:** `titleBarStyle: "Overlay"` (macOS native traffic lights with custom titlebar). `Titlebar.vue` has `data-tauri-drag-region`.
- **Timer events:** Rust spawns a background thread that ticks every 1s, emitting `timer:tick` (TimerState) and `timer:complete` ({ sessionType }) events.
- **System tray:** Menu with start/pause toggle, skip, and quit actions.
- **Timer cycle:** Work → Short Break → Work → ... → Long Break (every 4 cycles). Defaults: 25m/5m/15m.
- **Design tokens:** Dark theme with zinc bg, emerald (work), sky (short break), violet (long break) accents.
- **Env prefix:** Both `VITE_` and `TAURI_` prefixes are exposed to frontend.

## Gotchas

- **`update_task` requires ALL fields** — not partial updates. Always send the full `UpdateTaskInput` object with all required fields.
- **IPC parameter names must match exactly** between TypeScript `invoke()` calls and Rust `#[tauri::command]` function signatures.
- **Tauri v2 uses capabilities** (not v1 allowlists). Permissions are in `src-tauri/capabilities/default.json`.
- **Timer defaults live in TWO places** — `timer.rs` (`TimerSettings::default()`) and `stores/settings.ts` (`AppSettings` defaults). They must stay in sync.
- **Settings store does type coercion** — SQLite stores all settings as strings. The settings Pinia store converts them to bool/number based on the default value's type.
- **Dates are strings everywhere** — Timestamps are RFC3339 in SQLite. Stats endpoints expect `YYYY-MM-DD` format, not RFC3339.
- **Session count filtering** — Stats/session counts filter by `completed = 1` AND `session_type = "work"` (SQLite INTEGER boolean).
