#!/usr/bin/env bash
# =============================================================================
#  install — mang.sh installer
#  https://mang.sh
#
#  Usage:
#    curl -fsSL https://mang.sh/install | bash
#
#  Or run directly:
#    bash install
#
#  What this does:
#    1. Detects if mang.sh (yo) is already installed and shows current version
#    2. Checks for Rust/Cargo — installs via rustup automatically if missing
#    3. Clones the repo (shallow) and builds a release binary
#    4. Installs the binary as `yo` — at the same location if reinstalling
#    5. Adds `hi` and `hello` as shell aliases (skips if already present)
#
#  Safe to re-run: replaces the binary in-place. Config is never modified.
#
#  Other scripts:
#    curl -fsSL https://mang.sh/update    | bash   — update to latest
#    curl -fsSL https://mang.sh/uninstall | bash   — full removal
# =============================================================================

set -euo pipefail

REPO="https://github.com/paulfxyz/mang"
RAW="https://raw.githubusercontent.com/paulfxyz/mang/main"
BIN_NAME="yo"
TMP_DIR="$(mktemp -d)"
INSTALL_DIR=""
SUDO=""

# -- Colours (ANSI-C quoting — stores actual ESC byte, not literal \033) ------
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

# Cleanup temp dir on any exit (success or failure)
trap 'rm -rf "$TMP_DIR"' EXIT

# -- Banner -------------------------------------------------------------------
printf "\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "${CYN}  |         Installing  mang.sh  句芒        |${RST}\n"
printf "${CYN}  |   The spirit messenger for your shell   |${RST}\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "\n"

# =============================================================================
#  Step 1 -- Detect existing install
# =============================================================================
EXISTING_BIN="$(command -v yo 2>/dev/null || true)"

if [[ -n "$EXISTING_BIN" ]]; then
    EXISTING_VERSION="$(strings "$EXISTING_BIN" 2>/dev/null \
        | grep -oE 'v[0-9]+\.[0-9]+\.[0-9]+' \
        | head -1 || echo "unknown")"
    warn "mang.sh is already installed at $EXISTING_BIN (${EXISTING_VERSION})"
    info "Reinstalling will replace the binary. Your config is not affected."
    printf "\n"
fi

# Show target version
LATEST_VERSION="$(curl -fsSL --max-time 10 "$RAW/Cargo.toml" 2>/dev/null \
    | grep '^version' | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' \
    || echo "")"
if [[ -n "$LATEST_VERSION" ]]; then
    info "Target version: ${BLD}v${LATEST_VERSION}${RST}"
    printf "\n"
fi

# =============================================================================
#  Step 2 -- Check / install Rust
# =============================================================================
# shellcheck disable=SC1090
source "$HOME/.cargo/env" 2>/dev/null || true

if ! command -v cargo &>/dev/null; then
    warn "Rust not found — installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --quiet
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env" 2>/dev/null || true
    command -v cargo &>/dev/null || die "Rust installation failed. Install manually: https://rustup.rs"
    ok "Rust installed."
else
    ok "Rust: $(rustc --version)"
fi

# =============================================================================
#  Step 3 -- Clone and build
# =============================================================================
log "Cloning mang.sh source..."
git clone --depth 1 "$REPO" "$TMP_DIR/mang" &>/dev/null

log "Building release binary (~30 s on first build, faster after)..."
(cd "$TMP_DIR/mang" && cargo build --release --quiet 2>&1)

BINARY="$TMP_DIR/mang/target/release/yo"
[[ -f "$BINARY" ]] || die "Build failed — binary not found. Please open an issue: $REPO/issues"
ok "Build complete."

# =============================================================================
#  Step 4 -- Install binary
# =============================================================================
if [[ -n "$EXISTING_BIN" ]]; then
    INSTALL_DIR="$(dirname "$EXISTING_BIN")"
elif [[ -w /usr/local/bin ]]; then
    INSTALL_DIR="/usr/local/bin"
elif sudo -n true 2>/dev/null; then
    INSTALL_DIR="/usr/local/bin"
    SUDO="sudo"
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

if [[ ! -w "$INSTALL_DIR" ]]; then
    if sudo -n true 2>/dev/null; then
        SUDO="sudo"
    else
        warn "Need elevated permissions to write to $INSTALL_DIR."
        SUDO="sudo"
    fi
fi

${SUDO} cp "$BINARY" "$INSTALL_DIR/$BIN_NAME"
${SUDO} chmod +x "$INSTALL_DIR/$BIN_NAME"
ok "Installed: $INSTALL_DIR/$BIN_NAME"

# =============================================================================
#  Step 5 -- Shell aliases
# =============================================================================
SHELL_RC=""
case "$SHELL" in
    */zsh)  SHELL_RC="$HOME/.zshrc"  ;;
    */bash) SHELL_RC="$HOME/.bashrc" ;;
esac

if [[ -n "$SHELL_RC" ]]; then
    if ! grep -q "mang aliases" "$SHELL_RC" 2>/dev/null; then
        printf "\n# mang aliases -- added by installer\nalias hi='yo'\nalias hello='yo'\n" >> "$SHELL_RC"
        ok "Aliases added to $SHELL_RC  (hi / hello -> yo)"
    else
        ok "Aliases already present in $SHELL_RC"
    fi
fi

# =============================================================================
#  Done
# =============================================================================
printf "\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "${CYN}  |       mang.sh installed!  句芒 is ready  |${RST}\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "\n"
if [[ -n "$SHELL_RC" ]]; then
    printf "  Reload your shell:  ${BLD}source %s${RST}\n" "$SHELL_RC"
    printf "\n"
fi
printf "  Then type  ${CYN}${BLD}yo${RST}  to summon the spirit messenger.\n"
printf "\n"
printf "  ${DIM}Update:    curl -fsSL https://mang.sh/update    | bash${RST}\n"
printf "  ${DIM}Uninstall: curl -fsSL https://mang.sh/uninstall | bash${RST}\n"
printf "\n"
