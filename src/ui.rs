// =============================================================================
//  ui.rs — Terminal UI: banner, help, suggestion display, prompts
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  All visual output lives here.  No business logic, no I/O beyond stdout.
//  Every public function is self-contained and stateless.
//
//  v2.0.0 additions:
//    • Dry-run indicator in banner and suggestion display
//    • Backend indicator (OpenRouter vs Ollama) in intro line
//    • Context turn counter in help screen
//    • !context / !clear shortcuts documented
//    • print_context_summary() — show what the AI currently "remembers"
//    • print_empty_suggestion() — graceful display when AI returns no commands
// =============================================================================

use crate::ai::Suggestion;
use crate::config::Config;
use crate::context::ConversationContext;
use colored::Colorize;

/// Current version — single source of truth for the banner.
/// Keep in sync with Cargo.toml `version` field.
/// Future improvement: replace with env!("CARGO_PKG_VERSION") at compile time.
const VERSION: &str = "v2.3.2";

// =============================================================================
//  print_banner
//
//  Split-panel layout:
//    LEFT  — ASCII robot (antenna, eyes, arms, mouth, chest, legs)
//    RIGHT — Block-letter "YO," logo (cyan) + "RUST!" logo (white/bold)
//
//  Dry-run mode adds a visible [DRY RUN] badge so the user always knows
//  the session is non-destructive at a glance.
// =============================================================================
pub fn print_banner(dry_run: bool) {
    println!();

    // Each entry: (line_text, colour_code)
    //   0 = cyan            robot parts + YO, logo
    //   1 = white + bold    RUST! logo
    //   2 = cyan + dimmed   outer frame, footer
    let lines: &[(&str, u8)] = &[
        ("  ╔══════════════════════════════════════════════════════════════════╗", 2),
        ("  ║                                                                  ║", 2),
        ("  ║           ╷▲╷             ██╗   ██╗ ██████╗                     ║", 0),
        ("  ║      ┌────┴─┴────┐        ╚██╗ ██╔╝██╔═══██╗                    ║", 0),
        ("  ║      │ ╔═══╗╔═══╗│         ╚████╔╝ ██║   ██║                    ║", 0),
        ("  ║      │ ║◈  ◈║◈  ◈║│          ╚██╔╝  ██║   ██║                    ║", 0),
        ("  ║      │ ╚═══╝╚═══╝│           ██║   ╚██████╔╝                    ║", 0),
        ("  ║ ┌──┐ │ ┌─────────┐ │ ┌──┐    ╚═╝    ╚═════╝                     ║", 0),
        ("  ║ │░░├─┤ │ · · · · │ ├─░░│                                        ║", 0),
        ("  ║ └──┘ │ ┌──┬──┬──┐ │ └──┘   ██████╗ ██╗   ██╗███████╗████████╗  ║", 1),
        ("  ║      │ │▓▓│▓▓│▓▓│ │        ██╔══██╗██║   ██║██╔════╝╚══██╔══╝  ║", 1),
        ("  ║      │ └──┴──┴──┘ │        ██████╔╝██║   ██║███████╗   ██║     ║", 1),
        ("  ║      └─────┬─┬────┘        ██╔══██╗██║   ██║╚════██║   ██║     ║", 1),
        ("  ║            │ │             ██║  ██║╚██████╔╝███████║   ██║     ║", 1),
        ("  ║           ┌┘ └┐            ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝     ║", 1),
        ("  ║          ┌┴─┐┌─┴┐                                                ║", 2),
    ];

    for (line, code) in lines {
        match code {
            0 => println!("{}", line.cyan()),
            1 => println!("{}", line.white().bold()),
            _ => println!("{}", line.cyan().dimmed()),
        }
    }

    // Version + repo footer — VERSION const drives this line
    println!(
        "{}",
        format!("  ║          │░░││░░│           {VERSION}  ·  github.com/paulfxyz       ║")
            .cyan()
            .dimmed()
    );
    println!(
        "{}",
        "  ║          └──┘└──┘                                                ║"
            .cyan()
            .dimmed()
    );
    println!(
        "{}",
        "  ╚══════════════════════════════════════════════════════════════════╝"
            .cyan()
            .dimmed()
    );

    println!();
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Natural language → Terminal commands, powered by AI.".white()
    );
    if dry_run {
        println!(
            "  {}  {}",
            "◈".yellow().bold(),
            "DRY-RUN MODE — commands will be shown but never executed."
                .yellow()
                .bold()
        );
    }
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Type !help for all options.".dimmed()
    );
    println!();
}

