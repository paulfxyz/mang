// =============================================================================
//  main.rs — mang.sh v3.0.0 entry point
//  https://mang.sh
//
//  OVERVIEW
//  ────────
//  mang.sh is a natural-language terminal assistant.  The user describes what
//  they want to do; an AI translates it to shell commands; mang.sh shows them,
//  asks for confirmation, runs them, appends them to shell history, and
//  remembers the last N turns for follow-up context.
//
//  NEW IN v3.0.2
//  ─────────────────
//  • Advanced Prompt Mode (!prompt / !p)
//      Short Socratic dialogue (up to 3 rounds) that helps users who are stuck.
//      The AI generates targeted clarifying questions; the user's answers are
//      synthesised into a precise prompt fed to the normal suggest_commands()
//      pipeline.  Triggers manually (!prompt / !p) or automatically when the
//      AI returns an empty commands array.
//
//  NEW IN v2.3.5
//  ─────────────
//  • Update check   Background version check on launch; !update / !check
//  • N = refine     Pressing N on a suggestion opens an iterative refinement
//                   tunnel — user can keep improving until the command is right
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
//       ──► config   (load/save ~/.config/mang/config.json)
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
mod feedback;
mod history;
mod prompt_wizard;
mod safety;
mod shell;
mod shortcuts;
mod telemetry;
mod ui;
mod updater;

