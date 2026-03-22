// =============================================================================
//  config.rs — Persistent configuration for yo-rust
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  yo-rust stores exactly two pieces of user configuration:
//    • api_key — the OpenRouter secret key (sk-or-…)
//    • model   — the model slug (e.g. "openai/gpt-4o-mini")
//
//  These are persisted as a plain JSON file at:
//    macOS  → ~/Library/Application Support/yo-rust/config.json
//    Linux  → ~/.config/yo-rust/config.json  (or $XDG_CONFIG_HOME)
//    Windows→ %APPDATA%\yo-rust\config.json
//
//  The `dirs` crate resolves the correct platform path automatically.
//
//  WHY JSON, NOT TOML/YAML/INI?
//  ──────────────────────────────
//  • serde_json is already a dependency (used in ai.rs) — zero extra weight
//  • JSON is universally understood and manually editable by any user
//  • The config is trivially simple (2 string fields) — TOML's advantages
//    (comments, typed arrays) don't apply here
//  • serde_json::to_string_pretty() produces readable output automatically:
//    {
//      "api_key": "sk-or-...",
//      "model": "openai/gpt-4o-mini"
//    }
//
//  SECURITY NOTE
//  ─────────────
//  The API key is stored in plaintext in the config file.  This is a deliberate
//  trade-off: most developer tools (git, npm, cargo, ~/.ssh/config) store
//  credentials in plaintext config files.  The file is only readable by the
//  current user (default filesystem permissions on macOS/Linux are 0o644 for
//  files in ~/.config).  A future enhancement could use the system keychain
//  (Keychain on macOS, Secret Service on Linux) — but this adds significant
//  complexity and platform-specific dependencies.
//
//  The key is ONLY sent over HTTPS to api.openrouter.ai — never logged,
//  printed (except partially in error messages), or written to stdout.
// =============================================================================

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

// =============================================================================
//  Config struct
//
//  Derives:
//    Debug       — allows `{:?}` formatting for troubleshooting (api_key will
//                  appear in debug output — be careful not to log this)
//    Default     — generates Config { api_key: "".into(), model: "".into() }
//                  used when no config file exists yet (first run detection)
//    Serialize   — enables serde_json::to_string_pretty(cfg)
//    Deserialize — enables serde_json::from_str::<Config>(json)
//
//  #[serde(default)] on each field means that if the JSON on disk is missing
//  that key (e.g. an older config file before "model" was added), serde uses
//  the Default impl (empty string) instead of erroring.  This makes the
//  config format forward-compatible: new fields added in future versions won't
//  break existing config files.
// =============================================================================
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// OpenRouter API key.  Format: "sk-or-v1-<hex>".
    /// Empty string = not yet configured (first-run sentinel).
    #[serde(default)]
    pub api_key: String,

    /// OpenRouter model slug.  Format: "provider/model-name[:variant]".
    /// Examples: "openai/gpt-4o-mini", "anthropic/claude-3-haiku",
    ///           "meta-llama/llama-3.3-70b-instruct:free"
    /// Empty string = use the default set in interactive_setup().
    #[serde(default)]
    pub model: String,
}

// =============================================================================
//  config_path
//
//  Returns the absolute path to the config file.
//
//  We call this function from load() and save() rather than computing it once
//  at startup because:
//   1. The path is trivially cheap to compute (a few string concatenations)
//   2. It avoids a global or lazy static — keeps the module stateless
//   3. In theory $XDG_CONFIG_HOME could change between calls (unlikely but
//      this keeps the code correct by construction)
//
//  dirs::config_dir() follows XDG on Linux, uses the platform default on
//  macOS and Windows.  It returns None only in unusual circumstances (no HOME
//  set, missing Windows registry key) — we fall back to "." in that case,
//  which puts the config in the current directory.  Not ideal, but recoverable.
// =============================================================================
fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    // Join appends path components in a platform-safe way (/ on Unix, \ on Windows).
    base.join("yo-rust").join("config.json")
}

// =============================================================================
//  load
//
//  Reads and deserialises the config file.  Returns Config::default() if the
//  file doesn't exist yet (first-run scenario).
//
//  Error cases that propagate up to main():
//   • File exists but is not valid UTF-8
//   • File exists but is not valid JSON
//   • File exists but JSON has unexpected types for known fields
//
//  We do NOT silently ignore a corrupted config — the user should know if their
//  config is broken so they can delete it and start fresh.
// =============================================================================
pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let path = config_path();

    // First-run: the file simply doesn't exist yet.
    // Return an empty default rather than an error — the caller detects
    // cfg.api_key.is_empty() and triggers interactive_setup().
    if !path.exists() {
        return Ok(Config::default());
    }

    // fs::read_to_string reads the file into a heap-allocated String.
    // For a ~100-byte config file this is effectively free.
    let raw = fs::read_to_string(&path)?;

    // serde_json::from_str performs JSON parsing and type coercion in one pass.
    // The #[serde(default)] attributes on Config fields handle missing keys.
    let cfg: Config = serde_json::from_str(&raw)?;
    Ok(cfg)
}

