// =============================================================================
//  main.rs — yo-rust entry point
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  yo-rust is a natural-language terminal assistant.  The user types a plain-
//  English description of what they want to do; the program asks an LLM
//  (via OpenRouter) to translate that into one or more shell commands, shows
//  them with an explanation, and waits for explicit Y/N confirmation before
//  running anything.
//
//  This file owns:
//    • Program entry point and top-level flow
//    • The interactive REPL loop (Read → Evaluate → Print → Loop)
//    • Built-in shortcut dispatch (!help, !api, !exit)
//    • Natural-language intent detection for reconfiguration
//    • Shell command execution with inherited stdio
//
//  MODULE GRAPH
//  ────────────
//  main  ──uses──►  ui      (ASCII art, help text, suggestion display)
//        ──uses──►  config  (load/save ~/.config/yo-rust/config.json)
//        ──uses──►  ai      (OpenRouter API call, intent detection)
//
//  DESIGN PHILOSOPHY
//  ─────────────────
//  • Safety first: nothing executes without an explicit "Y" from the user.
//  • Zero magic: every decision (parse, detect, run) is visible in this file.
//  • Minimal state: the only mutable state is the Config struct and the
//    rustyline history buffer — both live on the stack of main().
//  • No async: a single blocking HTTP call per prompt is not a bottleneck.
//    Adding an async runtime (tokio) would triple compile time for zero gain.
// =============================================================================

// ── Declare sub-modules (each maps to src/<name>.rs) ─────────────────────────
mod ai;
mod config;
mod ui;

use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::process;

