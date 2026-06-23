#!/bin/sh
set -e

REPO="leoditto/chromashell"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

get_arch() {
    arch=$(uname -m)
    case "$arch" in
        x86_64|amd64) echo "x86_64" ;;
        arm64|aarch64) echo "aarch64" ;;
        *) echo "Unsupported architecture: $arch" >&2; exit 1 ;;
    esac
}

get_os() {
    os=$(uname -s)
    case "$os" in
        Darwin) echo "apple-darwin" ;;
        Linux) echo "unknown-linux-gnu" ;;
        *) echo "Unsupported OS: $os" >&2; exit 1 ;;
    esac
}

ARCH=$(get_arch)
OS=$(get_os)
TARGET="${ARCH}-${OS}"

echo "Detected: ${TARGET}"

# Get latest release tag
LATEST=$(curl -sL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | head -1 | cut -d'"' -f4)

if [ -z "$LATEST" ]; then
    echo "Error: could not find latest release" >&2
    exit 1
fi

echo "Installing chromashell ${LATEST}..."

URL="https://github.com/${REPO}/releases/download/${LATEST}/chromashell-${TARGET}.tar.gz"

TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

curl -sL "$URL" -o "${TMPDIR}/chromashell.tar.gz"
tar xzf "${TMPDIR}/chromashell.tar.gz" -C "$TMPDIR"

if [ ! -w "$INSTALL_DIR" ]; then
    echo "Installing to ${INSTALL_DIR} (requires sudo)..."
    sudo install -m 755 "${TMPDIR}/chromashell" "${INSTALL_DIR}/chromashell"
    sudo install -m 755 "${TMPDIR}/cs" "${INSTALL_DIR}/cs"
else
    install -m 755 "${TMPDIR}/chromashell" "${INSTALL_DIR}/chromashell"
    install -m 755 "${TMPDIR}/cs" "${INSTALL_DIR}/cs"
fi

echo "Installed chromashell and cs to ${INSTALL_DIR}"
echo "Run 'cs' to get started!"
