#!/bin/bash

# Define paths
BIN_NAME="typy"
LOCAL_DIR="$HOME/.local/share"
BIN_PATH="/usr/bin/$BIN_NAME"
CONFIG_PATH="$LOCAL_DIR/$BIN_NAME/english.txt"
CONFIG_DIR="$LOCAL_DIR/$BIN_NAME"

# Function to remove the binary with appropriate privileges
remove_binary() {
    if command -v sudo &> /dev/null; then
        sudo rm -f "$BIN_PATH"
    elif command -v doas &> /dev/null; then
        doas rm -f "$BIN_PATH"
    elif [ "$(id -u)" -eq 0 ]; then
        rm -f "$BIN_PATH"
    else
        echo "Please run this script with elevated privileges (sudo, doas, or as root)."
        exit 1
    fi
}

# Remove the binary
echo "Removing $BIN_NAME binary..."
remove_binary

# Remove configuration files
echo "Removing configuration files..."
rm -f "$CONFIG_PATH"
rm -rf "$CONFIG_DIR"

echo "Cleanup complete! $BIN_NAME has been removed from your system."
