// =============================================================================
//  feedback.rs — !feedback / !fb shortcut handler
//  https://github.com/paulfxyz/mang-sh
//
//  OVERVIEW
//  ────────
//  This module owns everything related to the !feedback / !fb shortcut:
//  parsing subcommands, running the setup wizard, and dispatching actions.
//
//  SUBCOMMANDS
//  ───────────
//  !feedback  / !fb           Show current status (same as !feedback status)
//  !feedback status           Show current telemetry config at a glance
//  !feedback setup            Full interactive setup wizard
//  !feedback on               Enable community sharing immediately
//  !feedback off              Disable community sharing immediately
//  !feedback personal         Configure personal JSONBin (key + collection)
//  !feedback clear            Remove ALL telemetry settings
//  !feedback about            Explain JSONBin and the data pipeline
//
//  These commands are checked in main.rs's REPL loop before the AI call,
//  so they are always instant regardless of network state.
// =============================================================================

use crate::config::Config;
use crate::ui;
use colored::Colorize;
use std::io::{self, Write};

/// All possible !feedback / !fb subcommand inputs.
#[derive(Debug, PartialEq)]
pub enum FeedbackCommand {
    Status,
    Setup,
    On,
    Off,
    Personal,
    Clear,
    About,
    /// Send a test entry right now and show the result synchronously
    Test,
    Unknown(String),
}

/// Parse a REPL input line into a FeedbackCommand, or return None if it's not
/// a feedback command at all.
pub fn parse(line: &str) -> Option<FeedbackCommand> {
    let line = line.trim().to_lowercase();

    // Match both !feedback and !fb as the trigger
    let sub = if let Some(rest) = line.strip_prefix("!feedback") {
        rest.trim().to_string()
    } else if let Some(rest) = line.strip_prefix("!fb") {
        rest.trim().to_string()
    } else {
        return None;
    };

    Some(match sub.as_str() {
        "" | "status" => FeedbackCommand::Status,
        "setup"       => FeedbackCommand::Setup,
        "on"          => FeedbackCommand::On,
        "off"         => FeedbackCommand::Off,
        "personal"    => FeedbackCommand::Personal,
        "clear"       => FeedbackCommand::Clear,
        "about"       => FeedbackCommand::About,
        "test"        => FeedbackCommand::Test,
        other         => FeedbackCommand::Unknown(other.to_string()),
    })
}

/// Dispatch a FeedbackCommand.
///
/// Mutates `cfg` in-place and returns true if config was changed (caller saves).
pub fn dispatch(cmd: FeedbackCommand, cfg: &mut Config) -> bool {
    match cmd {
        FeedbackCommand::Status => {
            ui::print_feedback_status(cfg);
            false
        }

        FeedbackCommand::Setup => {
            run_setup_wizard(cfg)
        }

        FeedbackCommand::On => {
            cfg.telemetry_share_central = true;
            cfg.sessions_since_telemetry_prompt = 0;
            println!("{}", "  ✔  Community sharing: ON".green().bold());
            println!("  {}  {}", "◈".cyan(),
                "Your successful commands will be anonymously shared to improve mang.sh.".dimmed());
            println!("  {}  {}", "◈".cyan(), "Type !feedback off to disable at any time.".dimmed());
            println!();
            true
        }

        FeedbackCommand::Off => {
            cfg.telemetry_share_central = false;
            cfg.sessions_since_telemetry_prompt = 0;
            println!("{}", "  ✔  Community sharing: OFF".dimmed());
            println!("  {}  {}", "◈".cyan(), "Type !feedback on to re-enable.".dimmed());
            println!();
            true
        }

        FeedbackCommand::Personal => {
            run_personal_wizard(cfg)
        }

        FeedbackCommand::Clear => {
            run_clear(cfg)
        }

        FeedbackCommand::Test => {
            run_test(cfg)
        }

        FeedbackCommand::About => {
            ui::print_feedback_about();
            false
        }

        FeedbackCommand::Unknown(sub) => {
            println!();
            println!("{}", format!("  ✗  Unknown subcommand: !feedback {sub}").red());
            println!("  {}", "Available subcommands:".white().bold());
            print_usage();
            false
        }
    }
}

// =============================================================================
//  Setup wizard — full interactive configuration
// =============================================================================

