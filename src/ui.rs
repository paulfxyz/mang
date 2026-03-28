// =============================================================================
//  ui.rs — Terminal UI: banner, help, suggestion display, prompts
//  https://mang.sh  |  github.com/paulfxyz/mang
//
//  OVERVIEW
//  ────────
//  All visual output lives here.  No business logic, no I/O beyond stdout.
//  Every public function is self-contained and stateless.
//
//  BRAND
//  ─────
//  mang.sh draws its name and spirit from Gou Mang (句芒) — the ancient
//  Chinese deity who serves as messenger between heaven and earth, between
//  the Emperor's will and the mortal world.  In mang.sh, Gou Mang bridges
//  human intent and machine syntax.  You speak; he translates; the shell listens.
//
//  The banner renders Gou Mang's tree motif (his sacred cosmic tree that
//  connects the celestial and terrestrial realms) alongside the MANG.SH
//  block-letter logotype.  The binary is still invoked as `yo` — a casual
//  direct summons.  No ceremony.  The god comes when called.
// =============================================================================

use crate::ai::Suggestion;
use crate::config::Config;
use crate::context::ConversationContext;
use colored::Colorize;

/// Current version — single source of truth for the banner.
/// Synced with Cargo.toml `version` field.
const VERSION: &str = "v3.0.4";