use clap::Parser;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::process;
use std::sync::mpsc;
use std::time::Duration;

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

    // ── 5. Background update check ──────────────────────────────────────────────
    // Spawn a thread immediately after startup to check GitHub for a newer version.
    // We use a background thread so startup latency is zero — the check completes
    // while the user is reading the banner and typing their first prompt.
    //
    // Rate-limited to once per 24 hours (stored in ~/.config/mang/last_update_check).
    // The result is collected before the first REPL prompt is shown.
    //
    // v3.0.7: the result travels over a channel and is polled, never joined.
    // Previously `join()` blocked startup for up to the 6 s HTTP timeout
    // whenever GitHub was unreachable (offline, VPN, a firewall prompt such as
    // Little Snitch waiting for an answer) — on every launch.
    let (update_tx, update_rx) = mpsc::channel::<updater::UpdateStatus>();
    std::thread::spawn(move || {
        // A send error just means the REPL already exited; nothing to do.
        let _ = update_tx.send(updater::check_for_update(false));
    });

    // ── 6. Periodic telemetry reminder ─────────────────────────────────────────
    // Every 10 sessions, if telemetry is off and user has never been asked
    // (or dismissed previously), gently remind them about community sharing.
    // We increment the counter here and save it.
    if !cfg.telemetry_share_central && cfg.telemetry_user_key.is_empty() {
        cfg.sessions_since_telemetry_prompt += 1;
        // Show reminder at session 10, 20, 30, ... (every 10 sessions)
        if cfg.sessions_since_telemetry_prompt >= 10 {
            cfg.sessions_since_telemetry_prompt = 0;
            println!(
                "\n  {}  {}",
                "◈".cyan().bold(),
                "Tip: mang.sh can share anonymised command data to help improve the tool.".white()
            );
            println!(
                "  {}  {}",
                "◈".cyan().bold(),
                "Type !api to configure community sharing (fully optional, off by default).".dimmed()
            );
            println!();
        }
        let _ = config::save(&cfg); // save updated counter (non-fatal if fails)
    }

    // ── 7. Load saved command shortcuts ──────────────────────────────────────
    // Loaded once at startup; !save and !forget persist immediately to disk.
    let mut shortcut_store = shortcuts::ShortcutStore::load();

    // ── 8. Pending telemetry thread handles ─────────────────────────────────
    // Background HTTP threads for telemetry submissions.
    // CRITICAL: We store the JoinHandle for every spawned thread and join them
    // all before process exit.  Without this, if the user exits immediately
    // after confirming a command, the process terminates and kills all threads
    // before any HTTP request completes — the collection stays empty.
    let mut pending_telemetry: Vec<std::thread::JoinHandle<()>> = Vec::new();

    // ── 9. Initialise multi-turn context window ────────────────────────────────
    // Capacity is config.context_size (default 5), or 0 if --no-context passed.
    let ctx_size = if args.no_context { 0 } else { cfg.context_size };
    let mut conversation = context::ConversationContext::new(ctx_size);

    // ── 10. History flag resolution ────────────────────────────────────────────
    // --no-history flag overrides config.history_enabled for this session.
    let history_enabled = cfg.history_enabled && !args.no_history;

    // Per-session switches handed to the shared helpers at the bottom of this file.
    let opts = RunOpts { dry_run: args.dry_run, history_enabled };

    // ── 11. Usage hint ─────────────────────────────────────────────────────────
    ui::print_intro(&cfg, args.dry_run);

    // ── 11b. Pick up the update check result ──────────────────────────────────
    // Wait at most 400 ms so a fast check still announces itself before the
    // first prompt; a slow or failed one is simply polled again each turn.
    let mut available_update: Option<String> = None;
    poll_update(&update_rx, Some(Duration::from_millis(400)), &mut available_update);

    // ── 12. Initialise line editor ─────────────────────────────────────────────
    // Provides arrow key editing, Ctrl-W word delete, in-session ↑/↓ history.
    let mut rl = DefaultEditor::new().unwrap_or_else(|e| {
        eprintln!("{}", format!("  ✗  Readline init failed: {e}").red());
        process::exit(1);
    });

    // ── 13. Main REPL loop ─────────────────────────────────────────────────────
    loop {
        // ── 9a. Read input ────────────────────────────────────────────────────
        // Announce a background update result that arrived since the last turn.
        poll_update(&update_rx, None, &mut available_update);

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
            Err(ReadlineError::Eof)       => {
                println!("\n{}", "  Later. ✌".dimmed());
                // Flush pending telemetry before exit
                for h in pending_telemetry { let _ = h.join(); }
                return;
            }
            Err(ReadlineError::Interrupted) => {
                println!("\n{}", "  Interrupted. ✌".dimmed());
                for h in pending_telemetry { let _ = h.join(); }
                return;
            }
            Err(e) => {
                eprintln!("{}", format!("  ✗  Input error: {e}").red());
                for h in pending_telemetry { let _ = h.join(); }
                return;
            }
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
            "!shortcuts" | "!sc" => {
                shortcut_store.print_all();
                continue;
            }
            // !feedback / !fb — telemetry & community data management
            cmd if feedback::parse(cmd).is_some() => {
                if let Some(fb_cmd) = feedback::parse(cmd) {
                    if feedback::dispatch(fb_cmd, &mut cfg) {
                        if let Err(e) = config::save(&cfg) {
                            eprintln!("{}", format!("  ✗  Could not save: {e}").red());
                        }
                    }
                }
                continue;
            }
            // !update / !check — check for and optionally install an update
            "!update" | "!upd" | "!check" => {
                // Use the already-fetched result if we have it; otherwise re-check now.
                let status = if let Some(ref v) = available_update {
                    updater::UpdateStatus::UpdateAvailable(v.clone())
                } else {
                    println!("{}", "  ◌  Checking for updates…".dimmed());
                    updater::check_for_update(true)
                };
                match status {
                    updater::UpdateStatus::UpdateAvailable(v) => {
                        updater::print_update_notice(&v);
                        println!("{}", "  Install this update now?".yellow().bold());
                        let ans = match rl.readline("  [Y/n] › ") {
                            Ok(a) => a.trim().to_lowercase(),
                            Err(_) => "n".to_string(),
                        };
                        if matches!(ans.as_str(), "y" | "yes" | "") {
                            if let Err(e) = updater::run_update() {
                                eprintln!("{}", format!("  ✗  {e}").red());
                            } else {
                                // Update script ran — exit mang.sh so user restarts fresh
                                for h in pending_telemetry { let _ = h.join(); }
                                return;
                            }
                        } else {
                            println!("{}", "  ◈  Skipped — you can update any time with !update.".dimmed());
                        }
                    }
                    updater::UpdateStatus::UpToDate => {
                        println!("{}", "  ✔  Already on the latest version.".green());
                    }
                    updater::UpdateStatus::Unavailable => {
                        println!("{}", "  ◈  Could not reach GitHub — check your connection.".yellow());
                        // Show the manual fallback command so the user is never stuck
                        println!(
                            "  {}  {}",
                            "◈".dimmed(),
                            "Manual update:".dimmed()
                        );
                        println!(
                            "  {}  {}",
                            " ".dimmed(),
                            "curl -fsSL https://mang.sh/update | bash".cyan().bold()
                        );
                        println!(
                            "  {}  {}",
                            " ".dimmed(),
                            "Windows: iwr -useb https://mang.sh/update.ps1 | iex".cyan()
                        );
                    }
                }
                println!();
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
            // ── Credits screen ────────────────────────────────────────────────
            // Shows author info, project links, build stack, and the Gou Mang
            // mythology.  No config needed — pure static display.
            "!credits" | "!cr" => {
                ui::print_credits();
                continue;
            }

            "!exit" | "!quit" | "!q" => {
                println!("{}", "  Later. ✌".dimmed());
                for h in pending_telemetry { let _ = h.join(); }
                return;
            }

            // ── Advanced Prompt Mode (explicit trigger) ───────────────────────
            // User typed !prompt or !p at the yo › prompt.
            // We run the wizard with an empty "original" so it starts fresh and
            // asks a completely open first question.
            "!prompt" | "!p" => {
                if let prompt_wizard::WizardResult::Prompt(refined) =
                    prompt_wizard::run(&mut rl, &cfg, &conversation, "")
                {
                    // Feed the synthesised prompt through the normal pipeline:
                    // AI → show → risk check → Y/N → execute → record.
                    ask_and_run(&mut rl, &cfg, &mut conversation, opts, Ask {
                    request: &refined, record_as: &refined,
                    empty_hint: "Still unclear — try rephrasing or use !prompt again.",
                    label: "Run it?",
                });
                }
                // WizardResult::Abandoned → nothing to do, back to the prompt.
                continue;
            }

            _ => {}
        }

        // ── 9c. Named shortcut dispatch ──────────────────────────────────────────
        // Checked before the AI so shortcuts are always instant (no network call).
        match shortcuts::parse_shortcut_input(&line) {
            shortcuts::ShortcutInput::List => {
                shortcut_store.print_all();
                continue;
            }
            shortcuts::ShortcutInput::Save(name) => {
                // Save the most recently executed command set
                if let Some(last_cmds) = conversation.turns().last().map(|t| t.commands.clone()) {
                    match shortcut_store.save_shortcut(&name, &last_cmds) {
                        Ok(_) => {
                            println!("{}", format!("  ✔  Saved as !{name}  — type !{name} anytime to run instantly.", name=name).green());
                            println!("  {}  {}", "◈".cyan(), "Commands saved:".dimmed());
                            for cmd in &last_cmds {
                                println!("       {}  {}", "$".dimmed(), cmd.white());
                            }
                        }
                        Err(e) => eprintln!("{}", format!("  ✗  {e}").red()),
                    }
                } else {
                    println!("{}", "  ◈  No commands run yet in this session — run something first.".yellow());
                }
                println!();
                continue;
            }
            shortcuts::ShortcutInput::Forget(name) => {
                if shortcut_store.forget(&name) {
                    println!("{}", format!("  ✔  Shortcut !{name} removed.", name=name).green());
                } else {
                    println!("{}", format!("  ◈  No shortcut named !{name} found.", name=name).yellow());
                }
                println!();
                continue;
            }
            shortcuts::ShortcutInput::Run(name) => {
                if let Some(cmds) = shortcut_store.get(&name).cloned() {
                    println!("{}", format!("  ◈  Running shortcut !{name}", name=name).cyan());
                    println!();
                    // Shortcuts skip the AI step, but a saved `rm -rf` is still
                    // an `rm -rf`: risky ones get the same explicit confirmation.
                    if !safety::assess(&cmds).is_empty()
                        && !confirm_run(&mut rl, "Run shortcut?", &cmds)
                    {
                        println!("{}", "  ◈  Skipped.".dimmed());
                        println!();
                        continue;
                    }
                    execute_commands(&cmds);
                    if history_enabled {
                        history::append_to_history(&cmds);
                    }
                    conversation.push(&format!("!{name}", name=name), &cmds);
                } else {
                    println!("{}", format!("  ◈  No shortcut named !{name}. Type !shortcuts to see all.", name=name).yellow());
                    println!("  {}  {}", "◈".cyan().bold(), "Or use !save <name> after running a command to create one.".dimmed());
                }
                println!();
                continue;
            }
            shortcuts::ShortcutInput::NotAShortcut => {}
        }

        // ── 9d. Natural-language intent detection ─────────────────────────────
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
                // Use the smart error printer that gives actionable diagnostics
                // for connection-level failures (firewall, VPN, proxy, etc.)
                ui::print_network_error(&e.to_string());
                continue;
            }
            Ok(s) => s,
        };

        if suggestion.commands.is_empty() {
            ui::print_empty_suggestion(&suggestion);

            // ── Auto-trigger Advanced Prompt Mode ─────────────────────────────
            // The AI couldn't pin down a command from the user's prompt.
            // Rather than returning to a blank prompt and leaving the user
            // stranded, we automatically enter the wizard with the original
            // vague prompt as context so its first question is targeted.
            // The user can opt out immediately by pressing Enter (abandons gracefully).
            println!();
            println!(
                "  {}  {}",
                "◈".yellow().bold(),
                "Let's clarify your request with a few quick questions.".white()
            );

            if let prompt_wizard::WizardResult::Prompt(refined) =
                prompt_wizard::run(&mut rl, &cfg, &conversation, &line)
            {
                ask_and_run(&mut rl, &cfg, &mut conversation, opts, Ask {
                    request: &refined, record_as: &refined,
                    empty_hint: "Still couldn't find a command — try a completely different phrasing.",
                    label: "Run it?",
                });
            }

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
        let confirmed = confirm_run(&mut rl, "Run it?", &suggestion.commands);

        if !confirmed {
            // ── Refinement tunnel ─────────────────────────────────────────────
            // N does not mean "give up" — it means "not quite right".
            // We enter an iterative refinement loop: the user describes what
            // they want changed, we send that to the AI with the original
            // suggestion as context, and show a new suggestion.
            // The loop continues until the user accepts (Y) or explicitly
            // abandons (!skip, blank Enter, or Ctrl-D).
            println!();
            println!("  {}  {}", "◈".yellow().bold(), "Let's refine — what should be different?".white().bold());
            println!("  {}  {}", "◈".cyan(), "(Describe the change, or press Enter / type !skip to cancel)".dimmed());
            println!();

            let refinement = match rl.readline(&format!("{} ", "  yo ›".cyan().bold())) {
                Ok(r) => {
                    let t = r.trim().to_string();
                    if !t.is_empty() { let _ = rl.add_history_entry(&t); }
                    t
                }
                Err(_) => String::new(),
            };

            // Empty input or !skip → truly cancel
            if refinement.is_empty() || refinement == "!skip" {
                println!("{}", "  ◈  Cancelled.".dimmed());
                println!();
                continue;
            }

            // Build a context-aware refinement prompt so the AI knows
            // what the original suggestion was and what to change.
            let refinement_prompt = format!(
                "Original request: {line}\nI suggested: {cmds}\nThe user wants this changed: {refinement}\nPlease provide an improved command.",
                cmds = suggestion.commands.join(" && ")
            );

            ask_and_run(&mut rl, &cfg, &mut conversation, opts, Ask {
                    request: &refinement_prompt, record_as: &line,
                    empty_hint: "Couldn't improve on that — try describing the change differently.",
                    label: "Run it?",
                });
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

            // ── Telemetry: background submission with tracked handle ─────────────
            // We call submit_background() which returns a JoinHandle.
            // We store it in pending_telemetry so we can join() it at exit,
            // ensuring the HTTP request completes even if the user exits
            // immediately after this command.
            let shell_label = crate::shell::ShellKind::detect().label().to_string();
            let telem_entry = telemetry::TelemetryEntry::new(
                &line,
                &suggestion.commands,
                &cfg.model,
                &cfg.backend,
                &shell_label,
                Some(true),
            );
            if let Some(handle) = telemetry::submit_background(
                telem_entry,
                cfg.telemetry_share_central,
                if cfg.telemetry_user_key.is_empty() { None } else { Some(cfg.telemetry_user_key.clone()) },
                if cfg.telemetry_user_collection.is_empty() { None } else { Some(cfg.telemetry_user_collection.clone()) },
            ) {
                pending_telemetry.push(handle);
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

            ask_and_run(&mut rl, &cfg, &mut conversation, opts, Ask {
                    request: &refinement_prompt, record_as: &line,
                    empty_hint: "Couldn't find a better command — try describing the problem differently.",
                    label: "Run refined command?",
                });
        }
    }
    // REPL loop ends — Rust drops all heap allocations automatically.
}

