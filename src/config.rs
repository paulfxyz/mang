// =============================================================================
//  config.rs — Persistent configuration for yo-rust
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  yo-rust stores all user preferences in a single JSON file:
//    macOS  → ~/Library/Application Support/yo-rust/config.json
//    Linux  → ~/.config/yo-rust/config.json  (or $XDG_CONFIG_HOME)
//    Windows→ %APPDATA%\yo-rust\config.json
//
//  v2.0.0 adds four new configuration fields:
//    backend        — "openrouter" (default) or "ollama" (local)
//    ollama_url     — base URL for the Ollama API (default: http://localhost:11434)
//    history_enabled— append confirmed commands to shell history (default: true)
//    context_size   — number of recent turns to include as follow-up context (default: 5)
//
//  All new fields use #[serde(default)] so existing v1.x config files are
//  read without error — missing fields fall back to sensible defaults.
//
//  BACKEND SELECTION
//  ─────────────────
//  "openrouter" — sends requests to https://openrouter.ai/api/v1/chat/completions.
//    Requires an API key.  Supports any model on OpenRouter's catalogue.
//
//  "ollama"    — sends requests to http://localhost:11434 (or ollama_url).
//    Requires a running Ollama instance (https://ollama.ai).
//    No API key required.  Model must be pulled locally (ollama pull <model>).
//    Ideal for air-gapped environments or privacy-sensitive workloads.
//
//  SECURITY NOTE
//  ─────────────
//  The API key is stored in plaintext.  This is standard practice for
//  developer CLI tools (~/.ssh/config, ~/.npmrc, ~/.cargo/credentials).
//  The file permissions default to 0o600 on Unix (user-readable only).
//  The key is only transmitted over HTTPS to api.openrouter.ai.
// =============================================================================

use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

// =============================================================================
//  Config struct
// =============================================================================

/// All persistent user preferences for yo-rust.
///
/// Every field has a `#[serde(default)]` so that deserialising an older config
/// file (with fewer fields) never panics — missing fields use the Default impl.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    // ── v1.x fields ──────────────────────────────────────────────────────────

    /// OpenRouter API key.  Empty = not yet configured (first-run sentinel).
    #[serde(default)]
    pub api_key: String,

    /// Model slug for OpenRouter, e.g. "openai/gpt-4o-mini".
    /// When backend == "ollama", this is the Ollama model name, e.g. "llama3.2".
    #[serde(default = "default_model")]
    pub model: String,

    // ── v2.0.0 fields ────────────────────────────────────────────────────────

    /// AI backend: "openrouter" (default) or "ollama".
    /// Determines which API endpoint is used and whether an API key is required.
    #[serde(default = "default_backend")]
    pub backend: String,

    /// Base URL for the Ollama API server.
    /// Only used when backend == "ollama".
    /// Change if Ollama is running on a remote host or non-default port.
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,

    /// Whether to append confirmed commands to the shell history file.
    /// true = append to ~/.zsh_history / ~/.bash_history after each confirmed run.
    /// false = no history appending (equivalent to --no-history flag).
    /// Default: true.
    #[serde(default = "default_true")]
    pub history_enabled: bool,

    /// Number of recent prompt/command pairs to include as context in each
    /// new AI request, enabling follow-up prompts ("now do the same for X").
    /// 0 = disabled.  Default: 5.
    #[serde(default = "default_context_size")]
    pub context_size: usize,

    // ── v2.3.0 telemetry fields ─────────────────────────────────────────────────────────────────────────

    /// Whether to share anonymised prompt/command pairs with the central
    /// yo-rust community dataset (Paul's JSONBin collection).
    /// Default: false (opt-in, not opt-out).
    #[serde(default)]
    pub telemetry_share_central: bool,

    /// User's personal JSONBin Master Key for their own private history bin.
    /// Empty = not configured.
    #[serde(default)]
    pub telemetry_user_key: String,

    /// User's personal JSONBin Collection ID.
    #[serde(default)]
    pub telemetry_user_collection: String,

    /// How many sessions have run since the last telemetry opt-in prompt.
    /// Used to periodically remind the user about the feature.
    /// Reset to 0 after the prompt is shown.
    #[serde(default)]
    pub sessions_since_telemetry_prompt: u32,
}

// ── Default value functions (required for #[serde(default = "fn_name")] ) ────
// These must be free functions (not closures) — serde's limitation.

