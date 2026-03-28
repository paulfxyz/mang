#!/usr/bin/env bash
# =============================================================================
#  uninstall.sh -- Remove mang.sh from your system
#  https://github.com/paulfxyz/mang
#
#  Usage (pipe-safe -- reads from /dev/tty, not stdin):
#    curl -fsSL https://mang.sh/uninstall.sh | bash
#
#  Or run directly after download:
#    bash uninstall.sh
#
#  What this script removes:
#    - The `yo` binary (searched in PATH and common install locations)
#    - The config directory -- ASKS BEFORE DELETING (keeps your API key by default)
#    - The mang.sh alias block from ~/.zshrc, ~/.bashrc, ~/.bash_profile
#
#  What it leaves alone:
#    - Rust / rustup (you may use it for other projects)
#    - Any other binaries or shell configuration
#
#  Works with: v1.0.0 and later
# =============================================================================

set -euo pipefail

SUDO=""

# -- Colours (pure ANSI escape codes, no Unicode in variable names) -----------
RED=$'\033[0;31m'
GRN=$'\033[0;32m'
CYN=$'\033[0;36m'
YLW=$'\033[1;33m'
BLD=$'\033[1m'
DIM=$'\033[2m'
RST=$'\033[0m'

ok()    { printf "  ${GRN}[ok]${RST}  %s\n"   "$1"; }
warn()  { printf "  ${YLW}[!!]${RST}  %s\n"   "$1"; }
info()  { printf "  ${DIM}      %s${RST}\n"    "$1"; }
skip()  { printf "  ${DIM}[--]  %s${RST}\n"   "$1"; }
die()   { printf "  ${RED}[!!]${RST}  %s\n"   "$1"; exit 1; }

# ask <question>
# Reads from /dev/tty so this works whether the script is piped or run directly.
# Prints [Y/N] (uppercase Y = default yes, uppercase N = default no).
# Returns 0 for yes, 1 for no.
ask_yes() {
    local reply
    # /dev/tty is the actual terminal even when stdin is a pipe
    printf "  ${YLW}[??]${RST}  %s ${DIM}[Y/N]${RST} " "$1"
    read -r reply </dev/tty
    [[ "$reply" =~ ^[Yy]$ ]]
}

ask_no() {
    # Like ask_yes but default is No (user must explicitly type Y)
    local reply
    printf "  ${YLW}[??]${RST}  %s ${DIM}[y/N]${RST} " "$1"
    read -r reply </dev/tty
    [[ "$reply" =~ ^[Yy]$ ]]
}

# -- Banner -------------------------------------------------------------------
printf "\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "${CYN}  |       Uninstalling  mang.sh           |${RST}\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "\n"
printf "  ${DIM}This will remove mang.sh from your system.${RST}\n"
printf "  ${DIM}Your OpenRouter API key will be kept unless you say otherwise.${RST}\n"
printf "\n"

# -- Confirm intent -----------------------------------------------------------
if ! ask_yes "Are you sure you want to uninstall mang.sh?"; then
    printf "\n"
    printf "  ${DIM}Cancelled. Nothing was changed.${RST}\n"
    printf "\n"
    exit 0
fi
printf "\n"

# =============================================================================
#  Step 1 -- Find and remove the binary
# =============================================================================
YO_BIN=""

# First: check PATH
if command -v yo &>/dev/null; then
    YO_BIN="$(command -v yo)"
else
    # PATH miss -- check common install locations manually
    for candidate in \
        /usr/local/bin/yo \
        /usr/bin/yo \
        "$HOME/.local/bin/yo" \
        "$HOME/bin/yo"; do
        if [[ -f "$candidate" ]]; then
            YO_BIN="$candidate"
            break
        fi
    done
fi

if [[ -n "$YO_BIN" ]]; then
    INSTALL_DIR="$(dirname "$YO_BIN")"

    # Determine whether we need sudo
    if [[ ! -w "$INSTALL_DIR" ]]; then
        if sudo -n true 2>/dev/null; then
            SUDO="sudo"
        else
            warn "Need elevated permissions to remove $YO_BIN"
            warn "You may be prompted for your password."
            SUDO="sudo"
        fi
    fi

    ${SUDO} rm -f "$YO_BIN"
    ok "Removed binary: $YO_BIN"
else
    skip "Binary not found in PATH or common locations -- may already be removed."
fi

# =============================================================================
#  Step 2 -- Remove config directory (ask first)
# =============================================================================

