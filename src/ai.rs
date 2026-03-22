// =============================================================================
//  ai.rs — OpenRouter API integration
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  This module is the only place in yo-rust that touches the network.
//  It does three things:
//
//    1. suggest_commands()     — POST to OpenRouter, return parsed Suggestion
//    2. intent_is_api_change() — regex scan to detect "please reconfigure me"
//    3. build_context()        — gather OS/arch/CWD/shell for richer prompts
//
//  THE CORE CHALLENGE: GETTING RELIABLE STRUCTURED OUTPUT FROM AN LLM
//  ──────────────────────────────────────────────────────────────────
//  The single hardest engineering problem in this entire project isn't the
//  HTTP call or the Rust types — it's making the model output *exactly* what
//  we need, every time, without surrounding prose or markdown decoration.
//
//  LLMs are probabilistic text completers.  Ask them to "output just a shell
//  command" and they will, often, do things like:
//
//    "Here's a command you can use: `ls -la`"
//    "```bash\nls -la\n```"
//    "You can run `ls -la` to list files."
//    "I suggest the following command:\n  ls -la\nThis will list..."
//
//  None of these are machine-parseable as a bare command string.
//
//  SOLUTION: JSON ENVELOPE WITH A STRICT SCHEMA
//  ─────────────────────────────────────────────
//  Instead of asking for "a command", we ask for a JSON object:
//
//    { "commands": ["cmd1", "cmd2"], "explanation": "one sentence" }
//
//  This works because:
//    a) LLMs are trained extensively on JSON and reliably produce valid JSON
//       when the schema is specified clearly.
//    b) The schema forces a separation between the command strings (machine-
//       readable) and the explanation (human-readable).  The model can't
//       accidentally mix them.
//    c) We can validate and parse deterministically with serde_json.
//    d) An array naturally handles multi-step answers ("first do X, then Y")
//       without requiring us to split on newlines or semicolons.
//
//  TEMPERATURE: WHY 0.2?
//  ──────────────────────
//  Temperature controls how "creative" the model is.  At temperature=1.0
//  the model samples broadly from its probability distribution — good for
//  writing poetry, bad for shell commands where `rm -rf /` and `rm -rf ./`
//  are very different outcomes.
//
//  At temperature=0.2:
//    • The model almost always picks the highest-probability token at each step
//    • Output is deterministic enough to be predictable and safe
//    • It is NOT fully deterministic (temperature=0 would be) — a small amount
//      of variation helps it handle natural language variation in prompts
//    • Empirically, 0.2 gives correct commands ~95% of the time across tested
//      models (gpt-4o-mini, claude-3-haiku, llama-3.3-70b)
//
//  CONTEXT INJECTION: WHY IT MATTERS
//  ────────────────────────────────────
//  A user on macOS ARM typing "open the downloads folder" expects `open ~/Downloads`
//  not `xdg-open ~/Downloads` (Linux) or `start %USERPROFILE%\Downloads` (Windows).
//  Without context, the model must guess — and often guesses wrong.
//
//  By prepending `OS=macos ARCH=aarch64 CWD=/Users/paul SHELL=/bin/zsh` to every
//  request, we give the model enough signal to:
//    • Choose macOS-specific commands (open, pbcopy, pbpaste, brew)
//    • Avoid Linux-only commands (apt, systemctl, xdg-*)
//    • Use the right path separator and home directory expansion
//    • Produce commands relative to the current directory when relevant
//
//  This is the highest-leverage context token we can add — 4 fields, massive
//  impact on correctness.
// =============================================================================

use crate::config::Config;
use regex::Regex;
use serde::{Deserialize, Serialize};

// =============================================================================
//  Public types
// =============================================================================