fn run_setup_wizard(cfg: &mut Config) -> bool {
    println!();
    println!("{}", "  ╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "  ║      📊  Feedback Setup Wizard                      ║".cyan().bold());
    println!("{}", "  ╚══════════════════════════════════════════════════════╝".cyan());
    println!();
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "mang.sh can share anonymised data to help improve future versions.".white()
    );
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "All sharing is opt-in. You can change these at any time.".dimmed()
    );
    println!();

    // ── Step 1: Community sharing ─────────────────────────────────────────────
    println!("  {}", "STEP 1 — Community sharing".white().bold());
    println!("  {}", "  Share anonymised prompt/command pairs with the mang.sh community".dimmed());
    println!("  {}", "  dataset? Data goes to a private JSONBin collection reviewed weekly".dimmed());
    println!("  {}", "  by the maintainer to improve the AI system prompt.".dimmed());
    println!();
    println!("  {}", "  Collected: prompt · commands · OS · shell · model · worked (Y/N)".dimmed());
    println!("  {}", "  Never collected: API keys · paths · CWD · output · identity".dimmed());
    println!();

    let current_hint = if cfg.telemetry_share_central { "[Y/n]" } else { "[y/N]" };
    print!("  {}  ", format!("Enable community sharing? {} ›", current_hint).yellow().bold());
    io::stdout().flush().unwrap_or(());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    cfg.telemetry_share_central = match input.trim().to_lowercase().as_str() {
        "y" | "yes" => true,
        "n" | "no"  => false,
        ""           => cfg.telemetry_share_central,
        _            => false,
    };

    if cfg.telemetry_share_central {
        println!("{}", "  ✔  Community sharing: ON".green());
    } else {
        println!("{}", "  ✔  Community sharing: OFF".dimmed());
    }

    // ── Step 2: Personal JSONBin ───────────────────────────────────────────────
    println!();
    println!("  {}", "STEP 2 — Personal JSONBin (optional)".white().bold());
    println!("  {}", "  Store your own command history in your own private JSONBin account.".dimmed());
    println!("  {}", "  Only you can read it. Completely independent of the community data.".dimmed());
    println!("  {}", "  Create a free account at: https://jsonbin.io".dimmed());
    println!();

    let has_personal = !cfg.telemetry_user_key.is_empty();
    let personal_hint = if has_personal { "configured, reconfigure? [y/N]" } else { "[y/N]" };
    print!("  {}  ", format!("Set up personal JSONBin? {} ›", personal_hint).yellow().bold());
    io::stdout().flush().unwrap_or(());
    input.clear();
    io::stdin().read_line(&mut input).unwrap_or(0);

    if matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
        run_personal_wizard(cfg);
    } else if has_personal {
        println!("{}", "  ✔  Personal JSONBin: kept as-is.".dimmed());
    } else {
        println!("{}", "  ✔  Personal JSONBin: skipped.".dimmed());
    }

    cfg.sessions_since_telemetry_prompt = 0;

    println!();
    println!("{}", "  ✔  Feedback settings saved.".green().bold());
    println!("  {}  {}", "◈".cyan(), "Type !feedback to review at any time.".dimmed());
    println!();
    true
}

// =============================================================================
//  Personal JSONBin wizard
// =============================================================================

fn run_personal_wizard(cfg: &mut Config) -> bool {
    println!();
    println!("  {}", "Personal JSONBin Setup".white().bold());
    println!("  {}", "  1. Create a free account at https://jsonbin.io".dimmed());
    println!("  {}", "  2. Copy your Master Key from the API Keys page".dimmed());
    println!("  {}", "  3. Create a Collection named e.g. 'my-mang-history'".dimmed());
    println!("  {}", "  4. Paste the Master Key and Collection ID below".dimmed());
    println!();

    // Master Key
    let key_hint = if cfg.telemetry_user_key.is_empty() {
        "paste your Master Key ›".to_string()
    } else {
        "new Master Key (Enter to keep current) ›".to_string()
    };
    print!("  {}  ", key_hint.yellow().bold());
    io::stdout().flush().unwrap_or(());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);
    let key = input.trim().to_string();
    if !key.is_empty() {
        cfg.telemetry_user_key = key;
    }

    if cfg.telemetry_user_key.is_empty() {
        println!("{}", "  ✔  Personal JSONBin: skipped (no key entered).".dimmed());
        println!();
        return false;
    }

    // Collection ID
    println!();
    let col_hint = if cfg.telemetry_user_collection.is_empty() {
        "Collection ID (from JSONBin dashboard) ›".to_string()
    } else {
        format!("Collection ID (Enter to keep {}) ›", &cfg.telemetry_user_collection)
    };
    print!("  {}  ", col_hint.yellow().bold());
    io::stdout().flush().unwrap_or(());
    input.clear();
    io::stdin().read_line(&mut input).unwrap_or(0);
    let col = input.trim().to_string();
    if !col.is_empty() {
        cfg.telemetry_user_collection = col;
    }

    if cfg.telemetry_user_collection.is_empty() {
        println!();
        println!("{}", "  ⚠  Collection ID not set — personal JSONBin will be skipped.".yellow());
        println!("  {}  Run !feedback personal again to add it.", "◈".cyan());
    } else {
        // Quick connectivity test
        println!();
        println!("{}", "  ◌  Testing connection to your JSONBin…".dimmed());
        match test_personal_bin(&cfg.telemetry_user_key, &cfg.telemetry_user_collection) {
            Ok(bin_id) => {
                println!("{}", format!("  ✔  Connected! Test bin created: {bin_id}").green());
                println!("  {}  {}", "◈".cyan(), "Cleaning up test entry…".dimmed());
                // Delete the test bin — we have the master key
                let _ = delete_test_bin(&cfg.telemetry_user_key, &bin_id);
                println!("{}", "  ✔  Personal JSONBin: configured and verified.".green().bold());
            }
            Err(e) => {
                println!("{}", format!("  ✗  Connection test failed: {e}").red());
                println!("  {}  {}", "◈".cyan(), "Check your Master Key and Collection ID.".dimmed());
                println!("  {}  {}", "◈".cyan(), "Settings saved anyway — will retry on next use.".dimmed());
            }
        }
    }

    println!();
    true
}

