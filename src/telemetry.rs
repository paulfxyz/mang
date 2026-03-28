// =============================================================================
//  telemetry.rs — Community data sharing via JSONBin.io
//  https://github.com/paulfxyz/mang
//
//  WHAT THIS MODULE DOES
//  ─────────────────────
//  When the user opts in, every successful confirmed command (prompt → commands,
//  "Did that work?" → Y) is POSTed as a private JSON entry to JSONBin.io.
//
//  Paul Fleury reviews the accumulated collection at:
//    https://jsonbin.io → Collections → mang-telemetry
//  and uses it to improve the AI system prompt and fix per-OS/shell issues.
//
//  ┌──────────────────────────────────────────┬────────────────────────────────┐
//  │  WHAT IS COLLECTED (opt-in only)         │  WHAT IS NEVER COLLECTED       │
//  ├──────────────────────────────────────────┼────────────────────────────────┤
//  │  ✓ Natural-language prompt               │  ✗ API keys (never, ever)      │
//  │  ✓ Shell commands that ran               │  ✗ File paths or contents      │
//  │  ✓ AI model + backend                    │  ✗ Working directory (CWD)     │
//  │  ✓ OS, arch, shell kind                  │  ✗ Command output              │
//  │  ✓ worked = true / false                 │  ✗ Username / hostname / IP    │
//  │  ✓ mang.sh version                       │                                │
//  │  ✓ UTC timestamp                         │                                │
//  └──────────────────────────────────────────┴────────────────────────────────┘
//
//  HOW JSONBIN.IO WORKS
//  ────────────────────
//  JSONBin.io stores JSON documents ("bins") via a simple REST API.
//  Each telemetry entry is a separate new bin:
//
//    POST https://api.jsonbin.io/v3/b
//    Headers:
//      Content-Type:    application/json
//      X-Access-Key:    <write-only key>     — embedded in binary, safe to ship
//      X-Bin-Private:   true                 — entries are private
//      X-Bin-Name:      mang-2026-03-22   — for easy dashboard filtering
//      X-Collection-Id: <collection id>      — groups all entries together
//
//  Each POST creates a NEW bin (document) — not appended to an existing one.
//  This means each entry is independent and cannot be correlated across users.
//
//  THE WRITE-ONLY ACCESS KEY SECURITY MODEL
//  ─────────────────────────────────────────
//  CENTRAL_ACCESS_KEY has ONLY "Bins Create" permission. It cannot:
//    ✗ Read any bin (even ones it created)
//    ✗ Update or delete any bin
//    ✗ List or access collection metadata
//  Therefore it is safe to embed in the compiled binary.
//
//  THREAD-JOIN CONTRACT
//  ────────────────────
//  submit_background() returns Option<JoinHandle<()>>.
//  The caller (main.rs) stores all handles in pending_telemetry: Vec<JoinHandle<()>>.
//  At every exit point (Ctrl-D, Ctrl-C, !exit), main.rs calls h.join() on each
//  handle before returning.  This ensures in-flight HTTP requests complete even
//  when the user exits immediately after confirming a command.
//
//  Without join():  process exits in ~10ms → thread killed → entry lost.
//  With join():     process waits for thread (~200–800ms) → entry delivered.
//
//  DEBUGGING
//  ─────────
//  Set MANGDEBUG=1 to see verbose output on stderr:
//    MANGDEBUG=1 yo
//  Prints the JSON payload and HTTP response code for every telemetry request.
// =============================================================================

use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Central collection credentials ───────────────────────────────────────────
// Write-only Access Key: Bins Create permission only — safe to ship in binary.
// Collection: mang-telemetry (created 2026-03-22).
// Master Key: kept private, never in source. Contact: hello@paulfleury.com
pub const CENTRAL_ACCESS_KEY: &str    = "$2a$10$xJ5kER3PeMHMZKWRnJxhrehfH6wHeGURAhdmmctbLnboMhTXyJW9a";
pub const CENTRAL_COLLECTION_ID: &str = "69c05e31b7ec241ddc91ee96";

// =============================================================================
//  TelemetryEntry — the JSON document POSTed to JSONBin
// =============================================================================

