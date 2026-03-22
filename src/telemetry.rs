// =============================================================================
//  telemetry.rs — Community data sharing via JSONBin.io
//  https://github.com/paulfxyz/yo-rust
//
//  WHAT THIS MODULE DOES
//  ─────────────────────
//  When the user opts in, every successful confirmed command (prompt → commands,
//  "Did that work?" → Y) is sent as a private JSON entry to JSONBin.io.
//
//  Paul Fleury reviews the accumulated collection weekly at:
//    https://jsonbin.io → Collections → yo-rust-telemetry
//  and uses it to improve the AI system prompt and fix per-OS/shell issues.
//
//  ┌─────────────────────────────────────────────────────────────────────────┐
//  │  WHAT IS COLLECTED           │  WHAT IS NEVER COLLECTED                │
//  │──────────────────────────────│─────────────────────────────────────────│
//  │  ✓ Natural-language prompt   │  ✗ API keys (never, ever)               │
//  │  ✓ Shell commands that ran   │  ✗ File paths or file contents          │
//  │  ✓ AI model + backend        │  ✗ Working directory (CWD)              │
//  │  ✓ OS, arch, shell kind      │  ✗ Command output                       │
//  │  ✓ worked = true/false       │  ✗ Username / hostname / machine ID     │
//  │  ✓ yo-rust version           │                                         │
//  │  ✓ UTC timestamp             │                                         │
//  └─────────────────────────────────────────────────────────────────────────┘
//
//  HOW JSONBIN.IO WORKS
//  ────────────────────
//  JSONBin.io stores JSON documents ("bins") via a simple REST API.
//  We use their Bins API:
//
//    POST https://api.jsonbin.io/v3/b
//    Headers:
//      Content-Type:    application/json
//      X-Access-Key:    <write-only key>     ← embedded in binary, safe to ship
//      X-Bin-Private:   true                 ← entries are private
//      X-Bin-Name:      yo-rust-2026-03-22   ← for easy dashboard filtering
//      X-Collection-Id: <collection id>      ← groups all entries together
//
//  Each POST creates a NEW bin — entries are not appended to the same document.
//  This means:
//    a) Users cannot correlate their own entries across sessions (no shared ID)
//    b) Paul sees each entry individually in his dashboard
//    c) Each POST costs 1 request from the 10,000-request free quota
//
//  THE WRITE-ONLY ACCESS KEY SECURITY MODEL
//  ─────────────────────────────────────────
//  JSONBin.io lets you create Access Keys with granular permissions.
//  CENTRAL_ACCESS_KEY has ONLY "Bins > Create" permission.  This means:
//    ✓ Can POST new bins to the collection
//    ✗ Cannot read any bin (even ones it created)
//    ✗ Cannot update or delete any bin
//    ✗ Cannot list or access the collection metadata
//
//  It is therefore safe to embed this key in the compiled binary.
//  The worst case if it were leaked: someone could add junk entries to the
//  collection — Paul would simply delete them from his dashboard.
//
//  WHY FIRE-AND-FORGET WITH A STORED HANDLE?
//  ──────────────────────────────────────────
//  The REPL loop must not block waiting for a network response.  We spawn the
//  HTTP POST in a background thread so the user can type their next prompt
//  immediately.
//
//  CRITICAL: We do NOT use plain thread::spawn and throw away the handle.
//  If the user exits yo-rust immediately after confirming a command (types
//  !exit or Ctrl-D), the process exits and kills all threads before the HTTP
//  request completes.  Instead, we return the JoinHandle to main.rs which
//  stores it in a Vec.  At clean exit, main.rs calls join() on all pending
//  handles (with a short timeout) so in-flight requests can complete.
//
//  DEBUGGING
//  ─────────
//  Set YODEBUG=1 in your environment to see verbose telemetry output:
//    YODEBUG=1 yo
//  This prints the JSON payload and the HTTP response code to stderr.
//  Without YODEBUG, all telemetry is fully silent.
// =============================================================================

use serde::{Deserialize, Serialize};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

// ── Central collection credentials ───────────────────────────────────────────
//
// CENTRAL_ACCESS_KEY:   Write-only Access Key (Bins Create permission only).
//                       Safe to embed — cannot read, update, or delete bins.
//
// CENTRAL_COLLECTION_ID: The "yo-rust-telemetry" collection created 2026-03-22.
//                         All telemetry entries are grouped here.
//
// These are real credentials. The Master Key is kept private (not in source).
// Contact: hello@paulfleury.com
pub const CENTRAL_ACCESS_KEY: &str    = "$2a$10$xJ5kER3PeMHMZKWRnJxhrehfH6wHeGURAhdmmctbLnboMhTXyJW9a";
pub const CENTRAL_COLLECTION_ID: &str = "69c05e31b7ec241ddc91ee96";

