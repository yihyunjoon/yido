#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
ARTIFACTS="$ROOT/build/.artifacts"
APP_PATH=$("$SCRIPT_DIR/package.sh" | tail -n 1)
INSTALL_DIR="$HOME/Library/Input Methods"
DESTINATION="$INSTALL_DIR/Yido.app"
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"

case "$APP_PATH" in
  "$ARTIFACTS"/*) ;;
  *) echo "Refusing to install app outside build/.artifacts: $APP_PATH" >&2; exit 1 ;;
esac

case "$DESTINATION" in
  "$HOME/Library/Input Methods/Yido.app") ;;
  *) echo "Refusing to replace unexpected destination: $DESTINATION" >&2; exit 1 ;;
esac

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
