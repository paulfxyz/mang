// =============================================================================
//  main.rs — yo-rust v2.0.0 entry point
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  yo-rust is a natural-language terminal assistant.  The user describes what
//  they want to do; an AI translates it to shell commands; yo-rust shows them,
//  asks for confirmation, runs them, appends them to shell history, and
//  remembers the last N turns for follow-up context.
//
//  NEW IN v2.0.0
//  ─────────────
//  • --dry / -d flag        Dry-run mode: suggest but never execute
//  • Multi-turn context     Follow-up prompts ("now do the same for X") work
//  • Shell history          Confirmed commands go to ~/.zsh_history / ~/.bash_history
//  • Ollama backend         Local AI inference, no API key, no network
//  • Post-execution feedback  Ask "did that work?" and refine if not
//  • Windows support        cmd.exe execution path; PowerShell-aware context
//
//  MODULE GRAPH
//  ────────────
//  main ──► cli      (clap argument parsing: --dry, --no-history, --no-context)
//       ──► config   (load/save ~/.config/yo-rust/config.json)
//       ──► ai       (OpenRouter + Ollama HTTP calls, intent detection)
//       ──► context  (rolling window of prior turns for follow-up support)
//       ──► history  (append confirmed commands to shell history file)
//       ──► ui       (ASCII banner, help, suggestion display, prompts)
//
//  EXECUTION FLOW (one REPL turn)
//  ───────────────────────────────
//  1. Read prompt from user via rustyline
//  2. Check for built-in shortcuts (!help, !api, !exit)
//  3. Check for natural-language config intent (regex, no API call)
//  4. Call AI backend → receive Suggestion { commands, explanation }
//  5. Display suggestion (or [DRY RUN] if --dry)
//  6. Y/N confirmation (skipped in dry-run)
//  7. Execute commands via OS shell (sh -c on Unix, cmd /C on Windows)
//  8. Post-execution feedback: "Did that work? [Y/n]"
//     → Y: record turn in context, append to shell history, next prompt
//     → N: loop back with "What went wrong?" for a refined suggestion
//  9. Record confirmed turn in ConversationContext for follow-up support
// 10. Append to shell history file
//
//  DESIGN PRINCIPLES
//  ─────────────────
//  • Safety: nothing executes without explicit Y confirmation
//  • Transparency: every decision is visible in this file, no hidden behaviour
//  • Fail-soft: history/context failures are warnings, not fatal errors
//  • No async: one blocking HTTP call per turn — tokio adds ~200 KB and 30 s
//    compile time for zero benefit at this call frequency
// =============================================================================

mod ai;
mod cli;
mod config;
mod context;
mod history;
mod shell;
mod ui;

use clap::Parser;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::process;

