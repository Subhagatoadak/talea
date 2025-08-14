#!/usr/bin/env bash
set -euo pipefail

# Build packages for the Talea Tauri IDE. Requires Node.js and the Tauri CLI.

ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
DIST_DIR="$ROOT_DIR/dist"

cd "$ROOT_DIR/ide"

# Install Node dependencies if needed and build the Tauri application
if [ ! -d node_modules ]; then
  npm install
fi
npm run tauri build

BUNDLE_DIR="src-tauri/target/release/bundle"
mkdir -p "$DIST_DIR"

# Copy platform-specific bundles if they exist
cp "$BUNDLE_DIR"/dmg/*.dmg "$DIST_DIR" 2>/dev/null || echo "No DMG produced"
cp "$BUNDLE_DIR"/msi/*.msi "$DIST_DIR" 2>/dev/null || true
cp "$BUNDLE_DIR"/appimage/*.AppImage "$DIST_DIR" 2>/dev/null || true

# On macOS, launch the IDE so the interpreter is immediately available
APP_BUNDLE="$BUNDLE_DIR/macos/Talea IDE.app"
if command -v open >/dev/null 2>&1 && [ -d "$APP_BUNDLE" ]; then
  echo "Launching Talea IDE..."
  open "$APP_BUNDLE"
fi

echo "Artifacts written to $DIST_DIR"
