#!/bin/sh
set -e

REPO="Saksham1387/Pulse"
INSTALL_DIR="/usr/local/bin"
BIN_NAME="pulse"

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)
    case "$ARCH" in
      x86_64) ARTIFACT="pulse-linux-x86_64" ;;
      *)
        echo "Unsupported architecture: $ARCH"
        echo "Please build from source: cargo install --git https://github.com/$REPO"
        exit 1
        ;;
    esac
    ;;
  Darwin)
    case "$ARCH" in
      x86_64) ARTIFACT="pulse-macos-x86_64" ;;
      arm64)  ARTIFACT="pulse-macos-arm64" ;;
      *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Unsupported OS: $OS"
    echo "Please build from source: cargo install --git https://github.com/$REPO"
    exit 1
    ;;
esac

# Get latest release tag
LATEST=$(curl -sSf "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"tag_name"' \
  | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST" ]; then
  echo "Could not determine latest release. Check https://github.com/$REPO/releases"
  exit 1
fi

URL="https://github.com/$REPO/releases/download/$LATEST/$ARTIFACT"

echo "Installing pulse $LATEST ($ARTIFACT)..."
curl -sSfL "$URL" -o "/tmp/$BIN_NAME"
chmod +x "/tmp/$BIN_NAME"

# Move to install dir (may need sudo)
if [ -w "$INSTALL_DIR" ]; then
  mv "/tmp/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
  echo "Requires sudo to write to $INSTALL_DIR"
  sudo mv "/tmp/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
fi

echo "pulse installed to $INSTALL_DIR/$BIN_NAME"
echo "Run 'pulse' to start."
