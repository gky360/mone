#!/bin/sh

# This install script is intended to download and install the latest available
# release of the mone.
#
# It attempts to identify the current platform and an error will be thrown if
# the platform is not supported.
#
# Environment variables:
# - INSTALL_DIRECTORY (optional): defaults to $HOME/.cargo/bin
# - MONE_RELEASE_TAG (optional): defaults to fetching the latest release
# - MONE_OS (optional): use a specific value for OS (mostly for testing)
# - MONE_ARCH (optional): use a specific value for ARCH (mostly for testing)
#
# You can install using this script:
# $ curl https://raw.githubusercontent.com/gky360/mone/master/install.sh | sh

set -e

RELEASES_URL="https://github.com/gky360/mone/releases"

downloadJSON() {
    url="$2"

    echo "Fetching $url ..."
    if test -x "$(command -v curl)"; then
        response=$(curl -s -L -w 'HTTPSTATUS:%{http_code}' -H 'Accept: application/json' "$url")
        body=$(echo "$response" | sed -e 's/HTTPSTATUS\:.*//g')
        code=$(echo "$response" | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
    elif test -x "$(command -v wget)"; then
        temp=$(mktemp)
        body=$(wget -q --header='Accept: application/json' -O - --server-response "$url" 2> "$temp")
        code=$(awk '/^  HTTP/{print $2}' < "$temp" | tail -1)
        rm "$temp"
    else
        echo "Neither curl nor wget was available to perform http requests."
        exit 1
    fi
    if [ "$code" != 200 ]; then
        echo "Request failed with code $code"
        exit 1
    fi

    eval "$1='$body'"
}

downloadFile() {
    url="$1"
    destination="$2"

    echo "Fetching $url ..."
    if test -x "$(command -v curl)"; then
        code=$(curl -s -w '%{http_code}' -L "$url" -o "$destination")
    elif test -x "$(command -v wget)"; then
        code=$(wget -q -O "$destination" --server-response "$url" 2>&1 | awk '/^  HTTP/{print $2}' | tail -1)
    else
        echo "Neither curl nor wget was available to perform http requests."
        exit 1
    fi

    if [ "$code" != 200 ]; then
        echo "Request failed with code $code"
        exit 1
    fi
}

initArch() {
    ARCH=$(uname -m)
    if [ -n "$MONE_ARCH" ]; then
        echo "Using MONE_ARCH"
        ARCH="$MONE_ARCH"
    fi
    case $ARCH in
        x86_64) ARCH="x86_64";;
        *) echo "Architecture ${ARCH} is not supported by this installation script"; exit 1;;
    esac
    echo "ARCH = $ARCH"
}

initOSAndVendor() {
    OS=$(uname | tr '[:upper:]' '[:lower:]')
    OS_CYGWIN=0
    if [ -n "$MONE_OS" ]; then
        echo "Using MONE_OS"
        OS="$MONE_OS"
    fi
    case "$OS" in
        darwin) OS='darwin'; VENDOR='apple';;
        linux) OS='linux-musl'; VENDOR='unknown';;
        *) echo "OS ${OS} is not supported by this installation script"; exit 1;;
    esac
    echo "OS = $OS"
}

# identify platform based on uname output
initArch
initOSAndVendor

# determine install directory if required
if [ -z "$INSTALL_DIRECTORY" ]; then
    INSTALL_DIRECTORY="$HOME/.cargo/bin"
fi
mkdir -p "$INSTALL_DIRECTORY"
echo "Will install into $INSTALL_DIRECTORY"

# if MONE_RELEASE_TAG was not provided, assume latest
if [ -z "$MONE_RELEASE_TAG" ]; then
    downloadJSON LATEST_RELEASE "$RELEASES_URL/latest"
    MONE_RELEASE_TAG=$(echo "${LATEST_RELEASE}" | tr -s '\n' ' ' | sed 's/.*"tag_name":"//' | sed 's/".*//' )
fi
echo "Release Tag = $MONE_RELEASE_TAG"

# assemble expected release artifact name
TAR="mone-${MONE_RELEASE_TAG}-${ARCH}-${VENDOR}-${OS}.tar.gz"

# fetch the real release data to make sure it exists before we attempt a download
downloadJSON RELEASE_DATA "$RELEASES_URL/tag/$MONE_RELEASE_TAG"
TAR_URL="$RELEASES_URL/download/$MONE_RELEASE_TAG/$TAR"
DOWNLOAD_DIR=$(mktemp -d)
downloadFile "$TAR_URL" "$DOWNLOAD_DIR/$TAR"

echo "Extracting downloaded tar file ..."
tar -xzf "$DOWNLOAD_DIR/$TAR" -C "$DOWNLOAD_DIR"

echo "Setting executable permissions ..."
INSTALL_NAME="mone"
if [ "$OS" = "windows" ]; then
    INSTALL_NAME="$INSTALL_NAME.exe"
fi
chmod +x "$DOWNLOAD_DIR/$INSTALL_NAME"

echo "Moving executable to $INSTALL_DIRECTORY/$INSTALL_NAME ..."
mv -i "$DOWNLOAD_DIR/$INSTALL_NAME" "$INSTALL_DIRECTORY/$INSTALL_NAME"

# clean up temp dir
echo "Cleaning up temp directory ..."
rm -rf "$DOWNLOAD_DIR"

echo "Installed mone to $INSTALL_DIRECTORY/$INSTALL_NAME"
