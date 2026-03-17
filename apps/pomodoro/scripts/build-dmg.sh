#!/bin/bash
set -euo pipefail

# Build Pomodoro.app and DMG for macOS distribution
# Usage: ./scripts/build-dmg.sh

cd "$(dirname "$0")/.."

echo "==> Building Tauri app..."
cargo tauri build

# Find the built .app bundle
APP_DIR="src-tauri/target/release/bundle/macos"
APP_PATH="$APP_DIR/Pomodoro.app"

if [ ! -d "$APP_PATH" ]; then
    echo "ERROR: $APP_PATH not found. Build may have failed."
    exit 1
fi

# Ad-hoc code sign (helps with some Gatekeeper scenarios)
echo "==> Ad-hoc signing the app bundle..."
codesign --force --deep -s - "$APP_PATH"

echo ""
echo "==> Build complete!"
echo "    App:  $APP_PATH"
echo "    DMG:  src-tauri/target/release/bundle/dmg/"
echo ""
echo "NOTE: This app is NOT notarized. Recipients need to bypass Gatekeeper:"
echo "  Option 1: Right-click the app → Open → Open (first time only)"
echo "  Option 2: Run in Terminal: xattr -cr /path/to/Pomodoro.app"
echo ""
echo "See INSTALL.md for recipient instructions."
