// =============================================================================
//  prompt_wizard.rs — Advanced Prompt Mode
//  https://mang.sh
//
//  OVERVIEW
//  ────────
//  The prompt wizard is a short Socratic dialogue that helps users who are
//  stuck — either because their request was too vague for the AI, or because
//  they proactively want a more guided experience.
//
//  It runs up to MAX_ROUNDS rounds.  Each round:
//    1. The AI generates ONE targeted clarifying question (coach mode).
//    2. The user answers (or presses Enter / !skip / !done to stop early).
//    3. The answer is stored and fed into the next question.
//
//  After all rounds, the accumulated context is synthesised into a single
//  precise prompt that is passed to the normal suggest_commands() pipeline.
//
//  TRIGGER PATHS
//  ─────────────
//    Explicit:   user types !prompt or !p at any yo › prompt
//    Automatic:  AI returned an empty commands array (couldn't interpret request)
//
//  WHY A SEPARATE COACH CALL (suggest_raw)?
//  ─────────────────────────────────────────
//  The normal suggest_commands() enforces a strict JSON schema:
//    { "commands": [...], "explanation": "..." }
//  That schema is what makes command parsing reliable.  But for clarifying
//  questions we want freeform text — a short conversational sentence.
//
//  suggest_raw() makes the same HTTP request but with a different system
//  prompt (just "answer concisely") and returns the raw content string.
//  This avoids hacking the JSON parser to accept prose.
//
//  DESIGN DECISIONS
//  ─────────────────
//  • Max 3 rounds: enough to disambiguate almost anything without feeling
//    like an interrogation.  The number is named MAX_ROUNDS so it's easy
//    to tune.
//  • Early exit: Enter / !skip / !done at any round fires with gathered context.
//    This lets power users skip straight to the command after one answer.
//  • Synthesis is pure string ops (no AI call): we join subject + answers
//    into one coherent sentence.  The downstream suggest_commands() call then
//    generates the actual command.  Keeping synthesis deterministic means the
//    wizard always progresses even if the AI service is slow.
//  • The wizard does NOT execute commands — it returns WizardResult::Prompt
//    so the caller (main.rs) runs the full normal Y/N execution pipeline.
//    This preserves the safety guarantee: nothing runs without user confirmation.
//
//  LESSON LEARNED (v3.0.2)
//  ─────────────────────────
//  Original design tried to parse the AI's clarifying question as a Suggestion
//  JSON blob, then strip the commands and use the explanation field as the
//  question.  That was fragile — small models sometimes wrap freeform text in
//  the JSON and sometimes don't.  suggest_raw() is the cleaner solution: one
//  function, one contract (return plain text), no parsing gymnastics.
// =============================================================================

use colored::Colorize;
use rustyline::DefaultEditor;

use crate::ai;
use crate::config::Config;
use crate::context::ConversationContext;

/// Maximum number of clarifying questions the wizard asks before synthesising.
pub const MAX_ROUNDS: usize = 3;

// =============================================================================
//  Public API
// =============================================================================

/// Return value from the wizard — tells the caller what to do next.
pub enum WizardResult {
    /// Wizard produced a refined prompt.  Pass this to `suggest_commands()`.
    Prompt(String),
    /// User abandoned the wizard (pressed Enter on an empty first answer, or
    /// typed !skip before providing any useful context).
    Abandoned,
}

/// Run the interactive Advanced Prompt Mode wizard.
///
/// Conducts up to `MAX_ROUNDS` rounds of AI-generated clarifying questions,
/// collects user answers, then synthesises everything into one clear prompt.
///
/// # Arguments
/// * `rl`           — rustyline editor, shared with the main REPL loop
/// * `cfg`          — current configuration (for AI backend calls)
/// * `conversation` — current conversation context (passed through, not modified)
/// * `original`     — the user's original vague prompt; empty string if the
///                    wizard was invoked directly via `!prompt`
///
/// # Returns
/// `WizardResult::Prompt(s)` with the synthesised prompt on success, or
/// `WizardResult::Abandoned` if the user bailed out with no useful input.
pub fn run(
    rl:           &mut DefaultEditor,
    cfg:          &Config,
    conversation: &ConversationContext,
    original:     &str,
) -> WizardResult {
    print_wizard_header();

    // Accumulated Q&A pairs across rounds.
    // Used to give the AI context for each successive question.
    let mut qa_pairs: Vec<(String, String)> = Vec::new();

    // Normalise the original prompt for use as "subject" in the coach call.
    // If there is no original prompt (wizard invoked via !prompt with nothing
    // prior), we use a neutral placeholder that tells the coach to ask an
    // open-ended first question.
    let subject = if original.trim().is_empty() {
        "the user needs help with a terminal task but hasn't specified it yet".to_string()
    } else {
        original.trim().to_string()
    };

    for round in 0..MAX_ROUNDS {
        // ── Generate a targeted clarifying question via the AI ─────────────────
        let question = match ai::suggest_raw(cfg, conversation, &coach_prompt(&subject, &qa_pairs)) {
            Ok(q)  => q,
            Err(e) => {
                eprintln!("{}", format!("  ✗  Wizard AI call failed: {e}").red());
                // Degrade gracefully: skip to synthesis with what we have
                break;
            }
        };

        // ── Display the question ───────────────────────────────────────────────
        print_wizard_question(round + 1, MAX_ROUNDS, &question);

        // ── Read the user's answer ─────────────────────────────────────────────
        let answer = match rl.readline(&format!("{} ", "  yo ›".cyan().bold())) {
            Ok(a) => {
                let t = a.trim().to_string();
                if !t.is_empty() {
                    let _ = rl.add_history_entry(&t);
                }
                t
            }
            Err(_) => String::new(), // Ctrl-D / interrupt → treat as skip
        };

        // ── Early-exit triggers ────────────────────────────────────────────────
        // Empty answer, !skip, or !done → stop collecting and synthesise now.
        // Special case: if this is round 0 AND nothing was collected yet AND the
        // subject is the neutral placeholder, the user gave us nothing to work
        // with — truly abandon.
        let is_skip = answer.is_empty()
            || matches!(answer.to_lowercase().as_str(), "!skip" | "!done" | "!s");

        if is_skip {
            if qa_pairs.is_empty()
                && subject == "the user needs help with a terminal task but hasn't specified it yet"
            {
                println!("{}", "  ◈  Wizard cancelled — nothing to work with.".dimmed());
                println!();
                return WizardResult::Abandoned;
            }
            println!("{}", "  ◌  Got it — building your command…".dimmed());
            println!();
            break;
        }

        // Store the Q&A pair for context in subsequent rounds
        qa_pairs.push((question, answer));

        // After the final round, show the synthesis message
        if round == MAX_ROUNDS - 1 {
            println!("{}", "  ◌  Building your command…".dimmed());
            println!();
        }
    }

    // ── Synthesise all collected context into one precise prompt ──────────────
    // This is a pure string operation — no AI call.  We combine the original
    // subject with all the user's answers to form a detailed, unambiguous request.
    // The downstream suggest_commands() call turns this into actual shell commands.
    let refined = synthesise(&subject, &qa_pairs);

    // Show the user what refined prompt will be sent, so they can see the value
    println!(
        "  {}  {}",
        "◈".cyan().bold(),
        format!("Refined: {refined}").white()
    );
    println!();

    WizardResult::Prompt(refined)
}

