#!/usr/bin/env bash
set -euo pipefail

# This script builds a macOS .app bundle and DMG for Talea.
# It also packages optional example downloads.
# Requires: macOS with Xcode command line tools, Node.js, and npx create-dmg.

ROOT_DIR="$(cd "$(dirname "$0")"/.. && pwd)"
DIST_DIR="$ROOT_DIR/dist"
APP_NAME="Talea"

cd "$ROOT_DIR"

# Build release binary
cargo build --release

VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
APP_DIR="$DIST_DIR/${APP_NAME}.app"

# Prepare bundle structure
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp target/release/talea "$APP_DIR/Contents/MacOS/${APP_NAME}"
cp assets/icon.icns "$APP_DIR/Contents/Resources/icon.icns"

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key><string>${APP_NAME}</string>
    <key>CFBundleExecutable</key><string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key><string>org.talea.app</string>
    <key>CFBundleVersion</key><string>${VERSION}</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>LSMinimumSystemVersion</key><string>10.13</string>
    <key>CFBundleIconFile</key><string>icon</string>
</dict>
</plist>
PLIST

# Sign bundle with an ad-hoc identity to avoid codesign errors on macOS
if command -v codesign >/dev/null 2>&1; then
    codesign --force --deep --sign - "$APP_DIR"
fi

mkdir -p "$DIST_DIR"
DMG_NAME="$DIST_DIR/${APP_NAME}-${VERSION}.dmg"

# Create DMG (requires macOS hdiutil)
if command -v hdiutil >/dev/null 2>&1; then
    npx --yes create-dmg "$APP_DIR" "$DIST_DIR" --overwrite >/tmp/dmg.log && tail -n 20 /tmp/dmg.log
    mv "$DIST_DIR/${APP_NAME} ${VERSION}.dmg" "$DMG_NAME" 2>/dev/null || true
else
    echo "hdiutil not found; skipping DMG creation. Run this on macOS." >&2
fi

# Optional downloads: package examples
if [ -d examples ]; then
    tar -czf "$DIST_DIR/${APP_NAME}-examples.tar.gz" examples
fi

echo "Artifacts written to $DIST_DIR"