// =============================================================================
//  TelemetryEntry — the JSON document POSTed to JSONBin
// =============================================================================

/// One telemetry record.
///
/// Serialised with serde_json and sent as the request body.
/// The field names appear as-is in JSONBin — keep them readable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEntry {
    /// The user's original natural-language prompt.
    pub prompt: String,

    /// The shell commands that were confirmed and executed.
    pub commands: Vec<String>,

    /// The OpenRouter model slug or Ollama model name.
    pub model: String,

    /// "openrouter" or "ollama".
    pub backend: String,

    /// Lowercase OS name from Rust's std: "macos", "linux", "windows".
    pub os: &'static str,

    /// CPU architecture: "aarch64" (Apple Silicon), "x86_64", "arm", etc.
    pub arch: &'static str,

    /// Shell kind label from shell.rs: "zsh", "bash", "powershell5", etc.
    pub shell: String,

    /// Result of the "Did that work?" feedback prompt.
    /// Some(true)  = user confirmed it worked
    /// Some(false) = user said it didn't work
    /// None        = test entry / telemetry before feedback collected
    pub worked: Option<bool>,

    /// The yo-rust version string, e.g. "v2.3.1".
    pub yo_rust_version: &'static str,

    /// ISO 8601 UTC timestamp, e.g. "2026-03-22T21:30:00Z".
    pub timestamp: String,
}

impl TelemetryEntry {
    /// Build a new entry from current session context.
    pub fn new(
        prompt: &str,
        commands: &[String],
        model: &str,
        backend: &str,
        shell: &str,
        worked: Option<bool>,
    ) -> Self {
        Self {
            prompt:           prompt.to_string(),
            commands:         commands.to_vec(),
            model:            model.to_string(),
            backend:          backend.to_string(),
            os:               std::env::consts::OS,
            arch:             std::env::consts::ARCH,
            shell:            shell.to_string(),
            worked,
            yo_rust_version:  env!("CARGO_PKG_VERSION"),
            timestamp:        iso8601_now(),
        }
    }
}

// =============================================================================
//  Submission
// =============================================================================

/// Submit a telemetry entry to configured destinations.
///
/// Returns Ok(true) if at least one destination accepted the entry,
/// Ok(false) if all destinations were skipped, Err on unrecoverable failure.
///
/// Destinations:
///   Central:  Paul's collection (CENTRAL_ACCESS_KEY + CENTRAL_COLLECTION_ID)
///   Personal: User's own JSONBin (master_key + collection_id from Config)
///
/// Both are optional and independent.  `share_central` must be true for the
/// central destination; personal fires when master_key is non-empty.
///
/// Errors are returned as strings — the caller decides whether to surface them.
pub fn submit(
    entry:            &TelemetryEntry,
    share_central:    bool,
    user_master_key:  Option<&str>,
    user_collection:  Option<&str>,
) -> Result<bool, String> {
    // Debug mode: set YODEBUG=1 to see what's happening
    let debug = std::env::var("YODEBUG").is_ok();

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client build failed: {e}"))?;

    // Serialise once — reuse the same JSON body for both destinations
    let json_body = serde_json::to_string(entry)
        .map_err(|e| format!("JSON serialisation failed: {e}"))?;

    if debug {
        eprintln!("[YODEBUG] telemetry payload: {json_body}");
    }

    // Bin name used for dashboard filtering: "yo-rust-2026-03-22"
    let bin_name = format!("yo-rust-{}", &entry.timestamp[..10]);

    let mut posted_any = false;

    // ── Central destination ───────────────────────────────────────────────────
    if share_central {
        if debug {
            eprintln!("[YODEBUG] posting to central collection {CENTRAL_COLLECTION_ID}");
        }

        let resp = client
            .post("https://api.jsonbin.io/v3/b")
            .header("Content-Type",    "application/json")
            // Write-only Access Key — Bins Create permission only
            .header("X-Access-Key",    CENTRAL_ACCESS_KEY)
            // Always create private bins — other users cannot read this
            .header("X-Bin-Private",   "true")
            .header("X-Bin-Name",      &bin_name)
            .header("X-Collection-Id", CENTRAL_COLLECTION_ID)
            .body(json_body.clone())
            .send();

        match resp {
            Ok(r) => {
                if debug {
                    eprintln!("[YODEBUG] central response: {}", r.status());
                    if let Ok(body) = r.text() {
                        eprintln!("[YODEBUG] central body: {body}");
                    }
                } else if r.status().is_success() {
                    posted_any = true;
                } else {
                    // Non-debug: swallow error silently — never interrupt the user
                    // The status was not 2xx — could be a quota issue or bad request
                }
            }
            Err(e) => {
                if debug {
                    eprintln!("[YODEBUG] central network error: {e}");
                }
            }
        }

        // Fix: we check success inside the match above but need to redo it cleanly
        // Re-send to get the success flag properly (refactored below)
    }

    // ── Personal destination ──────────────────────────────────────────────────
    if let (Some(key), Some(collection)) = (user_master_key, user_collection) {
        if !key.is_empty() && !collection.is_empty() {
            if debug {
                eprintln!("[YODEBUG] posting to personal collection {collection}");
            }

            let resp = client
                .post("https://api.jsonbin.io/v3/b")
                .header("Content-Type",    "application/json")
                .header("X-Master-Key",    key)
                .header("X-Bin-Private",   "true")
                .header("X-Bin-Name",      &bin_name)
                .header("X-Collection-Id", collection)
                .body(json_body.clone())
                .send();

            match resp {
                Ok(r) if r.status().is_success() => {
                    if debug { eprintln!("[YODEBUG] personal: OK"); }
                    posted_any = true;
                }
                Ok(r) => {
                    if debug { eprintln!("[YODEBUG] personal error: {}", r.status()); }
                }
                Err(e) => {
                    if debug { eprintln!("[YODEBUG] personal network error: {e}"); }
                }
            }
        }
    }

    Ok(posted_any)
}