fn main() {
    // ── 1. Parse command-line arguments ───────────────────────────────────────
    // clap handles --help, --version, and unknown flags automatically.
    let args = cli::Args::parse();

    // ── 2. Print banner ───────────────────────────────────────────────────────
    ui::print_banner(args.dry_run);

    // ── 3. Load configuration ─────────────────────────────────────────────────
    // Returns Config::default() on first run (no file yet).
    // Hard errors (corrupted JSON, unreadable file) exit immediately.
    let mut cfg = match config::load() {
        Ok(c)  => c,
        Err(e) => {
            eprintln!("{}", format!("  ✗  Could not load config: {e}").red());
            process::exit(1);
        }
    };

    // ── 4. First-run setup ────────────────────────────────────────────────────
    // First-run is detected by an empty api_key when backend == "openrouter",
    // or any first launch when backend is unset (empty = default to openrouter).
    // For Ollama, no API key is needed so we only prompt once.
    let needs_setup = cfg.backend.is_empty()
        || (cfg.backend == "openrouter" && cfg.api_key.is_empty());

    if needs_setup {
        println!(
            "\n{}",
            "  ◈  First run — let's get you set up. Takes 30 seconds.".yellow().bold()
        );
        config::interactive_setup(&mut cfg);
        if let Err(e) = config::save(&cfg) {
            eprintln!("{}", format!("  ✗  Could not save config: {e}").red());
        }
    }

    // ── 5. Initialise multi-turn context window ────────────────────────────────
    // Capacity is config.context_size (default 5), or 0 if --no-context passed.
    let ctx_size = if args.no_context { 0 } else { cfg.context_size };
    let mut conversation = context::ConversationContext::new(ctx_size);

    // ── 6. History flag resolution ────────────────────────────────────────────
    // --no-history flag overrides config.history_enabled for this session.
    let history_enabled = cfg.history_enabled && !args.no_history;

    // ── 7. Usage hint ─────────────────────────────────────────────────────────
    ui::print_intro(&cfg, args.dry_run);

    // ── 8. Initialise line editor ─────────────────────────────────────────────
    // Provides arrow key editing, Ctrl-W word delete, in-session ↑/↓ history.
    let mut rl = DefaultEditor::new().unwrap_or_else(|e| {
        eprintln!("{}", format!("  ✗  Readline init failed: {e}").red());
        process::exit(1);
    });

    // ── 9. Main REPL loop ─────────────────────────────────────────────────────
    loop {
        // ── 9a. Read input ────────────────────────────────────────────────────
        let context_indicator = if !conversation.is_empty() {
            format!(" [+{}]", conversation.len())
        } else {
            String::new()
        };

        let prompt_str = format!("{}{} ",
            "  yo ›".cyan().bold(),
            context_indicator.dimmed()
        );

        let line = match rl.readline(&prompt_str) {
            Ok(l) => {
                let t = l.trim().to_string();
                if !t.is_empty() {
                    let _ = rl.add_history_entry(&t);
                }
                t
            }
            Err(ReadlineError::Eof)       => { println!("\n{}", "  Later. ✌".dimmed()); break; }
            Err(ReadlineError::Interrupted) => { println!("\n{}", "  Interrupted. ✌".dimmed()); break; }
            Err(e) => { eprintln!("{}", format!("  ✗  Input error: {e}").red()); break; }
        };

        if line.is_empty() { continue; }

        // ── 9b. Built-in shortcuts ────────────────────────────────────────────
        match line.as_str() {
            "!help" | "!h" => {
                ui::print_help(&cfg, args.dry_run, history_enabled, ctx_size);
                continue;
            }
            "!api" => {
                config::interactive_setup(&mut cfg);
                if let Err(e) = config::save(&cfg) {
                    eprintln!("{}", format!("  ✗  Could not save: {e}").red());
                }
                println!("{}", "  ✔  Config updated.".green());
                continue;
            }
            "!context" | "!ctx" => {
                ui::print_context_summary(&conversation);
                continue;
            }
            "!clear" => {
                // Clear the conversation context for a fresh start
                conversation = context::ConversationContext::new(ctx_size);
                println!("{}", "  ✔  Context cleared.".green());
                continue;
            }
            "!exit" | "!quit" | "!q" => {
                println!("{}", "  Later. ✌".dimmed());
                break;
            }
            _ => {}
        }

        // ── 9c. Natural-language intent detection ─────────────────────────────
        // Detected before any API call — zero latency, zero cost.
        if ai::intent_is_api_change(&line) {
            println!("{}", "  ◈  Sounds like you want to update your config.".yellow());
            config::interactive_setup(&mut cfg);
            if let Err(e) = config::save(&cfg) {
                eprintln!("{}", format!("  ✗  Could not save: {e}").red());
            }
            println!("{}", "  ✔  Config updated.".green());
            continue;
        }

        // ── 9d. AI request ────────────────────────────────────────────────────
        println!("{}", "  ◌  Thinking…".dimmed());

        let suggestion = match ai::suggest_commands(&cfg, &conversation, &line) {
            Err(e)  => {
                eprintln!("{}", format!("  ✗  AI request failed: {e}").red());
                continue;
            }
            Ok(s) => s,
        };

        if suggestion.commands.is_empty() {
            ui::print_empty_suggestion(&suggestion);
            continue;
        }

        ui::print_suggestion(&suggestion, args.dry_run);

        // ── 9e. Dry-run path: no execution, no feedback, record for context ───
        if args.dry_run {
            println!("{}", "  ◈  [dry-run] Commands shown above — nothing was executed.".yellow().bold());
            // Still record in context so follow-ups work even in dry-run mode
            conversation.push(&line, &suggestion.commands);
            continue;
        }

        // ── 9f. Y/N confirmation ──────────────────────────────────────────────
        let confirmed = loop {
            let ans = match rl.readline(&format!("{} ", "  Run it? [Y/n] ›".yellow().bold())) {
                Ok(a)  => a.trim().to_lowercase(),
                Err(_) => "n".to_string(),
            };
            match ans.as_str() {
                "y" | "yes" | "" => break true,
                "n" | "no"       => break false,
                _ => println!("{}", "  Please press Y or N.".yellow()),
            }
        };

        if !confirmed {
            println!("{}", "  ◈  Skipped — rephrase your prompt and try again.".dimmed());
            continue;
        }

        // ── 9g. Execute commands ──────────────────────────────────────────────
        let all_succeeded = execute_commands(&suggestion.commands);

        // ── 9h. Post-execution feedback loop ─────────────────────────────────
        //
        // After running the commands, we ask "Did that work?".
        // This closes the loop: if something went wrong the user can explain
        // what happened and get a refined suggestion without leaving the REPL.
        //
        // Flow:
        //   Y / Enter → success; record context; append to history; next prompt
        //   N         → ask "What went wrong?"; send follow-up to AI; loop back
        //
        // We show the feedback prompt even if a command exited non-zero, because
        // non-zero exit codes are not always failures (e.g. grep returns 1 when
        // no lines match, which is informational not an error).
        let _ = all_succeeded; // used to decide prompt wording below

        let worked = loop {
            let prompt_text = if all_succeeded {
                "  Did that work? [Y/n] ›".green().bold().to_string()
            } else {
                "  Command exited with an error. Did it still do what you wanted? [y/N] ›"
                    .yellow().bold().to_string()
            };

            let ans = match rl.readline(&format!("{} ", prompt_text)) {
                Ok(a)  => a.trim().to_lowercase(),
                Err(_) => "y".to_string(), // Ctrl-D at feedback = "yes, done"
            };

            // For the success path, blank Enter = "yes"
            // For the error path, blank Enter = "no" (safer default)
            let default_yes = all_succeeded;
            match ans.as_str() {
                "y" | "yes"      => break true,
                "n" | "no"       => break false,
                ""               => break default_yes,
                _                => println!("{}", "  Please press Y or N.".yellow()),
            }
        };

        if worked {
            // ── Success path ──────────────────────────────────────────────────
            // Record the turn in context so follow-up prompts work.
            conversation.push(&line, &suggestion.commands);

            // Append to shell history if enabled.
            if history_enabled {
                history::append_to_history(&suggestion.commands);
            }

            println!("{}", "  ◈  Great! What else?".dimmed());
        } else {
            // ── Refinement path ───────────────────────────────────────────────
            // Ask what went wrong, then immediately loop back for another AI call.
            println!("{}", "  ◈  What went wrong? (describe the problem)".yellow());

            let follow_up = match rl.readline(&format!("{} ", "  yo ›".cyan().bold())) {
                Ok(f)  => {
                    let t = f.trim().to_string();
                    if !t.is_empty() { let _ = rl.add_history_entry(&t); }
                    t
                }
                Err(_) => {
                    println!("{}", "  Skipped.".dimmed());
                    continue;
                }
            };

            if follow_up.is_empty() {
                continue;
            }

            // Build a refinement prompt that includes the original request,
            // what ran, and the user's description of the problem.
            let refinement_prompt = format!(
                "I ran: {}\nThe problem was: {}\nPlease suggest a corrected command.",
                suggestion.commands.join(" && "),
                follow_up
            );

            println!("{}", "  ◌  Thinking…".dimmed());

            match ai::suggest_commands(&cfg, &conversation, &refinement_prompt) {
                Err(e) => eprintln!("{}", format!("  ✗  AI request failed: {e}").red()),
                Ok(refined) => {
                    ui::print_suggestion(&refined, false);

                    // Re-prompt for confirmation on the refined suggestion
                    let confirmed2 = loop {
                        let ans = match rl.readline(
                            &format!("{} ", "  Run refined command? [Y/n] ›".yellow().bold()),
                        ) {
                            Ok(a)  => a.trim().to_lowercase(),
                            Err(_) => "n".to_string(),
                        };
                        match ans.as_str() {
                            "y" | "yes" | "" => break true,
                            "n" | "no"       => break false,
                            _ => println!("{}", "  Please press Y or N.".yellow()),
                        }
                    };

                    if confirmed2 {
                        execute_commands(&refined.commands);
                        conversation.push(&refinement_prompt, &refined.commands);
                        if history_enabled {
                            history::append_to_history(&refined.commands);
                        }
                    } else {
                        println!("{}", "  ◈  Skipped.".dimmed());
                    }
                }
            }
        }
    }
    // REPL loop ends — Rust drops all heap allocations automatically.
}

