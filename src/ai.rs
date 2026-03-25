// =============================================================================
//  ai.rs — AI backend integration (OpenRouter + Ollama)
//  https://github.com/paulfxyz/mang-sh
//
//  OVERVIEW
//  ────────
//  This module is the only place in mang.sh that touches the network.
//  It abstracts over two backends behind a single public function:
//
//    suggest_commands(cfg, history_ctx, prompt) → Result<Suggestion>
//
//  The caller never needs to know whether the request went to OpenRouter
//  or a local Ollama instance — the same Suggestion struct comes back.
//
//  BACKENDS
//  ────────
//  OpenRouter  https://openrouter.ai/api/v1/chat/completions
//    • OpenAI-compatible chat completions API
//    • Requires an API key in the Authorization header
//    • Supports any model on OpenRouter's catalogue (100+ models)
//    • Used when config.backend == "openrouter"
//
//  Ollama      http://localhost:11434/api/chat  (or custom URL)
//    • Local model inference — no API key, no network, complete privacy
//    • Must have Ollama installed and a model pulled (e.g. `ollama pull llama3.2`)
//    • Uses Ollama's native /api/chat endpoint (not the OpenAI-compat endpoint)
//      because the native format is more stable across Ollama versions
//    • Used when config.backend == "ollama"
//
//  THE CORE CHALLENGE: RELIABLE STRUCTURED OUTPUT FROM AN LLM
//  ─────────────────────────────────────────────────────────────
//  Every LLM wants to be helpful and verbose.  Ask "give me a shell command
//  to list files" and you'll often get:
//
//    "Here's a command you can use: `ls -la`"
//    "```bash\nls -la\n```\nThis lists all files including hidden ones."
//
//  Neither is machine-parseable as a bare command string.
//
//  SOLUTION: STRICT JSON ENVELOPE
//  ───────────────────────────────
//  The system prompt instructs the model to always reply with this exact JSON:
//
//    {
//      "commands": ["cmd1", "cmd2"],
//      "explanation": "one plain-English sentence"
//    }
//
//  Why this works reliably:
//    a) Modern LLMs have seen enormous amounts of JSON in training — they are
//       very good at following a JSON schema when it's stated clearly.
//    b) Separating commands (machine-readable) from explanation (human-readable)
//       prevents the model from accidentally mixing them.
//    c) serde_json gives us deterministic parsing with clear error messages.
//    d) An array handles multi-step answers without string splitting.
//    e) We strip markdown fences as a fallback — belt and suspenders.
//
//  MULTI-TURN CONTEXT
//  ──────────────────
//  v2.0.0 introduces follow-up prompts ("now do the same for staging").
//  We pass the last N confirmed prompt/command pairs as prior "user" and
//  "assistant" turns in the messages array.  This gives the model enough
//  conversational context to resolve pronouns and relative references.
//
//  Each prior turn is injected as:
//    { role: "user",      content: "<prior prompt>" }
//    { role: "assistant", content: "<prior commands JSON>" }
//
//  This mirrors how a real multi-turn conversation looks to the model and
//  ensures it interprets follow-ups correctly.
//
//  TEMPERATURE: 0.2
//  ─────────────────
//  Shell commands are not creative.  At temperature=0.2 the model consistently
//  picks the most conventional, highest-probability command — no hallucinated
//  flags, no invented tool names.  Tested across GPT-4o-mini, Claude 3 Haiku,
//  Llama 3.2, and Mistral.
//
//  CONTEXT INJECTION
//  ─────────────────
//  OS, architecture, CWD, and shell are prepended to every user message.
//  This single addition has the largest impact on command accuracy:
//  `brew` vs `apt`, `open` vs `xdg-open`, `pbcopy` vs `xclip`,
//  arm64 vs x86_64 binary downloads, Windows-native paths vs POSIX paths.
// =============================================================================

use crate::config::Config;
use crate::context::ConversationContext;
use crate::shell::ShellKind;
use regex::Regex;
use serde::{Deserialize, Serialize};

// =============================================================================
//  Public output type
// =============================================================================

/// The result of one AI round-trip.
///
/// Contains the shell commands to run (machine-readable) and an optional
/// plain-English explanation (human-readable, shown before Y/N prompt).
#[derive(Debug, Clone)]
pub struct Suggestion {
    /// Shell commands to execute in order.  Each is passed verbatim to the
    /// OS shell (`sh -c` on Unix, `cmd /C` on Windows), so pipelines,
    /// redirections, globs, and compound operators all work correctly.
    pub commands: Vec<String>,

