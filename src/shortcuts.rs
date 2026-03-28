// =============================================================================
//  shortcuts.rs — Named command shortcuts (user feature request)
//  https://github.com/paulfxyz/mang-sh
//
//  OVERVIEW
//  ────────
//  Feature requested by Wayne (Windows user, 2026-03-22):
//
//    "Create shortcuts that can access previously run commands without
//     confirming to run, example: !restartdocker which will restart Docker
//     that was previously ran.  Allow to tag commands with the shortcut
//     that starts with ! and the shortcut name."
//
//  HOW IT WORKS
//  ────────────
//  Users can save any command set as a named shortcut:
//
//    yo ›  !save restartdocker
//    → saves the most recently confirmed commands as "!restartdocker"
//
//  Then replay it at any time, instantly, no AI call, no confirmation:
//
//    yo ›  !restartdocker
//    → runs the saved commands immediately
//
//  SHORTCUTS STORAGE
//  ─────────────────
//  Shortcuts are persisted to disk at:
//    macOS/Linux: ~/.config/mang-sh/shortcuts.json
//    Windows:     %APPDATA%\mang.sh\shortcuts.json
//
//  Format: a flat JSON map of  shortcut_name → [cmd1, cmd2, ...]
//  Example:
//    {
//      "restartdocker": ["docker restart mycontainer"],
//      "deployapp":     ["git push heroku main", "heroku logs --tail"]
//    }
//
//  Keys are stored WITHOUT the leading ! — the ! prefix is the invocation
//  syntax.  This avoids confusion in the JSON file.
//
//  DESIGN DECISIONS
//  ────────────────
//  • No confirmation on replay.  This is explicitly what the user asked for:
//    "without confirming to run".  The user named and saved this intentionally.
//  • Names are case-insensitive (stored lowercase) so !RestartDocker and
//    !restartdocker both work.
//  • Names must be alphanumeric + underscore/hyphen only — no spaces, no
//    special characters — to keep parsing simple and unambiguous.
//  • Shortcuts persist across sessions (saved to disk) but can be managed
//    with !shortcuts (list), !save <name>, and !forget <name>.
//  • We use the last *confirmed and executed* commands as the save source,
//    not the AI suggestion — we only save things that actually ran.
// =============================================================================

use std::collections::HashMap;
use std::path::PathBuf;

use colored::Colorize;
use serde::{Deserialize, Serialize};

/// A map of shortcut name → command list.
/// Stored as plain JSON on disk for easy manual editing.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ShortcutStore {
    /// Maps shortcut name (without !) to the list of shell commands.
    #[serde(default)]
    pub shortcuts: HashMap<String, Vec<String>>,
}

impl ShortcutStore {
    // ── Persistence ──────────────────────────────────────────────────────────

    /// Load shortcuts from disk.  Returns an empty store if the file doesn't
    /// exist yet (first run, or no shortcuts saved yet).
    pub fn load() -> Self {
        let path = shortcuts_path();
        if !path.exists() {
            return Self::default();
        }
        match std::fs::read_to_string(&path) {
            Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
            Err(_)  => Self::default(),
        }
    }

    /// Save shortcuts to disk.  Non-fatal on error — prints a warning.
    pub fn save(&self) {
        let path = shortcuts_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                if let Err(e) = std::fs::write(&path, json) {
                    eprintln!("{}", format!("  ✗  Could not save shortcuts: {e}").red());
                }
            }
            Err(e) => {
                eprintln!("{}", format!("  ✗  Could not serialise shortcuts: {e}").red());
            }
        }
    }

    // ── Core operations ───────────────────────────────────────────────────────

    /// Save a set of commands under a given shortcut name.
    ///
    /// # Arguments
    /// * `name`     — shortcut name WITHOUT the leading `!`
    /// * `commands` — the commands to save
    ///
    /// Returns `Err` with a message if the name is invalid.
    pub fn save_shortcut(&mut self, name: &str, commands: &[String]) -> Result<(), String> {
        let normalised = normalise_name(name)?;
        self.shortcuts.insert(normalised, commands.to_vec());
        self.save();
        Ok(())
    }

    /// Remove a shortcut by name.  Returns true if it existed.
    pub fn forget(&mut self, name: &str) -> bool {
        let Ok(normalised) = normalise_name(name) else { return false; };
        let existed = self.shortcuts.remove(&normalised).is_some();
        if existed {
            self.save();
        }
        existed
    }

    /// Look up a shortcut by name.  Returns `None` if not found.
    pub fn get(&self, name: &str) -> Option<&Vec<String>> {
        let Ok(normalised) = normalise_name(name) else { return None; };
        self.shortcuts.get(&normalised)
    }

    /// Returns true if a shortcut with this name exists.
    #[allow(dead_code)]
    pub fn exists(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    // ── Display ───────────────────────────────────────────────────────────────

    /// Print all saved shortcuts to stdout.
    pub fn print_all(&self) {
        println!();
        if self.shortcuts.is_empty() {
            println!("  {}  {}", "◈".cyan(), "No shortcuts saved yet.".dimmed());
            println!("  {}  {}", "◈".cyan(), "After confirming a command, type !save <name> to save it.".dimmed());
            println!();
            return;
        }

        println!("{}", "  ╔══════════════════════════════════════════════════════╗".cyan());
        println!("{}", "  ║              Saved Shortcuts                         ║".cyan().bold());
        println!("{}", "  ╚══════════════════════════════════════════════════════╝".cyan());
        println!();

        // Sort alphabetically for stable display
        let mut names: Vec<&String> = self.shortcuts.keys().collect();
        names.sort();

        for name in names {
            println!("  {}  {}", format!("!{name}").yellow().bold(), "→".dimmed());
            if let Some(cmds) = self.shortcuts.get(name) {
                for cmd in cmds {
                    println!("       {}  {}", "$".dimmed(), cmd.white());
                }
            }
            println!();
        }

        println!(
            "  {}  {}",
            "◈".cyan(),
            "Run any shortcut instantly: type !name at the yo › prompt.".dimmed()
        );
        println!(
            "  {}  {}",
            "◈".cyan(),
            "Remove a shortcut: !forget <name>".dimmed()
        );
        println!();
    }
}

