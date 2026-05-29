#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
cd "$ROOT"

APP_NAME="Yido"
BUNDLE_ID="com.yido.inputmethod.Yido"
CONNECTION_NAME="${BUNDLE_ID}_Connection"
ARTIFACTS="$ROOT/build/.artifacts"
APP="$ARTIFACTS/macos/${APP_NAME}.app"
SWIFT_SCRATCH="$ARTIFACTS/swift"
CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$ARTIFACTS/cargo}"
RUST_DYLIB="$CARGO_TARGET_DIR/release/libyido.dylib"

remove_artifact() {
  local path="$1"

  case "$path" in
    "$ARTIFACTS"/*) rm -rf "$path" ;;
    *) echo "Refusing to remove path outside build/.artifacts: $path" >&2; exit 1 ;;
  esac
}

export CARGO_TARGET_DIR
cargo build --manifest-path yido/Cargo.toml -p yido-ffi --release

# FFI C 헤더 생성 (cbindgen은 소스를 파싱하므로 빌드 프로파일과 무관)
FFI_HEADER_DIR="$ROOT/yido-swift/Sources/CYidoFFI/include"
mkdir -p "$ARTIFACTS/ffi" "$FFI_HEADER_DIR"
cbindgen yido/ffi --config cbindgen.toml --output "$ARTIFACTS/ffi/yido_ffi.h"
cp "$ARTIFACTS/ffi/yido_ffi.h" "$FFI_HEADER_DIR/yido_ffi.h"
SWIFT_BUILD_DIR=$(swift build \
  --package-path yido-swift \
  --scratch-path "$SWIFT_SCRATCH" \
  -c release \
  --show-bin-path)
swift build \
  --package-path yido-swift \
  --scratch-path "$SWIFT_SCRATCH" \
  -c release \
  --product "$APP_NAME"

remove_artifact "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$APP/Contents/Frameworks"

cp "$SWIFT_BUILD_DIR/$APP_NAME" "$APP/Contents/MacOS/$APP_NAME"
chmod +x "$APP/Contents/MacOS/$APP_NAME"
cp "$RUST_DYLIB" "$APP/Contents/Frameworks/libyido.dylib"

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
    <key>InputMethodServerPreferencesWindowControllerClass</key>
    <string>YidoPreferencesWindowController</string>
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
codesign --force --sign - "$APP/Contents/Frameworks/libyido.dylib"
codesign --force --sign - "$APP"

echo "$APP"
