# GripDL - macOS Download Manager

A production-ready, high-performance Internet Download Manager clone specifically for macOS. GripDL integrates with Firefox to intercept downloads, supports multi-threaded downloading (segmentation), and manages file queues.

## Features

- **Smart Segmentation**: Split files into up to 32 parts using HTTP `Range` headers and download them concurrently
- **Browser Integration**: Firefox extension captures Download URL, Cookies, Referrer, and User-Agent
- **File Assembly**: Efficiently merge file parts upon completion without freezing the UI
- **Persistence**: Save download state to SQLite to allow pausing/resuming downloads even after restarting
- **System Tray**: Minimize to the macOS menu bar

## Tech Stack

- **Core/Backend**: Rust (Tokio for async I/O, Reqwest for HTTP)
- **GUI**: Tauri v2 (React + TypeScript + Vite)
- **UI Library**: Shadcn/ui + Tailwind CSS
- **Browser Extension**: Mozilla WebExtensions API (Manifest V3), TypeScript
- **Communication**: Native Messaging protocol

## Project Structure

```
GripDL/
├── app/                    # Tauri application
│   ├── src/               # React frontend source
│   ├── src-tauri/         # Rust backend source
│   └── package.json
├── extension/             # Firefox extension
│   ├── manifest.json
│   ├── background.ts
│   └── package.json
└── README.md
```

## Development

### Prerequisites

- Rust (latest stable)
- Node.js 18+
- macOS (for building)

### Setup

1. Install Rust dependencies:
```bash
cd app/src-tauri
cargo build
```

2. Install frontend dependencies:
```bash
cd app
npm install
```

3. Install extension dependencies:
```bash
cd extension
npm install
```

### Building

```bash
# Build Tauri app
cd app
npm run tauri build

# Build extension
cd extension
npm run build
```

## License

AGPL-3.0