// =============================================================================
//  Shared REPL helpers (v3.0.7)
//
//  Before v3.0.7 the "show → ask Y/N → execute → record" sequence was copy-
//  pasted five times in main().  The copies had already drifted apart (some
//  recorded failed commands in the conversation context, some did not; the
//  error wording differed).  One implementation means one place to get the
//  safety behaviour right.
// =============================================================================

/// Session-wide switches that the helpers need.  `Copy` so it can be passed
/// freely without borrow-checker gymnastics.
#[derive(Clone, Copy)]
struct RunOpts {
    /// `--dry`: show suggestions but never execute.
    dry_run: bool,
    /// Whether confirmed commands are appended to the shell history file.
    history_enabled: bool,
}

/// Poll the background update check.
///
/// * `wait = Some(d)` blocks for at most `d` (used once, at startup).
/// * `wait = None` is a non-blocking peek (used every REPL turn).
///
/// When a newer version is reported it is announced once and remembered in
/// `slot` so that `!update` can reuse it without another network call.
fn poll_update(
    rx:   &mpsc::Receiver<updater::UpdateStatus>,
    wait: Option<Duration>,
    slot: &mut Option<String>,
) {
    if slot.is_some() {
        return; // already announced
    }
    let msg = match wait {
        Some(d) => rx.recv_timeout(d).ok(),
        None    => rx.try_recv().ok(),
    };
    if let Some(updater::UpdateStatus::UpdateAvailable(v)) = msg {
        updater::print_update_notice(&v);
        *slot = Some(v);
    }
}

