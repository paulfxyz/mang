// =============================================================================
//  ui.rs — Terminal UI: banner, help text, suggestion display
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  This module owns all visual output in yo-rust.  No logic, no I/O beyond
//  writing to stdout — purely presentation.
//
//  Design goals:
//    • Fits in an 80-column terminal (the standard minimum for CLI tools)
//    • ANSI colour via the `colored` crate (respects NO_COLOR env var)
//    • Every function is self-contained — no shared mutable state
//    • Educational: the banner shows version and repo on every launch
//
//  COLOUR STRATEGY
//  ────────────────
//  We use three visual tiers:
//    cyan          — primary brand colour (logo, borders, prompt marker)
//    white / bold  — emphasis (commands, section headers, important text)
//    dimmed        — secondary info (hints, descriptions, footer text)
//    yellow        — interactive prompts and warnings
//    green         — success states
//    red           — errors (handled in main.rs and config.rs)
//
//  WHY THE `colored` CRATE OVER RAW ANSI ESCAPE CODES?
//  ─────────────────────────────────────────────────────
//  Raw ANSI codes (\x1b[36m etc.) work fine but are:
//    • Fragile: easy to forget the reset code and leak colour to the next line
//    • Opaque: "\x1b[1;36m" is harder to read than ".cyan().bold()"
//    • Non-portable: Windows requires additional setup for ANSI in older consoles
//  The `colored` crate handles all of this and also checks the NO_COLOR
//  environment variable (standardised at no-color.org) so power users can
//  disable colour output cleanly.
//
//  BANNER DESIGN NOTES
//  ────────────────────
//  The banner uses a split-panel layout:
//    LEFT  — ASCII robot body (antenna, eyes, arms, mouth, chest panel, legs)
//    RIGHT — Block-letter "YO," logo + block-letter "RUST!" logo
//
//  The robot is rendered in cyan (brand) and the "YO," text matches cyan.
//  "RUST!" is white+bold to create a two-tone effect that reads clearly even
//  on monochrome terminals.
//
//  Each line is stored as a (&str, u8) tuple where u8 is a colour code:
//    0 → cyan       (robot structure + "YO," logo)
//    1 → white+bold ("RUST!" logo)
//    2 → cyan+dimmed (frame, version, footer)
//
//  This approach avoids per-character colour calls — the entire line is
//  coloured as one unit.  This is a performance non-issue (it's a few dozen
//  println! calls at startup) but keeps the code readable.
// =============================================================================

use crate::ai::Suggestion;
use colored::Colorize;

/// Current application version.  Defined here as a const so it appears in the
/// banner at runtime and is easy to update in one place.
///
/// IMPORTANT: keep this in sync with Cargo.toml `version` field.
/// A future improvement: use env!("CARGO_PKG_VERSION") to read it at compile
/// time from Cargo.toml automatically, eliminating the manual sync requirement.
const VERSION: &str = "v1.1.0";

// =============================================================================
//  print_banner
//
//  The first thing the user sees on every `yo` invocation.
//  Printed unconditionally — even before config is loaded — so the UI appears
//  instantly while any slow I/O happens afterward.
//
//  80-COLUMN FIT
//  ─────────────
//  The outer box is 68 characters wide (2 leading spaces + ╔ + 64 interior + ╗).
//  This fits within 80 columns with room to spare.  Each interior line is padded
//  to exactly 64 characters so the right ║ border always aligns.
// =============================================================================
pub fn print_banner() {
    println!();

    // Each tuple: (line_text, colour_code)
    //   0 = cyan             — robot parts + YO, logo
    //   1 = white + bold     — RUST! logo
    //   2 = cyan + dimmed    — outer frame + footer metadata
    let lines: &[(&str, u8)] = &[
        // ── Outer frame top ───────────────────────────────────────────────────
        ("  ╔══════════════════════════════════════════════════════════════════╗", 2),

        // ── Antenna ───────────────────────────────────────────────────────────
        ("  ║                                                                  ║", 2),
        ("  ║           ╷▲╷             ██╗   ██╗ ██████╗                     ║", 0),
        ("  ║      ┌────┴─┴────┐        ╚██╗ ██╔╝██╔═══██╗                    ║", 0),

        // ── Eyes + top of YO logo ─────────────────────────────────────────────
        ("  ║      │ ╔═══╗╔═══╗│         ╚████╔╝ ██║   ██║                    ║", 0),
        ("  ║      │ ║◈  ◈║◈  ◈║│          ╚██╔╝  ██║   ██║                    ║", 0),
        ("  ║      │ ╚═══╝╚═══╝│           ██║   ╚██████╔╝                    ║", 0),

        // ── Arms + mouth + bottom of YO logo ─────────────────────────────────
        ("  ║ ┌──┐ │ ┌─────────┐ │ ┌──┐    ╚═╝    ╚═════╝                     ║", 0),
        ("  ║ │░░├─┤ │ · · · · │ ├─░░│                                        ║", 0),

        // ── Chest panel + RUST! logo ──────────────────────────────────────────
        ("  ║ └──┘ │ ┌──┬──┬──┐ │ └──┘   ██████╗ ██╗   ██╗███████╗████████╗  ║", 1),
        ("  ║      │ │▓▓│▓▓│▓▓│ │        ██╔══██╗██║   ██║██╔════╝╚══██╔══╝  ║", 1),
        ("  ║      │ └──┴──┴──┘ │        ██████╔╝██║   ██║███████╗   ██║     ║", 1),
        ("  ║      └─────┬─┬────┘        ██╔══██╗██║   ██║╚════██║   ██║     ║", 1),
        ("  ║            │ │             ██║  ██║╚██████╔╝███████║   ██║     ║", 1),
        ("  ║           ┌┘ └┐            ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝     ║", 1),

        // ── Legs ──────────────────────────────────────────────────────────────
        ("  ║          ┌┴─┐┌─┴┐                                                ║", 2),
    ];

    // Print all banner lines except the version/footer row (handled below).
    for (line, color_code) in lines {
        match color_code {
            0 => println!("{}", line.cyan()),
            1 => println!("{}", line.white().bold()),
            _ => println!("{}", line.cyan().dimmed()),
        }
    }

    // ── Version + repo footer — injected dynamically so VERSION const drives it ──
    // We use format! rather than a static string so changing the VERSION const
    // at the top of this file is the only edit needed for a version bump.
    println!("{}", format!("  ║          │░░││░░│           {VERSION}  ·  github.com/paulfxyz       ║").cyan().dimmed());
    println!("{}", "  ║          └──┘└──┘                                                ║".cyan().dimmed());

    // ── Outer frame bottom ────────────────────────────────────────────────────
    println!("{}", "  ╚══════════════════════════════════════════════════════════════════╝".cyan().dimmed());

    println!();

    // ── Taglines ──────────────────────────────────────────────────────────────
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Natural language → Terminal commands, powered by AI.".white()
    );
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Type !help for options.  Type !api to configure.".dimmed()
    );
    println!();
}

