#!/bin/bash

# Define paths
BIN_NAME="typy"
LOCAL_DIR="$HOME/.local/share"
BIN_PATH="/usr/bin/$BIN_NAME"
CONFIG_PATH="$LOCAL_DIR/$BIN_NAME/words.txt"
CONFIG_DIR="$LOCAL_DIR/$BIN_NAME"

# Remove the binary
echo "Removing $BIN_NAME binary..."
sudo rm -f "$BIN_PATH"

# Remove configuration files
echo "Removing configuration files..."
rm -f "$CONFIG_PATH"
rm -rf "$CONFIG_DIR"

echo "Cleanup complete! $BIN_NAME has been removed from your system."
