# Pomodoro macOS `.dmg` build

This workspace already uses Tauri's native macOS bundling. Running the app build produces both a macOS `.app` bundle and a `.dmg` installer for direct download.

## Prerequisites

- macOS with Xcode Command Line Tools installed
- Rust toolchain available to Cargo
- Dependencies installed with `vp install`

## Build command

Run the Tauri production build from the workspace root:

```bash
vp run pomodoro#tauri build
```

The command executes the Vue production build first (`beforeBuildCommand: "vp build"`) and then bundles the desktop app through Tauri.

## Output artifacts

After a successful build, Tauri writes the release artifacts to:

```text
apps/pomodoro/src-tauri/target/release/bundle/
```

The relevant macOS outputs are:

- `macos/Pomodoro.app`
- `dmg/Pomodoro_<version>_<arch>.dmg`

On Apple Silicon, the generated file name includes `aarch64`. On Intel Macs, the suffix follows the host architecture used to build the app.

## Local install test

1. Run `vp run pomodoro#tauri build`
2. Open the generated `.dmg`
3. Drag `Pomodoro.app` into `Applications`
4. Launch the app from `Applications`

For a realistic check, repeat the install on a separate macOS user or machine that does not have the source tree open.

## Current limitation: unsigned build

The current `.dmg` is suitable for local testing and manual sharing, but it is **not** signed or notarized yet.

Because of that, macOS may block the first launch with a Gatekeeper warning. For local testing, use the standard macOS override flow:

- right-click the app and choose `Open`, or
- open it once from `Applications` and then allow it in `System Settings > Privacy & Security`

## Future release hardening

For public distribution without Gatekeeper warnings, add these steps later:

- Apple Developer account
- `Developer ID Application` certificate
- notarization credentials for `notarytool`
- Tauri signing/notarization secrets in the release environment
- final validation that the built `.app` and `.dmg` are signed, notarized, and stapled