// =============================================================================
//  print_intro
//
//  Printed after the banner and after any first-run setup, immediately before
//  the first REPL prompt.  Serves as a quick usage reminder — brief because
//  the banner already drew attention.
// =============================================================================
pub fn print_intro() {
    println!();
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Describe what you want to do — I'll suggest the commands.".white()
    );
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        "Press Y to run, N to refine, Ctrl+D to exit.".dimmed()
    );
    println!();
}

// =============================================================================
//  print_help
//
//  Full help screen — triggered by !help or !h.
//  Intentionally verbose: a user who types !help wants comprehensive information,
//  not a one-liner.  Sections follow a logical reading order:
//    What it does → Examples → Keyboard shortcuts → NL triggers → Config
// =============================================================================
pub fn print_help() {
    println!();
    println!("{}", "  ╔══════════════════════════════════════════════════════╗".cyan());
    println!("{}", "  ║         🤖  Yo, Rust!  —  Help & Reference          ║".cyan().bold());
    println!("{}", "  ╚══════════════════════════════════════════════════════╝".cyan());
    println!();

    // ── What it does ──────────────────────────────────────────────────────────
    println!("  {}", "WHAT IT DOES".white().bold());
    println!("  {}", "  Type a plain-English task. yo-rust asks an AI (via OpenRouter),".dimmed());
    println!("  {}", "  then shows you the exact shell commands — with a brief explanation.".dimmed());
    println!("  {}", "  Press Y to execute, N to refine your prompt and try again.".dimmed());
    println!("  {}", "  Nothing ever runs without your explicit Y.".dimmed());
    println!();

    // ── Examples ──────────────────────────────────────────────────────────────
    // Showing prompt + resulting command pairs teaches the user what kinds of
    // requests work well — more valuable than abstract descriptions.
    println!("  {}", "EXAMPLES".white().bold());
    let examples: &[(&str, &str)] = &[
        ("find all .env files in this project",                "find . -name \".env\" -type f"),
        ("kill whatever is running on port 8080",              "lsof -ti:8080 | xargs kill -9"),
        ("show the 10 biggest files here",                     "du -ah . | sort -rh | head -n 10"),
        ("compress the uploads folder to tar.gz",              "tar -czf uploads.tar.gz uploads/"),
        ("git log last 5 commits with author",                 "git log -5 --pretty=format:\"%h %an: %s\""),
        ("list all running docker containers",                 "docker ps"),
        ("check my public IP address",                        "curl -s https://ifconfig.me"),
        ("show files changed in the last 24 hours",           "find . -mtime -1 -type f"),
        ("watch the nginx error log live",                     "tail -f /var/log/nginx/error.log"),
        ("count lines of Rust code in this project",          "find . -name '*.rs' | xargs wc -l"),
    ];
    for (prompt, cmd) in examples {
        println!("    {}  {}", "yo ›".cyan().bold(), prompt.white());
        println!("         {}  {}\n", "$".dimmed(), cmd.dimmed());
    }

    // ── Keyboard shortcuts ────────────────────────────────────────────────────
    println!("  {}", "SHORTCUTS".white().bold());
    let shortcuts: &[(&str, &str)] = &[
        ("!help  / !h",  "Show this help screen"),
        ("!api",         "Update your OpenRouter API key and model"),
        ("!exit  / !q",  "Quit yo-rust"),
        ("Y / Enter",    "Accept and run the suggested command(s)"),
        ("N",            "Decline — refine your prompt and try again"),
        ("Ctrl+D",       "Exit yo-rust at any time"),
        ("↑ / ↓",        "Recall previous prompts in this session"),
    ];
    for (key, desc) in shortcuts {
        println!(
            "    {}  {}",
            format!("{:<16}", key).yellow().bold(),
            desc.dimmed()
        );
    }
    println!();

    // ── Natural-language config triggers ──────────────────────────────────────
    println!("  {}", "NATURAL LANGUAGE TRIGGERS".white().bold());
    println!("  {}", "  These phrases auto-trigger !api — detected before any API call:".dimmed());
    for trigger in &[
        "\"change my API key\"",
        "\"update my openrouter key\"",
        "\"switch to a different model\"",
        "\"use a new model\"",
    ] {
        println!("    {}  {}", "›".cyan(), trigger.dimmed());
    }
    println!();

    // ── Config location ───────────────────────────────────────────────────────
    println!("  {}", "CONFIG FILE".white().bold());
    println!("    {}  {}", "macOS:".dimmed(),  "~/Library/Application Support/yo-rust/config.json".yellow());
    println!("    {}  {}", "Linux:".dimmed(),  "~/.config/yo-rust/config.json".yellow());
    println!("    {}", "Plain JSON — editable manually if needed.".dimmed());
    println!();

    // ── Footer ────────────────────────────────────────────────────────────────
    println!("  {}  {}  {}  github.com/paulfxyz/yo-rust",
        "◈".cyan(),
        VERSION.dimmed(),
        "·".dimmed()
    );
    println!();
}