    /// One-sentence description of what the commands do.
    /// Displayed before the confirmation prompt so the user can make an
    /// informed decision.  `None` if the model omitted it (shouldn't happen
    /// with a well-behaved model and our system prompt, but we handle it).
    pub explanation: Option<String>,
}

// =============================================================================
//  OpenRouter wire types
// =============================================================================

/// Request body for the OpenRouter (OpenAI-compatible) chat completions API.
/// Lifetime `'a` borrows model slug and message content — avoids cloning per request.
#[derive(Serialize)]
struct OrChatRequest<'a> {
    model:      &'a str,
    messages:   Vec<OrMessage<'a>>,
    temperature: f32,
    max_tokens:  u32,
}

#[derive(Serialize)]
struct OrMessage<'a> {
    role:    &'a str,
    content: String, // owned — context turns may be built dynamically
}

#[derive(Deserialize)]
struct OrChatResponse {
    choices: Vec<OrChoice>,
}

#[derive(Deserialize)]
struct OrChoice {
    message: OrResponseMessage,
}

#[derive(Deserialize)]
struct OrResponseMessage {
    content: String,
}

// =============================================================================
//  Ollama wire types
//  Docs: https://github.com/ollama/ollama/blob/main/docs/api.md#generate-a-chat-completion
// =============================================================================

#[derive(Serialize)]
struct OllamaChatRequest {
    model:    String,
    messages: Vec<OllamaMessage>,
    stream:   bool,  // false = single response JSON, not a streaming SSE
    options:  OllamaOptions,
}

#[derive(Serialize)]
struct OllamaMessage {
    role:    String,
    content: String,
}

#[derive(Serialize)]
struct OllamaOptions {
    temperature: f32,
    num_predict: u32, // Ollama's equivalent of max_tokens
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaResponseMessage,
}

#[derive(Deserialize)]
struct OllamaResponseMessage {
    content: String,
}

// =============================================================================
//  System prompt
//
//  This is the most important "code" in the project.  It constrains the model's
//  output to the exact JSON schema we need, and encodes our safety preferences.
//
//  Design notes:
//    - Rules are numbered for easy reference when debugging model behaviour
//    - Rule 1 and 2 are redundant by design: restating the format constraint
//      twice reduces the chance of the model "forgetting" it mid-generation
//    - Rule 4 specifies POSIX sh compatibility because we invoke via `sh -c`
//      on Unix; on Windows we map to cmd.exe — see build_context()
//    - Rule 5 is a soft safety rail — it asks for a comment, not a refusal,
//      so legitimate destructive operations still work with the Y/N gate
//    - Rule 7 provides a clean escape hatch for non-shell requests so the
//      empty-commands case is handled gracefully in ui.rs
// =============================================================================
const SYSTEM_PROMPT: &str = r#"You are mang.sh, a terminal assistant that converts natural language requests into shell commands.

RULES:
1. Reply ONLY with a valid JSON object — no prose, no markdown fences, no preamble.
2. The JSON must match this exact schema:
   {
     "commands": ["<cmd1>", "<cmd2>"],
     "explanation": "<one concise sentence describing what the commands accomplish>"
   }
3. Produce the minimal set of commands required — prefer composable one-liners.
4. The system context always includes a SHELL= and syntax= field:
   - syntax=posix      → use POSIX sh syntax (pipes, redirections, &&, $VAR)
   - syntax=powershell → use PowerShell syntax; SHELL=powershell5 means avoid && (use ; or -and instead); SHELL=powershell7 supports &&
   - syntax=cmd        → use cmd.exe syntax (pipe with |, chain with & not &&, use %VAR%)
   Always match the syntax to the syntax= field, not just the OS.
5. Never suggest catastrophically destructive commands (rm -rf /, format C:, dd to a disk device) without adding an explicit safety comment inside the command string.
6. If the request is ambiguous, make the safest reasonable interpretation.
7. If the request cannot be expressed as shell commands, return:
   { "commands": [], "explanation": "I cannot express this as a shell command." }
"#;

// =============================================================================
//  suggest_commands — main public API
// =============================================================================