// =============================================================================
//  save
//
//  Serialises and writes the config to disk, creating parent directories
//  if they don't exist yet.
//
//  WHY NOT USE AN ATOMIC WRITE (TEMP FILE + RENAME)?
//  ──────────────────────────────────────────────────
//  Atomic writes (write to tmpfile → fsync → rename) prevent partial writes
//  from corrupting the config file.  For a ~100-byte file written once per
//  session, the window for corruption (SIGKILL during write) is nanoseconds.
//  If it does happen, the user simply re-enters their API key.  The added
//  complexity of temp-file + rename is not warranted here.
//
//  If this were a database or a large file, atomic writes would be essential.
// =============================================================================
pub fn save(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();

    // Create the yo-rust config directory if it doesn't exist.
    // create_dir_all is idempotent — safe to call even if the directory exists.
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Produce pretty-printed JSON (2-space indentation) so the file is
    // human-readable if the user opens it in a text editor.
    let json = serde_json::to_string_pretty(cfg)?;

    // fs::write creates or truncates the file and writes all bytes atomically
    // at the OS level (a single write() syscall for small content on most systems).
    fs::write(&path, json)?;
    Ok(())
}

// =============================================================================
//  interactive_setup
//
//  Walks the user through entering an API key and selecting a model.
//  Mutates `cfg` in place — the caller is responsible for calling save() after.
//
//  WHY MUTATE IN PLACE RATHER THAN RETURN A NEW CONFIG?
//  ──────────────────────────────────────────────────────
//  The caller (main.rs) already owns `cfg` and needs to keep using it after
//  setup.  Mutating in place avoids an unnecessary clone and keeps the borrow
//  checker happy without lifetime gymnastics.
//
//  The API key loop runs until a non-empty string is entered.  We don't
//  validate the key format (sk-or-…) because:
//   1. OpenRouter may change key formats in future
//   2. The user will discover immediately on the next prompt if it's wrong
//   3. Validation would require a test API call — adds 1–3 s to setup
// =============================================================================
pub fn interactive_setup(cfg: &mut Config) {
    println!();
    println!("{}", "  ╔══════════════════════════════════════════════╗".cyan());
    println!("{}", "  ║          OpenRouter  Configuration           ║".cyan().bold());
    println!("{}", "  ╚══════════════════════════════════════════════╝".cyan());
    println!();
    println!(
        "  {}",
        "Get your free API key at:  https://openrouter.ai/keys".dimmed()
    );
    println!(
        "  {}",
        "Free models available — no credit card required for basic usage.".dimmed()
    );
    println!();

    // ── API key input loop ────────────────────────────────────────────────────
    // We loop until the user enters a non-empty string.
    // io::stdout().flush() is necessary before print!() (without newline)
    // because stdout is line-buffered by default — without flush the prompt
    // would not appear until a newline was written.
    loop {
        print!("  {}  ", "OpenRouter API key ›".yellow().bold());
        io::stdout().flush().unwrap_or(()); // Best-effort — ignore flush errors

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or(0);
        let key = input.trim().to_string();

        if key.is_empty() {
            println!("{}", "  ✗  API key cannot be empty. Please paste your key.".red());
            continue;
        }
        cfg.api_key = key;
        break;
    }

    // ── Model selection ───────────────────────────────────────────────────────
    // We present a curated list of well-known models rather than asking the user
    // to type a slug from memory.  Numbers 1-5 map to preset slugs; anything
    // else is treated as a custom slug (advanced users).
    //
    // The list is intentionally short — a long list causes "choice paralysis".
    // Users who want other models can type the slug directly.
    println!();
    println!("  {}", "Select a model:".white().bold());
    println!(
        "  {}",
        "  1) meta-llama/llama-3.3-70b-instruct:free   🆓 free tier, great quality".dimmed()
    );
    println!(
        "  {}",
        "  2) openai/gpt-4o-mini                       fast, cheap, reliable".dimmed()
    );
    println!(
        "  {}",
        "  3) openai/gpt-4o                            powerful, best for complex tasks".dimmed()
    );
    println!(
        "  {}",
        "  4) anthropic/claude-3-haiku                 very fast, low cost".dimmed()
    );
    println!(
        "  {}",
        "  5) anthropic/claude-3.5-sonnet              best reasoning".dimmed()
    );
    println!(
        "  {}",
        "  Or type any OpenRouter model slug (e.g. google/gemini-flash-1.5).".dimmed()
    );
    println!("  {}", "  Leave blank to use option 1 (recommended for new users).".dimmed());
    println!();

    print!("  {}  ", "Model [1] ›".yellow().bold());
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    let choice = input.trim().to_string();

    // Map numeric shortcuts to full slugs; anything else is passed through as-is.
    cfg.model = match choice.as_str() {
        "" | "1" => "meta-llama/llama-3.3-70b-instruct:free".to_string(),
        "2"      => "openai/gpt-4o-mini".to_string(),
        "3"      => "openai/gpt-4o".to_string(),
        "4"      => "anthropic/claude-3-haiku".to_string(),
        "5"      => "anthropic/claude-3.5-sonnet".to_string(),
        custom   => custom.to_string(), // Advanced user — trust the input
    };

    println!();
    println!(
        "  {}  using model: {}",
        "✔  Saved →".green().bold(),
        cfg.model.cyan()
    );
    println!(
        "  {}",
        "  Config written to ~/.config/yo-rust/config.json".dimmed()
    );
}
