#!/bin/sh
set -eu

# Packages a built .app into an unsigned, drag-to-install .dmg (AD-10).
# Extracted out of .github/workflows/release.yml (code-review fix, Story 4.3)
# so it's a single, locally-runnable source of truth instead of two hand-kept
# copies (the workflow YAML and whatever a developer retypes at a terminal to
# verify it) — same reasoning that already justified scripts/build-engine.sh.
#
# Usage: scripts/package-dmg.sh <path-to-.app>
# Produces: <app-name>.dmg in the current working directory.

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <path-to-.app>" >&2
  exit 1
fi

APP_PATH="$1"
APP_NAME="$(basename "$APP_PATH" .app)"
STAGING_DIR="dmg-staging"

rm -rf "$STAGING_DIR" "$APP_NAME.dmg"
mkdir -p "$STAGING_DIR"
cp -R "$APP_PATH" "$STAGING_DIR/"
ln -s /Applications "$STAGING_DIR/Applications"
hdiutil create -volname "$APP_NAME" -srcfolder "$STAGING_DIR" -ov -format UDZO "$APP_NAME.dmg"
rm -rf "$STAGING_DIR"