# Resolve the correct config directory for this OS
CONFIG_DIR=""
case "$(uname -s)" in
    Darwin)
        # macOS: dirs crate uses Application Support
        MACOS_CFG="$HOME/Library/Application Support/mang"
        XDG_CFG="${XDG_CONFIG_HOME:-$HOME/.config}/mang"
        if   [[ -d "$MACOS_CFG" ]]; then CONFIG_DIR="$MACOS_CFG"
        elif [[ -d "$XDG_CFG"   ]]; then CONFIG_DIR="$XDG_CFG"
        fi
        ;;
    Linux|*)
        CONFIG_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/mang"
        ;;
esac

printf "\n"
if [[ -n "$CONFIG_DIR" && -d "$CONFIG_DIR" ]]; then
    warn "Config found: $CONFIG_DIR"
    info "Contains your OpenRouter API key and model preference."
    info "If you keep it, reinstalling will pick up your settings automatically."
    printf "\n"
    if ask_no "Delete config? (you will need to re-enter your API key if you reinstall)"; then
        rm -rf "$CONFIG_DIR"
        ok "Removed config: $CONFIG_DIR"
    else
        skip "Config kept at: $CONFIG_DIR"
    fi
else
    skip "Config directory not found -- nothing to remove."
fi


# =============================================================================
#  Step 2b -- Remove legacy yo-rust config (from versions before v3.0.0)
# =============================================================================
YO_RUST_CONFIGS=()
case "$(uname -s)" in
    Darwin)
        YO_RUST_CONFIGS+=(
            "$HOME/Library/Application Support/yo-rust"
            "${XDG_CONFIG_HOME:-$HOME/.config}/yo-rust"
        )
        ;;
    Linux|*)
        YO_RUST_CONFIGS+=(
            "${XDG_CONFIG_HOME:-$HOME/.config}/yo-rust"
        )
        ;;
esac

# Also check for mang-sh (the name used before v3.0.4)
case "$(uname -s)" in
    Darwin)
        YO_RUST_CONFIGS+=(
            "$HOME/Library/Application Support/mang-sh"
            "${XDG_CONFIG_HOME:-$HOME/.config}/mang-sh"
        )
        ;;
    Linux|*)
        YO_RUST_CONFIGS+=(
            "${XDG_CONFIG_HOME:-$HOME/.config}/mang-sh"
        )
        ;;
esac

for LEGACY_CFG in "${YO_RUST_CONFIGS[@]}"; do
    if [[ -d "$LEGACY_CFG" ]]; then
        warn "Legacy yo-rust config found: $LEGACY_CFG"
        if ask_no "Delete legacy yo-rust config?"; then
            rm -rf "$LEGACY_CFG"
            ok "Removed legacy config: $LEGACY_CFG"
        else
            skip "Legacy config kept at: $LEGACY_CFG"
        fi
    fi
done

# =============================================================================
#  Step 3 -- Remove shell aliases
# =============================================================================
printf "\n"

RC_FILES=()
for f in "$HOME/.zshrc" "$HOME/.bashrc" "$HOME/.bash_profile"; do
    [[ -f "$f" ]] && RC_FILES+=("$f")
done

ALIASES_REMOVED=0
for RC_FILE in "${RC_FILES[@]}"; do
    if grep -qE "mang aliases|yo-rust aliases" "$RC_FILE" 2>/dev/null; then
        # Write to a temp file then move -- avoids corrupt rc file on crash
        TMP_RC="$(mktemp)"
        grep -v \
            -e "mang aliases" \
            -e "yo-rust aliases" \
            -e "alias hi='yo'" \
            -e "alias hello='yo'" \
            -e "alias yo=" \
            "$RC_FILE" > "$TMP_RC"
        mv "$TMP_RC" "$RC_FILE"
        ok "Removed mang aliases from $RC_FILE"
        ALIASES_REMOVED=1
    fi
done

if [[ $ALIASES_REMOVED -eq 0 ]]; then
    skip "No mang aliases found in shell config files."
fi

# =============================================================================
#  Done
# =============================================================================
printf "\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "${CYN}  |          Uninstall complete!            |${RST}\n"
printf "${CYN}  +==========================================+${RST}\n"
printf "\n"
printf "  mang.sh has been removed.\n"
printf "\n"
printf "  ${DIM}Rust itself was NOT removed.${RST}\n"
printf "  ${DIM}To remove Rust too: ${BLD}rustup self uninstall${RST}\n"
printf "\n"
printf "  ${DIM}To reinstall mang.sh at any time:${RST}\n"
printf "  ${CYN}  curl -fsSL https://mang.sh/install | bash${RST}\n"
printf "\n"
