#!/usr/bin/env bash
# ==============================================================================
# QORDY Esnaf POS - macOS Application (.app) & DMG Builder
# ==============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
VERSION="3.0.0"
APP_NAME="QORDY Esnaf POS"
BUNDLE_ID="com.qordy.esnafpos"
DIST_DIR="$PROJECT_ROOT/dist"
STAGING_DIR="$PROJECT_ROOT/build/macos_staging"
APP_BUNDLE="$STAGING_DIR/$APP_NAME.app"
DMG_NAME="QORDY_Esnaf_POS_v${VERSION}_macOS.dmg"

echo "============================================================"
echo "  QORDY Esnaf POS - macOS App & DMG Packaging v$VERSION"
echo "============================================================"

cd "$PROJECT_ROOT"

# 1. Compile Release Binary
echo "[1/5] macOS release binary derleniyor..."
cargo build --release

# 2. Prepare Clean Staging Directory
echo "[2/5] macOS .app bundle hazırlanıyor..."
rm -rf "$STAGING_DIR"
mkdir -p "$DIST_DIR"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

# Copy binary & strip debug symbols for minimum size
cp "target/release/qordy_pos_rs" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
strip "$APP_BUNDLE/Contents/MacOS/$APP_NAME" 2>/dev/null || true

# Copy assets and AppIcon
cp -r "assets" "$APP_BUNDLE/Contents/Resources/"
cp "assets/AppIcon.icns" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"

# Create Enterprise Info.plist
cat > "$APP_BUNDLE/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>tr</string>
    <key>CFBundleDisplayName</key>
    <string>QORDY Esnaf POS</string>
    <key>CFBundleExecutable</key>
    <string>$APP_NAME</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSUIElement</key>
    <false/>
</dict>
</plist>
EOF

echo "[3/5] .app paketi $APP_BUNDLE dizininde başarıyla oluşturuldu."

# 3. Create Applications symlink for drag-and-drop installer
echo "[4/5] Applications sürükle-bırak kısayolu ekleniyor..."
rm -f "$STAGING_DIR/Applications"
ln -s /Applications "$STAGING_DIR/Applications"

# 4. Build DMG using native hdiutil
echo "[5/5] Apple hdiutil ile sıkıştırılmış DMG imajı üretiliyor..."
FINAL_DMG="$DIST_DIR/$DMG_NAME"
rm -f "$FINAL_DMG"
hdiutil create \
    -volname "QORDY Esnaf POS Kurulum" \
    -srcfolder "$STAGING_DIR" \
    -ov \
    -format UDZO \
    "$FINAL_DMG"

DMG_SIZE="$(du -sh "$FINAL_DMG" | cut -f1)"
shasum -a 256 "$FINAL_DMG" > "$DIST_DIR/SHA256SUMS_macOS.txt"

echo "============================================================"
echo "  BAŞARILI: macOS .app ve Kurulum DMG Paketi Hazır!"
echo "  App: $APP_BUNDLE"
echo "  DMG: $FINAL_DMG ($DMG_SIZE)"
echo "============================================================"