// =============================================================================
//  print_intro
//
//  Shown after banner + optional first-run setup, immediately before the
//  first REPL prompt.  Shows active backend, dry-run status, and feature flags.
// =============================================================================
pub fn print_intro(cfg: &Config, dry_run: bool) {
    println!();

    // Backend indicator
    let backend_str = if cfg.backend == "ollama" {
        format!(
            "Ollama  ({})  model: {}",
            cfg.ollama_url.dimmed(),
            cfg.model.cyan()
        )
    } else {
        format!("OpenRouter  model: {}", cfg.model.cyan())
    };
    println!("  {}  Backend: {}", "◈".cyan().bold(), backend_str);

    if dry_run {
        println!(
            "  {}  {}",
            "◈".yellow().bold(),
            "Dry-run active — nothing will execute.".yellow()
        );
    }
    if cfg.history_enabled && !dry_run {
        println!(
            "  {}  {}",
            "◈".cyan().bold(),
            "Shell history: on  (confirmed commands saved to your history file)".dimmed()
        );
    }
    if cfg.context_size > 0 {
        println!(
            "  {}  {}",
            "◈".cyan().bold(),
            format!(
                "Context: {} turns  (follow-up prompts like \"now do the same for X\" work)",
                cfg.context_size
            )
            .dimmed()
        );
    }

    println!();
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Describe what you want to do — I'll suggest the commands.".white()
    );
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Y to run · N to skip · !help for all shortcuts.".dimmed()
    );
    println!();
}