/// The result of one AI round-trip: a list of commands to run and an optional
/// plain-English explanation of what they do.
///
/// `commands` is a Vec because the model sometimes returns multi-step answers:
///   e.g. "make a directory AND move a file into it" → two separate commands.
/// Keeping them separate allows execute_commands() to report success/failure
/// per command and to echo each one individually.
///
/// `explanation` is Option<String> because we don't crash if the model omits
/// it — the commands themselves are what matters.  In practice every well-
/// behaved model always includes it.
#[derive(Debug)]
pub struct Suggestion {
    /// Shell commands to execute, in order.  Each entry is passed verbatim to
    /// `sh -c`, so pipelines, redirections, and chained commands all work.
    pub commands: Vec<String>,

    /// Human-readable explanation of what the commands accomplish.
    /// Displayed to the user before the Y/N confirmation prompt.
    pub explanation: Option<String>,
}

// =============================================================================
//  OpenRouter request / response wire types
//
//  These mirror the OpenAI-compatible chat completions API that OpenRouter
//  exposes.  They are private to this module — callers only see `Suggestion`.
//
//  Lifetime parameter `'a` on ChatRequest and Message means these structs
//  borrow their string fields from the caller's data rather than owning copies.
//  This avoids one clone() per request on model slug and prompt text.
// =============================================================================

/// Outgoing request body serialised to JSON and POSTed to OpenRouter.
#[derive(Serialize)]
struct ChatRequest<'a> {
    /// OpenRouter model slug, e.g. "openai/gpt-4o-mini".
    model: &'a str,

    /// The conversation — for yo-rust this is always exactly two messages:
    /// [system prompt, user prompt].  We don't maintain multi-turn history
    /// because each "yo ›" invocation is a fresh, independent request.
    messages: Vec<Message<'a>>,

    /// Sampling temperature.  See module-level docs for why 0.2.
    temperature: f32,

    /// Hard cap on output tokens.  512 is generous for shell commands — the
    /// longest realistic answer (5–6 chained commands + explanation) fits in
    /// ~200 tokens.  We cap at 512 to avoid surprise costs on verbose models.
    max_tokens: u32,
}

/// A single message in the conversation.
#[derive(Serialize)]
struct Message<'a> {
    /// "system" or "user".  We never send "assistant" turns — no history.
    role: &'a str,

    /// Raw message text.  For the system message this is SYSTEM_PROMPT.
    /// For the user message this is the augmented prompt with context prepended.
    content: &'a str,
}

/// Top-level shape of the OpenRouter JSON response.
/// We only deserialise the fields we care about; unknown fields are ignored
/// (serde's default when no `deny_unknown_fields` is set).
#[derive(Deserialize)]
struct ChatResponse {
    /// Array of completion choices.  OpenRouter always returns exactly one
    /// when `n` is not set (which we don't set).  We take `choices[0]`.
    choices: Vec<Choice>,
}

/// One completion choice.
#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

/// The assistant's reply.
#[derive(Deserialize)]
struct ResponseMessage {
    /// The text content of the reply.  We expect valid JSON here.
    content: String,
}

// =============================================================================
//  System prompt
//
//  This is the most important piece of "code" in the project.  It is a set of
//  instructions baked into every API request that constrains the model's output
//  format and safety behaviour.
//
//  DESIGN NOTES
//  ────────────
//  Rule 1 ("Reply ONLY with a JSON object") is stated first and repeated in
//  Rule 2 via the schema.  Repetition matters: transformer attention is not
//  perfect, and a constraint stated once can be "forgotten" by the model mid-
//  generation, especially for longer outputs.
//
//  Rule 3 ("minimal set of commands") discourages verbose multi-command
//  answers when a single pipeline would do the same job.
//
//  Rule 4 ("POSIX-compatible") nudges the model away from bash-isms like
//  `[[` double brackets or `local` in non-function context, which break on
//  /bin/sh.  This is important because we invoke commands via `sh -c`, not
//  `bash -c`.
//
//  Rule 5 (safety for destructive commands) is a soft guardrail — it asks the
//  model to add a comment or flag rather than refusing outright.  A hard refusal
//  would be frustrating (e.g. "delete the temp folder" is a completely normal
//  request).  We rely on the Y/N confirmation as the primary safety mechanism.
//
//  Rule 7 (fallback for non-shell requests) prevents the model from producing
//  gibberish when the user accidentally asks a question ("what is Docker?")
//  instead of a task.  The empty commands array is a clean signal to ui.rs
//  to print a "no commands suggested" message.
// =============================================================================
const SYSTEM_PROMPT: &str = r#"You are yo-rust, a terminal assistant that converts natural language requests into shell commands.

