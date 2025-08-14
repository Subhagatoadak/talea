#!/usr/bin/env bash
set -euo pipefail

# Build packages for the Talea Tauri IDE. Requires Node.js and the Tauri CLI.

ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
DIST_DIR="$ROOT_DIR/dist"

cd "$ROOT_DIR/ide"

# Build the Tauri application for the host platform
npm run tauri build

BUNDLE_DIR="src-tauri/target/release/bundle"
mkdir -p "$DIST_DIR"

# Copy platform-specific bundles if they exist
cp "$BUNDLE_DIR"/dmg/*.dmg "$DIST_DIR" 2>/dev/null || echo "No DMG produced"
cp "$BUNDLE_DIR"/msi/*.msi "$DIST_DIR" 2>/dev/null || true
cp "$BUNDLE_DIR"/appimage/*.AppImage "$DIST_DIR" 2>/dev/null || true

echo "Artifacts written to $DIST_DIR"