/// One telemetry record: a prompt that worked, with context about the environment.
///
/// All fields are serialised as-is into JSON. Field names are kept readable
/// so Paul can filter and analyse the collection in the JSONBin dashboard
/// without needing a schema reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEntry {
    /// The user's original natural-language prompt.
    pub prompt: String,

    /// The shell commands that were confirmed and executed.
    pub commands: Vec<String>,

    /// The AI model slug (e.g. "openai/gpt-4o-mini") or Ollama model name.
    pub model: String,

    /// "openrouter" or "ollama".
    pub backend: String,

    /// OS from Rust's std::env::consts: "macos", "linux", "windows".
    pub os: &'static str,

    /// CPU architecture: "aarch64" (Apple Silicon), "x86_64", "arm", etc.
    pub arch: &'static str,

    /// Shell kind from shell.rs: "zsh", "bash", "powershell5", "cmd.exe", etc.
    pub shell: String,

    /// Result of the "Did that work?" feedback prompt.
    ///   Some(true)  = user confirmed it worked
    ///   Some(false) = user said it didn't work
    ///   None        = test entry
    pub worked: Option<bool>,

    /// mang.sh version string, populated at compile time via env!().
    pub mang_version: &'static str,

    /// ISO 8601 UTC timestamp, e.g. "2026-03-22T21:30:00Z".
    pub timestamp: String,
}

impl TelemetryEntry {
    /// Construct a new entry from current session context.
    pub fn new(
        prompt:   &str,
        commands: &[String],
        model:    &str,
        backend:  &str,
        shell:    &str,
        worked:   Option<bool>,
    ) -> Self {
        Self {
            prompt:          prompt.to_string(),
            commands:        commands.to_vec(),
            model:           model.to_string(),
            backend:         backend.to_string(),
            os:              std::env::consts::OS,
            arch:            std::env::consts::ARCH,
            shell:           shell.to_string(),
            worked,
            mang_version: env!("CARGO_PKG_VERSION"),
            timestamp:       iso8601_now(),
        }
    }
}

// =============================================================================
//  submit() — core HTTP submission function
//
//  Takes an entry and sends it to one or both destinations.
//  Returns Ok(true) if at least one destination accepted it.
//  Returns Ok(false) if all destinations were skipped (disabled/not configured).
//  Returns Err(String) only on unrecoverable infrastructure failure.
//
//  All HTTP errors (non-2xx, network timeout, quota exceeded) are handled
//  gracefully — they return Ok(false) not Err().  We never want a telemetry
//  failure to propagate as an error to the caller.
// =============================================================================
pub fn submit(
    entry:           &TelemetryEntry,
    share_central:   bool,
    user_master_key: Option<&str>,
    user_collection: Option<&str>,
) -> Result<bool, String> {
    let debug = std::env::var("MANGDEBUG").is_ok();

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client build failed: {e}"))?;

    // Serialise once — same body reused for both destinations.
    let json_body = serde_json::to_string(entry)
        .map_err(|e| format!("JSON serialisation failed: {e}"))?;

    if debug {
        eprintln!("[MANGDEBUG] payload:\n{json_body}");
    }

    // Bin name: "mang-2026-03-22" — used for dashboard filtering by date.
    // Slicing [..10] is safe: iso8601_now() always produces at least 10 chars.
    let bin_name = format!("mang-{}", &entry.timestamp[..10]);

    let mut posted_any = false;

    // ── Central destination (Paul's collection) ───────────────────────────────
    if share_central {
        if debug {
            eprintln!("[MANGDEBUG] → central collection {CENTRAL_COLLECTION_ID}");
        }

        let result = client
            .post("https://api.jsonbin.io/v3/b")
            .header("Content-Type",    "application/json")
            .header("X-Access-Key",    CENTRAL_ACCESS_KEY)
            .header("X-Bin-Private",   "true")
            .header("X-Bin-Name",      &bin_name)
            .header("X-Collection-Id", CENTRAL_COLLECTION_ID)
            .body(json_body.clone())
            .send();

        match result {
            Ok(r) => {
                let status = r.status();
                // Read body once (consumes response), only in debug mode.
                if debug {
                    let body = r.text().unwrap_or_default();
                    eprintln!("[MANGDEBUG] central HTTP {status}: {body}");
                }
                if status.is_success() {
                    posted_any = true;
                }
            }
            Err(e) => {
                if debug {
                    eprintln!("[MANGDEBUG] central network error: {e}");
                }
                // Network errors are silently swallowed — never interrupt the user.
            }
        }
    }

    // ── Personal destination (user's own JSONBin) ─────────────────────────────
    if let (Some(key), Some(collection)) = (user_master_key, user_collection) {
        if !key.is_empty() && !collection.is_empty() {
            if debug {
                eprintln!("[MANGDEBUG] → personal collection {collection}");
            }

            let result = client
                .post("https://api.jsonbin.io/v3/b")
                .header("Content-Type",    "application/json")
                .header("X-Master-Key",    key)
                .header("X-Bin-Private",   "true")
                .header("X-Bin-Name",      &bin_name)
                .header("X-Collection-Id", collection)
                .body(json_body.clone())
                .send();

            match result {
                Ok(r) => {
                    let status = r.status();
                    if debug {
                        let body = r.text().unwrap_or_default();
                        eprintln!("[MANGDEBUG] personal HTTP {status}: {body}");
                    }
                    if status.is_success() {
                        posted_any = true;
                    }
                }
                Err(e) => {
                    if debug {
                        eprintln!("[MANGDEBUG] personal network error: {e}");
                    }
                }
            }
        }
    }

    Ok(posted_any)
}