// ── Parse shortcut invocation from user input ─────────────────────────────────

/// Represents what kind of shortcut-related input the user typed.
#[derive(Debug, PartialEq)]
pub enum ShortcutInput {
    /// User typed `!save <name>` — save last commands under this name
    Save(String),
    /// User typed `!forget <name>` — remove this shortcut
    Forget(String),
    /// User typed `!shortcuts` — list all shortcuts
    List,
    /// User typed `!<name>` — run the named shortcut
    Run(String),
    /// Not a shortcut-related input
    NotAShortcut,
}

/// Parse a user input string into a `ShortcutInput`.
///
/// This is called in the main REPL loop before any AI processing.
/// We handle the following patterns:
///   `!shortcuts`        → List
///   `!save <name>`      → Save("name")
///   `!forget <name>`    → Forget("name")
///   `!<alphanumeric>`   → Run("name")  (if name looks valid)
///   anything else       → NotAShortcut
pub fn parse_shortcut_input(line: &str) -> ShortcutInput {
    let line = line.trim();

    if !line.starts_with('!') {
        return ShortcutInput::NotAShortcut;
    }

    // Already-handled built-in shortcuts (don't intercept them here).
    // !prompt / !p are handled in main.rs before this function is called.
    match line {
        "!help" | "!h" | "!api" | "!exit" | "!quit" | "!q"
        | "!context" | "!ctx" | "!clear"
        | "!prompt" | "!p"
        | "!credits" | "!cr"
        => return ShortcutInput::NotAShortcut,
        _ => {}
    }

    if line == "!shortcuts" {
        return ShortcutInput::List;
    }

    if let Some(rest) = line.strip_prefix("!save ") {
        let name = rest.trim().to_string();
        return if name.is_empty() {
            ShortcutInput::NotAShortcut
        } else {
            ShortcutInput::Save(name)
        };
    }

    if let Some(rest) = line.strip_prefix("!forget ") {
        let name = rest.trim().to_string();
        return if name.is_empty() {
            ShortcutInput::NotAShortcut
        } else {
            ShortcutInput::Forget(name)
        };
    }

    // Anything else starting with ! that looks like a valid name is a Run attempt
    let name = &line[1..]; // strip leading !
    if !name.is_empty() && is_valid_shortcut_name(name) {
        return ShortcutInput::Run(name.to_lowercase());
    }

    ShortcutInput::NotAShortcut
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Returns the path to the shortcuts JSON file.
fn shortcuts_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("mang").join("shortcuts.json")
}

/// Normalise and validate a shortcut name.
/// Allowed: letters, digits, underscores, hyphens.  No spaces.
fn normalise_name(name: &str) -> Result<String, String> {
    // Strip a leading ! if the user accidentally included it
    let name = name.strip_prefix('!').unwrap_or(name).trim();
    if name.is_empty() {
        return Err("Shortcut name cannot be empty.".to_string());
    }
    if !is_valid_shortcut_name(name) {
        return Err(format!(
            "Invalid shortcut name '{name}'. Use only letters, digits, - and _."
        ));
    }
    Ok(name.to_lowercase())
}

/// Returns true if the name contains only alphanumeric chars, hyphens, underscores.
fn is_valid_shortcut_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}
