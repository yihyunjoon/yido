#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"

APP_NAME="Yido"
BUNDLE_ID="com.yido.inputmethod.Yido"
CONNECTION_NAME="${BUNDLE_ID}_Connection"
APP="$ROOT/build/${APP_NAME}.app"
SWIFT_BUILD_DIR="$ROOT/yido-swift/.build/arm64-apple-macosx/release"
RUST_DYLIB="$ROOT/target/release/libyido_ffi.dylib"

cargo build -p yido-ffi --release
mise run ffi:header
swift build \
  --package-path yido-swift \
  -c release \
  --product "$APP_NAME"

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$APP/Contents/Frameworks"

cp "$SWIFT_BUILD_DIR/$APP_NAME" "$APP/Contents/MacOS/$APP_NAME"
chmod +x "$APP/Contents/MacOS/$APP_NAME"
cp "$RUST_DYLIB" "$APP/Contents/Frameworks/libyido_ffi.dylib"

if compgen -G "$SWIFT_BUILD_DIR/"'*.bundle' >/dev/null; then
  cp -R "$SWIFT_BUILD_DIR/"*.bundle "$APP/Contents/Resources/"
fi

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>이도</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>CFBundleSupportedPlatforms</key>
    <array>
        <string>MacOSX</string>
    </array>
    <key>InputMethodConnectionName</key>
    <string>${CONNECTION_NAME}</string>
    <key>InputMethodServerControllerClass</key>
    <string>YidoInputController</string>
    <key>InputMethodServerDelegateClass</key>
    <string>YidoInputController</string>
    <key>InputMethodSessionController</key>
    <string>YidoInputController</string>
    <key>LSMinimumSystemVersion</key>
    <string>26.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>TISInputSourceID</key>
    <string>${BUNDLE_ID}</string>
    <key>TISIntendedLanguage</key>
    <string>ko</string>
    <key>tsInputMethodCharacterRepertoireKey</key>
    <array>
        <string>Hang</string>
    </array>
    <key>ComponentInputModeDict</key>
    <dict>
        <key>tsInputModeListKey</key>
        <dict>
            <key>${BUNDLE_ID}.ko</key>
            <dict>
                <key>TISIconLabels</key>
                <dict>
                    <key>Primary</key>
                    <string>이도</string>
                </dict>
                <key>TISInputSourceID</key>
                <string>${BUNDLE_ID}.ko</string>
                <key>TISIntendedLanguage</key>
                <string>ko</string>
                <key>tsInputModeCharacterRepertoireKey</key>
                <array>
                    <string>Hang</string>
                </array>
                <key>tsInputModeIsVisibleKey</key>
                <true/>
                <key>tsInputModePrimaryInScriptKey</key>
                <false/>
                <key>tsInputModeScriptKey</key>
                <string>smUnicode</string>
            </dict>
        </dict>
        <key>tsVisibleInputModeOrderedArrayKey</key>
        <array>
            <string>${BUNDLE_ID}.ko</string>
        </array>
    </dict>
</dict>
</plist>
PLIST

plutil -lint "$APP/Contents/Info.plist"
xattr -cr "$APP"
codesign --force --sign - "$APP/Contents/Frameworks/libyido_ffi.dylib"
codesign --force --sign - "$APP"

echo "$APP"
