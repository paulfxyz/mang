// =============================================================================
//  history.rs — Shell history integration
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  When a user confirms and runs a command via yo-rust, that command is never
//  added to their shell's history file.  This means:
//    - Pressing ↑ in the terminal after running yo-rust won't show the command
//    - `history | grep` won't find commands run through yo-rust
//    - The user has no persistent record of what ran
//
//  This module fixes that by appending each confirmed command directly to the
//  appropriate shell history file after execution.
//
//  SUPPORTED SHELLS
//  ────────────────
//  zsh   → ~/.zsh_history
//    Format:  ": <unix_timestamp>:0;<command>\n"
//    zsh's EXTENDED_HISTORY format includes a timestamp and duration (always 0
//    for externally-appended entries).  Most zsh configs use this format.
//    Plain format (just the command) is also valid but loses timestamp.
//    We use EXTENDED_HISTORY format so the entry integrates naturally.
//
//  bash  → ~/.bash_history  (or $HISTFILE if set)
//    Format:  "<command>\n"
//    Bash history is plain — one command per line, no metadata.
//
//  fish  → ~/.local/share/fish/fish_history
//    Format:  "- cmd: <command>\n  when: <unix_timestamp>\n"
//    YAML-like format.  Supported but less common.
//
//  DETECTION
//  ─────────
//  We detect the shell from the SHELL environment variable.  If SHELL is
//  not set (unusual but possible), we check HISTFILE for bash, then try
//  zsh_history as the most common fallback.
//
//  SAFETY
//  ──────
//  We open the history file in append-only mode — we never read or truncate it.
//  A failed write (disk full, read-only filesystem) is a soft warning, not
//  a fatal error.  The command already ran successfully; history is bonus.
//
//  MULTI-COMMAND ENTRIES
//  ──────────────────────
//  When yo-rust suggests multiple commands ("first do X, then do Y"), each
//  command is appended as a separate history entry rather than joining them
//  with &&.  This matches how the user would have typed them interactively
//  and makes individual commands findable via history search.
//
//  NOTE ON LIVE SHELL HISTORY
//  ──────────────────────────
//  Writing to the history file does NOT make the commands immediately visible
//  in the current shell session's ↑ recall — that buffer is managed in memory
//  by the shell process itself.  A new terminal window (or `history -r` in zsh,
//  `history -n` in bash) will pick up the appended entries.  This is the same
//  behaviour as any external process that writes to the history file.
// =============================================================================

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Append a slice of confirmed commands to the user's shell history file.
///
/// Each command in `commands` is written as a separate history entry.
/// Failures are non-fatal: an error message is printed but execution continues.
///
/// # Arguments
/// * `commands` — the commands that were just confirmed and executed
pub fn append_to_history(commands: &[String]) {
    if commands.is_empty() {
        return;
    }

    let shell = std::env::var("SHELL").unwrap_or_default();

    if shell.contains("zsh") {
        append_zsh(commands);
    } else if shell.contains("fish") {
        append_fish(commands);
    } else {
        // Default to bash format for bash, sh, dash, and unknowns.
        // HISTFILE overrides the default ~/.bash_history location.
        append_bash(commands);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  zsh history
//  Format: ": <timestamp>:0;<command>\n"
// ─────────────────────────────────────────────────────────────────────────────
fn append_zsh(commands: &[String]) {
    let path = zsh_history_path();
    let ts = unix_timestamp();

    match OpenOptions::new().append(true).create(true).open(&path) {
        Ok(mut f) => {
            for cmd in commands {
                // zsh EXTENDED_HISTORY format.
                // The ":0" is elapsed time in seconds (always 0 for us).
                let entry = format!(": {}:0;{}\n", ts, cmd);
                if let Err(e) = f.write_all(entry.as_bytes()) {
                    eprintln!("  [history] Could not write to {:?}: {}", path, e);
                }
            }
        }
        Err(e) => {
            eprintln!("  [history] Could not open {:?}: {}", path, e);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  bash / generic history
//  Format: "<command>\n"
// ─────────────────────────────────────────────────────────────────────────────
fn append_bash(commands: &[String]) {
    let path = bash_history_path();

    match OpenOptions::new().append(true).create(true).open(&path) {
        Ok(mut f) => {
            for cmd in commands {
                let entry = format!("{}\n", cmd);
                if let Err(e) = f.write_all(entry.as_bytes()) {
                    eprintln!("  [history] Could not write to {:?}: {}", path, e);
                }
            }
        }
        Err(e) => {
            eprintln!("  [history] Could not open {:?}: {}", path, e);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  fish history
//  Format (YAML-like):
//    - cmd: <command>
//      when: <timestamp>
// ─────────────────────────────────────────────────────────────────────────────
fn append_fish(commands: &[String]) {
    let path = fish_history_path();
    let ts = unix_timestamp();

    match OpenOptions::new().append(true).create(true).open(&path) {
        Ok(mut f) => {
            for cmd in commands {
                let entry = format!("- cmd: {}\n  when: {}\n", cmd, ts);
                if let Err(e) = f.write_all(entry.as_bytes()) {
                    eprintln!("  [history] Could not write to {:?}: {}", path, e);
                }
            }
        }
        Err(e) => {
            eprintln!("  [history] Could not open {:?}: {}", path, e);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Path resolution helpers
// ─────────────────────────────────────────────────────────────────────────────

fn zsh_history_path() -> PathBuf {
    // ZDOTDIR lets users keep zsh config outside $HOME.  If set, the history
    // file lives there instead.
    if let Ok(zdotdir) = std::env::var("ZDOTDIR") {
        PathBuf::from(zdotdir).join(".zsh_history")
    } else {
        home_dir().join(".zsh_history")
    }
}

fn bash_history_path() -> PathBuf {
    // HISTFILE overrides the default location.
    if let Ok(histfile) = std::env::var("HISTFILE") {
        if !histfile.is_empty() {
            return PathBuf::from(histfile);
        }
    }
    home_dir().join(".bash_history")
}

fn fish_history_path() -> PathBuf {
    // Fish history lives under XDG_DATA_HOME (or ~/.local/share on Linux,
    // ~/Library/Application Support on macOS).
    if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("fish").join("fish_history")
    } else {
        home_dir().join(".local/share/fish/fish_history")
    }
}

/// Returns the user's home directory, falling back to "." if unresolvable.
fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Returns the current Unix timestamp in seconds.
fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
