# Cirnotorrent

A production-grade, modern BitTorrent client with a dual-mode architecture: a cross-platform desktop application built with **Tauri v2**, and a headless server daemon designed for **Docker** self-hosting.

## Features

- **Dual Mode**: Desktop app (Tauri) + Headless server (Docker/Axum)
- **Glassmorphic UI**: Dark/light mode, customizable accent colors, smooth animations
- **BitTorrent Engine**: Powered by librqbit (DHT, PEX, LSD, uTP, TCP, magnet links)
- **Auto-Extraction**: Automatically extract .zip, .rar, .7z, .tar.gz archives on completion
- **RSS Auto-Downloader**: Poll feeds, match with regex/wildcard, auto-add torrents
- **Friend Sharing**: Generate shareable links with password protection and expiry
- **Remote Access**: One-click SSH reverse tunnel via localhost.run
- **UPnP Port Mapping**: Automatic port forwarding
- **Per-Torrent Controls**: Speed limits, ratio limits, seeding time limits, categories, tags
- **Sequential Download**: Toggle per-torrent for streaming while downloading
- **Search**: Integrated Jackett/Prowlarr search
- **Real-Time Stats**: WebSocket-powered live speed graphs and peer counts
- **Keyboard Shortcuts**: Space (pause/resume), Delete (remove), Ctrl+L (add magnet)

## Quick Start

### Desktop App

Download the latest release from [GitHub Releases](../../releases):
- **Linux**: `.AppImage`
- **Windows**: `.exe` (NSIS installer)

### Docker One-Liner

```bash
docker run -d \
  --name cirnotorrent \
  -p 8080:8080 \
  -p 6881:6881 \
  -p 6881:6881/udp \
  -v ./downloads:/data/downloads \
  -v ./config:/data/config \
  -e USERNAME=admin \
  -e PASSWORD=yourpassword \
  cirnotorrent
```

### Docker Compose

```yaml
services:
  cirnotorrent:
    build: .
    container_name: cirnotorrent
    restart: unless-stopped
    ports:
      - "8080:8080"
      - "6881:6881"
      - "6881:6881/udp"
    volumes:
      - ./data/downloads:/data/downloads
      - ./data/config:/data/config
      - ./data/watch:/data/watch
    environment:
      - USERNAME=admin
      - PASSWORD=changeme
      - MAX_DOWN_SPEED=0
      - MAX_UP_SPEED=0
```

```bash
docker compose up -d
```

Then open `http://localhost:8080` in your browser.

## Remote Access Setup

Cirnotorrent supports one-click remote access via SSH reverse tunnel:

1. Open the **Sharing Hub** page
2. Click **Start Tunnel**
3. Copy the generated `https://*.localhost.run` URL
4. Access your Cirnotorrent instance from anywhere

The tunnel uses `ssh -R 80:localhost:PORT nokey@localhost.run` — no additional binaries required.

## Building from Source

### Prerequisites

- Rust 1.75+
- Node.js 18+
- **Linux**: `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
- **Windows**: WebView2 (pre-installed on Windows 10/11)

### Desktop App

```bash
# Install Tauri CLI
cargo install tauri-cli --version "^2"

# Install frontend dependencies
cd frontend && npm install && cd ..

# Build
cargo tauri build
```

Output:
- Linux: `src-tauri/target/release/bundle/appimage/*.AppImage`
- Windows: `src-tauri/target/release/bundle/nsis/*.exe`

### Headless Server

```bash
# Build frontend
cd frontend && npm install && npm run build && cd ..

# Build server binary
cargo build --release -p cirnotorrent-server

# Run
./target/release/cirnotorrent-server \
  --port 8080 \
  --download-path ./downloads \
  --username admin \
  --password mypassword
```

### Docker

```bash
docker build -t cirnotorrent .
docker run -d -p 8080:8080 -p 6881:6881 -p 6881:6881/udp cirnotorrent
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Space` | Pause/Resume selected torrent |
| `Delete` | Remove selected torrent |
| `Ctrl+L` | Add magnet link |
| `Ctrl+O` | Open .torrent file picker (desktop) |

## Architecture

```
┌─────────────────────────────────────────────┐
│           Frontend (React + TS)              │
│    Vite · TailwindCSS · Framer Motion        │
├──────────────────┬──────────────────────────┤
│  Tauri Desktop   │   Headless Server         │
│  (src-tauri)     │   (crates/server)         │
│  IPC Commands    │   Axum REST + WebSocket   │
├──────────────────┴──────────────────────────┤
│              Core Library                     │
│              (crates/core)                    │
│  ┌──────────┬──────────┬──────────┐          │
│  │ Torrent  │ Database │ RSS      │          │
│  │ Manager  │ (SQLite) │ Manager  │          │
│  │(librqbit)│          │          │          │
│  ├──────────┼──────────┼──────────┤          │
│  │Extraction│ Sharing  │ API      │          │
│  │ Manager  │ Hub      │ (Axum)   │          │
│  └──────────┴──────────┴──────────┘          │
└─────────────────────────────────────────────┘
```

## Project Structure

```
cirnotorrent/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── core/               # Core library
│   │   └── src/
│   │       ├── lib.rs      # Engine struct
│   │       ├── db.rs       # SQLite database
│   │       ├── torrent.rs  # librqbit wrapper
│   │       ├── extraction.rs # Auto-extractor
│   │       ├── rss.rs      # RSS manager
│   │       ├── sharing.rs  # UPnP + tunnels
│   │       └── api.rs      # REST + WebSocket
│   └── server/             # Headless binary
│       └── src/main.rs
├── src-tauri/              # Desktop app
│   └── src/main.rs
├── frontend/               # React UI
│   └── src/
│       ├── api/client.ts   # Unified API client
│       ├── store/          # Zustand store
│       ├── components/     # Layout, Sidebar
│       └── pages/          # All UI pages
├── Dockerfile
├── docker-compose.yml
└── .github/workflows/
```

## License

MIT
