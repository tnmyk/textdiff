# diff

Live: **[diff.tanmay.sh](https://diff.tanmay.sh)**

A fast, privacy-first text diff tool. Paste two texts, see exactly what changed — side by side, word by word.

All processing happens locally. Nothing is sent to a server.

## Features

- Line-level diff with inline word highlighting
- Synchronized scrolling across both panes
- Runs natively and in the browser (WebAssembly)

## Stack

- **Rust** + **egui / eframe** — UI
- **similar** — diffing engine
- **Cloudflare Workers** — hosting

## Run locally

```sh
cargo run
```

## Develop web version

```sh
trunk serve
```

Then open <http://127.0.0.1:8080>. Auto-reloads on change.

## Build for web

```sh
trunk build --release
```

## Deploy

```sh
wrangler deploy
```
