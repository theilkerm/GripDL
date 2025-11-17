# GripDL Project Structure

```
GripDL/
├── app/                          # Tauri application
│   ├── src/                      # React frontend source
│   │   ├── components/           # React components
│   │   │   ├── DownloadList.tsx
│   │   │   └── DownloadItem.tsx
│   │   ├── App.tsx              # Main app component
│   │   ├── main.tsx             # Entry point
│   │   └── index.css            # Global styles
│   ├── src-tauri/                # Rust backend
│   │   ├── src/
│   │   │   ├── main.rs          # Tauri entry point
│   │   │   ├── downloader.rs    # Core download engine with segmentation
│   │   │   ├── native_messaging.rs  # Native Messaging Host implementation
│   │   │   ├── persistence.rs   # SQLite persistence layer
│   │   │   └── state.rs         # Application state management
│   │   ├── Cargo.toml           # Rust dependencies
│   │   ├── build.rs             # Build script
│   │   └── tauri.conf.json      # Tauri configuration
│   ├── package.json             # Frontend dependencies
│   ├── vite.config.ts           # Vite configuration
│   ├── tailwind.config.js       # Tailwind CSS configuration
│   └── tsconfig.json            # TypeScript configuration
│
├── extension/                    # Firefox extension
│   ├── src/
│   │   └── background.ts        # Extension background script
│   ├── manifest.json            # Extension manifest (Manifest V3)
│   ├── package.json             # Extension build dependencies
│   └── tsconfig.json           # TypeScript configuration
│
├── scripts/                      # Build and installation scripts
│   ├── register-native-messaging.sh    # macOS native messaging registration
│   └── register-native-messaging.ps1   # PowerShell version (reference)
│
├── README.md                     # Project documentation
├── LICENSE.md                    # License file
└── .gitignore                    # Git ignore rules
```

## Key Components

### Rust Backend (`app/src-tauri/src/`)

- **`downloader.rs`**: Core download engine implementing:
  - Multi-threaded segmented downloads (up to 32 segments)
  - HTTP Range header support
  - Progress tracking
  - Pause/Resume/Cancel functionality
  - File assembly from segments

- **`native_messaging.rs`**: Native Messaging Host server that:
  - Listens for JSON messages from Firefox extension
  - Parses download requests (URL, cookies, referrer, user-agent)
  - Forwards requests to download manager

- **`persistence.rs`**: SQLite database layer for:
  - Storing download state
  - Resuming downloads after app restart
  - Tracking download segments

- **`state.rs`**: Application state management with RwLock for thread-safe access

### Frontend (`app/src/`)

- **`App.tsx`**: Main application component that:
  - Manages download list state
  - Listens for download updates via Tauri events
  - Handles user actions (pause/resume/cancel)

- **`DownloadList.tsx`**: Component displaying all downloads

- **`DownloadItem.tsx`**: Individual download item with:
  - Progress bar
  - Status indicators
  - Control buttons

### Firefox Extension (`extension/`)

- **`background.ts`**: Service worker that:
  - Intercepts downloads using `downloads.onCreated`
  - Extracts cookies, referrer, and user-agent
  - Sends messages to native app via Native Messaging

## Build Process

1. **Rust Backend**: Built via Cargo as part of Tauri build
2. **Frontend**: Built with Vite (React + TypeScript)
3. **Extension**: Built with esbuild (TypeScript → JavaScript)

## Native Messaging Setup

The native messaging host must be registered in:
```
~/Library/Application Support/Mozilla/NativeMessagingHosts/com.gripdl.app.json
```

The registration script (`scripts/register-native-messaging.sh`) creates this manifest file pointing to the GripDL executable.

