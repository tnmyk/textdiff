# Text Diff

Live: **[diff.tanmay.sh](https://diff.tanmay.sh)**

A fast, privacy-first text diff tool. Paste two texts, see exactly what changed — side by side, word by word.

All processing happens locally. Nothing is sent to a server.

## Features

- Line-level diff with grapheme-safe inline highlighting
- Ignore case, whitespace, and blank lines
- Collapse unchanged sections and jump between changes
- Open or drop text files into either pane
- Keyboard shortcuts for compare, swap, and clear
- Side-by-side rows keep both panes aligned
- Runs natively and in the browser (WebAssembly)
- Swap, clear, and example text actions
- Responsive mobile layout with a tabbed editor and unified diff

## Stack

- **Rust** + **GPUI / gpui-kit** — UI
- **similar** — diffing engine
- **Trunk** — web build
- **Cloudflare Workers** — hosting

## Run locally

The project pins a nightly Rust toolchain because GPUI currently uses unstable features.

```sh
cargo run
```

## Develop web version

```sh
trunk serve
```

Then open <http://127.0.0.1:8080>. Auto-reloads on change.

The dev server sets the `Cross-Origin-Opener-Policy` and `Cross-Origin-Embedder-Policy` headers GPUI's web runtime needs.

## Build for web

```sh
trunk build --release --no-sri
```

Optionally shrink the wasm before deploying. `binaryen` provides `wasm-opt`:

```sh
./scripts/optimize-wasm.sh
```

The deploy workflow runs this pass. It also writes `dist/diff_bg.wasm.br`,
which the Worker serves to browsers that accept brotli; other clients get the
original wasm.

## Deploy

```sh
wrangler deploy
```

`_headers` is copied into `dist/` during the Trunk build, so Cloudflare serves the wasm with the required cross-origin isolation headers.
