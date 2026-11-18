#!/usr/bin/env bash
set -euo pipefail

# GPUI's debug-friendly build leaves a large wasm module. Run this after
# `trunk build --release` to cut the transfer size. `--disable-compact-imports`
# keeps the output loadable by browsers that do not enable the compact-imports
# proposal by default.
wasm-opt -Oz --all-features --disable-compact-imports \
  -o dist/diff_bg.wasm dist/diff_bg.wasm
