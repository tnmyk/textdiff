#!/usr/bin/env bash
set -euo pipefail

# GPUI's debug-friendly build leaves a large wasm module. Run this after
# `trunk build --release` to cut the transfer size. `--disable-compact-imports`
# keeps the output loadable by browsers that do not enable the compact-imports
# proposal by default.
wasm-opt -Oz --all-features --disable-compact-imports \
  -o dist/diff_bg.wasm dist/diff_bg.wasm

# Ship a precompressed copy next to the wasm. The Worker serves it to clients
# that accept brotli, which is smaller than Cloudflare's on-the-fly compression.
# Quality 11 is the smallest output; the extra build time is paid once in CI.
if command -v brotli >/dev/null 2>&1; then
  brotli -q 11 --force -o dist/diff_bg.wasm.br dist/diff_bg.wasm
  echo "brotli: dist/diff_bg.wasm.br ($(wc -c < dist/diff_bg.wasm.br | tr -d ' ') bytes)"
else
  echo "warning: brotli CLI not found; skipping dist/diff_bg.wasm.br" >&2
fi
