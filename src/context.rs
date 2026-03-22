// =============================================================================
//  context.rs — Multi-turn conversation context
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  Without context, every `yo ›` prompt is completely independent.  The AI
//  has no memory of what it suggested before, so follow-up requests like:
//
//    yo ›  list all log files older than 7 days
//    yo ›  now delete them                          ← "them" = ?
//
//  ...break down because the model has no idea what "them" refers to.
//
//  This module solves that by maintaining a rolling window of the last N
//  (config.context_size, default 5) confirmed prompt/command pairs and
//  serialising them as prior conversation turns in each new API request.
//
//  HOW IT WORKS
//  ────────────
//  Each confirmed execution is recorded as a `Turn`:
//    - prompt:   the user's original natural-language request
//    - commands: the commands that were confirmed and run
//
//  When the AI module builds the next request, it prepends these turns as
//  "assistant" messages in the chat history, giving the model enough context
//  to resolve pronouns ("them", "it", "the same folder") and follow-up
//  intentions ("do it recursively", "now for staging instead of production").
//
//  WINDOW MANAGEMENT
//  ─────────────────
//  We keep at most `max_turns` entries.  When the window is full, the oldest
//  turn is dropped (FIFO / sliding window).  This prevents the context from
//  growing unboundedly and consuming excessive tokens on long sessions.
//
//  The window is in-memory only — it is not persisted to disk.  Rationale:
//    1. Context is session-specific; stale context from a past session
//       would confuse rather than help the model.
//    2. The prompt history may contain sensitive paths or data.
//    3. Keeping it in RAM keeps the code simple and safe.
//
//  TOKEN BUDGET AWARENESS
//  ──────────────────────
//  Each turn in the context costs tokens.  At 5 turns × ~50 tokens/turn =
//  ~250 tokens overhead — well within the 4k–32k context windows of modern
//  models and negligible against our 512-token max_tokens cap on output.
//  If a future version needs to trim aggressively, it can truncate the oldest
//  turns first since they are least relevant to the current request.
// =============================================================================

/// One completed interaction: the user's prompt and the commands that ran.
///
/// Both fields are `String` (owned) because this struct lives across multiple
/// REPL iterations and must outlive the borrow of any particular turn's data.
#[derive(Debug, Clone)]
pub struct Turn {
    /// The original natural-language prompt the user typed.
    pub prompt: String,

    /// The shell commands that were confirmed and executed (or shown in dry-run).
    /// Stored as a single joined string for compact representation in the AI
    /// context window: "cmd1 && cmd2" reads more naturally than a JSON array.
    pub commands_summary: String,
}

/// A rolling window of recent turns, used to provide follow-up context to
/// the AI on each new request.
///
/// The window is bounded by `max_turns`.  When full, the oldest entry is
/// dropped to make room for new ones (FIFO sliding window).
#[derive(Debug)]
pub struct ConversationContext {
    /// The turns in chronological order, oldest first.
    turns: Vec<Turn>,

    /// Maximum number of turns to retain.  Configurable via config.context_size.
    /// Default is 5.  Set to 0 to disable context entirely.
    max_turns: usize,
}

impl ConversationContext {
    /// Create a new, empty context window with the given capacity.
    ///
    /// Passing `max_turns = 0` creates a context that never stores anything,
    /// effectively disabling multi-turn memory.  This is used when `--no-context`
    /// is passed on the command line.
    pub fn new(max_turns: usize) -> Self {
        Self {
            turns: Vec::with_capacity(max_turns.min(20)), // cap initial alloc
            max_turns,
        }
    }

    /// Record a completed turn (prompt + commands) into the context window.
    ///
    /// If the window is at capacity, the oldest turn is removed first.
    /// No-op when `max_turns == 0`.
    pub fn push(&mut self, prompt: &str, commands: &[String]) {
        if self.max_turns == 0 {
            return;
        }

        // Evict the oldest entry if we're at capacity.
        // VecDeque would be O(1) for front removal, but Vec::remove(0) is fine
        // here — max_turns is small (≤20) and this runs at most once per user turn.
        if self.turns.len() >= self.max_turns {
            self.turns.remove(0);
        }

        // Summarise multiple commands as a semicolon-separated string.
        // This is more readable than a JSON array when injected into a prompt.
        let commands_summary = commands.join(" ; ");

        self.turns.push(Turn {
            prompt: prompt.to_string(),
            commands_summary,
        });
    }

    /// Returns a slice of all recorded turns, oldest first.
    ///
    /// The AI module iterates over this to build the prior-context portion
    /// of the chat messages array.
    pub fn turns(&self) -> &[Turn] {
        &self.turns
    }

    /// Returns true if there are any recorded turns.
    pub fn is_empty(&self) -> bool {
        self.turns.is_empty()
    }

    /// Returns the number of turns currently held.
    pub fn len(&self) -> usize {
        self.turns.len()
    }
}