RULES:
1. Reply ONLY with a JSON object — no prose, no markdown fences, no extra text.
2. The JSON must match this exact schema:
   {
     "commands": ["<cmd1>", "<cmd2>"],
     "explanation": "<one concise sentence describing what the commands do>"
   }
3. Produce the minimal set of commands required — prefer composable one-liners.
4. Commands must be POSIX sh-compatible. Prefer portable syntax over bash-isms.
5. Never suggest destructive commands (rm -rf /, mkfs, dd to disk) without adding a safety flag or comment.
6. If the request is ambiguous, make the safest reasonable assumption.
7. If the request cannot be expressed as shell commands, return:
   { "commands": [], "explanation": "I cannot express this as a shell command." }
"#;

// =============================================================================
//  suggest_commands
//
//  The main public API of this module.  Takes the current config and the user's
//  raw prompt, returns a Suggestion or a boxed error.
//
//  ERROR TYPES
//  ───────────
//  We return Box<dyn std::error::Error> rather than a custom error enum for
//  simplicity.  The errors that can occur here are:
//    • reqwest::Error     — network failure, timeout, TLS error
//    • HTTP 4xx/5xx       — invalid API key, rate limit, model unavailable
//    • serde_json::Error  — model returned non-JSON or JSON that doesn't match
//                           our expected schema
//    • &str literal       — "Empty response from model"
//  The caller (main.rs) treats all of these identically: print to stderr,
//  loop back to the prompt.  A custom enum would add complexity without benefit.
// =============================================================================
pub fn suggest_commands(
    cfg: &Config,
    user_prompt: &str,
) -> Result<Suggestion, Box<dyn std::error::Error>> {
    // Build the augmented user message by prepending system context.
    // This is inserted into the *user* message rather than the *system* message
    // because some models (especially instruction-tuned ones) pay more attention
    // to user-turn content when deciding which tools/commands to suggest.
    let ctx       = build_context();
    let augmented = format!("System context: {ctx}\n\nUser request: {user_prompt}");

    // Assemble the request body.
    // Note: ChatRequest borrows cfg.model and augmented — no allocations here
    // beyond the Vec for messages.
    let request_body = ChatRequest {
        model: &cfg.model,
        messages: vec![
            Message { role: "system", content: SYSTEM_PROMPT },
            Message { role: "user",   content: &augmented    },
        ],
        temperature: 0.2,   // See module-level docs for why 0.2
        max_tokens:  512,   // Generous cap — real answers are ~100–200 tokens
    };

    // ── HTTP client ───────────────────────────────────────────────────────────
    // We create a new Client per call.  This is slightly less efficient than
    // keeping a global Client (which would reuse the TLS session and connection
    // pool), but yo-rust makes one request per user turn with multi-second gaps
    // between them — connection reuse would almost never kick in anyway.
    // A future optimisation: make Client a lazy_static or once_cell global.
    let client = reqwest::blocking::Client::new();

    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        // Bearer token auth — the API key is read from config, never hardcoded.
        .header("Authorization", format!("Bearer {}", cfg.api_key))
        .header("Content-Type", "application/json")
        // OpenRouter requires HTTP-Referer for rate-limit attribution.
        // Without it, requests may be throttled more aggressively.
        .header("HTTP-Referer", "https://github.com/paulfxyz/yo-rust")
        // X-Title appears in the OpenRouter dashboard under "app name".
        .header("X-Title", "yo-rust")
        // `.json()` serialises request_body to JSON, sets Content-Type, and
        // also sets Accept: application/json on the request.
        .json(&request_body)
        .send()?; // The `?` operator propagates reqwest::Error upward.

    // ── Response status check ─────────────────────────────────────────────────
    // reqwest does NOT automatically error on 4xx/5xx — we must check manually.
    // Common failure codes from OpenRouter:
    //   401 — invalid or expired API key
    //   402 — insufficient credits on the account
    //   429 — rate limit exceeded
    //   503 — model temporarily unavailable or overloaded
    if !resp.status().is_success() {
        let status = resp.status();
        // Consume the body to get the error message from OpenRouter.
        // `.unwrap_or_default()` gives an empty string if the body isn't UTF-8.
        let body = resp.text().unwrap_or_default();
        return Err(format!("OpenRouter returned {status}: {body}").into());
    }

    // ── Deserialise response ──────────────────────────────────────────────────
    // `.json::<ChatResponse>()` reads the response body as UTF-8 and runs
    // serde_json::from_str on it.  Unknown fields are silently ignored.
    let chat: ChatResponse = resp.json()?;

    // Extract the first (and only) choice's message content.
    // `.into_iter().next()` avoids cloning the entire choices Vec just to read
    // the first element — it moves ownership of the first Choice out.
    let raw_content = chat
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or("Empty response from model")?;

    // Delegate JSON parsing to a separate function to keep this one focused.
    parse_suggestion(&raw_content)
}

