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
  br_tmp="dist/diff_bg.wasm.br.tmp"
  br_out="dist/diff_bg.wasm.br"
  rm -f "$br_tmp"
  brotli -q 11 -o "$br_tmp" dist/diff_bg.wasm

  # Never publish an empty precompressed asset: the Worker would serve it as a
  # valid brotli response and the app would fail to load.
  if [ ! -s "$br_tmp" ]; then
    rm -f "$br_tmp"
    echo "error: brotli produced an empty $br_out" >&2
    exit 1
  fi

  mv -f "$br_tmp" "$br_out"
  echo "brotli: $br_out ($(wc -c < "$br_out" | tr -d ' ') bytes)"
else
  echo "warning: brotli CLI not found; skipping dist/diff_bg.wasm.br" >&2
fi