fn default_model()        -> String { "openai/gpt-4o-mini".to_string() }
fn default_backend()      -> String { "openrouter".to_string() }
fn default_ollama_url()   -> String { "http://localhost:11434".to_string() }
fn default_true()         -> bool   { true }
fn default_context_size() -> usize  { 5 }

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key:                         String::new(),
            model:                           default_model(),
            backend:                         default_backend(),
            ollama_url:                      default_ollama_url(),
            history_enabled:                 default_true(),
            context_size:                    default_context_size(),
            telemetry_share_central:         false,
            telemetry_user_key:              String::new(),
            telemetry_user_collection:       String::new(),
            sessions_since_telemetry_prompt: 0,
        }
    }
}

// =============================================================================
//  Path resolution
// =============================================================================

/// Returns the absolute path to the config JSON file.
///
/// We recompute this on every call (rather than caching it) because:
///   1. The path is cheap to compute (a few string joins).
///   2. It keeps the module stateless — no global or lazy_static needed.
///   3. XDG_CONFIG_HOME could theoretically change between calls.
fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("yo-rust").join("config.json")
}

// =============================================================================
//  Load
// =============================================================================

/// Read and deserialise the config file.
///
/// Returns `Config::default()` if the file does not exist (first-run).
/// Propagates errors for corrupted/unreadable files — the caller exits.
pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = fs::read_to_string(&path)?;
    let cfg: Config = serde_json::from_str(&raw)?;
    Ok(cfg)
}

// =============================================================================
//  Save
// =============================================================================

/// Serialise and write the config to disk.
///
/// Creates parent directories if they don't exist yet.
/// On failure, the caller logs a warning but the session continues.
pub fn save(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(cfg)?;
    fs::write(&path, json)?;
    Ok(())
}

// =============================================================================
//  Interactive setup
// =============================================================================

/// Walk the user through the full configuration wizard.
///
/// Mutates `cfg` in-place.  The caller is responsible for calling `save()`
/// after this returns.
///
/// v2.0.0 adds:
///   - Backend selection (OpenRouter vs Ollama)
///   - Ollama URL prompt (only when Ollama is selected)
///   - History preference toggle
///   - Context size preference
pub fn interactive_setup(cfg: &mut Config) {
    println!();
    println!("{}", "  ╔════════════════════════════════════════════════╗".cyan());
    println!("{}", "  ║            yo-rust Configuration               ║".cyan().bold());
    println!("{}", "  ╚════════════════════════════════════════════════╝".cyan());
    println!();

    // ── 1. Backend selection ──────────────────────────────────────────────────
    println!("  {}", "AI Backend:".white().bold());
    println!("  {}", "  1) OpenRouter  (cloud — GPT-4o, Claude, Llama, etc.)".dimmed());
    println!("  {}", "  2) Ollama      (local — private, offline, no API key needed)".dimmed());
    println!();
    print!("  {}  ", "Backend [1] ›".yellow().bold());
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    match input.trim() {
        "2" => {
            cfg.backend = "ollama".to_string();
            setup_ollama(cfg);
        }
        _ => {
            cfg.backend = "openrouter".to_string();
            setup_openrouter(cfg);
        }
    }

    // ── 2. Shell history preference ────────────────────────────────────────
    println!();
    println!("  {}", "Shell history:".white().bold());
    println!("  {}", "  Append confirmed commands to ~/.zsh_history / ~/.bash_history?".dimmed());
    println!("  {}", "  This makes them available via ↑ in your terminal.".dimmed());
    println!();
    print!("  {}  ", "Enable history appending? [Y/n] ›".yellow().bold());
    io::stdout().flush().unwrap_or(());

    input.clear();
    io::stdin().read_line(&mut input).unwrap_or(0);
    cfg.history_enabled = !matches!(input.trim().to_lowercase().as_str(), "n" | "no");

    // ── 3. Context size preference ─────────────────────────────────────────
    println!();
    println!("  {}", "Follow-up context:".white().bold());
    println!("  {}", "  How many recent commands to remember for follow-up prompts?".dimmed());
    println!("  {}", "  (0 = disabled, default = 5, max = 20)".dimmed());
    println!();
    print!("  {}  ", "Context size [5] ›".yellow().bold());
    io::stdout().flush().unwrap_or(());

    input.clear();
    io::stdin().read_line(&mut input).unwrap_or(0);
    cfg.context_size = input.trim().parse::<usize>().unwrap_or(5).min(20);

    // ── Telemetry / community sharing setup ────────────────────────────────────────────
    // Delegate to the feedback module's setup wizard — single source of truth.
    crate::feedback::dispatch(crate::feedback::FeedbackCommand::Setup, cfg);

    println!();
    println!("{}", "  ✔  Configuration saved.".green().bold());
}