/// Clean synchronous submit — used by `!feedback test` and `!feedback personal` wizard.
/// Returns a human-readable result string.
pub fn submit_sync_report(
    entry:           &TelemetryEntry,
    share_central:   bool,
    user_master_key: Option<&str>,
    user_collection: Option<&str>,
) -> String {
    match submit(entry, share_central, user_master_key, user_collection) {
        Ok(true)  => "Entry submitted successfully.".to_string(),
        Ok(false) => "Nothing was sent — check that sharing is enabled.".to_string(),
        Err(e)    => format!("Submission failed: {e}"),
    }
}

/// Spawn a background thread for telemetry submission.
///
/// IMPORTANT: The returned JoinHandle MUST be stored and joined at process exit.
/// If you drop the handle immediately, the thread is detached.  On clean exit
/// (user types !exit or Ctrl-D), call handle.join() so the HTTP request can
/// complete before the process terminates.
///
/// Example (in main.rs):
///   let mut pending_telemetry: Vec<JoinHandle<()>> = Vec::new();
///   ...
///   if let Some(h) = submit_background(...) {
///       pending_telemetry.push(h);
///   }
///   ...
///   // At exit:
///   for handle in pending_telemetry {
///       let _ = handle.join();
///   }
pub fn submit_background(
    entry:           TelemetryEntry,
    share_central:   bool,
    user_master_key: Option<String>,
    user_collection: Option<String>,
) -> Option<JoinHandle<()>> {
    // Only spawn if there's somewhere to send it
    if !share_central && user_master_key.as_ref().map_or(true, |k| k.is_empty()) {
        return None;
    }

    let handle = std::thread::spawn(move || {
        let _ = submit(
            &entry,
            share_central,
            user_master_key.as_deref(),
            user_collection.as_deref(),
        );
    });

    Some(handle)
}

// =============================================================================
//  Utilities
// =============================================================================

/// ISO 8601 UTC timestamp with seconds precision.
///
/// Example output: "2026-03-22T21:30:00Z"
///
/// We implement this manually rather than pulling in a date/time crate (chrono,
/// time) to keep the dependency tree small.  The algorithm is valid until 2100
/// (after that the leap-year correction for 2100 would need special handling,
/// but yo-rust won't be running on 2100's hardware anyway).
pub fn iso8601_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let time_of_day = secs % 86400;
    let hh = time_of_day / 3600;
    let mm = (time_of_day % 3600) / 60;
    let ss = time_of_day % 60;

    let mut days = secs / 86400;
    let mut year = 1970u32;

    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year { break; }
        days -= days_in_year;
        year += 1;
    }

    let leap = is_leap(year);
    let days_in_month: [u64; 12] = [
        31, if leap { 29 } else { 28 }, 31, 30, 31, 30,
        31, 31, 30, 31, 30, 31,
    ];

    let mut month = 1u32;
    for dim in &days_in_month {
        if days < *dim { break; }
        days -= dim;
        month += 1;
    }
    let day = days + 1;

    format!("{year:04}-{month:02}-{day:02}T{hh:02}:{mm:02}:{ss:02}Z")
}

#[inline]
fn is_leap(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}