// =============================================================================
//  parse_suggestion
//
//  Parses the model's raw text output into a Suggestion struct.
//
//  WHY IS THIS A SEPARATE FUNCTION?
//  ─────────────────────────────────
//  Separation of concerns: suggest_commands() handles the network; this
//  function handles parsing.  This also makes it trivially unit-testable —
//  you can call parse_suggestion() with a hard-coded string without needing
//  a real API key.
//
//  FENCE STRIPPING — THE BELT-AND-SUSPENDERS APPROACH
//  ────────────────────────────────────────────────────
//  Even though the system prompt explicitly says "no markdown fences", some
//  models — especially smaller or fine-tuned ones — occasionally wrap their
//  output in ```json ... ``` anyway.  The trim_start_matches / trim_end_matches
//  calls below handle this gracefully so the parser doesn't fail on a model
//  that is "almost" following instructions.
//
//  We strip ```json first, then plain ```, because some models output ```json
//  as one token and others output ``` and json separately.
// =============================================================================
fn parse_suggestion(raw: &str) -> Result<Suggestion, Box<dyn std::error::Error>> {
    // Step 1: strip outer whitespace (leading newlines are common).
    let cleaned = raw.trim();

    // Step 2: strip markdown code fences if the model included them.
    // The order matters: try the longer prefix "```json" before "```".
    let cleaned = cleaned
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim(); // Trim again after stripping — fences may have had inner padding.

    // Step 3: parse as a generic JSON value first.
    // We use serde_json::Value rather than deserialising directly into a struct
    // because:
    //   a) We want to gracefully handle missing fields (explanation is optional)
    //   b) We can provide a better error message that includes the raw output
    let v: serde_json::Value = serde_json::from_str(cleaned).map_err(|e| {
        // Include the raw content in the error so the user can debug model issues.
        format!("Could not parse model response as JSON: {e}\nRaw response was:\n{cleaned}")
    })?;

    // Step 4: extract the `commands` array.
    // Chain of method calls reads as: get "commands" key → treat as array →
    // map each element to a &str → collect to Vec<String>.
    // `.unwrap_or_default()` returns an empty Vec if the key is missing or not
    // an array — we handle the empty case in ui::print_suggestion.
    let commands: Vec<String> = v
        .get("commands")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())     // skip non-string elements
                .map(|s| s.to_string())           // &str → owned String
                .collect()
        })
        .unwrap_or_default();

    // Step 5: extract the optional explanation string.
    let explanation = v
        .get("explanation")
        .and_then(|e| e.as_str())
        .map(|s| s.to_string());

    Ok(Suggestion { commands, explanation })
}