/// Sub-wizard for OpenRouter setup.
fn setup_openrouter(cfg: &mut Config) {
    println!();
    println!(
        "  {}",
        "Get your free API key at: https://openrouter.ai/keys".dimmed()
    );
    println!(
        "  {}",
        "Free models available — no credit card required for basic usage.".dimmed()
    );
    println!();

    // API key loop — retry until non-empty
    loop {
        print!("  {}  ", "OpenRouter API key ›".yellow().bold());
        io::stdout().flush().unwrap_or(());
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap_or(0);
        let key = input.trim().to_string();
        if key.is_empty() {
            println!("{}", "  ✗  API key cannot be empty.".red());
            continue;
        }
        cfg.api_key = key;
        break;
    }

    // Model selection
    println!();
    println!("  {}", "Select a model:".white().bold());
    println!("  {}", "  1) openai/gpt-4o-mini              ★ recommended — fast, reliable".dimmed());
    println!("  {}", "  2) openai/gpt-4o                   most powerful".dimmed());
    println!("  {}", "  3) anthropic/claude-3.5-sonnet     best reasoning".dimmed());
    println!("  {}", "  4) anthropic/claude-3-haiku        very fast, low cost".dimmed());
    println!("  {}", "  5) meta-llama/llama-3.3-70b-instruct:free   free tier".dimmed());
    println!("  {}", "  Or type any OpenRouter model slug directly.".dimmed());
    println!("  {}", "  Leave blank for gpt-4o-mini (recommended).".dimmed());
    println!();

    print!("  {}  ", "Model [1] ›".yellow().bold());
    io::stdout().flush().unwrap_or(());

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    cfg.model = match input.trim() {
        "" | "1" => "openai/gpt-4o-mini".to_string(),
        "2"      => "openai/gpt-4o".to_string(),
        "3"      => "anthropic/claude-3.5-sonnet".to_string(),
        "4"      => "anthropic/claude-3-haiku".to_string(),
        "5"      => "meta-llama/llama-3.3-70b-instruct:free".to_string(),
        custom   => custom.to_string(),
    };

    println!();
    println!(
        "  {}  model: {}",
        "✔  OpenRouter configured →".green().bold(),
        cfg.model.cyan()
    );
}

/// Sub-wizard for Ollama setup.
fn setup_ollama(cfg: &mut Config) {
    println!();
    println!("  {}", "Ollama — local AI backend".white().bold());
    println!("  {}", "  Install Ollama: https://ollama.ai".dimmed());
    println!("  {}", "  Pull a model:   ollama pull llama3.2".dimmed());
    println!();

    // Ollama base URL
    print!("  {}  ", "Ollama URL [http://localhost:11434] ›".yellow().bold());
    io::stdout().flush().unwrap_or(());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    let url = input.trim().to_string();
    cfg.ollama_url = if url.is_empty() {
        "http://localhost:11434".to_string()
    } else {
        url
    };

    // Model name
    println!();
    println!("  {}", "Common Ollama models (must be pulled first):".white().bold());
    println!("  {}", "  1) llama3.2          (recommended general-purpose)".dimmed());
    println!("  {}", "  2) llama3.1          (older, widely available)".dimmed());
    println!("  {}", "  3) mistral           (fast, good at code/commands)".dimmed());
    println!("  {}", "  4) codellama         (code-optimised)".dimmed());
    println!("  {}", "  Or type any model name you have pulled locally.".dimmed());
    println!();

    print!("  {}  ", "Model [1] ›".yellow().bold());
    io::stdout().flush().unwrap_or(());

    input.clear();
    io::stdin().read_line(&mut input).unwrap_or(0);
    cfg.model = match input.trim() {
        "" | "1" => "llama3.2".to_string(),
        "2"      => "llama3.1".to_string(),
        "3"      => "mistral".to_string(),
        "4"      => "codellama".to_string(),
        custom   => custom.to_string(),
    };

    // Clear the API key when switching to Ollama — it's not needed.
    // We keep any existing key in case the user switches back.

    println!();
    println!(
        "  {}  model: {}  url: {}",
        "✔  Ollama configured →".green().bold(),
        cfg.model.cyan(),
        cfg.ollama_url.dimmed()
    );
}
