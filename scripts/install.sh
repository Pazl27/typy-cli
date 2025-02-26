#!/bin/bash

# Define paths
BIN_NAME="typy"
LOCAL_DIR="$HOME/.local/share"
CONFIG_PATH="$LOCAL_DIR/$BIN_NAME/english.txt"
INSTALL_DIR="$HOME/your-repo"
GIT_TAG="v0.8.0"

# Function to move binary with appropriate privileges
move_binary() {
    if command -v sudo &> /dev/null; then
        sudo mv /tmp/$BIN_NAME /usr/bin/$BIN_NAME
        sudo chmod +x /usr/bin/$BIN_NAME
    elif command -v doas &> /dev/null; then
        doas mv /tmp/$BIN_NAME /usr/bin/$BIN_NAME
        doas chmod +x /usr/bin/$BIN_NAME
    elif [ "$(id -u)" -eq 0 ]; then
        mv /tmp/$BIN_NAME /usr/bin/$BIN_NAME
        chmod +x /usr/bin/$BIN_NAME
    else
        echo "Please run this script with elevated privileges (sudo, doas, or as root)."
        exit 1
    fi
}

# Create directories if they don't exist
mkdir -p "$LOCAL_DIR/$BIN_NAME"

# Download the binary and move it to the correct location
echo "Downloading and installing $BIN_NAME..."
curl -L https://github.com/Pazl27/typy-cli/releases/download/$GIT_TAG/$BIN_NAME -o /tmp/$BIN_NAME

# Move the binary to /usr/bin and make it executable
move_binary

# Move any required files to the ~/.local folder (e.g., configuration files)
echo "Setting up configuration files..."
curl -L https://github.com/Pazl27/typy-cli/releases/download/$GIT_TAG/english.txt -o "$CONFIG_PATH"

echo "Installation complete! You can now run the CLI tool by typing '$BIN_NAME' in your terminal."