// =============================================================================
//  print_help
// =============================================================================
pub fn print_help(cfg: &Config, dry_run: bool, history_enabled: bool, ctx_size: usize) {
    println!();
    println!(
        "{}",
        "  ╔══════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "  ║         🤖  Yo, Rust!  —  Help & Reference          ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ╚══════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    // Session status
    println!("  {}", "SESSION".white().bold());
    println!(
        "    {}  {}",
        "Backend:".dimmed(),
        if cfg.backend == "ollama" {
            format!("Ollama  ({})  model: {}", cfg.ollama_url, cfg.model)
        } else {
            format!("OpenRouter  model: {}", cfg.model)
        }
    );
    println!(
        "    {}  {}",
        "Dry-run:".dimmed(),
        if dry_run {
            "yes — nothing will execute"
        } else {
            "no"
        }
    );
    println!(
        "    {}  {}",
        "History:".dimmed(),
        if history_enabled { "on" } else { "off" }
    );
    println!(
        "    {}  {}",
        "Context:".dimmed(),
        if ctx_size > 0 {
            format!("{ctx_size} turns")
        } else {
            "off".to_string()
        }
    );
    println!();

    // Examples
    println!("  {}", "EXAMPLES".white().bold());
    let examples: &[(&str, &str)] = &[
        (
            "find all .env files in this project",
            "find . -name \".env\" -type f",
        ),
        (
            "kill whatever is on port 8080",
            "lsof -ti:8080 | xargs kill -9",
        ),
        (
            "show the 10 biggest files here",
            "du -ah . | sort -rh | head -n 10",
        ),
        (
            "compress the uploads folder",
            "tar -czf uploads.tar.gz uploads/",
        ),
        (
            "git log last 5 commits with author",
            "git log -5 --pretty=format:\"%h %an: %s\"",
        ),
        ("list running docker containers", "docker ps"),
        ("check my public IP",            "curl -s https://ifconfig.me"),
        (
            "count lines of Rust code in this project",
            "find . -name '*.rs' | xargs wc -l | tail -1",
        ),
        (
            "watch nginx error log live",
            "tail -f /var/log/nginx/error.log",
        ),
        (
            "show files changed in the last 24 hours",
            "find . -mtime -1 -type f",
        ),
    ];
    for (prompt, cmd) in examples {
        println!("    {}  {}", "yo ›".cyan().bold(), prompt.white());
        println!("         {}  {}\n", "$".dimmed(), cmd.dimmed());
    }

    // Shortcuts
    println!("  {}", "SHORTCUTS".white().bold());
    let shortcuts: &[(&str, &str)] = &[
        ("!help  / !h",     "This help screen"),
        ("!api",            "Update backend, API key, model, history & context settings"),
        ("!feedback  / !fb",  "Feedback & data sharing — status, setup, on/off"),
        ("!fb setup",          "Run the full feedback setup wizard"),
        ("!fb on / !fb off",   "Toggle community sharing instantly"),
        ("!fb personal",       "Configure your own personal JSONBin"),
        ("!fb about",          "Explain the data pipeline & JSONBin.io"),
        ("!shortcuts / !sc",   "List all saved command shortcuts"),
        ("!save <name>",     "Save last ran commands as !<name> (instant replay)"),
        ("!forget <name>",   "Delete a saved shortcut"),
        ("!<name>",          "Run a saved shortcut instantly — no AI, no confirmation"),
        ("!context / !ctx",  "Show what the AI currently remembers (last N turns)"),
        ("!clear",           "Clear conversation context — start fresh"),
        ("!exit  / !q",     "Quit yo-rust"),
        ("Y / Enter",       "Confirm and run the suggested command(s)"),
        ("N",               "Skip — rephrase and try again"),
        ("↑ / ↓",           "Recall previous prompts in this session"),
        ("Ctrl+D",          "Exit at any time"),
    ];
    for (key, desc) in shortcuts {
        println!(
            "    {}  {}",
            format!("{:<22}", key).yellow().bold(),
            desc.dimmed()
        );
    }
    println!();

    // CLI flags
    println!("  {}", "LAUNCH FLAGS".white().bold());
    println!(
        "    {}  {}",
        "--dry  / -d   ".yellow().bold(),
        "Dry-run: show commands but never execute them"
    );
    println!(
        "    {}  {}",
        "--no-history  ".yellow().bold(),
        "Disable shell history appending for this session"
    );
    println!(
        "    {}  {}",
        "--no-context  ".yellow().bold(),
        "Disable multi-turn context for this session"
    );
    println!();

    // Natural-language triggers
    println!("  {}", "NATURAL LANGUAGE CONFIG TRIGGERS".white().bold());
    println!(
        "  {}",
        "  These phrases auto-trigger !api without typing the shortcut:".dimmed()
    );
    for phrase in &[
        "\"change my API key\"",
        "\"switch to a different model\"",
        "\"use ollama\"  /  \"use openrouter\"",
        "\"change backend\"",
    ] {
        println!("    {}  {}", "›".cyan(), phrase.dimmed());
    }
    println!();

    // Config location
    println!("  {}", "CONFIG FILE".white().bold());
    println!("    {}  {}", "macOS:  ".dimmed(), "~/Library/Application Support/yo-rust/config.json".yellow());
    println!("    {}  {}", "Linux:  ".dimmed(), "~/.config/yo-rust/config.json".yellow());
    println!("    {}  {}", "Windows:".dimmed(), "%APPDATA%\\yo-rust\\config.json".yellow());
    println!("    {}", "Plain JSON — editable manually if needed.".dimmed());
    println!();

    // Footer
    println!(
        "  {}  {}  {}  github.com/paulfxyz/yo-rust",
        "◈".cyan(),
        VERSION.dimmed(),
        "·".dimmed()
    );
    println!();
}

// =============================================================================
//  print_suggestion
// =============================================================================
pub fn print_suggestion(suggestion: &Suggestion, dry_run: bool) {
    println!();

    if let Some(ref expl) = suggestion.explanation {
        println!("  {}  {}", "◈".cyan().bold(), expl.white());
        println!();
    }

    // Box width adapts to the longest command (minimum 46 chars inner width)
    let inner_w = suggestion
        .commands
        .iter()
        .map(|c| c.len() + 7)
        .max()
        .unwrap_or(46)
        .max(46);

    let bar = "─".repeat(inner_w);

    if dry_run {
        println!("  {}{}{}", "┌".yellow(), bar.yellow(), "┐".yellow());
        for cmd in &suggestion.commands {
            let pad = inner_w.saturating_sub(cmd.len() + 5);
            println!(
                "  {}  {}  {}{}{}",
                "│".yellow(),
                "$".dimmed(),
                cmd.white().bold(),
                " ".repeat(pad),
                "│".yellow()
            );
        }
        println!("  {}{}{}", "└".yellow(), bar.yellow(), "┘".yellow());
        println!("  {}", "[dry-run — not executed]".yellow().dimmed());
    } else {
        println!("  {}{}{}", "┌".cyan(), bar.cyan(), "┐".cyan());
        for cmd in &suggestion.commands {
            let pad = inner_w.saturating_sub(cmd.len() + 5);
            println!(
                "  {}  {}  {}{}{}",
                "│".cyan(),
                "$".dimmed(),
                cmd.white().bold(),
                " ".repeat(pad),
                "│".cyan()
            );
        }
        println!("  {}{}{}", "└".cyan(), bar.cyan(), "┘".cyan());
    }

    println!();
}

// =============================================================================
//  print_empty_suggestion
// =============================================================================
pub fn print_empty_suggestion(suggestion: &Suggestion) {
    println!();
    println!(
        "  {}  {}",
        "⚠".yellow(),
        "No commands were suggested — try rephrasing.".yellow()
    );
    if let Some(ref expl) = suggestion.explanation {
        println!("  {}  {}", "◈".cyan(), expl.dimmed());
    }
    println!();
}

// =============================================================================
//  print_context_summary — show what the AI currently "remembers"
// =============================================================================
pub fn print_context_summary(ctx: &ConversationContext) {
    println!();
    if ctx.is_empty() {
        println!(
            "{}",
            "  ◈  No context recorded yet — run some commands first.".dimmed()
        );
        println!();
        return;
    }
    println!(
        "{}",
        "  ╔══════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "  ║         Current Conversation Context                ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ╚══════════════════════════════════════════════════════╝".cyan()
    );
    println!();
    for (i, turn) in ctx.turns().iter().enumerate() {
        println!(
            "  {}  {}",
            format!("[{}]", i + 1).cyan().bold(),
            turn.prompt.white()
        );
        println!("       {}  {}", "$".dimmed(), turn.commands_summary.dimmed());
        println!();
    }
    println!(
        "  {}  {}",
        "◈".cyan(),
        format!(
            "{} turn(s) in context.  Type !clear to reset.",
            ctx.len()
        )
        .dimmed()
    );
    println!();
}

// =============================================================================
//  print_feedback_status — show current telemetry configuration at a glance
// =============================================================================
pub fn print_feedback_status(cfg: &crate::config::Config) {
    println!();
    println!("{}", "  ╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "  ║         📊  Feedback & Data Sharing Status          ║".cyan().bold());
    println!("{}", "  ╚══════════════════════════════════════════════════════╝".cyan());
    println!();

    // ── Community sharing ─────────────────────────────────────────────────────
    let community_status = if cfg.telemetry_share_central {
        "ON  — sharing with yo-rust community dataset".green().to_string()
    } else {
        "OFF — not sharing".dimmed().to_string()
    };
    println!("  {}  Community sharing:   {}", "◈".cyan().bold(), community_status);

    // ── Personal JSONBin ──────────────────────────────────────────────────────
    let personal_status = if !cfg.telemetry_user_key.is_empty() {
        format!("ON  — collection: {}",
            if cfg.telemetry_user_collection.is_empty() {
                "not set (run !feedback to configure)".to_string()
            } else {
                cfg.telemetry_user_collection.clone()
            }
        ).green().to_string()
    } else {
        "OFF — no personal JSONBin configured".dimmed().to_string()
    };
    println!("  {}  Personal JSONBin:    {}", "◈".cyan().bold(), personal_status);

    println!();
    println!("  {}", "WHAT IS COLLECTED  (only when sharing is ON)".white().bold());
    println!("  {}", "  ✓  Your natural-language prompt".dimmed());
    println!("  {}", "  ✓  The commands that ran".dimmed());
    println!("  {}", "  ✓  OS, shell, AI model, yo-rust version".dimmed());
    println!("  {}", "  ✓  Whether it worked (your Y/N feedback)".dimmed());
    println!();
    println!("  {}", "WHAT IS NEVER COLLECTED".white().bold());
    println!("  {}", "  ✗  API keys (never, ever)".dimmed());
    println!("  {}", "  ✗  File paths or contents".dimmed());
    println!("  {}", "  ✗  Your working directory".dimmed());
    println!("  {}", "  ✗  Any command output".dimmed());
    println!("  {}", "  ✗  Username, hostname, or any identity".dimmed());
    println!();
    println!("  {}", "ACTIONS".white().bold());
    println!("    {}  Run the full setup wizard", "!feedback setup".yellow().bold());
    println!("    {}  Toggle community sharing on/off", "!feedback on  /  !feedback off".yellow().bold());
    println!("    {}  Configure your personal JSONBin", "!feedback personal".yellow().bold());
    println!("    {}  Clear all telemetry settings", "!feedback clear".yellow().bold());
    println!("    {}  Learn about JSONBin.io", "!feedback about".yellow().bold());
    println!();
    println!("  {}  {}  {}  https://jsonbin.io",
        "◈".cyan(), "Personal JSONBin →".dimmed(), "·".dimmed());
    println!();
}

// =============================================================================
//  print_feedback_about — explain JSONBin and the data pipeline
// =============================================================================
pub fn print_feedback_about() {
    println!();
    println!("{}", "  ╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "  ║     📊  About Community Feedback & JSONBin.io       ║".cyan().bold());
    println!("{}", "  ╚══════════════════════════════════════════════════════╝".cyan());
    println!();
    println!("  {}", "THE GOAL".white().bold());
    println!("  {}", "  Every week, Paul Fleury reviews the accumulated data to see which".dimmed());
    println!("  {}", "  prompts worked, which failed, and which OS/shell combinations need".dimmed());
    println!("  {}", "  better system prompt rules. This directly improves yo-rust for everyone.".dimmed());
    println!();
    println!("  {}", "HOW THE DATA FLOWS".white().bold());
    println!("  {}", "  1. You confirm a command worked (Y at the feedback prompt)".dimmed());
    println!("  {}", "  2. yo-rust POSTs an anonymised JSON entry to JSONBin.io".dimmed());
    println!("  {}", "  3. It lands in a private collection only Paul can read".dimmed());
    println!("  {}", "  4. Paul reviews weekly → improves the system prompt → new release".dimmed());
    println!();
    println!("  {}", "JSONBIN.IO".white().bold());
    println!("  {}", "  JSONBin.io is a simple JSON storage API. yo-rust uses a write-only".dimmed());
    println!("  {}", "  Access Key — it can create bins but CANNOT read, update, or delete.".dimmed());
    println!("  {}", "  Your entries are private and cannot be seen by other users.".dimmed());
    println!();
    println!("  {}", "YOUR OWN JSONBIN".white().bold());
    println!("  {}", "  You can optionally store your own command history in your own".dimmed());
    println!("  {}", "  JSONBin account — completely separate from the community dataset.".dimmed());
    println!("  {}", "  Only you can read it. Useful for personal analytics.".dimmed());
    println!();
    println!("    {}  https://jsonbin.io  (free account, 10,000 requests)", "→".cyan());
    println!("    {}  Run:  !feedback personal  to configure your own bin", "→".cyan());
    println!();
}