/// Call the configured AI backend and return a shell command suggestion.
///
/// # Arguments
/// * `cfg`     — user configuration (backend, model, api_key, ollama_url)
/// * `context` — rolling window of prior prompt/command pairs for follow-up support
/// * `prompt`  — the user's current natural-language request
pub fn suggest_commands(
    cfg:     &Config,
    context: &ConversationContext,
    prompt:  &str,
) -> Result<Suggestion, Box<dyn std::error::Error>> {
    match cfg.backend.as_str() {
        "ollama" => suggest_ollama(cfg, context, prompt),
        _        => suggest_openrouter(cfg, context, prompt),
    }
}

// =============================================================================
//  OpenRouter backend
// =============================================================================

fn suggest_openrouter(
    cfg:     &Config,
    context: &ConversationContext,
    prompt:  &str,
) -> Result<Suggestion, Box<dyn std::error::Error>> {
    // Build the messages array:
    //   [system] + [prior turn pairs...] + [current user message]
    let mut messages: Vec<OrMessage> = Vec::new();

    // System message — always first
    messages.push(OrMessage {
        role:    "system",
        content: SYSTEM_PROMPT.to_string(),
    });

    // Prior conversation turns for follow-up context
    for turn in context.turns() {
        // Prior user prompt
        messages.push(OrMessage {
            role:    "user",
            content: format!("System context: {}\n\nUser request: {}", build_context(), turn.prompt),
        });
        // Prior assistant response (reconstructed as JSON)
        messages.push(OrMessage {
            role:    "assistant",
            content: format!(
                r#"{{"commands": [{}], "explanation": "Previous command."}}"#,
                turn.commands_summary
                    .split(" ; ")
                    .map(|c| format!("\"{}\"", c.replace('"', "\\\"")))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }

    // Current user message — augmented with system context
    let user_content = format!(
        "System context: {}\n\nUser request: {}",
        build_context(),
        prompt
    );
    messages.push(OrMessage { role: "user", content: user_content });

    let body = OrChatRequest {
        model:       &cfg.model,
        messages,
        temperature: 0.2,
        max_tokens:  512,
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization",  format!("Bearer {}", cfg.api_key))
        .header("Content-Type",   "application/json")
        .header("HTTP-Referer",   "https://github.com/paulfxyz/mang-sh")
        .header("X-Title",        "mang.sh")
        .json(&body)
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text   = resp.text().unwrap_or_default();
        return Err(format!("OpenRouter {status}: {text}").into());
    }

    let chat: OrChatResponse = resp.json()?;
    let raw = chat.choices.into_iter().next()
        .map(|c| c.message.content)
        .ok_or("Empty response from OpenRouter")?;

    parse_suggestion(&raw)
}

// =============================================================================
//  Ollama backend
// =============================================================================

fn suggest_ollama(
    cfg:     &Config,
    context: &ConversationContext,
    prompt:  &str,
) -> Result<Suggestion, Box<dyn std::error::Error>> {
    // Ollama uses the same message structure as OpenAI but its own endpoint.
    let mut messages: Vec<OllamaMessage> = Vec::new();

    messages.push(OllamaMessage {
        role:    "system".to_string(),
        content: SYSTEM_PROMPT.to_string(),
    });

    // Inject prior conversation turns
    for turn in context.turns() {
        messages.push(OllamaMessage {
            role:    "user".to_string(),
            content: format!("System context: {}\n\nUser request: {}", build_context(), turn.prompt),
        });
        messages.push(OllamaMessage {
            role:    "assistant".to_string(),
            content: format!(
                r#"{{"commands": [{}], "explanation": "Previous command."}}"#,
                turn.commands_summary
                    .split(" ; ")
                    .map(|c| format!("\"{}\"", c.replace('"', "\\\"")))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        });
    }

    messages.push(OllamaMessage {
        role:    "user".to_string(),
        content: format!("System context: {}\n\nUser request: {}", build_context(), prompt),
    });

    let body = OllamaChatRequest {
        model:    cfg.model.clone(),
        messages,
        stream:   false, // we want a single complete response, not SSE stream
        options:  OllamaOptions { temperature: 0.2, num_predict: 512 },
    };

    // Ollama chat endpoint: POST /api/chat
    let url = format!("{}/api/chat", cfg.ollama_url.trim_end_matches('/'));

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120)) // local models can be slow
        .build()?;

    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("Could not reach Ollama at {url}: {e}\nIs Ollama running? Try: ollama serve"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text   = resp.text().unwrap_or_default();
        return Err(format!("Ollama {status}: {text}\nMake sure the model is pulled: ollama pull {}", cfg.model).into());
    }

    let chat: OllamaChatResponse = resp.json()?;
    parse_suggestion(&chat.message.content)
}

// =============================================================================
//  JSON response parser
//
//  Shared by both backends — parses the model's text into a Suggestion.
//
//  FENCE STRIPPING
//  ───────────────
//  Even with an explicit "no markdown fences" rule, some models (especially
//  smaller Ollama models) occasionally wrap output in ```json ... ```.
//  We strip those before attempting JSON parse.  Order matters: try the
//  longer prefix "```json" first.
//
//  GENERIC VALUE PARSE
//  ────────────────────
//  We parse into serde_json::Value rather than a typed struct so that:
//    a) Missing fields produce a clean error, not a panic
//    b) We can embed the raw response in the error message for debugging
//    c) The explanation field is truly optional (Some/None) without requiring
//       an Option<String> in a derived struct that would need flatten logic
// =============================================================================
fn parse_suggestion(raw: &str) -> Result<Suggestion, Box<dyn std::error::Error>> {
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let v: serde_json::Value = serde_json::from_str(cleaned).map_err(|e| {
        format!("Model returned non-JSON output: {e}\n\nRaw response:\n{cleaned}")
    })?;

    let commands: Vec<String> = v
        .get("commands")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let explanation = v
        .get("explanation")
        .and_then(|e| e.as_str())
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    Ok(Suggestion { commands, explanation })
}

// =============================================================================
//  suggest_raw — freeform AI call for the prompt wizard
//
//  Unlike suggest_commands(), this does NOT enforce the JSON schema.
//  We want a plain conversational sentence (a clarifying question), not
//  structured data.  The function is intentionally minimal — it sends a
//  simple system+user message pair and returns the raw content string.
//
//  DESIGN
//  ──────
//  • Temperature 0.5 (vs 0.2 for commands): we want natural, varied phrasing
//    for questions, not the single most-probable token sequence.
//  • max_tokens 256: clarifying questions are short — no need for the full
//    512-token budget we allocate for command generation.
//  • Same HTTP infrastructure as suggest_commands() — no new client setup.
//  • The caller (prompt_wizard.rs) handles empty/error responses gracefully
//    by skipping to synthesis with whatever context was already collected.
// =============================================================================

/// Call the AI backend and return a raw freeform text response.
///
/// Used by the prompt wizard to generate clarifying questions.
/// Unlike `suggest_commands()`, this does NOT parse JSON — it returns
/// the model's raw output as a `String`.
///
/// # Arguments
/// * `cfg`     — user configuration (backend, model, api_key, ollama_url)
/// * `_ctx`    — conversation context (accepted but not injected, to keep
///               the coach call self-contained and unaffected by prior turns)
/// * `prompt`  — the full coach prompt (already formatted by the wizard)
pub fn suggest_raw(
    cfg:  &Config,
    _ctx: &ConversationContext,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    match cfg.backend.as_str() {
        "ollama" => suggest_raw_ollama(cfg, prompt),
        _        => suggest_raw_openrouter(cfg, prompt),
    }
}

/// Raw freeform call via OpenRouter (no JSON schema enforcement).
fn suggest_raw_openrouter(
    cfg:    &Config,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // Minimal message array: a permissive system prompt + the coach prompt.
    // We deliberately do NOT inject our strict JSON system prompt here.
    let messages = vec![
        OrMessage {
            role:    "system",
            content: "You are a helpful assistant. Answer concisely and conversationally. \
                      Do not use markdown, JSON, or bullet points. \
                      One or two short sentences maximum.".to_string(),
        },
        OrMessage {
            role:    "user",
            content: prompt.to_string(),
        },
    ];

    let body = OrChatRequest {
        model:       &cfg.model,
        messages,
        temperature: 0.5,  // slightly higher for natural-sounding questions
        max_tokens:  256,  // questions are short
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", cfg.api_key))
        .header("Content-Type",  "application/json")
        .header("HTTP-Referer",  "https://github.com/paulfxyz/mang-sh")
        .header("X-Title",       "mang.sh")
        .json(&body)
        .send()?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text   = resp.text().unwrap_or_default();
        return Err(format!("OpenRouter {status}: {text}").into());
    }

    let chat: OrChatResponse = resp.json()?;
    let raw = chat.choices.into_iter().next()
        .map(|c| c.message.content)
        .ok_or("Empty response from OpenRouter")?;

    Ok(raw.trim().to_string())
}

/// Raw freeform call via Ollama (no JSON schema enforcement).
fn suggest_raw_ollama(
    cfg:    &Config,
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let messages = vec![
        OllamaMessage {
            role:    "system".to_string(),
            content: "You are a helpful assistant. Answer concisely and conversationally. \
                      Do not use markdown, JSON, or bullet points. \
                      One or two short sentences maximum.".to_string(),
        },
        OllamaMessage {
            role:    "user".to_string(),
            content: prompt.to_string(),
        },
    ];

    let body = OllamaChatRequest {
        model:    cfg.model.clone(),
        messages,
        stream:   false,
        options:  OllamaOptions { temperature: 0.5, num_predict: 256 },
    };

    let url = format!("{}/api/chat", cfg.ollama_url.trim_end_matches('/'));

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    let resp = client
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("Could not reach Ollama at {url}: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text   = resp.text().unwrap_or_default();
        return Err(format!("Ollama {status}: {text}").into());
    }

    let chat: OllamaChatResponse = resp.json()?;
    Ok(chat.message.content.trim().to_string())
}

// =============================================================================
//  Intent detection — natural language config triggers
//
//  These regex patterns intercept prompts that mean "please reconfigure me"
//  before they are sent to the AI.  Benefits:
//    • Instant response (microseconds vs 1-3 s API round-trip)
//    • Zero token cost
//    • More reliable than asking the LLM to classify intent
//
//  Patterns use `.*` between key terms to catch variations:
//    "change my api key"  → "change.*api"  ✓
//    "can I update the api please"  → "update.*api"  ✓
// =============================================================================

/// Returns true if the prompt is asking to reconfigure the API key or model.
pub fn intent_is_api_change(prompt: &str) -> bool {
    let lower = prompt.to_lowercase();
    let patterns = [
        r"change.*api",
        r"update.*api",
        r"new.*api.*key",
        r"set.*api.*key",
        r"switch.*model",
        r"change.*model",
        r"update.*model",
        r"different.*model",
        r"change.*backend",
        r"switch.*backend",
        r"use.*ollama",
        r"use.*openrouter",
    ];
    patterns.iter().any(|p| {
        Regex::new(p).map(|re| re.is_match(&lower)).unwrap_or(false)
    })
}

// =============================================================================
//  System context builder
//
//  Injects runtime environment facts into every prompt.
//  These 4 fields have the highest impact on command correctness:
//
//    OS=macos   → use `open`, `brew`, `pbcopy`; avoid `apt`, `xdg-open`
//    OS=windows → use `cmd.exe` syntax, backslash paths, `start` instead of `open`
//    ARCH=aarch64 → Apple Silicon; some binary installs differ from x86_64
//    CWD=...    → relative paths in suggestions match the actual working dir
//    SHELL=...  → zsh-specific syntax vs bash vs fish vs PowerShell
// =============================================================================
/// Build a rich context string injected into every AI prompt.
///
/// v2.0.0: uses shell::ShellKind::detect() for precise shell identification
/// rather than a raw $SHELL env var read.  This correctly distinguishes
/// PowerShell 5 from PowerShell 7, Git Bash from native bash, etc.
pub fn build_context() -> String {
    let os   = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Detect shell precisely — see shell.rs for the full detection matrix
    let shell = ShellKind::detect();
    let shell_label = shell.label();

    // Include a posix_compat hint so the AI knows whether to use POSIX syntax,
    // PowerShell syntax, or cmd.exe syntax without having to infer it.
    let syntax_hint = if shell.is_powershell() {
        "syntax=powershell"
    } else if matches!(shell, ShellKind::Cmd) {
        "syntax=cmd"
    } else {
        "syntax=posix"
    };

    format!("OS={os} ARCH={arch} CWD={cwd} SHELL={shell_label} {syntax_hint}")
}
