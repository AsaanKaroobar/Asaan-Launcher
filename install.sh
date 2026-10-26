#!/bin/sh
# Install Asaaan Launcher from the latest GitHub release.
#
# Linux: installs the AppImage under the user's XDG data directory, links the
#        asaaan-launcher command into ~/.local/bin, and creates a desktop entry.
# macOS: copies Asaaan Launcher.app from the release DMG into ~/Applications.
#
# No sudo is used. Debian packages remain a separate manual install. Windows
# users should use the NSIS .exe installer.

set -eu

REPO="AsaanKaroobar/Asaan-Launcher"
RELEASES_URL="https://github.com/${REPO}/releases"
LATEST_JSON="${ASAAAN_LATEST_JSON:-${RELEASES_URL}/latest/download/latest.json}"

VERSION=""
PREFIX=""
BIN_DIR=""
INSTALL_DESKTOP=1
UNINSTALL=0

usage() {
    cat <<'EOF'
Usage: install.sh [options]

Install the latest Asaaan Launcher release for Linux (AppImage) or macOS (DMG).
Linux installs to ~/.local/share/asaaan-launcher and optionally creates a
desktop entry. macOS installs to ~/Applications. No sudo is required.

Options:
  --version VERSION   Install a specific release (default: latest)
  --prefix DIR        Linux: directory for the AppImage
                      (default: $XDG_DATA_HOME/asaaan-launcher)
                      macOS: directory containing the .app
                      (default: ~/Applications)
  --bin-dir DIR       Linux: directory for the asaaan-launcher symlink
                      (default: $XDG_BIN_HOME or ~/.local/bin)
  --no-desktop        Linux: do not create a desktop entry
  --uninstall         Remove the installed app and desktop entry
  -h, --help          Show this help

Examples:
  sh install.sh
  sh install.sh --version 0.3.0
  sh install.sh --uninstall
EOF
}

die() {
    printf 'install.sh: %s\n' "$*" >&2
    exit 1
}

have() {
    command -v "$1" >/dev/null 2>&1
}

while [ "$#" -gt 0 ]; do
    case "$1" in
        --version)
            [ "$#" -ge 2 ] || die "--version needs a value"
            VERSION=$2
            shift 2
            ;;
        --version=*)
            VERSION=${1#*=}
            shift
            ;;
        --prefix)
            [ "$#" -ge 2 ] || die "--prefix needs a value"
            PREFIX=$2
            shift 2
            ;;
        --prefix=*)
            PREFIX=${1#*=}
            shift
            ;;
        --bin-dir)
            [ "$#" -ge 2 ] || die "--bin-dir needs a value"
            BIN_DIR=$2
            shift 2
            ;;
        --bin-dir=*)
            BIN_DIR=${1#*=}
            shift
            ;;
        --no-desktop)
            INSTALL_DESKTOP=0
            shift
            ;;
        --uninstall)
            UNINSTALL=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown option: $1 (try --help)"
            ;;
    esac
done

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64|amd64) ;;
            *) die "no Linux build for architecture: $ARCH" ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            arm64|aarch64) ;;
            *) die "no macOS build for architecture: $ARCH (Apple Silicon only)" ;;
        esac
        ;;
    *)
        die "unsupported operating system: $OS (use the Windows NSIS installer on Windows)"
        ;;
esac

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
if [ "$OS" = "Darwin" ]; then
    PREFIX=${PREFIX:-"$HOME/Applications"}
    APP="$PREFIX/Asaaan Launcher.app"
else
    PREFIX=${PREFIX:-"$DATA_HOME/asaaan-launcher"}
    BIN_DIR=${BIN_DIR:-"${XDG_BIN_HOME:-$HOME/.local/bin}"}
    APPIMAGE="$PREFIX/Asaaan-Launcher.AppImage"
    DESKTOP_FILE="$DATA_HOME/applications/asaaan-launcher.desktop"
    ICON_FILE="$DATA_HOME/icons/hicolor/128x128/apps/asaaan-launcher.png"
fi

if [ "$UNINSTALL" -eq 1 ]; then
    if [ "$OS" = "Darwin" ]; then
        rm -rf "$APP"
        printf 'Removed %s\n' "$APP"
    else
        rm -f "$APPIMAGE"
        if [ -L "$BIN_DIR/asaaan-launcher" ]; then
            rm -f "$BIN_DIR/asaaan-launcher"
        fi
        rm -f "$DESKTOP_FILE" "$ICON_FILE"
        printf 'Removed Asaaan Launcher from %s\n' "$PREFIX"
    fi
    printf 'User settings were left in place.\n'
    exit 0
fi

have curl || die "curl is required"
if [ "$OS" = "Darwin" ]; then
    have hdiutil || die "hdiutil is required on macOS"
    have ditto || die "ditto is required on macOS"
fi

TMP_DIR=$(mktemp -d "${TMPDIR:-/tmp}/asaaan-install.XXXXXX")
MOUNT_DIR=""

cleanup() {
    if [ -n "${MOUNT_DIR:-}" ] && [ -d "$MOUNT_DIR" ]; then
        hdiutil detach "$MOUNT_DIR" >/dev/null 2>&1 || true
    fi
    if [ -n "${TMP_DIR:-}" ]; then
        rm -rf "$TMP_DIR"
    fi
}
trap cleanup 0
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

fetch() {
    curl -fL --silent --show-error --proto '=https' --retry 3 --retry-delay 1 -o "$2" "$1" ||
        die "download failed: $1"
}

fetch_optional() {
    curl -fsSL --proto '=https' --retry 2 -o "$2" "$1" 2>/dev/null
}

sum_file() {
    if have sha256sum; then
        sha256sum "$1" | awk '{print $1}'
    elif have shasum; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        die "sha256sum or shasum is required"
    fi
}

