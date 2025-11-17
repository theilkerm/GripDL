# GripDL Setup Guide

## Prerequisites

- **Rust**: Latest stable version (install from [rustup.rs](https://rustup.rs/))
- **Node.js**: Version 18 or higher
- **macOS**: 10.13 or later
- **Firefox**: Version 109 or later

## Building the Application

### 1. Install Dependencies

```bash
# Install Rust dependencies
cd app/src-tauri
cargo build

# Install frontend dependencies
cd ../..
cd app
npm install

# Install extension dependencies
cd ../extension
npm install
```

### 2. Build the Tauri Application

```bash
cd app
npm run tauri:build
```

This will create a `.app` bundle in `app/src-tauri/target/release/bundle/macos/`.

### 3. Build the Firefox Extension

```bash
cd extension
npm run build
```

This creates `background.js` in the extension directory.

## Installing the Extension

1. Open Firefox and navigate to `about:debugging`
2. Click "This Firefox"
3. Click "Load Temporary Add-on"
4. Select `extension/manifest.json`

## Registering Native Messaging Host

After building the app, register it as a Native Messaging Host:

```bash
chmod +x scripts/register-native-messaging.sh
./scripts/register-native-messaging.sh "/path/to/GripDL.app"
```

Or manually create the manifest file at:
```
~/Library/Application Support/Mozilla/NativeMessagingHosts/com.gripdl.app.json
```

With the following content (adjust the path to your app):

```json
{
  "name": "com.gripdl.app",
  "description": "GripDL Native Messaging Host",
  "path": "/Applications/GripDL.app/Contents/MacOS/gripdl",
  "type": "stdio",
  "allowed_extensions": ["gripdl@example.com"]
}
```

**Important**: The extension ID in `manifest.json` must match the `allowed_extensions` in the native messaging manifest.

## Development Mode

### Running the Tauri App in Development

```bash
cd app
npm run tauri:dev
```

### Testing the Extension

1. Load the extension in Firefox (as described above)
2. Make a download in Firefox
3. The download should be intercepted and sent to GripDL

## Architecture Notes

### Native Messaging

The native messaging host communicates with Firefox using the Native Messaging protocol:
- Messages are sent as JSON with a 4-byte length prefix (little-endian)
- The host reads from stdin and writes to stdout
- In production, you may want to create a separate binary for the native messaging host that communicates with the main app via IPC

### Download Flow

1. User initiates download in Firefox
2. Extension intercepts via `downloads.onCreated`
3. Extension extracts cookies, referrer, and user-agent
4. Extension sends message to native app via Native Messaging
5. Native app receives message and starts download
6. Download manager handles segmentation and progress
7. Frontend displays download status

## Troubleshooting

### Extension Not Connecting

- Verify the native messaging manifest file exists and is valid JSON
- Check that the path to the executable is correct
- Ensure Firefox has permission to run the native app
- Restart Firefox after registering the native messaging host

### Downloads Not Starting

- Check browser console for extension errors
- Verify the native messaging host is running
- Check Rust logs for download manager errors

### Build Issues

- Ensure all Rust dependencies are installed: `cargo build`
- Clear build cache: `cargo clean` and `rm -rf node_modules`
- Check that Tauri CLI is installed: `npm install -g @tauri-apps/cli`

## Production Deployment

For production, consider:

1. **Code Signing**: Sign the macOS app for distribution
2. **Notarization**: Notarize the app with Apple
3. **Extension Distribution**: Publish extension to Firefox Add-ons
4. **Native Messaging Binary**: Create a separate binary for native messaging that communicates with the main app via IPC or local server
5. **Auto-updates**: Implement update mechanism for both app and extension