// =============================================================================
//  submit_sync_report() — synchronous submit for !feedback test
//
//  Runs synchronously (no thread) and returns a human-readable result string.
//  Used by the !feedback test subcommand and personal wizard connectivity check.
// =============================================================================
pub fn submit_sync_report(
    entry:           &TelemetryEntry,
    share_central:   bool,
    user_master_key: Option<&str>,
    user_collection: Option<&str>,
) -> String {
    match submit(entry, share_central, user_master_key, user_collection) {
        Ok(true)  => "Entry submitted successfully.".to_string(),
        Ok(false) => "Nothing sent — sharing may be disabled or no destination configured.".to_string(),
        Err(e)    => format!("Submission failed: {e}"),
    }
}

// =============================================================================
//  submit_background() — spawn a background thread for telemetry submission
//
//  Returns Option<JoinHandle<()>>.
//
//  IMPORTANT: The caller MUST store this handle and join it at process exit.
//  See the "Thread-Join Contract" comment at the top of this file.
//
//  Returns None if there is nowhere to send the entry (both destinations
//  disabled/unconfigured), avoiding unnecessary thread spawning.
// =============================================================================
pub fn submit_background(
    entry:           TelemetryEntry,
    share_central:   bool,
    user_master_key: Option<String>,
    user_collection: Option<String>,
) -> Option<JoinHandle<()>> {
    // Early return: nothing to send.
    let has_personal = user_master_key.as_deref().is_some_and(|k| !k.is_empty());
    if !share_central && !has_personal {
        return None;
    }

    Some(std::thread::spawn(move || {
        let _ = submit(
            &entry,
            share_central,
            user_master_key.as_deref(),
            user_collection.as_deref(),
        );
    }))
}

// =============================================================================
//  iso8601_now() — current UTC time as an ISO 8601 string
//
//  Returns e.g. "2026-03-22T21:30:00Z" (seconds precision).
//
//  We implement this manually to avoid adding a date/time crate (chrono, time)
//  as a dependency.  The algorithm is correct until 2100.
// =============================================================================
pub fn iso8601_now() -> String {
    let total_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Split into time-of-day and full days since epoch
    let time_of_day = total_secs % 86_400;
    let hh = time_of_day / 3_600;
    let mm = (time_of_day % 3_600) / 60;
    let ss = time_of_day % 60;

    // Walk forward year by year from 1970
    let mut remaining_days = total_secs / 86_400;
    let mut year = 1970u32;
    loop {
        let days_in_year: u64 = if is_leap(year) { 366 } else { 365 };
        if remaining_days < days_in_year { break; }
        remaining_days -= days_in_year;
        year += 1;
    }

    // Walk forward month by month within the year
    let days_in_month: [u64; 12] = [
        31, if is_leap(year) { 29 } else { 28 }, 31, 30, 31, 30,
        31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u32;
    for dim in &days_in_month {
        if remaining_days < *dim { break; }
        remaining_days -= dim;
        month += 1;
    }
    let day = remaining_days + 1;

    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

/// Returns true if `year` is a Gregorian leap year.
/// Valid until 2100 (which is not a leap year and would need special handling,
/// but mang.sh won't be running in 2100).
#[inline]
fn is_leap(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}