// =============================================================================
//  execute_commands
//  ────────────────
//  Runs each shell command sequentially and returns true if ALL commands
//  exited with status 0 (conventional success).
//
//  SHELL SELECTION — Windows vs Unix
//  ───────────────────────────────────
//  On Unix (macOS, Linux):  sh -c "<cmd>"
//  On Windows:              cmd.exe /C "<cmd>"
//
//  We could use PowerShell on Windows (`powershell -Command "<cmd>"`), but
//  cmd.exe is always present and the AI is instructed to generate cmd.exe
//  syntax when OS=windows.  A future `!pwsh` shortcut could switch to PS.
//
//  WHY sh -c / cmd /C AND NOT DIRECT exec()?
//  ─────────────────────────────────────────
//  LLM-generated commands routinely use shell features that only the shell
//  interprets:
//    Unix:    pipelines (|), redirections (> >>), globs (*.log), chaining (&& ||)
//    Windows: piped commands, environment variable expansion (%VAR%), FOR loops
//
//  Parsing these in Rust would mean reimplementing a shell.  Delegating to
//  the OS shell is correct, safe, and keeps the code to ~10 lines.
//
//  WHY INHERITED STDIO?
//  ────────────────────
//  Stdio::inherit() gives the child process our terminal's file descriptors.
//  This is essential for:
//    • Interactive programs (vim, htop, less) — they need a real TTY
//    • Streaming output (cargo build, npm install) — real-time, not buffered
//    • Colour output — many programs disable colour when stdout is a pipe
// =============================================================================
fn execute_commands(commands: &[String]) -> bool {
    let mut all_ok = true;

    for cmd in commands {
        println!("\n{}  {}", "  ►".green().bold(), cmd.white().bold());

        // Dispatch to the detected shell — see shell.rs for full detection
        // logic covering zsh, bash, fish, sh, PowerShell 5/7, cmd.exe, Git Bash.
        let status = shell::run_in_shell(cmd);

        match status {
            Ok(s) if s.success() => {
                println!("{}", "  ✔  Done.".green());
            }
            Ok(s) => {
                eprintln!("{}", format!("  ✗  Exited with status {s}").red());
                all_ok = false;
            }
            Err(e) => {
                eprintln!("{}", format!("  ✗  Failed to launch shell: {e}").red());
                all_ok = false;
            }
        }
    }

    all_ok
}
