# GripDL Architecture

## Overview

GripDL is a production-ready download manager for macOS that integrates with Firefox to intercept downloads and manage them with advanced features like multi-threaded segmentation.

## System Architecture

```
┌─────────────┐
│   Firefox   │
│  Extension  │
└──────┬──────┘
       │ Native Messaging Protocol
       │ (JSON over stdio)
       ▼
┌─────────────────────────┐
│ Native Messaging Host   │
│ (Separate Binary)       │
└──────┬──────────────────┘
       │ IPC/HTTP/Unix Socket
       ▼
┌─────────────────────────┐
│   GripDL Tauri App       │
│                         │
│  ┌──────────────────┐  │
│  │  React Frontend   │  │
│  │  (UI Components)  │  │
│  └────────┬─────────┘  │
│           │            │
│  ┌────────▼─────────┐  │
│  │  Tauri Commands  │  │
│  └────────┬─────────┘  │
│           │            │
│  ┌────────▼─────────┐  │
│  │ Download Manager  │  │
│  │  (Rust Engine)    │  │
│  └────────┬─────────┘  │
│           │            │
│  ┌────────▼─────────┐  │
│  │   Persistence    │  │
│  │   (SQLite DB)    │  │
│  └──────────────────┘  │
└─────────────────────────┘
       │
       ▼
┌─────────────────────────┐
│   HTTP Downloads        │
│   (Reqwest + Tokio)     │
└─────────────────────────┘
```

## Component Details

### 1. Firefox Extension (`extension/`)

**Purpose**: Intercept downloads from Firefox and forward them to the native app.

**Key Features**:
- Listens to `downloads.onCreated` event
- Extracts cookies, referrer, and user-agent
- Sends messages via Native Messaging protocol

**Files**:
- `src/background.ts`: Service worker that handles download interception
- `manifest.json`: Extension manifest (Manifest V3)

### 2. Native Messaging Host (`app/src-tauri/src/bin/native-messaging-host.rs`)

**Purpose**: Bridge between Firefox extension and GripDL app.

**Protocol**:
- Reads JSON messages from stdin (4-byte length prefix + JSON)
- Writes JSON responses to stdout
- Communicates with main app via IPC (to be implemented)

**Note**: Currently acknowledges messages. In production, should communicate with main app via:
- Unix domain socket
- HTTP localhost server
- Named pipe

### 3. Tauri Application (`app/`)

#### Frontend (`app/src/`)

**Technology**: React + TypeScript + Vite + Tailwind CSS

**Components**:
- `App.tsx`: Main application component
- `DownloadList.tsx`: List of all downloads
- `DownloadItem.tsx`: Individual download item with progress

**Features**:
- Real-time download updates via Tauri events
- Pause/Resume/Cancel controls
- Progress bars and status indicators

#### Backend (`app/src-tauri/src/`)

**Technology**: Rust + Tokio + Reqwest

**Modules**:

##### `downloader.rs` - Core Download Engine

**Features**:
- **Smart Segmentation**: Up to 32 concurrent segments using HTTP Range headers
- **Automatic Detection**: Checks server support for Range requests
- **Progress Tracking**: Real-time progress updates
- **Pause/Resume**: State management for paused downloads
- **File Assembly**: Efficient merging of downloaded segments

**Algorithm**:
1. HEAD request to check file size and Range support
2. Calculate optimal number of segments (max 32, min 1MB per segment)
3. Download segments concurrently
4. Merge segments into final file
5. Clean up temporary files

##### `persistence.rs` - SQLite Database

**Schema**:
- `downloads` table: Download metadata and state
- `download_segments` table: Segment progress tracking

**Features**:
- Save download state on progress updates
- Load downloads on app startup
- Resume paused downloads after restart

##### `native_messaging.rs` - Native Messaging Integration

**Purpose**: Handle messages from Firefox extension (when running in main app context).

**Note**: For production, native messaging should use a separate binary that communicates with the main app.

##### `state.rs` - Application State

**Purpose**: Thread-safe state management using `RwLock`.

## Data Flow

### Download Initiation

1. User clicks download link in Firefox
2. Extension intercepts via `downloads.onCreated`
3. Extension cancels original download
4. Extension extracts cookies, referrer, user-agent
5. Extension sends message to Native Messaging Host
6. Native Messaging Host forwards to GripDL app (via IPC)
7. GripDL app starts download via Download Manager
8. Download Manager checks server capabilities
9. Download Manager creates segments and starts concurrent downloads
10. Progress updates emitted to frontend
11. Segments merged on completion
12. Download marked as completed

### Progress Updates

1. Download Manager updates progress (every 1MB)
2. Persistence layer saves to SQLite
3. Tauri event emitted to frontend
4. React component updates UI

## Performance Considerations

### Segmentation Strategy

- **Minimum Segment Size**: 1MB (prevents overhead for small files)
- **Maximum Segments**: 32 (balance between parallelism and overhead)
- **Dynamic Calculation**: Based on file size and server capabilities

### Concurrency

- Uses Tokio async runtime for non-blocking I/O
- Each segment downloads concurrently
- File writes are buffered and async

### Memory Management

- Segments downloaded to temporary files
- Final file assembled by copying segments sequentially
- Temporary files cleaned up after merge

## Security Considerations

1. **Cookie Handling**: Cookies passed from extension, not stored long-term
2. **File Paths**: Downloads saved to user's Downloads directory
3. **Native Messaging**: Only registered extensions can communicate
4. **Input Validation**: All URLs and file paths validated

## Future Enhancements

1. **Resume Support**: Resume partial downloads using Range headers
2. **Bandwidth Limiting**: Throttle download speed
3. **Scheduling**: Schedule downloads for specific times
4. **Categories**: Organize downloads into categories
5. **Search**: Search downloads by name, URL, or status
6. **Notifications**: macOS notifications for completed downloads
7. **System Tray**: Minimize to menu bar
8. **Multiple Browser Support**: Extend to Chrome/Safari

## Error Handling

- **Network Errors**: Retry logic with exponential backoff
- **File System Errors**: Graceful error messages to user
- **Invalid URLs**: Validation before starting download
- **Server Errors**: Handle HTTP error codes appropriately

## Testing Strategy

1. **Unit Tests**: Test downloader logic in isolation
2. **Integration Tests**: Test extension ↔ native messaging ↔ app flow
3. **E2E Tests**: Test full download flow from Firefox to completion
4. **Performance Tests**: Measure download speeds with various segment counts

