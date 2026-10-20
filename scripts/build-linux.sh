#!/usr/bin/env bash
# ==============================================================================
# Music Tools v2.0 - Linux Release Packaging & Build Script
# Compiles optimized release binary with LTO and generates standalone package
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Music Tools v2.0 (Rust + GPUI) - Linux Release Builder ===${NC}"
echo -e "Project Root: ${ROOT_DIR}"
echo ""

# 1. Check for required build tools
echo -e "${BLUE}[1/5] Checking build environment & tools...${NC}"

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo is not installed. Please install Rust via https://rustup.rs/${NC}"
    exit 1
fi

if ! command -v rustc &> /dev/null; then
    echo -e "${RED}Error: rustc compiler is not installed.${NC}"
    exit 1
fi

RUST_VER="$(rustc --version)"
echo -e "  Found Rust: ${GREEN}${RUST_VER}${NC}"

# 2. Check for Linux system libraries (Fontconfig, Alsa, Wayland/X11)
echo -e "${BLUE}[2/5] Checking Linux system graphics and audio dependencies...${NC}"

check_pkg() {
    local pkg_name="$1"
    local desc="$2"
    if command -v pkg-config &> /dev/null && pkg-config --exists "$pkg_name"; then
        echo -e "  ${GREEN}✓${NC} ${desc} (${pkg_name})"
        return 0
    else
        echo -e "  ${YELLOW}?${NC} ${desc} (${pkg_name}) - pkg-config check skipped or library not detected in default path"
        return 0
    fi
}

check_pkg "fontconfig" "Fontconfig (Text shaping & fonts)"
check_pkg "alsa" "ALSA audio library"
check_pkg "wayland-client" "Wayland client library"
check_pkg "x11" "X11 core library"
check_pkg "xkbcommon" "XKB Common keymap handling"

# 3. Check for runtime dependencies
echo -e "${BLUE}[3/5] Checking runtime tools (yt-dlp, ffmpeg)...${NC}"
if command -v yt-dlp &> /dev/null; then
    echo -e "  ${GREEN}✓${NC} yt-dlp: $(yt-dlp --version)"
else
    echo -e "  ${YELLOW}!${NC} yt-dlp not found in PATH (Music Tools can download it automatically on first run)"
fi

if command -v ffmpeg &> /dev/null; then
    echo -e "  ${GREEN}✓${NC} ffmpeg: available"
else
    echo -e "  ${YELLOW}!${NC} ffmpeg not found in PATH (Recommended for audio transcoding and ID3 tagging)"
fi

# 4. Build Release Binary with Cargo
echo -e "${BLUE}[4/5] Building release binary with LTO (crates/app)...${NC}"
cd "${ROOT_DIR}"

CARGO_PROFILE_RELEASE_LTO=thin cargo build --release -p app

RELEASE_BIN="${ROOT_DIR}/target/release/app"
if [ ! -f "${RELEASE_BIN}" ]; then
    echo -e "${RED}Error: Release binary not found at ${RELEASE_BIN}${NC}"
    exit 1
fi

BIN_SIZE="$(du -h "${RELEASE_BIN}" | cut -f1)"
echo -e "  Binary compiled: ${GREEN}${RELEASE_BIN} (${BIN_SIZE})${NC}"

# 5. Assemble Linux Distribution Package
echo -e "${BLUE}[5/5] Assembling Linux Distribution Package...${NC}"

DIST_DIR="${ROOT_DIR}/target/dist-linux"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/bin"
mkdir -p "${DIST_DIR}/share/applications"
mkdir -p "${DIST_DIR}/share/icons/hicolor/scalable/apps"

# Copy binary
cp "${RELEASE_BIN}" "${DIST_DIR}/bin/music-tools"
chmod +x "${DIST_DIR}/bin/music-tools"

# Strip binary if strip is available to minimize footprint
if command -v strip &> /dev/null; then
    echo -e "  Stripping debug symbols..."
    strip "${DIST_DIR}/bin/music-tools"
    STRIPPED_SIZE="$(du -h "${DIST_DIR}/bin/music-tools" | cut -f1)"
    echo -e "  Stripped binary size: ${GREEN}${STRIPPED_SIZE}${NC}"
fi

# Copy Desktop entry and Icon
cp "${ROOT_DIR}/assets/music-tools.desktop" "${DIST_DIR}/share/applications/"
cp "${ROOT_DIR}/assets/music-tools.svg" "${DIST_DIR}/share/icons/hicolor/scalable/apps/"

# Create runnable portable launcher
cat > "${DIST_DIR}/run.sh" << 'EOF'
#!/usr/bin/env bash
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export PATH="${SCRIPT_DIR}/bin:${PATH}"
exec "${SCRIPT_DIR}/bin/music-tools" "$@"
EOF
chmod +x "${DIST_DIR}/run.sh"

echo ""
echo -e "${GREEN}===================================================================${NC}"
echo -e "${GREEN}✓ Build completed successfully!${NC}"
echo -e "Package ready at: ${BLUE}${DIST_DIR}${NC}"
echo ""
echo -e "To run directly:"
echo -e "  ${YELLOW}${DIST_DIR}/run.sh${NC}"
echo ""
echo -e "To install to your local user directory (~/.local):"
echo -e "  cp ${DIST_DIR}/bin/music-tools ~/.local/bin/"
echo -e "  cp ${DIST_DIR}/share/applications/music-tools.desktop ~/.local/share/applications/"
echo -e "  cp ${DIST_DIR}/share/icons/hicolor/scalable/apps/music-tools.svg ~/.local/share/icons/hicolor/scalable/apps/"
echo -e "${GREEN}===================================================================${NC}"
