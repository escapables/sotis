#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-$ROOT_DIR/release}"
BINARY_PATH="$ROOT_DIR/target/release/sotis-gui"
LIB_DIR="$OUT_DIR/lib"
TESSDATA_DIR="$OUT_DIR/share/tessdata"

need_cmd() {
  local name="$1"
  if ! command -v "$name" >/dev/null 2>&1; then
    echo "error: required command '$name' is not available" >&2
    exit 1
  fi
}

find_tessdata() {
  local candidates=(
    "${TESSDATA_PREFIX:-}/eng.traineddata"
    "/usr/share/tesseract-ocr/5/tessdata/eng.traineddata"
    "/usr/share/tesseract-ocr/4.00/tessdata/eng.traineddata"
    "/usr/share/tessdata/eng.traineddata"
    "/usr/local/share/tessdata/eng.traineddata"
  )

  for candidate in "${candidates[@]}"; do
    if [[ -n "$candidate" && -f "$candidate" ]]; then
      echo "$candidate"
      return 0
    fi
  done

  return 1
}

find_lib_on_system() {
  local lib_name="$1"
  local lib_prefix="${lib_name}."
  local search_roots=(
    "/usr/lib"
    "/usr/lib64"
    "/usr/local/lib"
    "/lib"
    "/lib64"
    "/usr/lib/x86_64-linux-gnu"
    "/lib/x86_64-linux-gnu"
  )

  for root in "${search_roots[@]}"; do
    [[ -d "$root" ]] || continue
    local found
    found="$(
      find "$root" -maxdepth 2 \( -type f -o -type l \) \
        \( -name "$lib_name" -o -name "${lib_prefix}*" \) \
        2>/dev/null | head -n 1 || true
    )"
    if [[ -n "$found" ]]; then
      echo "$found"
      return 0
    fi
  done

  return 1
}

copy_required_lib() {
  local source_name="$1"
  local dest_name="$2"
  local root_override="$ROOT_DIR/$source_name"
  local source_path=""

  if [[ -f "$root_override" ]]; then
    source_path="$root_override"
  else
    source_path="$(find_lib_on_system "$source_name" || true)"
  fi

  if [[ -z "$source_path" ]]; then
    echo "error: could not locate $source_name (checked repo root and common system lib dirs)" >&2
    exit 1
  fi

  cp -f "$source_path" "$LIB_DIR/$dest_name"
}

need_cmd cargo
need_cmd find

echo "[bundle] building OCR-enabled release binary"
cargo build --release -p sotis-gui --features ocr

if [[ ! -x "$BINARY_PATH" ]]; then
  echo "error: expected binary not found at $BINARY_PATH" >&2
  exit 1
fi

echo "[bundle] preparing output at $OUT_DIR"
rm -rf "$OUT_DIR"
mkdir -p "$LIB_DIR" "$TESSDATA_DIR"

cp -f "$BINARY_PATH" "$OUT_DIR/sotis-gui"
copy_required_lib "libpdfium.so" "libpdfium.so"
copy_required_lib "libtesseract.so.5" "libtesseract.so.5"
copy_required_lib "libleptonica.so.6" "libleptonica.so.6"

tessdata_source="$(find_tessdata || true)"
if [[ -z "$tessdata_source" ]]; then
  echo "error: could not locate eng.traineddata (set TESSDATA_PREFIX or install tesseract data)" >&2
  exit 1
fi
cp -f "$tessdata_source" "$TESSDATA_DIR/eng.traineddata"

cat >"$OUT_DIR/run.sh" <<'EOF'
#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)"
export LD_LIBRARY_PATH="$DIR/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export TESSDATA_PREFIX="$DIR/share/tessdata"
exec "$DIR/sotis-gui" "$@"
EOF
chmod +x "$OUT_DIR/run.sh"

echo "[bundle] complete"
echo "[bundle] bundle contents:"
find "$OUT_DIR" -maxdepth 3 -type f | sort
