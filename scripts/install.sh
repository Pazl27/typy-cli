#!/bin/bash

# Define paths
BIN_NAME="typy"
LOCAL_DIR="$HOME/.local/share"
BIN_PATH="$/usr/bin"
CONFIG_PATH="$LOCAL_DIR/$BIN_NAME/words.txt"
INSTALL_DIR="$HOME/your-repo"
GIT_TAG="v0.5.0-alpha"

# Create directories if they don't exist
mkdir -p "$LOCAL_DIR/$BIN_NAME"

# Download the binary and move it to the correct location
echo "Downloading and installing $BIN_NAME..."
curl -L https://github.com/Pazl27/typy-cli/releases/download/$GIT_TAG/$BIN_NAME -o /tmp/$BIN_NAME

# Move the binary to /usr/bin and make it executable
sudo mv /tmp/$BIN_NAME /usr/bin/$BIN_NAME
sudo chmod +x /usr/bin/$BIN_NAME

# # Move any required files to the ~/.local folder (e.g., configuration files)
echo "Setting up configuration files..."
curl -L https://github.com/Pazl27/typy-cli/releases/download/$GIT_TAG/words.txt -o "$CONFIG_PATH"

echo "Installation complete! You can now run the CLI tool by typing '$BIN_NAME' in your terminal."