/// Ask the user whether to run `commands`.  Returns `true` to run.
///
/// SAFETY MODEL
/// ────────────
/// * Ordinary commands: `[Y/n]` — Enter accepts, because a confirmation that
///   needs a typed answer for `ls` trains people to stop reading it.
/// * Commands flagged by `safety::assess`: the reasons are printed and the
///   prompt flips to `[y/N]` — Enter cancels, and only an explicit `y`/`yes`
///   runs them.
/// * EOF / Ctrl-C at the prompt always means "no".
fn confirm_run(rl: &mut DefaultEditor, label: &str, commands: &[String]) -> bool {
    let reasons = safety::assess(commands);
    let risky = !reasons.is_empty();
    if risky {
        ui::print_risk_warning(&reasons);
    }

    let hint = if risky { "[y/N]" } else { "[Y/n]" };
    loop {
        let ans = match rl.readline(&format!("{} ", format!("  {label} {hint} ›").yellow().bold())) {
            Ok(a)  => a.trim().to_lowercase(),
            Err(_) => return false,
        };
        match ans.as_str() {
            "y" | "yes"      => return true,
            "n" | "no"       => return false,
            "" if !risky     => return true,  // Enter accepts only safe commands
            "" if risky      => return false, // …and declines risky ones
            _                => println!("{}", "  Please press Y or N.".yellow()),
        }
    }
}

