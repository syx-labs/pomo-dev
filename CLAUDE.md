# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Vite+ monorepo with three workspaces:

- **`apps/website`** — Vanilla TypeScript + Vite app (counter demo). Entry: `src/main.ts`, served via `index.html`.
- **`apps/pomodoro`** — Tauri v2 desktop app (Pomodoro timer + task manager). Vue 3 + Pinia frontend, Rust + SQLite backend. Entry: `src/main.ts` (Vue), `src-tauri/src/main.rs` (Rust).
- **`packages/utils`** — Publishable TypeScript library built with `vp pack` (tsdown). Exports from `src/index.ts`, tests in `tests/`.

Workspaces are defined in root `package.json` (`packages/*`, `apps/*`, `tools/*` — `tools/` does not exist yet). Package manager is npm (via `packageManager` field), but all commands go through the `vp` CLI.

**Requirements:** Node.js >=22.12.0, Rust toolchain (for Tauri apps).

## Commands

```bash
# Install dependencies (run first)
vp install

# Full validation (format + lint + test + build)
vp run ready

# Dev server (website)
vp run dev              # or: vp run website#dev

# Build everything
vp run build -r

# Test everything
vp run test -r

# Run a single test file
vp test packages/utils/tests/index.test.ts

# Lint & format & type-check
vp check                # runs fmt + lint + tsc
vp check --fix          # auto-fix

# Package-specific commands
vp run website#build    # build website only
vp run utils#test       # test utils only

# Pomodoro (Tauri desktop app — run from apps/pomodoro/)
vp run pomodoro#tauri dev   # Run Tauri app in dev mode (Vite on :1420 + Rust)
vp run pomodoro#tauri build # Build production binary

# Add a dependency
vp add <pkg> -w <workspace>   # e.g. vp add lodash -w utils
```

## Using Vite+, the Unified Toolchain for the Web

This project uses Vite+, a unified toolchain built on Vite, Rolldown, Vitest, tsdown, Oxlint, Oxfmt, and Vite Task. The global CLI is `vp`.

### Critical Rules

- **Never use npm/pnpm/yarn directly.** Use `vp` for everything (install, add, remove, run).
- **Never run `vp vitest` or `vp oxlint`.** Use `vp test` and `vp lint`.
- **Import from `vite-plus`, not `vite` or `vitest`.** Use `import { defineConfig } from 'vite-plus'` and `import { expect, test, vi } from 'vite-plus/test'`.
- **Do not install vitest, oxlint, oxfmt, or tsdown.** They are bundled in vite-plus.
- **Use `vp dlx` instead of `npx`.** For one-off binaries.
- **Script conflicts:** `vp` built-in commands take precedence over `package.json` scripts. Use `vp run <script>` to run package.json scripts explicitly (e.g., `vp run ready`).
- **Type-aware linting** is configured in `vite.config.ts` — no need for extra plugins.

### Pre-commit Hook

A pre-commit hook (`.vite-hooks/pre-commit`) runs `vp check --fix` on staged files via `vp staged`.

## Architecture Notes

- **`packages/utils`** uses `vp pack` with `dts.tsgo: true` for fast declaration generation via TypeScript Go. Exports are auto-configured from `package.json` exports field. Output goes to `dist/`.
- **`apps/website`** is a vanilla Vite app (no framework). It uses DOM manipulation directly in `src/main.ts`.
- **`apps/pomodoro`** is a Tauri v2 app. Frontend is Vue 3 + vue-router + Pinia. Backend is Rust with SQLite (schema inline in `src-tauri/src/db.rs`). IPC via `@tauri-apps/api/core` invoke. Plugins: notification, global-shortcut, autostart.
- Root `vite.config.ts` configures staged file checking and type-aware linting globally. Per-workspace configs extend or override as needed.