// =============================================================================
//  print_suggestion
//
//  Renders the AI's command suggestion inside a styled box with an explanation
//  line above it.
//
//  BOX SIZING
//  ──────────
//  The box width adapts to the longest command in the list:
//    inner_w = max(longest_command_length + padding, minimum_width)
//  This means short single commands get a compact box and long pipelines get
//  a wide box — always readable, never truncated.
//
//  The minimum width is 44 characters — wide enough to look intentional even
//  for very short commands like `ls` or `pwd`.
//
//  PADDING CALCULATION
//  ────────────────────
//  Each command row looks like:
//    ║  $  <command><padding>║
//  Where:
//    ║ = 1 char (drawn separately)
//    "  $  " = 5 chars of prefix (2 spaces + $ + 2 spaces)
//  So the padding to right-justify the closing ║ is:
//    padding = inner_w - command.len() - 5
//  We use .saturating_sub() to avoid underflow panic if command is longer than
//  inner_w (shouldn't happen given our max() call, but defensive code).
// =============================================================================
pub fn print_suggestion(suggestion: &Suggestion) {
    println!();

    // ── Edge case: model returned empty commands array ────────────────────────
    // This happens when the user asks something non-shell-able ("what is docker?")
    // or when the model decides the request is too risky/ambiguous.
    // The explanation field usually contains a helpful message in this case.
    if suggestion.commands.is_empty() {
        println!(
            "  {}  {}",
            "⚠".yellow(),
            "No commands were suggested. Try rephrasing your request.".yellow()
        );
        if let Some(ref expl) = suggestion.explanation {
            println!("  {}  {}", "◈".cyan(), expl.dimmed());
        }
        println!();
        return;
    }

    // ── Explanation line ──────────────────────────────────────────────────────
    // Print above the command box so the user reads the description before
    // deciding whether to confirm.
    if let Some(ref explanation) = suggestion.explanation {
        println!("  {}  {}", "◈".cyan().bold(), explanation.white());
        println!();
    }

    // ── Box width calculation ─────────────────────────────────────────────────
    // command.len() + 5 accounts for the "  $  " prefix inside the box.
    // We add 2 more for right-side padding before the closing ║.
    let inner_w = suggestion
        .commands
        .iter()
        .map(|c| c.len() + 7)  // 5 prefix + 2 right padding
        .max()
        .unwrap_or(46)
        .max(46); // Enforce minimum box width

    let h_bar = "─".repeat(inner_w);

    // ── Top border ────────────────────────────────────────────────────────────
    println!("  {}{}{}", "┌".cyan(), h_bar.cyan(), "┐".cyan());

    // ── Command rows ──────────────────────────────────────────────────────────
    for cmd in &suggestion.commands {
        // Compute right-side padding to keep the ║ border aligned.
        let padding = inner_w.saturating_sub(cmd.len() + 5);
        println!(
            "  {}  {}  {}{}{}",
            "│".cyan(),
            "$".dimmed(),
            cmd.white().bold(),
            " ".repeat(padding),
            "│".cyan()
        );
    }

    // ── Bottom border ─────────────────────────────────────────────────────────
    println!("  {}{}{}", "└".cyan(), h_bar.cyan(), "┘".cyan());
    println!();
}