if [ -z "$VERSION" ]; then
    fetch "$LATEST_JSON" "$TMP_DIR/latest.json"
    VERSION=$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$TMP_DIR/latest.json" | head -n 1)
fi

[ -n "$VERSION" ] || die "could not determine the latest release version"
case "$VERSION" in
    *[!A-Za-z0-9._-]*) die "invalid release version: $VERSION" ;;
esac

if [ "$OS" = "Darwin" ]; then
    ASSET="Asaaan-Launcher_${VERSION}_macos_aarch64.dmg"
else
    ASSET="Asaaan-Launcher_${VERSION}_linux_amd64.AppImage"
fi
BASE_URL="${RELEASES_URL}/download/${VERSION}"

fetch "${BASE_URL}/SHA256SUMS" "$TMP_DIR/SHA256SUMS"
fetch "${BASE_URL}/${ASSET}" "$TMP_DIR/${ASSET}"

EXPECTED=$(awk -v asset="$ASSET" '$2 == asset {print $1; exit}' "$TMP_DIR/SHA256SUMS")
[ -n "$EXPECTED" ] || die "SHA256SUMS does not list ${ASSET}"
ACTUAL=$(sum_file "$TMP_DIR/${ASSET}")
[ "$ACTUAL" = "$EXPECTED" ] || {
    printf 'expected %s\nactual   %s\n' "$EXPECTED" "$ACTUAL" >&2
    die "checksum verification failed for ${ASSET}"
}

if [ "$OS" = "Darwin" ]; then
    mkdir -p "$PREFIX"
    MOUNT_DIR=$(mktemp -d "${TMPDIR:-/tmp}/asaaan-dmg.XXXXXX")
    hdiutil attach -nobrowse -readonly -mountpoint "$MOUNT_DIR" "$TMP_DIR/${ASSET}" >/dev/null ||
        die "could not mount ${ASSET}"

    SRC_APP=""
    for candidate in "$MOUNT_DIR"/*.app; do
        if [ -d "$candidate" ]; then
            SRC_APP=$candidate
            break
        fi
    done
    [ -n "$SRC_APP" ] || die "no .app bundle found in ${ASSET}"

    NEW_APP="$PREFIX/.Asaaan Launcher.app.new.$$"
    rm -rf "$NEW_APP"
    ditto "$SRC_APP" "$NEW_APP" || die "could not copy the app bundle"
    rm -rf "$APP"
    mv "$NEW_APP" "$APP"
    hdiutil detach "$MOUNT_DIR" >/dev/null 2>&1 || true
    MOUNT_DIR=""

    printf '\nInstalled Asaaan Launcher %s\n' "$VERSION"
    printf '  App: %s\n' "$APP"
    printf '\nOpen it with:\n  open "%s"\n' "$APP"
    printf '\nThe app is unsigned and not notarized. If macOS blocks the first launch,\n'
    printf 'approve it in System Settings > Privacy & Security, or right-click > Open.\n'
    printf 'macOS automatic updates may also be blocked until the app is signed.\n'
    exit 0
fi

mkdir -p "$PREFIX" "$BIN_DIR"
cp "$TMP_DIR/${ASSET}" "$APPIMAGE.new"
chmod 755 "$APPIMAGE.new"
mv -f "$APPIMAGE.new" "$APPIMAGE"
ln -sfn "$APPIMAGE" "$BIN_DIR/asaaan-launcher"

if [ "$INSTALL_DESKTOP" -eq 1 ]; then
    mkdir -p "$DATA_HOME/applications" "$DATA_HOME/icons/hicolor/128x128/apps"
    ICON_NAME="application-x-executable"
    ICON_URL="https://raw.githubusercontent.com/${REPO}/${VERSION}/src-tauri/icons/128x128.png"
    if fetch_optional "$ICON_URL" "$TMP_DIR/icon.png" && [ -s "$TMP_DIR/icon.png" ]; then
        cp "$TMP_DIR/icon.png" "$ICON_FILE"
        chmod 644 "$ICON_FILE"
        ICON_NAME="asaaan-launcher"
    else
        printf 'Warning: could not download the icon; using a generic desktop icon.\n' >&2
    fi

    cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=Asaaan Launcher
Comment=Keyboard-first application launcher
Exec="$BIN_DIR/asaaan-launcher"
Icon=$ICON_NAME
Terminal=false
Categories=Utility;
StartupWMClass=asaaan-launcher
EOF
    chmod 644 "$DESKTOP_FILE"
    if have update-desktop-database; then
        update-desktop-database "$DATA_HOME/applications" >/dev/null 2>&1 || true
    fi
fi

printf '\nInstalled Asaaan Launcher %s\n' "$VERSION"
printf '  AppImage: %s\n' "$APPIMAGE"
printf '  Command:  %s\n' "$BIN_DIR/asaaan-launcher"
if [ "$INSTALL_DESKTOP" -eq 1 ]; then
    printf '  Desktop:  %s\n' "$DESKTOP_FILE"
fi
printf '\nRun it with:\n  %s\n' "$BIN_DIR/asaaan-launcher"
printf '\nOn X11, Ctrl+Alt+Space opens it. On Wayland, bind Ctrl+Alt+Space to:\n'
printf '  %s --toggle\n' "$BIN_DIR/asaaan-launcher"
printf '\nThis is the AppImage update channel; signed releases can update it in place.\n'
printf 'If the launcher is running, quit it from the tray and reopen the new version.\n'

case ":$PATH:" in
    *":$BIN_DIR:"*) ;;
    *) printf '\nNote: %s is not in PATH; the desktop entry uses the full path.\n' "$BIN_DIR" ;;
esac
