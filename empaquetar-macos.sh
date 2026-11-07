#!/usr/bin/env bash
# Crea "Rust Turbo Assembler.app" (binario universal Intel + Apple Silicon)
# y dist/rust-tasm-macos.zip. Se ejecuta en macOS (lo usa GitHub Actions).
set -euo pipefail
cd "$(dirname "$0")"
APPNAME="Rust Turbo Assembler"
VERSION="$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)"

rustup target add x86_64-apple-darwin aarch64-apple-darwin
export MACOSX_DEPLOYMENT_TARGET=11.0
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
T="${CARGO_TARGET_DIR:-target}"

rm -rf dist && mkdir -p dist
APP="dist/$APPNAME.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create -output "$APP/Contents/MacOS/rust-tasm" \
    "$T/x86_64-apple-darwin/release/rust-tasm" "$T/aarch64-apple-darwin/release/rust-tasm"
cp -R BIN docs locales examples "$APP/Contents/Resources/"

# Ícono .icns a partir del PNG de 1024 px.
ICONSET="dist/AppIcon.iconset"
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
    sips -z $s $s assets/icon-1024.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    sips -z $((s*2)) $((s*2)) assets/icon-1024.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
rm -rf "$ICONSET"

cat > "$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>$APPNAME</string>
    <key>CFBundleDisplayName</key><string>$APPNAME</string>
    <key>CFBundleIdentifier</key><string>io.github.fulish-code.rust-tasm</string>
    <key>CFBundleVersion</key><string>$VERSION</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundleExecutable</key><string>rust-tasm</string>
    <key>CFBundleIconFile</key><string>AppIcon</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>LSMinimumSystemVersion</key><string>11.0</string>
    <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
    <key>NSHighResolutionCapable</key><true/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key><string>Assembly source</string>
            <key>CFBundleTypeRole</key><string>Editor</string>
            <key>CFBundleTypeExtensions</key><array><string>asm</string><string>ASM</string><string>inc</string><string>INC</string></array>
        </dict>
    </array>
</dict>
</plist>
EOF

# Firma local (ad hoc): necesaria en Apple Silicon para que el sistema lo abra.
codesign --force --deep --sign - "$APP"
(cd dist && ditto -c -k --keepParent "$APPNAME.app" rust-tasm-macos.zip)
(cd dist && shasum -a 256 rust-tasm-macos.zip > rust-tasm-macos.zip.sha256)
lipo -info "$APP/Contents/MacOS/rust-tasm"
ls -la dist
