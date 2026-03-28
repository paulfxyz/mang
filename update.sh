#!/usr/bin/env bash
# =============================================================================
#  update.sh -- Update mang.sh to the latest version
#  https://github.com/paulfxyz/mang
#
#  Usage:
#    curl -fsSL https://mang.sh/update.sh | bash
#
#  Or run directly after download:
#    bash update.sh
#
#  What this script does:
#    1. Checks that yo is installed (exit if not)
#    2. Reads the currently installed version from the binary
#    3. Fetches the latest version number from Cargo.toml on GitHub
#    4. Exits cleanly if already up to date (no unnecessary builds)
#    5. Clones the latest source, builds a release binary
#    6. Replaces the binary at its current install location
#    7. Never touches your config (~/.config/mang/config.json)
#
#  Works with: v1.0.0 and later
# =============================================================================

set -euo pipefail

REPO="https://github.com/paulfxyz/mang"
RAW="https://raw.githubusercontent.com/paulfxyz/mang/main"
TMP_DIR="$(mktemp -d)"
SUDO=""

# -- Colours ------------------------------------------------------------------
RED=$'\033[0;31m'
GRN=$'\033[0;32m'
CYN=$'\033[0;36m'
YLW=$'\033[1;33m'
BLD=$'\033[1m'
DIM=$'\033[2m'
RST=$'\033[0m'

log()  { printf "  ${CYN}[..]${RST}  %s\n" "$1"; }
ok()   { printf "  ${GRN}[ok]${RST}  %s\n" "$1"; }
warn() { printf "  ${YLW}[!!]${RST}  %s\n" "$1"; }
info() { printf "  ${DIM}      %s${RST}\n"  "$1"; }
die()  { printf "  ${RED}[!!]${RST}  %s\n" "$1"; rm -rf "$TMP_DIR"; exit 1; }

# Cleanup on unexpected exit
trap 'rm -rf "$TMP_DIR"' EXIT

# -- Banner -------------------------------------------------------------------
printf "\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "${CYN}  |          Updating  mang.sh            |${RST}\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "\n"

# =============================================================================
#  Step 1 -- Verify yo is installed
# =============================================================================
YO_BIN="$(command -v yo 2>/dev/null || true)"

if [[ -z "$YO_BIN" ]]; then
    # Also check common locations in case PATH is not set up yet
    for candidate in /usr/local/bin/yo /usr/bin/yo "$HOME/.local/bin/yo" "$HOME/bin/yo"; do
        if [[ -f "$candidate" ]]; then
            YO_BIN="$candidate"
            break
        fi
    done
fi

if [[ -z "$YO_BIN" ]]; then
    warn "mang.sh does not appear to be installed."
    printf "\n"
    printf "  Install it first:\n"
    printf "  ${CYN}  curl -fsSL https://mang.sh/install | bash${RST}\n"
    printf "\n"
    exit 1
fi

INSTALL_DIR="$(dirname "$YO_BIN")"
ok "Found yo at: $YO_BIN"

# =============================================================================
#  Step 2 -- Detect installed version
# =============================================================================
# Extract version string embedded in the binary.
# `strings` reads printable strings from a binary -- works on any platform.
# The VERSION const in ui.rs is embedded as a literal "vX.Y.Z" in the binary.
INSTALLED_VERSION=""
if command -v strings &>/dev/null; then
    INSTALLED_VERSION="$(strings "$YO_BIN" 2>/dev/null \
        | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' \
        | head -1 || true)"
fi
# Fallback for stripped binaries or old versions without the const
if [[ -z "$INSTALLED_VERSION" ]]; then
    INSTALLED_VERSION="unknown"
fi
info "Installed: ${BLD}${INSTALLED_VERSION}${RST}"

# =============================================================================
#  Step 3 -- Fetch latest version from GitHub
# =============================================================================
log "Checking latest version on GitHub..."
LATEST_VERSION="$(curl -fsSL --max-time 10 "$RAW/Cargo.toml" 2>/dev/null \
    | grep '^version' \
    | head -1 \
    | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' \
    || true)"

if [[ -z "$LATEST_VERSION" ]]; then
    warn "Could not reach GitHub to check latest version."
    warn "Check your internet connection and try again."
    exit 1
fi
info "Latest:    ${BLD}v${LATEST_VERSION}${RST}"

# =============================================================================
#  Step 4 -- Early exit if already current
# =============================================================================
if [[ "$INSTALLED_VERSION" == "v${LATEST_VERSION}" ]]; then
    printf "\n"
    ok "Already up to date (${INSTALLED_VERSION}). Nothing to do."
    printf "\n"
    exit 0
fi

printf "\n"
log "Updating ${INSTALLED_VERSION} --> v${LATEST_VERSION}..."
printf "\n"

# =============================================================================
#  Step 5 -- Ensure Rust is available
# =============================================================================
if ! command -v cargo &>/dev/null; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env" 2>/dev/null || true
fi
if ! command -v cargo &>/dev/null; then
    die "Rust/Cargo not found. Run 'curl -fsSL https://mang.sh/install | bash' to reinstall (it will install Rust automatically)."
fi
info "Rust: $(rustc --version)"

# =============================================================================
#  Step 6 -- Clone and build
# =============================================================================
log "Cloning latest source..."
git clone --depth 1 "$REPO" "$TMP_DIR/mang" &>/dev/null

log "Building release binary..."
(cd "$TMP_DIR/mang" && cargo build --release --quiet 2>&1)

BINARY="$TMP_DIR/mang/target/release/yo"
[[ -f "$BINARY" ]] || die "Build failed -- binary not found. Please open an issue at $REPO/issues"
ok "Build complete."

# =============================================================================
#  Step 7 -- Replace binary in-place
# =============================================================================
if [[ ! -w "$INSTALL_DIR" ]]; then
    if sudo -n true 2>/dev/null; then
        SUDO="sudo"
    else
        warn "Need elevated permissions to update $YO_BIN (you may be prompted for your password)."
        SUDO="sudo"
    fi
fi

${SUDO} cp "$BINARY" "$YO_BIN"
${SUDO} chmod +x "$YO_BIN"
ok "Binary replaced at: $YO_BIN"

# =============================================================================
#  Done
# =============================================================================
printf "\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "${CYN}  |           Update complete!              |${RST}\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "\n"
printf "  ${BLD}mang.sh${RST}${BLD} is ready.${RST}\n"
printf "  ${DIM}Type ${BLD}yo${RST}${DIM} to start.${RST}\n"
printf "\n"
printf "  ${DIM}Your config was not changed.${RST}\n"
printf "\n"
