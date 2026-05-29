#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/.." && pwd)
APP_PATH=$("$ROOT/Scripts/package_input_method.sh" | tail -n 1)
INSTALL_DIR="$HOME/Library/Input Methods"
DESTINATION="$INSTALL_DIR/Yido.app"
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"

mkdir -p "$INSTALL_DIR"
pkill -x Yido 2>/dev/null || true
rm -rf "$DESTINATION"
cp -R "$APP_PATH" "$DESTINATION"
xattr -cr "$DESTINATION"
codesign --force --sign - "$DESTINATION"

if [[ -x "$LSREGISTER" ]]; then
  "$LSREGISTER" -f "$DESTINATION"
fi

killall cfprefsd 2>/dev/null || true

echo "Installed $DESTINATION"
echo "Open System Settings > Keyboard > Input Sources and add Yido if it is not already listed."