fn main() {
    // ── 1. Welcome banner ─────────────────────────────────────────────────────
    // Printed before anything else so the user sees the UI immediately,
    // even while config is loading from disk.
    ui::print_banner();

    // ── 2. Load configuration ─────────────────────────────────────────────────
    // config::load() reads ~/.config/yo-rust/config.json.
    // If the file doesn't exist yet (first run) it returns Config::default(),
    // which has empty api_key and model fields — we detect that below.
    // A hard error here (e.g. corrupted JSON) is unrecoverable, so we exit.
    let mut cfg = match config::load() {
        Ok(c)  => c,
        Err(e) => {
            // Use eprintln! for errors so they go to stderr, not stdout.
            // This matters if the user is piping stdout elsewhere.
            eprintln!("{}", format!("  ✗  Could not read config: {e}").red());
            process::exit(1);
        }
    };

    // ── 3. First-run setup ────────────────────────────────────────────────────
    // An empty api_key is the sentinel for "never configured".
    // We prompt once, save to disk, and never ask again unless the user
    // explicitly requests it via !api or a natural-language trigger.
    if cfg.api_key.is_empty() {
        println!(
            "\n{}",
            "  ◈  First run detected — let's get you set up.".yellow().bold()
        );
        // interactive_setup() fills cfg.api_key and cfg.model in-place.
        config::interactive_setup(&mut cfg);
        // Persist immediately so a Ctrl-C after setup doesn't lose the key.
        if let Err(e) = config::save(&cfg) {
            eprintln!("{}", format!("  ✗  Could not save config: {e}").red());
            // Non-fatal: the session can continue even if the write fails
            // (e.g. read-only filesystem), but warn the user.
        }
    }

    // ── 4. Brief usage hint ───────────────────────────────────────────────────
    ui::print_intro();

    // ── 5. Initialise the line editor ─────────────────────────────────────────
    // DefaultEditor provides:
    //   • Line editing (Ctrl-A/E, Ctrl-W, arrow keys)
    //   • In-session history (↑/↓ to recall previous prompts)
    //
    // We do NOT persist history to disk because user prompts may contain
    // sensitive paths or intentions the user doesn't want logged.
    // If a future version adds history persistence, it should go here via
    // `rl.load_history(path)` after this line.
    let mut rl = DefaultEditor::new().unwrap_or_else(|e| {
        eprintln!("{}", format!("  ✗  Readline init failed: {e}").red());
        process::exit(1);
    });

    // ── 6. Main REPL loop ─────────────────────────────────────────────────────
    // This loop runs until the user presses Ctrl-D (EOF), Ctrl-C, or types
    // !exit.  Each iteration is one "turn": read → maybe call AI → confirm →
    // maybe execute.
    loop {
        // ── 6a. Read ──────────────────────────────────────────────────────────
        // rl.readline() blocks until the user presses Enter.
        // The prompt string is printed by rustyline before each input.
        // We format it with colour here; rustyline handles cursor placement.
        let line = match rl.readline(&format!("{} ", "  yo ›".cyan().bold())) {
            Ok(l) => {
                let trimmed = l.trim().to_string();
                // Add to in-session history so ↑ recalls previous prompts.
                // Ignore errors (e.g. history disabled) — not worth surfacing.
                if !trimmed.is_empty() {
                    let _ = rl.add_history_entry(&trimmed);
                }
                trimmed
            }

            // Ctrl-D sends EOF — standard Unix "I'm done" signal.
            // This is the idiomatic way to exit a REPL.
            Err(ReadlineError::Eof) => {
                println!("\n{}", "  Later. ✌".dimmed());
                break;
            }

            // Ctrl-C sends SIGINT.  We treat it as a clean exit rather than
            // propagating a signal (which would skip cleanup code).
            Err(ReadlineError::Interrupted) => {
                println!("\n{}", "  Interrupted. Later. ✌".dimmed());
                break;
            }

            // Any other readline error (terminal lost, resized badly, etc.)
            // is unrecoverable — exit the loop.
            Err(e) => {
                eprintln!("{}", format!("  ✗  Input error: {e}").red());
                break;
            }
        };

        // Skip blank lines without printing anything — just re-show the prompt.
        if line.is_empty() {
            continue;
        }

        // ── 6b. Dispatch built-in shortcuts ───────────────────────────────────
        // These are checked BEFORE the AI intent detector and BEFORE any network
        // call, so they are always instant regardless of API latency.
        //
        // Pattern: match on a &str slice of the owned String.
        // Using `as_str()` is idiomatic; `&line[..]` also works but is noisier.
        match line.as_str() {
            // Show the full help screen with examples and keyboard reference.
            "!help" | "!h" => {
                ui::print_help();
                continue; // Back to the top of the REPL loop — re-show prompt.
            }

            // Reconfigure API key and model interactively.
            // We pass `&mut cfg` so interactive_setup can overwrite both fields.
            "!api" => {
                config::interactive_setup(&mut cfg);
                if let Err(e) = config::save(&cfg) {
                    eprintln!("{}", format!("  ✗  Could not save config: {e}").red());
                }
                println!("{}", "  ✔  API key & model updated.".green());
                continue;
            }

            // Clean exit — same effect as Ctrl-D.
            "!exit" | "!quit" | "!q" => {
                println!("{}", "  Later. ✌".dimmed());
                break;
            }

            // Not a shortcut — fall through to AI processing below.
            _ => {}
        }

        // ── 6c. Natural-language intent detection ─────────────────────────────
        // Before spending a network round-trip on a user message that is really
        // "please update my settings", we check for known phrases using a fast
        // regex scan.  See ai::intent_is_api_change for the pattern list and
        // rationale for why this is done client-side rather than asking the LLM.
        if ai::intent_is_api_change(&line) {
            println!(
                "{}",
                "  ◈  Sounds like you want to update your API config.".yellow()
            );
            config::interactive_setup(&mut cfg);
            if let Err(e) = config::save(&cfg) {
                eprintln!("{}", format!("  ✗  Could not save config: {e}").red());
            }
            println!("{}", "  ✔  API key & model updated.".green());
            continue;
        }

        // ── 6d. AI request ────────────────────────────────────────────────────
        // This is the only network I/O in the entire program.
        // ai::suggest_commands() blocks (no async) while the HTTP request
        // is in flight.  On a typical broadband connection this takes 0.5–3 s
        // depending on the model.  The "Thinking…" indicator is shown first
        // so the user knows the program is alive.
        println!("{}", "  ◌  Thinking…".dimmed());

        match ai::suggest_commands(&cfg, &line) {
            // ── Network/parse error ───────────────────────────────────────────
            Err(e) => {
                // Print to stderr and loop back — the session continues.
                // Common causes: wrong API key (401), no internet, model
                // overloaded (503), malformed JSON response from the model.
                eprintln!("{}", format!("  ✗  AI request failed: {e}").red());
            }

            // ── Successful suggestion ─────────────────────────────────────────
            Ok(suggestion) => {
                // Render the command block with a decorative box and explanation.
                ui::print_suggestion(&suggestion);

                // ── 6e. Y / N confirmation loop ───────────────────────────────
                // We loop here (rather than returning to the outer loop) so that
                // an invalid answer like "maybe" just re-asks, without losing
                // the displayed suggestion.
                loop {
                    let answer = match rl.readline(
                        &format!("{} ", "  Run it? [Y/n] ›".yellow().bold()),
                    ) {
                        Ok(a)  => a.trim().to_lowercase(),
                        // Ctrl-D at the confirmation prompt = "no" (safe default)
                        Err(_) => String::from("n"),
                    };

                    match answer.as_str() {
                        // YES — execute every command in the suggestion in order.
                        // Empty string (bare Enter) is treated as Y — ergonomic
                        // because pressing Enter is the natural "yes" motion.
                        "y" | "yes" | "" => {
                            execute_commands(&suggestion.commands);
                            break; // Return to outer REPL loop for next prompt.
                        }

                        // NO — skip execution, return to outer loop.
                        // The user can rephrase their original request.
                        "n" | "no" => {
                            println!(
                                "{}",
                                "  ◈  No worries — adjust your prompt and try again."
                                    .dimmed()
                            );
                            break;
                        }

                        // Anything else: nudge the user and re-ask.
                        _ => {
                            println!("{}", "  Please type Y (yes) or N (no).".yellow());
                        }
                    }
                }
            }
        }
    }
    // ── End of REPL loop ──────────────────────────────────────────────────────
    // Rust automatically drops all heap allocations here (cfg, rl, etc.).
    // No explicit cleanup is needed — the OS reclaims resources on process exit.
}