/// Send a test bin to the user's personal JSONBin to verify connectivity.
fn test_personal_bin(master_key: &str, collection_id: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post("https://api.jsonbin.io/v3/b")
        .header("Content-Type", "application/json")
        .header("X-Master-Key", master_key)
        .header("X-Bin-Private", "true")
        .header("X-Bin-Name", "mang-connection-test")
        .header("X-Collection-Id", collection_id)
        .body(r#"{"mang":"connection_test"}"#)
        .send()
        .map_err(|e| format!("Network error: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(format!("HTTP {status}: {body}"));
    }

    let json: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
    let bin_id = json["metadata"]["id"]
        .as_str()
        .ok_or("no bin ID in response")?
        .to_string();

    Ok(bin_id)
}

/// Delete a test bin using the master key.
fn delete_test_bin(master_key: &str, bin_id: &str) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://api.jsonbin.io/v3/b/{bin_id}");
    client
        .delete(&url)
        .header("X-Master-Key", master_key)
        .send()
        .map_err(|e| e.to_string())?;

    Ok(())
}

// =============================================================================
//  Clear all telemetry settings
// =============================================================================

fn run_clear(cfg: &mut Config) -> bool {
    println!();
    println!("{}", "  ⚠  This will clear ALL feedback settings.".yellow().bold());
    print!("  {}  ", "Are you sure? [y/N] ›".yellow().bold());
    io::stdout().flush().unwrap_or(());
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap_or(0);

    if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
        println!("{}", "  ✔  Cancelled — nothing changed.".dimmed());
        println!();
        return false;
    }

    cfg.telemetry_share_central         = false;
    cfg.telemetry_user_key              = String::new();
    cfg.telemetry_user_collection       = String::new();
    cfg.sessions_since_telemetry_prompt = 0;

    println!("{}", "  ✔  All feedback settings cleared.".green());
    println!("  {}  {}", "◈".cyan(), "Run !feedback setup to reconfigure at any time.".dimmed());
    println!();
    true
}

// =============================================================================
//  Test — send a live test entry and show the result synchronously
// =============================================================================

fn run_test(cfg: &Config) -> bool {
    use crate::telemetry;
    use crate::shell::ShellKind;

    println!();
    println!("{}", "  ◌  Sending test entry to JSONBin…".dimmed());

    let shell_label = ShellKind::detect().label().to_string();
    let entry = telemetry::TelemetryEntry::new(
        "!feedback test — connectivity check",
        &["echo mang-test".to_string()],
        &cfg.model,
        &cfg.backend,
        &shell_label,
        Some(true),
    );

    let user_key     = if cfg.telemetry_user_key.is_empty() { None } else { Some(cfg.telemetry_user_key.as_str()) };
    let user_col     = if cfg.telemetry_user_collection.is_empty() { None } else { Some(cfg.telemetry_user_collection.as_str()) };

    let result = telemetry::submit_sync_report(
        &entry,
        cfg.telemetry_share_central,
        user_key,
        user_col,
    );

    if result.contains("successfully") {
        println!("{}", format!("  ✔  {result}").green().bold());
        println!("  {}  {}", "◈".cyan(), "Check your JSONBin dashboard — the entry should appear there now.".dimmed());
        println!("  {}  {}", "◈".cyan(), "https://jsonbin.io → Collections → mang.sh-telemetry".dimmed());
    } else {
        println!("{}", format!("  ✗  {result}").red());
        println!("  {}  {}", "◈".cyan(), "Run with MANGDEBUG=1 for verbose output:  MANGDEBUG=1 yo".dimmed());
        println!("  {}  {}", "◈".cyan(), "Is sharing enabled?  Type !feedback on to enable.".dimmed());
    }
    println!();
    false
}

// =============================================================================
//  Usage hint (shown for unknown subcommands)
// =============================================================================

pub fn print_usage() {
    let cmds: &[(&str, &str)] = &[
        ("!feedback  / !fb",          "Show current feedback & telemetry status"),
        ("!feedback status",          "Same as above"),
        ("!feedback setup",           "Run the full interactive setup wizard"),
        ("!feedback on",              "Enable community sharing immediately"),
        ("!feedback off",             "Disable community sharing immediately"),
        ("!feedback personal",        "Configure your own personal JSONBin"),
        ("!feedback clear",           "Remove all feedback settings"),
        ("!feedback about",           "Explain JSONBin and the data pipeline"),
    ];
    println!();
    for (cmd, desc) in cmds {
        println!("    {}  {}", format!("{:<30}", cmd).yellow().bold(), desc.dimmed());
    }
    println!();
}