// =============================================================================
//  print_banner
//
//  Clean two-row block-letter design:
//    Row 1 (MANG)  — cyan: the spirit-messenger's name in full
//    Row 2 (.sh)   — bold white: the extension / invocation form
//
//  Header:  句芒  ·  Gou Mang  ·  Spirit Messenger  (Chinese glyphs, name, title)
//  Footer:  version · mang.sh · github
//
//  Colour scheme:
//    Cyan          — Chinese header line, MANG block letters
//    White + bold  — .sh block letters
//    Dimmed        — outer frame, footer metadata
// =============================================================================
pub fn print_banner(dry_run: bool) {
    println!();

    // ── frame + header ────────────────────────────────────────────────────────
    println!("{}", "  ╔═══════════════════════════════════════════════╗".dimmed());
    println!("{}", "  ║                                               ║".dimmed());

    // Chinese characters, romanised name, subtitle — all cyan
    println!(
        "  {}   {}   {}",
        "║".dimmed(),
        "句芒   ·   Gou Mang   ·   Spirit Messenger".cyan(),
        "║".dimmed()
    );

    println!("{}", "  ║                                               ║".dimmed());

    // ── MANG block letters (cyan) ─────────────────────────────────────────────
    println!(
        "  {}   {}        {}",
        "║".dimmed(),
        "███╗   ███╗ █████╗ ███╗  ██╗ ██████╗".cyan(),
        "║".dimmed()
    );
    println!(
        "  {}   {}        {}",
        "║".dimmed(),
        "████╗ ████║██╔══██╗████╗ ██║██╔════╝".cyan(),
        "║".dimmed()
    );
    println!(
        "  {}   {}       {}",
        "║".dimmed(),
        "██╔████╔██║███████║██╔██╗██║██║  ███╗".cyan(),
        "║".dimmed()
    );
    println!(
        "  {}   {}       {}",
        "║".dimmed(),
        "██║╚██╔╝██║██╔══██║██║╚████║██║   ██║".cyan(),
        "║".dimmed()
    );
    println!(
        "  {}   {}       {}",
        "║".dimmed(),
        "██║ ╚═╝ ██║██║  ██║██║ ╚███║╚██████╔╝".cyan(),
        "║".dimmed()
    );
    println!(
        "  {}   {}        {}",
        "║".dimmed(),
        "╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚══╝ ╚═════╝".cyan(),
        "║".dimmed()
    );

    println!("{}", "  ║                                               ║".dimmed());

    // ── .sh block letters (bold white) ───────────────────────────────────────
    println!(
        "  {}   {}                            {}",
        "║".dimmed(),
        "██████╗ ██╗  ██╗".bold(),
        "║".dimmed()
    );
    println!(
        "  {}   {}                            {}",
        "║".dimmed(),
        "██╔════╝██║  ██║".bold(),
        "║".dimmed()
    );
    println!(
        "  {}   {}                            {}",
        "║".dimmed(),
        "███████╗███████║".bold(),
        "║".dimmed()
    );
    println!(
        "  {}   {}                            {}",
        "║".dimmed(),
        "╚════██║██╔══██║".bold(),
        "║".dimmed()
    );
    println!(
        "  {}   {}                            {}",
        "║".dimmed(),
        "███████║██║  ██║".bold(),
        "║".dimmed()
    );
    println!(
        "  {}   {}                            {}",
        "║".dimmed(),
        "╚══════╝╚═╝  ╚═╝".bold(),
        "║".dimmed()
    );

    println!("{}", "  ║                                               ║".dimmed());

    // ── footer ────────────────────────────────────────────────────────────────
    println!(
        "  {}   {}  ·  mang.sh  ·  github.com/paulfxyz  {}",
        "║".dimmed(),
        VERSION.dimmed(),
        "║".dimmed()
    );
    println!("{}", "  ╚═══════════════════════════════════════════════╝".dimmed());

    println!();
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "The spirit messenger between you and your shell.".white()
    );
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Describe what you need. Gou Mang delivers the command.".white()
    );

    if dry_run {
        println!(
            "  {}  {}",
            "◈".yellow().bold(),
            "DRY-RUN MODE — commands will be shown but never executed.".yellow().bold()
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
// =============================================================================
pub fn print_intro(cfg: &Config, dry_run: bool) {
    println!();

    let backend_str = if cfg.backend == "ollama" {
        format!(
            "Ollama  ({}  model: {})",
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
        "Describe what you want to do — Mang will suggest the commands.".white()
    );
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Y to run · N to refine · !prompt to clarify · !help for all shortcuts.".dimmed()
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
        "  ║      句芒  mang.sh  —  Help & Reference             ║"
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
        ("find all .env files in this project",      "find . -name \".env\" -type f"),
        ("kill whatever is on port 8080",            "lsof -ti:8080 | xargs kill -9"),
        ("show the 10 biggest files here",           "du -ah . | sort -rh | head -n 10"),
        ("compress the uploads folder",              "tar -czf uploads.tar.gz uploads/"),
        ("git log last 5 commits with author",       "git log -5 --pretty=format:\"%h %an: %s\""),
        ("list running docker containers",           "docker ps"),
        ("check my public IP",                       "curl -s https://ifconfig.me"),
        ("count lines of Rust code in this project", "find . -name '*.rs' | xargs wc -l | tail -1"),
        ("watch nginx error log live",               "tail -f /var/log/nginx/error.log"),
        ("show files changed in the last 24 hours",  "find . -mtime -1 -type f"),
    ];
    for (prompt, cmd) in examples {
        println!("    {}  {}", "yo ›".cyan().bold(), prompt.white());
        println!("         {}  {}\n", "$".dimmed(), cmd.dimmed());
    }

    // Shortcuts
    println!("  {}", "SHORTCUTS".white().bold());
    let shortcuts: &[(&str, &str)] = &[
        ("!prompt / !p",     "Advanced Prompt Mode — guided questions to clarify vague requests"),
        ("!credits / !cr",   "About mang.sh — author, links, acknowledgements"),
        ("!help  / !h",      "This help screen"),
        ("!update / !check", "Check for a new version and offer to install it"),
        ("!api",             "Update backend, API key, model, history & context"),
        ("!feedback / !fb",  "Telemetry status, opt-in/out, personal JSONBin"),
        ("!shortcuts / !sc", "List all saved command shortcuts"),
        ("!save <name>",     "Save last commands as !<name> for instant replay"),
        ("!forget <name>",   "Remove a saved shortcut"),
        ("!<name>",          "Run a saved shortcut instantly — no AI, no Y/N"),
        ("!context / !ctx",  "Show what Gou Mang currently remembers"),
        ("!clear",           "Clear conversation context — fresh start"),
        ("!exit  / !q",      "Dismiss Mang for now"),
        ("Y / Enter",        "Confirm and run"),
        ("N",                "Refine — describe what to change, Mang adjusts"),
        ("↑ / ↓",            "Recall previous prompts in this session"),
        ("Ctrl+D",           "Exit at any time"),
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
        "    {}  Dry-run: show commands but never execute them",
        "--dry  / -d   ".yellow().bold()
    );
    println!(
        "    {}  Disable shell history appending for this session",
        "--no-history  ".yellow().bold()
    );
    println!(
        "    {}  Disable multi-turn context for this session",
        "--no-context  ".yellow().bold()
    );
    println!();

    // Natural language triggers
    println!("  {}", "NATURAL LANGUAGE TRIGGERS".white().bold());
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
    println!("    {}  {}", "macOS:  ".dimmed(), "~/Library/Application Support/mang/config.json".yellow());
    println!("    {}  {}", "Linux:  ".dimmed(), "~/.config/mang/config.json".yellow());
    println!("    {}  {}", "Windows:".dimmed(), "%APPDATA%\\mang\\config.json".yellow());
    println!("    {}", "Plain JSON — editable manually if needed.".dimmed());
    println!();

    // Footer
    println!(
        "  {}  {}  {}  mang.sh  ·  github.com/paulfxyz/mang",
        "句芒".cyan(),
        VERSION.dimmed(),
        "·".dimmed()
    );
    println!();
}

// =============================================================================
//  print_credits
//
//  Shows author info, project links, and acknowledgements.
//  Triggered by !credits / !cr from the REPL.
//
//  Design notes:
//  • All URLs are hard-coded strings — no config dependency, no network call.
//  • Width matches the help / context summary boxes (56 chars inner).
//  • The Perplexity Computer credit is intentional — this project was built
//    entirely in collaboration with AI pair programming.  The code is real
//    Rust, the architecture decisions are real, but the implementation speed
//    was only possible with AI assistance.  Honesty about that is a feature.
// =============================================================================
pub fn print_credits() {
    println!();
    println!(
        "{}",
        "  ╔══════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "  ║   句芒  mang.sh  —  Credits & About                  ║".cyan().bold()
    );
    println!(
        "{}",
        "  ╚══════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    // ── Author ────────────────────────────────────────────────────────────
    println!("  {}", "AUTHOR".white().bold());
    println!(
        "    {}  {}",
        "Name:    ".dimmed(),
        "Paul Fleury".white().bold()
    );
    println!(
        "    {}  {}",
        "Role:    ".dimmed(),
        "Founder, entrepreneur, developer".white()
    );
    println!(
        "    {}  {}",
        "Base:    ".dimmed(),
        "❤️  Lisbon, Portugal".white()
    );
    println!(
        "    {}  {}",
        "Web:     ".dimmed(),
        "https://paulfleury.com".cyan()
    );
    println!(
        "    {}  {}",
        "Email:   ".dimmed(),
        "hello@paulfleury.com".cyan()
    );
    println!(
        "    {}  {}",
        "GitHub:  ".dimmed(),
        "github.com/paulfxyz".cyan()
    );
    println!(
        "    {}  {}",
        "LinkedIn:".dimmed(),
        "linkedin.com/in/paulfxyz".cyan()
    );
    println!(
        "    {}  {}",
        "Twitter: ".dimmed(),
        "@paulfxyz".cyan()
    );
    println!();

    // ── Project ───────────────────────────────────────────────────────────
    println!("  {}", "PROJECT".white().bold());
    println!(
        "    {}  {}",
        "Name:    ".dimmed(),
        "mang.sh 句芒  (formerly yo-rust)".white().bold()
    );
    println!(
        "    {}  {}",
        "Version: ".dimmed(),
        VERSION.cyan().bold()
    );
    println!(
        "    {}  {}",
        "License: ".dimmed(),
        "MIT".white()
    );
    println!(
        "    {}  {}",
        "Website: ".dimmed(),
        "https://mang.sh".cyan()
    );
    println!(
        "    {}  {}",
        "Source:  ".dimmed(),
        "github.com/paulfxyz/mang".cyan()
    );
    println!(
        "    {}  {}",
        "Install: ".dimmed(),
        "curl -fsSL https://mang.sh/install | bash".yellow()
    );
    println!();

    // ── Built with ─────────────────────────────────────────────────────────
    println!("  {}", "BUILT WITH".white().bold());
    println!(
        "    {}  {}",
        "✓ ".green(),
        "Rust — memory safety, zero runtime, native binary".white()
    );
    println!(
        "    {}  {}",
        "✓ ".green(),
        "OpenRouter — access to GPT-4o, Claude, Llama and 100+ models".white()
    );
    println!(
        "    {}  {}",
        "✓ ".green(),
        "Ollama — local private AI inference, zero network".white()
    );
    println!(
        "    {}  {}",
        "✓ ".green(),
        "Perplexity Computer — AI pair programming throughout".white()
    );
    println!();

    // ── Mythology ────────────────────────────────────────────────────────
    println!("  {}", "THE NAME".white().bold());
    println!(
        "    {}",
        "句芒 (Gōu Mḁng) is an ancient Chinese spirit deity — the divine".dimmed()
    );
    println!(
        "    {}",
        "messenger between the Emperor of Heaven and the mortal world.".dimmed()
    );
    println!(
        "    {}",
        "He carries intent across the boundary between realms.".dimmed()
    );
    println!(
        "    {}",
        "mang.sh does the same: human language → machine command.".dimmed()
    );
    println!();

    // ── Footer ────────────────────────────────────────────────────────────
    println!(
        "  {}  {}  {}  mang.sh  ·  github.com/paulfxyz/mang",
        "句芒".cyan(),
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
                "│".yellow(), "$".dimmed(),
                cmd.white().bold(), " ".repeat(pad), "│".yellow()
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
                "│".cyan(), "$".dimmed(),
                cmd.white().bold(), " ".repeat(pad), "│".cyan()
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
//  print_context_summary
// =============================================================================
pub fn print_context_summary(ctx: &ConversationContext) {
    println!();
    if ctx.is_empty() {
        println!(
            "{}",
            "  ◈  No context yet — Gou Mang is listening from the beginning.".dimmed()
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
        "  ║         句芒  Mang's Current Memory                 ║"
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
            "{} turn(s) in memory.  Type !clear to start fresh.",
            ctx.len()
        )
        .dimmed()
    );
    println!();
}

// =============================================================================
//  print_feedback_status
// =============================================================================
pub fn print_feedback_status(cfg: &crate::config::Config) {
    println!();
    println!(
        "{}",
        "  ╔══════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "  ║      📊  Feedback & Data Sharing — Status           ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ╚══════════════════════════════════════════════════════╝".cyan()
    );
    println!();

    let community_status = if cfg.telemetry_share_central {
        "ON  — sharing with mang.sh community dataset".green().to_string()
    } else {
        "OFF — not sharing".dimmed().to_string()
    };
    println!(
        "  {}  Community sharing:   {}",
        "◈".cyan().bold(),
        community_status
    );

    let personal_status = if !cfg.telemetry_user_key.is_empty() {
        format!(
            "ON  — collection: {}",
            if cfg.telemetry_user_collection.is_empty() {
                "not set (run !feedback personal)".to_string()
            } else {
                cfg.telemetry_user_collection.clone()
            }
        )
        .green()
        .to_string()
    } else {
        "OFF — no personal JSONBin configured".dimmed().to_string()
    };
    println!(
        "  {}  Personal JSONBin:    {}",
        "◈".cyan().bold(),
        personal_status
    );

    println!();
    println!("  {}", "WHAT IS COLLECTED  (only when sharing is ON)".white().bold());
    println!("  {}", "  ✓  Your natural-language prompt".dimmed());
    println!("  {}", "  ✓  The commands that ran".dimmed());
    println!("  {}", "  ✓  OS, shell, AI model, mang.sh version".dimmed());
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
    println!(
        "    {}  Run the full setup wizard",
        "!feedback setup".yellow().bold()
    );
    println!(
        "    {}  Toggle community sharing on/off",
        "!feedback on  /  !feedback off".yellow().bold()
    );
    println!(
        "    {}  Configure your personal JSONBin",
        "!feedback personal".yellow().bold()
    );
    println!(
        "    {}  Send a live test entry and verify receipt",
        "!feedback test".yellow().bold()
    );
    println!(
        "    {}  Clear all telemetry settings",
        "!feedback clear".yellow().bold()
    );
    println!();
    println!(
        "  {}  {}  {}  https://jsonbin.io",
        "◈".cyan(),
        "Personal JSONBin →".dimmed(),
        "·".dimmed()
    );
    println!();
}

// =============================================================================
//  print_feedback_about
// =============================================================================
pub fn print_feedback_about() {
    println!();
    println!(
        "{}",
        "  ╔══════════════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "  ║     📊  About Community Feedback & JSONBin.io       ║"
            .cyan()
            .bold()
    );
    println!(
        "{}",
        "  ╚══════════════════════════════════════════════════════╝".cyan()
    );
    println!();
    println!("  {}", "THE GOAL".white().bold());
    println!(
        "  {}",
        "  Every week, Paul Fleury reviews the accumulated data to see which".dimmed()
    );
    println!(
        "  {}",
        "  prompts worked, which failed, and which OS/shell combinations need".dimmed()
    );
    println!(
        "  {}",
        "  better system prompt rules. This directly improves mang.sh for everyone.".dimmed()
    );
    println!();
    println!("  {}", "HOW THE DATA FLOWS".white().bold());
    println!(
        "  {}",
        "  1. You confirm a command worked (Y at the feedback prompt)".dimmed()
    );
    println!(
        "  {}",
        "  2. mang.sh POSTs an anonymised JSON entry to JSONBin.io".dimmed()
    );
    println!(
        "  {}",
        "  3. It lands in a private collection only Paul can read".dimmed()
    );
    println!(
        "  {}",
        "  4. Paul reviews weekly → improves the system prompt → new release".dimmed()
    );
    println!();
    println!(
        "  {}  {}  {}  https://jsonbin.io",
        "◈".cyan(),
        "Learn more:".dimmed(),
        "·".dimmed()
    );
    println!();
}
