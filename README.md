# 🎌 anime-manga-cli

A blazing fast ⚡ terminal media player for anime streaming and manga reading, built entirely in Rust.

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)

## ✨ Features

- 📺 **Anime streaming** — search, browse episodes, pick quality (1080p/720p/480p/360p), stream via mpv
- 📖 **Manga reading** — search, browse chapters, download pages, read via feh
- 🌆 **Animated TUI** — cyberpunk city skyline, rain effects, glitch animations, neon theme
- 🔍 **Category browsing** — trending, action, romance, fantasy and more
- ⚡ **Local API server** — scrapes AllAnime GraphQL API, decodes stream URLs
- 🎯 **Quality picker** — choose your preferred resolution before playing

## 📸 Preview

> Add your screen recording or screenshots here

## 🛠️ Requirements

- [Rust](https://rustup.rs/) (1.75+)
- [mpv](https://mpv.io/) — for anime playback
- [feh](https://feh.finalrewind.org/) — for manga reading
- [chafa](https://hpjansson.org/chafa/) — for terminal image preview

Install dependencies on Ubuntu/WSL:

```bash
sudo apt install feh chafa mpv -y
```

## 🚀 Installation

```bash
# Clone the repo
git clone https://github.com/YOUR_USERNAME/anime-manga-cli.git
cd anime-manga-cli

# Build
cargo build --release -p manga
cargo build --release -p server

# Run (needs two terminals)
cargo run -p server   # Terminal 1 — start API server
cargo run -p manga    # Terminal 2 — start TUI
```

## 🎮 Usage

**Main Menu**

- `← →` — switch between Anime and Manga mode
- `Enter` — confirm selection
- `q` — quit

**Search Screen**

- Start typing to search
- `Tab` — toggle between search and category browse
- `↑ ↓ ← →` — navigate categories
- `Enter` — search or select category

**Results**

- `↑ ↓` — navigate results
- `Enter` — select
- `Esc` — go back

**Anime Episodes**

- `Enter` — fetch stream qualities
- Pick resolution → mpv opens fullscreen

**Manga Chapters**

- `Enter` — download and open chapter in feh
- `← →` — navigate pages in feh

## 🏗️ Architecture

```
┌─────────────────────────────┐
│      manga crate (TUI)      │
│  ratatui · crossterm        │
│  MangaDex API (manga)       │
│  HTTP client → server       │
└──────────────┬──────────────┘
               │ HTTP localhost:8080
┌──────────────▼──────────────┐
│     server crate (API)      │
│  axum · reqwest             │
│  AllAnime GraphQL scraper   │
│  m3u8 stream URL decoder    │
└─────────────────────────────┘
```

## ⚠️ Disclaimer

This tool is for educational purposes only. It does not host, store, or distribute any media content. All content is fetched from third-party public APIs. Users are responsible for complying with the terms of service of any content provider and the copyright laws of their country.

This project is not affiliated with MangaDex, AllAnime, or any content provider.

## 📝 License

MIT License — see [LICENSE](LICENSE) for details.