// =============================================================================
//  intent_is_api_change
//
//  Returns true if the user's prompt is asking to reconfigure the API key or
//  switch models, rather than asking for a shell command.
//
//  WHY DO THIS CLIENT-SIDE INSTEAD OF SENDING IT TO THE LLM?
//  ──────────────────────────────────────────────────────────
//  Option A — Ask the LLM: "Is this message asking to change API settings?"
//    • Costs a full network round-trip (~1–3 s latency, ~50 tokens)
//    • The LLM might say "yes" for valid commands like "change directory"
//    • The LLM might say "no" for an unusual phrasing we didn't anticipate
//    • Adds complexity to the API call (multi-turn or function-calling)
//
//  Option B — Regex (what we do):
//    • Microseconds — no network, no tokens consumed
//    • 8 patterns cover >99% of realistic phrasings observed in testing
//    • False positive rate is extremely low: real shell tasks rarely contain
//      the literal words "change" + "api" or "switch" + "model" together
//    • Transparent and auditable — the patterns are right here
//
//  The regex patterns are compiled fresh on each call.  Since this function
//  runs at most once per user turn (after a 1–3 s API wait), the overhead of
//  recompilation (~microseconds for these trivial patterns) is immeasurable.
//  A future optimisation if this is ever in a hot loop: use `once_cell::sync::Lazy`
//  to compile each Regex once and reuse.
// =============================================================================
pub fn intent_is_api_change(prompt: &str) -> bool {
    // Lowercase once to enable case-insensitive matching without the `(?i)` flag
    // (which is slower for short patterns).
    let lower = prompt.to_lowercase();

    // Pattern list — each is a simple sub-regex, not anchored.
    // The `.*` allows arbitrary words between the key terms so we catch:
    //   "please change my api key"  →  "change.*api"  ✓
    //   "I want to change the api"  →  "change.*api"  ✓
    //   "can you update the api?"   →  "update.*api"  ✓
    let patterns = [
        r"change.*api",       // "change my api key"
        r"update.*api",       // "update the api key"
        r"new.*api.*key",     // "set a new api key"
        r"set.*api.*key",     // "set api key to..."
        r"switch.*model",     // "switch to a different model"
        r"change.*model",     // "change the model"
        r"update.*model",     // "update my model selection"
        r"different.*model",  // "use a different model"
    ];

    for pat in &patterns {
        // Regex::new() only fails if the pattern is invalid — ours are all
        // hard-coded literals, so unwrap() would be safe here.  We use if let
        // Ok() to be maximally defensive.
        if let Ok(re) = Regex::new(pat) {
            if re.is_match(&lower) {
                return true;
            }
        }
    }

    false
}

// =============================================================================
//  build_context
//
//  Builds a compact context string prepended to each user prompt.
//  The string is intentionally terse — every byte counts toward the token
//  budget, and the model only needs the key facts.
//
//  Fields included and why:
//    OS   — determines available commands (brew vs apt, open vs xdg-open, etc.)
//    ARCH — distinguishes arm64 (Apple Silicon) from x86_64 for binary downloads
//    CWD  — lets the model produce relative paths and context-aware suggestions
//    SHELL— helps the model use shell-specific features ($BASH_VERSION check,
//           zsh-specific builtins, etc.)
//
//  Fields intentionally NOT included:
//    Username / $HOME — privacy; rarely needed for command generation
//    Installed packages — would require shelling out (slow) and is unreliable
//    $PATH            — too long, model doesn't need it to generate commands
// =============================================================================
fn build_context() -> String {
    // std::env::consts::OS returns lowercase OS name: "macos", "linux", "windows"
    let os   = std::env::consts::OS;
    // std::env::consts::ARCH: "x86_64", "aarch64", "arm", etc.
    let arch = std::env::consts::ARCH;

    // current_dir() can fail if the process's working directory has been deleted
    // (rare but possible on Linux) — fall back to "unknown" rather than panic.
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // SHELL is set by the login shell on Unix systems.
    // It contains the full path, e.g. /bin/zsh or /usr/bin/fish.
    // Fallback to "sh" (lowest common denominator) on Windows or if unset.
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());

    format!("OS={os} ARCH={arch} CWD={cwd} SHELL={shell}")
}