// =============================================================================
//  execute_commands
//  ────────────────
//  Runs each command in `commands` sequentially by spawning a `sh -c` child
//  process.  Commands are run one at a time — if one fails, the rest still run
//  (the caller decides whether to surface individual failures).
//
//  WHY `sh -c` INSTEAD OF PARSING THE COMMAND OURSELVES?
//  ──────────────────────────────────────────────────────
//  LLM-generated commands often use shell features that are interpreted by the
//  shell, not by the OS exec() call:
//    • Pipelines:         ls -la | grep ".rs" | wc -l
//    • Redirections:      echo "hello" > file.txt
//    • Globbing:          rm *.tmp
//    • Env expansion:     cd $HOME
//    • Command chaining:  git add . && git commit -m "x"
//    • Subshells:         $(git rev-parse HEAD)
//
//  If we used Command::new("ls").args(&["-la"]) we would need to parse and
//  handle all of these ourselves — essentially reimplementing a shell.
//  Delegating to `sh -c` is safe, correct, and keeps the code tiny.
//
//  WHY INHERITED STDIO?
//  ────────────────────
//  Using Stdio::inherit() means the child process gets the same stdin/stdout/
//  stderr as our process — the terminal's file descriptors directly.  This is
//  essential for:
//    • Interactive programs: vim, htop, less, fzf, nano — they need a real TTY
//      and will break (or show nothing) if stdout is a pipe.
//    • Streaming output: long-running commands like `cargo build` or `npm install`
//      print progress in real time.  Capturing and replaying would buffer it.
//    • Colour output: many programs (grep, ls, cargo) disable colour when they
//      detect a non-TTY stdout.  Inheriting preserves colour.
//
//  The trade-off: we cannot capture stdout to add our own formatting around it.
//  We print "► <cmd>" before and "✔ Done." / "✗ exit N" after.  That's enough.
// =============================================================================
fn execute_commands(commands: &[String]) {
    for cmd in commands {
        // Echo the command being run so the user can see exactly what executed.
        // Use white+bold to distinguish it from normal output.
        println!("\n{}  {}", "  ►".green().bold(), cmd.white().bold());

        // Spawn `sh -c "<cmd>"`.
        // `.status()` blocks until the child process exits and returns its
        // exit status.  We do NOT use `.spawn()` because we want sequential
        // execution with inherited stdio — spawn() would require manual wait().
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();

        match status {
            // Exit code 0 → conventional success.
            Ok(s) if s.success() => {
                println!("{}", "  ✔  Done.".green());
            }
            // Non-zero exit code → the command reported failure.
            // We report but do NOT abort — let subsequent commands run.
            // This mirrors shell behaviour: `set -e` would abort on failure,
            // but the default is to continue.  A future version could add a
            // --stop-on-error flag.
            Ok(s) => {
                eprintln!(
                    "{}",
                    format!("  ✗  Command exited with status {s}").red()
                );
            }
            // OS-level error: couldn't spawn `sh` at all.  This is unusual
            // (it would mean `sh` isn't in PATH, or we hit an OS resource limit).
            Err(e) => {
                eprintln!("{}", format!("  ✗  Failed to run command: {e}").red());
            }
        }
    }
}
