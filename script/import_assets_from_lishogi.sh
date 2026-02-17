#!/usr/bin/env bash
set -euo pipefail

LISHOGI_ROOT="${1:-/home/harry/Play/lishogi}"
TARGET_ROOT="${2:-.}"

echo "Importing assets from: ${LISHOGI_ROOT}"
echo "Target project root: ${TARGET_ROOT}"

mkdir -p "${TARGET_ROOT}/assets/pieces/standard/western"
mkdir -p "${TARGET_ROOT}/assets/pieces/standard/ryoko_1kanji"
mkdir -p "${TARGET_ROOT}/assets/boards/lishogi"
mkdir -p "${TARGET_ROOT}/assets/sounds/lishogi/shogi"

cp -f "${LISHOGI_ROOT}/ui/@build/pieces/assets/standard/western/"*.svg \
  "${TARGET_ROOT}/assets/pieces/standard/western/"

cp -f "${LISHOGI_ROOT}/ui/@build/pieces/assets/standard/ryoko_1kanji/"*.svg \
  "${TARGET_ROOT}/assets/pieces/standard/ryoko_1kanji/"

cp -f "${LISHOGI_ROOT}/ui/@build/static/assets/images/boards/"*.jpg \
  "${TARGET_ROOT}/assets/boards/lishogi/" || true
cp -f "${LISHOGI_ROOT}/ui/@build/static/assets/images/boards/"*.png \
  "${TARGET_ROOT}/assets/boards/lishogi/" || true

cp -f "${LISHOGI_ROOT}/ui/@build/static/assets/sound/ogg/system/shogi/"*.ogg \
  "${TARGET_ROOT}/assets/sounds/lishogi/shogi/"

echo "Done."
