#!/usr/bin/env bash
set -euo pipefail

# Install dependencies and build the Talea IDE.
# This helper installs the IDE's JavaScript packages and
# produces platform-specific bundles via Tauri.

ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
IDE_DIR="$ROOT_DIR/ide"

if ! command -v npm >/dev/null 2>&1; then
  echo "npm is required to build the Talea IDE" >&2
  exit 1
fi

cd "$IDE_DIR"

# Install JS dependencies if missing
if [ ! -d node_modules ]; then
  npm install
fi

# Build the Tauri application (creates installers under src-tauri/target)
npm run tauri build

echo "Talea IDE bundles are located in ide/src-tauri/target/release/bundle"