// =============================================================================
//  Internal helpers
// =============================================================================

/// Build the coach prompt sent to the AI for each clarifying question.
///
/// The coach prompt uses a different framing than the normal command-generation
/// system prompt — it instructs the AI to act as a "prompt coach" and output
/// exactly ONE short question, with no preamble or JSON.
///
/// We include:
///   • The user's original request (subject)
///   • All prior Q&A pairs (so the AI avoids asking the same thing twice)
///   • A clear constraint: one question, conversational tone, max 2 sentences
fn coach_prompt(subject: &str, qa_pairs: &[(String, String)]) -> String {
    // Format prior Q&A as readable context for the AI
    let prior_context = if qa_pairs.is_empty() {
        String::new()
    } else {
        let pairs = qa_pairs
            .iter()
            .map(|(q, a)| format!("  Q: {q}\n  A: {a}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("\n\nContext gathered so far:\n{pairs}")
    };

    format!(
        "You are a shell command prompt coach. A user is trying to run something \
         in their terminal but their request is unclear.\n\
         \n\
         Original request: \"{subject}\"{prior_context}\n\
         \n\
         Ask ONE short, specific clarifying question to help determine exactly \
         what shell command they need. Be conversational — max 1-2 sentences. \
         No preamble, no numbering, no JSON. Just the question."
    )
}

/// Synthesise accumulated Q&A context into a single, precise command prompt.
///
/// This is pure string logic — no AI call, fully deterministic.
/// The strategy: combine the original subject with each answer, joined with
/// ". " to form a natural compound sentence that is rich enough for the
/// downstream `suggest_commands()` call to produce a precise suggestion.
///
/// Example:
///   subject  = "do the docker thing"
///   answers  = ["restart my container", "myapp", "also show logs"]
///   result   = "do the docker thing. restart my container. myapp. also show logs"
///
/// The downstream AI is good at extracting intent from compound sentences like
/// this — the extra detail is always better than vagueness.
fn synthesise(subject: &str, qa_pairs: &[(String, String)]) -> String {
    if qa_pairs.is_empty() {
        return subject.to_string();
    }

    // Start with the original subject if it's meaningful
    let mut parts: Vec<String> = Vec::new();
    if subject != "the user needs help with a terminal task but hasn't specified it yet" {
        parts.push(subject.to_string());
    }

    // Add each answer (questions are the AI's context, not the user's intent)
    for (_, answer) in qa_pairs {
        parts.push(answer.clone());
    }

    parts.join(". ")
}

// =============================================================================
//  UI rendering
// =============================================================================

/// Print the wizard header box shown at the start of every wizard session.
pub fn print_wizard_header() {
    println!();
    println!(
        "{}",
        "  ╔═══════════════════════════════════════════════╗".cyan()
    );
    println!(
        "{}",
        "  ║  ✦  Advanced Prompt Mode                      ║".cyan().bold()
    );
    println!(
        "{}",
        "  ║     I'll ask up to 3 questions to nail it.    ║".dimmed()
    );
    println!(
        "{}",
        "  ║     Press Enter or type !skip to fire early.  ║".dimmed()
    );
    println!(
        "{}",
        "  ╚═══════════════════════════════════════════════╝".cyan()
    );
    println!();
}

/// Print a single wizard question with its round counter.
fn print_wizard_question(round: usize, max: usize, question: &str) {
    println!(
        "  {}  {}  {}",
        "✦".cyan().bold(),
        format!("({round}/{max})").dimmed(),
        question.white().bold()
    );
    println!();
}
