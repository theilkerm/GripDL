#!/bin/bash

# Script to register GripDL as a Native Messaging Host on macOS
# This should be run during app installation

APP_NAME="com.gripdl.app"
APP_PATH="$1"  # Path to the GripDL.app bundle

if [ -z "$APP_PATH" ]; then
    echo "Usage: $0 <path-to-GripDL.app>"
    exit 1
fi

# Get the actual executable path
EXECUTABLE_PATH="$APP_PATH/Contents/MacOS/gripdl"

if [ ! -f "$EXECUTABLE_PATH" ]; then
    echo "Error: Executable not found at $EXECUTABLE_PATH"
    exit 1
fi

# Create the manifest JSON
MANIFEST_DIR="$HOME/Library/Application Support/Mozilla/NativeMessagingHosts"
mkdir -p "$MANIFEST_DIR"

MANIFEST_FILE="$MANIFEST_DIR/$APP_NAME.json"

cat > "$MANIFEST_FILE" << EOF
{
  "name": "$APP_NAME",
  "description": "GripDL Native Messaging Host",
  "path": "$EXECUTABLE_PATH",
  "type": "stdio",
  "allowed_extensions": ["gripdl@example.com"]
}
EOF

echo "Native Messaging Host registered at: $MANIFEST_FILE"
echo "Please restart Firefox for changes to take effect."

