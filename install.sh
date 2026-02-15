#!/bin/bash

# SSH Key Manager - Installation Script
# Usage: curl -sSL https://raw.githubusercontent.com/crystaldaking/ssh-key-manager/main/install.sh | bash

set -e

REPO="crystaldaking/ssh-key-manager"
INSTALL_DIR="/usr/local/bin"q
BINARY_NAME="skm"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            case "$ARCH" in
                x86_64)
                    PLATFORM="linux-amd64"
                    ;;
                aarch64|arm64)
                    PLATFORM="linux-arm64"
                    ;;
                *)
                    echo -e "${RED}Unsupported architecture: $ARCH${NC}"
                    exit 1
                    ;;
            esac
            ;;
        Darwin*)
            case "$ARCH" in
                x86_64)
                    PLATFORM="macos-amd64"
                    ;;
                arm64)
                    PLATFORM="macos-arm64"
                    ;;
                *)
                    echo -e "${RED}Unsupported architecture: $ARCH${NC}"
                    exit 1
                    ;;
            esac
            ;;
        *)
            echo -e "${RED}Unsupported operating system: $OS${NC}"
            exit 1
            ;;
    esac
}

# Get latest release version
get_latest_version() {
    echo -e "${YELLOW}Fetching latest release...${NC}"
    LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$LATEST_RELEASE" ]; then
        echo -e "${RED}Failed to fetch latest release${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Latest version: $LATEST_RELEASE${NC}"
}

# Download binary
download_binary() {
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_RELEASE/skm-$PLATFORM"
    TEMP_DIR=$(mktemp -d)
    TEMP_FILE="$TEMP_DIR/$BINARY_NAME"
    
    echo -e "${YELLOW}Downloading skm-$PLATFORM...${NC}"
    
    if ! curl -L -o "$TEMP_FILE" "$DOWNLOAD_URL"; then
        echo -e "${RED}Failed to download binary${NC}"
        rm -rf "$TEMP_DIR"
        exit 1
    fi
    
    chmod +x "$TEMP_FILE"
}

# Install binary
install_binary() {
    echo -e "${YELLOW}Installing to $INSTALL_DIR/$BINARY_NAME...${NC}"
    
    if [ -w "$INSTALL_DIR" ]; then
        mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
    else
        echo -e "${YELLOW}Requesting sudo privileges...${NC}"
        sudo mv "$TEMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    rm -rf "$TEMP_DIR"
}

# Verify installation
verify_installation() {
    if command -v "$BINARY_NAME" &> /dev/null; then
        VERSION=$($BINARY_NAME --version)
        echo -e "${GREEN}Successfully installed $VERSION${NC}"
        echo ""
        echo -e "${GREEN}Run 'skm' to start SSH Key Manager${NC}"
    else
        echo -e "${RED}Installation failed${NC}"
        exit 1
    fi
}

# Main installation flow
main() {
    echo -e "${GREEN}=== SSH Key Manager Installer ===${NC}"
    echo ""
    
    detect_platform
    get_latest_version
    download_binary
    install_binary
    verify_installation
}

# Allow custom install directory
if [ -n "$1" ]; then
    INSTALL_DIR="$1"
fi

main
