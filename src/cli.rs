// =============================================================================
//  cli.rs — Command-line argument parsing
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  yo-rust is primarily an interactive REPL, but accepts a small set of flags
//  that modify its behaviour for the current session.  All flags are optional;
//  the default behaviour (no flags) is identical to v1.x.
//
//  FLAGS
//  ─────
//  --dry / -d
//    Dry-run mode: show the suggested commands but NEVER execute them.
//    Useful for previewing what yo-rust would do, or for piping output to
//    another tool without any side effects.
//    The Y/N prompt is replaced with "[dry-run — not executed]".
//
//  --no-history
//    Disable shell history appending for this session only.
//    By default, confirmed commands are appended to the user's shell history
//    file (~/.zsh_history or ~/.bash_history) so they appear in ↑ recall.
//    Use this flag when running sensitive commands you don't want logged.
//
//  --no-context
//    Disable multi-turn context for this session only.
//    By default, yo-rust remembers the last N (config.context_size) commands
//    and includes them in each new request, enabling follow-up prompts like
//    "now do the same for the other folder".
//    Use this flag for a stateless session.
//
//  IMPLEMENTATION NOTE
//  ───────────────────
//  We use clap's derive API rather than the builder API.  The derive approach:
//    • Generates the Args struct from attribute annotations at compile time
//    • Automatically produces --help output matching our fields
//    • Handles short (-d) and long (--dry) flags transparently
//  The struct is small (3 bool fields) so compile overhead is negligible.
// =============================================================================

use clap::Parser;

/// yo-rust — Natural language terminal assistant.
///
/// Type `yo` and describe what you want to do in plain English.
/// yo-rust will suggest shell commands and ask for confirmation before running anything.
#[derive(Parser, Debug)]
#[command(
    name    = "yo",
    version = "2.0.0",
    about   = "Natural language terminal assistant powered by AI.",
    long_about = None,
)]
pub struct Args {
    /// Dry-run mode: show suggested commands but never execute them.
    ///
    /// All other behaviour is identical to normal mode — the AI is still
    /// called, commands are displayed with their explanation, but instead of
    /// a Y/N prompt the output is labelled "[dry-run]".
    #[arg(short = 'd', long = "dry", default_value_t = false)]
    pub dry_run: bool,

    /// Disable shell history appending for this session.
    ///
    /// Confirmed commands are normally appended to ~/.zsh_history or
    /// ~/.bash_history so they appear when you press ↑ in your shell.
    /// Pass this flag to skip that for the current session.
    #[arg(long = "no-history", default_value_t = false)]
    pub no_history: bool,

    /// Disable multi-turn context for this session.
    ///
    /// By default, the last N confirmed commands are included in each new
    /// AI request, enabling follow-up prompts ("now do the same for X").
    /// Pass this flag for a fully stateless session.
    #[arg(long = "no-context", default_value_t = false)]
    pub no_context: bool,
}