/// What to ask the AI and how to word the surrounding messages.
struct Ask<'a> {
    /// The text sent to the model.
    request: &'a str,
    /// The prompt stored in the conversation context on success (so follow-ups
    /// like "now do it for staging" resolve correctly).
    record_as: &'a str,
    /// Shown when the model returns no commands.
    empty_hint: &'a str,
    /// Wording of the confirmation question.
    label: &'a str,
}

/// Ask the AI, show the result, confirm and run it.
fn ask_and_run(
    rl:           &mut DefaultEditor,
    cfg:          &config::Config,
    conversation: &mut context::ConversationContext,
    opts:         RunOpts,
    ask:          Ask<'_>,
) {
    let Ask { request, record_as, empty_hint, label } = ask;
    println!("{}", "  ◌  Thinking…".dimmed());

    let suggestion = match ai::suggest_commands(cfg, conversation, request) {
        Ok(s)  => s,
        Err(e) => {
            ui::print_network_error(&e.to_string());
            return;
        }
    };

    if suggestion.commands.is_empty() {
        ui::print_empty_suggestion(&suggestion);
        println!("  {}  {}", "◈".yellow(), empty_hint.dimmed());
        return;
    }

    ui::print_suggestion(&suggestion, opts.dry_run);

    if opts.dry_run {
        // Nothing runs, but record the turn so follow-ups still work.
        conversation.push(record_as, &suggestion.commands);
        return;
    }

    if !confirm_run(rl, label, &suggestion.commands) {
        println!("{}", "  ◈  Skipped.".dimmed());
        return;
    }

    // Record only commands that actually succeeded: a failed command in the
    // context would mislead the model on the next follow-up.
    if execute_commands(&suggestion.commands) {
        conversation.push(record_as, &suggestion.commands);
        if opts.history_enabled {
            history::append_to_history(&suggestion.commands);
        }
        println!("{}", "  ◈  Great! What else?".dimmed());
    }
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
    for (idx, cmd) in commands.iter().enumerate() {
        println!("\n{}  {}", "  ►".green().bold(), cmd.white().bold());

        // `cd` is special: every command runs in its own child shell, so a
        // `cd` there would be forgotten before the next command starts.
        // A bare `cd <dir>` is therefore applied to this process instead,
        // and the following children inherit the new working directory.
        // (The audit saw GPT-4o-mini return ["cd dir", "git branch"] even
        // though the system prompt forbids it.)
        let outcome: Result<bool, String> = match shell::builtin_cd(cmd) {
            Some(Ok(dir)) => {
                println!("{}", format!("  ✔  Now in {}", dir.display()).green());
                continue;
            }
            Some(Err(e)) => Err(format!("cd failed: {e}")),
            // Dispatch to the detected shell — see shell.rs for the full
            // detection logic (zsh, bash, fish, sh, PowerShell 5/7, cmd, Git Bash).
            None => match shell::run_in_shell(cmd) {
                Ok(s) if s.success() => Ok(true),
                Ok(s) => Err(format!("Exited with {s}")),
                Err(e) => Err(format!("Failed to launch shell: {e}")),
            },
        };

        match outcome {
            Ok(_) => println!("{}", "  ✔  Done.".green()),
            Err(msg) => {
                eprintln!("{}", format!("  ✗  {msg}").red());
                // STOP at the first failure.  Steps are usually dependent
                // ("make dir", then "cd dir", then "rm *"); running the rest
                // after an error can act on the wrong directory.  Before
                // v3.0.7 the loop kept going.
                let skipped = commands.len() - idx - 1;
                if skipped > 0 {
                    eprintln!(
                        "{}",
                        format!("  ◈  Stopped — {skipped} remaining command(s) were not run.").yellow()
                    );
                }
                return false;
            }
        }
    }
    true
}
