# 📝 Changelog — Yo, Rust!

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) · Versioning: [SemVer](https://semver.org/)

---

## [2.3.2] — 2026-03-22

### 🐛 Fixed — Telemetry entries not appearing in JSONBin dashboard

Three bugs were causing the collection to stay empty:

**Bug 1 — Detached thread killed at process exit (critical)**
The previous code used a fire-and-forget `thread::spawn` whose handle was
immediately dropped.  When the user exited yo-rust (Ctrl-D, `!exit`, or even
just closing the terminal) the process terminated and killed all background
threads before any HTTP request completed.  A network POST to JSONBin takes
~200–800 ms; a typical REPL session exit is faster than that.

Fix: `submit_background()` now returns `Option<JoinHandle<()>>`.  The main
REPL loop stores all handles in `pending_telemetry: Vec<JoinHandle<()>>`.  At
every exit point (Ctrl-D, Ctrl-C, `!exit`) we call `h.join()` on each handle
before returning.  The HTTP requests complete before the process exits.

**Bug 2 — Silent error swallowing made debugging impossible**
The previous `submit()` function swallowed all errors silently.  A bad header,
quota exhausted, or network error all produced identical behaviour: nothing.

Fix: Added `YODEBUG=1` debug mode.  Set `YODEBUG=1` in your environment and
run `yo` to see the full JSON payload and HTTP response for every telemetry
request printed to stderr.  Without `YODEBUG`, all telemetry remains silent.

**Bug 3 — `submit` function had a logic error on success tracking**
The central destination branch had a comment `let _ = format!(...)` path that
never set `posted_any = true` on HTTP 200.  The code path was:
```rust
Ok(resp) if resp.status().is_success() => { posted_any = true; }   // OK
Ok(resp) => { let _ = format!(...); }  // error — never set posted_any
```
But the debug path consumed the response body, so success checking was broken
when debug mode was added inline.  Refactored into a clean match with explicit
`posted_any = true` only on 2xx status codes.

**Additional improvements to `telemetry.rs`:**
- `submit()` signature changed: takes `&TelemetryEntry` (borrow) not owned value,
  and takes `Option<&str>` not `Option<String>` to avoid unnecessary clones
- `submit_sync_report()` added: synchronous version that returns a human-readable
  result string, used by `!feedback test` and personal wizard connectivity check
- `iso8601_now()` extracted as `pub` for testability; `is_leap()` extracted inline
- Full doc comments on every public item

### ✨ New — `!feedback test` subcommand

Send a live test entry synchronously and see the result immediately:
```
yo ›  !feedback test
  ◌  Sending test entry to JSONBin…
  ✔  Entry submitted successfully.
  ◈  Check your JSONBin dashboard — the entry should appear there now.
```
If it fails, the error is shown inline.  Run with `YODEBUG=1 yo` for full
HTTP-level diagnostics.

### 📚 README updates

- "What's new in v2.0.0" renamed to "v2 — What changed in the major version"
  so it reads as a milestone summary rather than a version-specific section
- Inline changelog heading also updated to "v2 milestone highlights"

---

## [2.3.1] — 2026-03-22

### ✨ New — `!feedback` / `!fb` shortcut + live JSONBin credentials

**JSONBin.io collection is now live** — the central `yo-rust-telemetry` collection
is configured and accepting entries.  The write-only Access Key is embedded in
the binary; no setup required from users.

**New `!feedback` / `!fb` shortcut** with full subcommand UI:

| Command | What it does |
|---|---|
| `!feedback` / `!fb` | Show current telemetry status |
| `!feedback setup` | Full interactive setup wizard |
| `!feedback on` | Enable community sharing in one word |
| `!feedback off` | Disable community sharing in one word |
| `!feedback personal` | Configure personal JSONBin (with live connectivity test) |
| `!feedback clear` | Remove all telemetry settings |
| `!feedback about` | Explain JSONBin and the data pipeline |

New module `src/feedback.rs`:
- `parse()` — maps `!feedback <sub>` and `!fb <sub>` to `FeedbackCommand` enum
- `dispatch()` — runs the appropriate action, returns `bool` (config changed?)
- Personal JSONBin wizard includes a live connectivity test: creates a bin,
  verifies HTTP 200, then immediately deletes the test entry
- `!feedback personal` retains current key if Enter is pressed (non-destructive)
- `run_clear()` asks for confirmation before wiping settings

`ui.rs` additions:
- `print_feedback_status()` — colour-coded status panel with all active settings
- `print_feedback_about()` — plain-English explanation of JSONBin and data flow
- Help screen updated with all `!feedback` / `!fb` subcommands
- Intro shows "Community sharing: ON (type !feedback to manage)" when active

`src/telemetry.rs`:
- `CENTRAL_ACCESS_KEY` and `CENTRAL_COLLECTION_ID` now contain real credentials
- Collection `yo-rust-telemetry` created 2026-03-22
- `central_is_configured()` now returns `true`

---

## [2.3.0] — 2026-03-22

### ✨ New — Community data sharing & personal command history (JSONBin.io)

yo-rust can now optionally record successful prompt → command pairs and send
them to JSONBin.io.  This powers two things:

**1. Central community dataset (Paul Fleury's collection)**
  - A write-only Access Key is embedded in the binary.  It has `Bins Create`
    permission ONLY — no read, no update, no delete.
  - Users POST to it; Paul reads the collection from his JSONBin dashboard.
  - Each entry is a separate private bin — users can't see each other's data.
  - Paul reviews the data weekly and iterates on the system prompt.
  - **Default: OFF** — explicitly opt-in, never on without consent.

**2. User's own personal JSONBin (optional)**
  - Users can provide their own JSONBin Master Key + Collection ID.
  - All their entries go to their own private account, only they can read it.
  - Useful for personal command analytics / history review.
  - Completely independent of the central collection.

**What is collected:**
  - Natural-language prompt that ran
  - Shell commands that executed
  - AI model + backend used
  - OS, architecture, shell kind
  - Whether it worked (`true`/`false` from the feedback prompt)
  - yo-rust version
  - UTC timestamp

**What is NEVER collected:**
  - API keys — never
  - File contents or paths
  - Current working directory
  - Command output
  - Username, hostname, or any identity

**Opt-in flow:**
  - Asked once during first-run setup (new users) or `!api` (existing users)
  - Gentle reminder every 10 sessions if telemetry is off (type `!api` to configure)
  - Can be turned off at any time via `!api`

**Implementation details:**
  - New module `src/telemetry.rs`
  - Submission is fire-and-forget in a background thread — never blocks the REPL
  - Failures are silently ignored — a network error never interrupts the session
  - Config fields: `telemetry_share_central`, `telemetry_user_key`,
    `telemetry_user_collection`, `sessions_since_telemetry_prompt`
  - All new fields have `#[serde(default)]` — existing configs load without error
  - JSONBin.io API: `POST /v3/b` with `X-Bin-Private: true` + `X-Collection-Id`

**For Paul — setup required before v2.3.0 release:**
  1. Create JSONBin.io account at https://jsonbin.io
  2. Create a Collection named `yo-rust-telemetry`
  3. Create an Access Key with **Bins Create permission ONLY**
  4. Replace `CENTRAL_ACCESS_KEY` and `CENTRAL_COLLECTION_ID` constants
     in `src/telemetry.rs` with the real values
  5. The binary is safe to distribute — the Access Key is write-only

---

## [2.2.0] — 2026-03-22

### 🐛 Fixed — Windows PS5.1 `TerminatingError` on `cargo build`

Root cause (Wayne's machine: Windows 11, PowerShell 5.1.26100.7920 Desktop edition):

In PowerShell 5.1, any output to `stderr` from a native executable is captured
as an `ErrorRecord` object.  `cargo.exe` writes all progress output
("Updating crates.io index", "Compiling foo v1.0", etc.) to **stderr** even on
a completely successful build.  The previous `install.ps1` used:

```powershell
$ErrorActionPreference = "Stop"   # ← root cause
Set-StrictMode -Version Latest
... & cargo build --release 2>&1 | Out-Null  # ← amplified the problem
```

The `2>&1` redirection converted cargo's stderr lines into `ErrorRecord` objects.
With `Stop` mode active, the very first progress line from cargo immediately
triggered a `TerminatingError` and killed the script before the build completed.

Fix applied to `install.ps1` and `update.ps1`:
- Removed `$ErrorActionPreference = "Stop"` — kept at the default `"Continue"`
- Removed `Set-StrictMode -Version Latest` — this is a script, not a module
- Removed ALL `2>&1` and `| Out-Null` from native command calls
- Let `cargo` stdout and stderr flow directly to the host (visible output)
- Check `$LASTEXITCODE` after every native command — the correct PS idiom
- Used `System.Net.WebClient` instead of `Invoke-WebRequest -OutFile` for
  binary downloads — more reliable on PS5 with slow connections
- Used `Start-Process -Wait -PassThru` for `rustup-init.exe` so exit code
  is inspectable without triggering ErrorRecord issues
- Added `Register-EngineEvent Exiting` cleanup so temp dir is always removed
- Added helpful comment block in `install.ps1` explaining the root cause for
  future contributors

### ✨ New — Named command shortcuts (Wayne's feature request)

Save any command set as a named shortcut, then replay it instantly with one
word — no AI call, no confirmation prompt.

**Usage:**
```
yo ›  docker restart mycontainer
  ✔  Done.
  Did that work? [Y/n] › Y

yo ›  !save restartdocker
  ✔  Saved as !restartdocker

# Later, any time:
yo ›  !restartdocker
  ◈  Running shortcut !restartdocker
  ►  docker restart mycontainer
  ✔  Done.
```

- `!save <name>` — save last confirmed command(s) under this name
- `!<name>` — run the saved shortcut instantly (no AI, no confirmation)
- `!forget <name>` — remove a shortcut
- `!shortcuts` / `!sc` — list all saved shortcuts
- Shortcuts are persisted to `~/.config/yo-rust/shortcuts.json`
- Names are case-insensitive, alphanumeric + `-` + `_` only
- Multi-command shortcuts supported (saves all commands from the last run)
- Shortcuts are also recorded in the conversation context for follow-up support
- Running a shortcut appends to shell history if history is enabled

New module: `src/shortcuts.rs`

---

## [2.1.0] — 2026-03-22

### Root cause fixed

On Windows, PowerShell has a built-in alias called `curl` that maps to
`Invoke-WebRequest`. This is **not** the real curl binary. `Invoke-WebRequest`
does not accept `-fsSL` flags, so running the Unix install command:

```
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
```

...fails immediately with:
```
Invoke-WebRequest : A parameter cannot be found that matches parameter name 'fsSL'.
```

And even if the download succeeded, `bash` is not available in native Windows
PowerShell (without Git Bash or WSL2 installed), so `yo.sh` could never run.

### 🪟 New: PowerShell native scripts

- **`install.ps1`** — native PowerShell installer. Works in PS5 and PS7 without
  Git Bash, WSL, or the real curl binary. Does everything `yo.sh` does:
  detects existing install and version, downloads `rustup-init.exe` and installs
  Rust if missing, downloads source ZIP, builds release binary, installs to
  `%LOCALAPPDATA%\yo-rust\bin\yo.exe`, adds to user PATH, adds `yo`/`hi`/`hello`
  aliases to `$PROFILE`.
  Install command: `iwr -useb https://raw.githubusercontent.com/paulfxyz/yo-rust/main/install.ps1 | iex`

- **`update.ps1`** — native PowerShell updater. Reads installed version from
  binary, checks latest in Cargo.toml on GitHub, early-exits if current, builds
  and replaces in-place. Config never touched.
  Update command: `iwr -useb https://raw.githubusercontent.com/paulfxyz/yo-rust/main/update.ps1 | iex`

- **`uninstall.ps1`** — native PowerShell uninstaller. Removes binary, removes
  install dir from user PATH, asks before removing config, cleans aliases from
  `$PROFILE` using regex replacement.
  Uninstall command: `iwr -useb https://raw.githubusercontent.com/paulfxyz/yo-rust/main/uninstall.ps1 | iex`

### 📚 Documentation

- **README** — install section now shows macOS/Linux vs Windows commands side by
  side at the very top. Prominent warning explains the `curl` alias issue.
  Windows section updated with `install.ps1` as option 1.
- **INSTALL.md** — Windows section fully rewritten: all three options (PS native,
  Git Bash, WSL2), troubleshooting row for the curl alias error.
- **Code structure table** — lists all 6 scripts (3 Unix + 3 PowerShell).

---

## [2.0.0] — 2026-03-22

This is a major release.  All v1.x config files are forward-compatible — new fields
default to sensible values when missing.

### ✨ New Features

#### 🏠 Ollama backend — local, private, offline
- New backend option: `ollama`.  Routes requests to a local Ollama instance
  (`http://localhost:11434` by default, configurable).
- No API key required.  No outbound network traffic.  Complete privacy.
- Supports any Ollama model: `llama3.2`, `mistral`, `codellama`, `qwen2.5-coder`, etc.
- Full setup wizard: backend selection, Ollama URL, model name.
- Natural-language triggers: "use ollama", "use openrouter", "change backend" all
  detected client-side before any API call.
- 120-second timeout (vs 60 for OpenRouter) — local inference can be slower.
- Error messages include actionable hints: "Is Ollama running? Try: ollama serve"

#### 🔁 Multi-turn conversation context
- New module `context.rs`: rolling window of the last N confirmed prompt/command pairs.
- Default window size: 5 turns (configurable in setup or `config.json`).
- Prior turns are injected as `user`/`assistant` message pairs before the current
  prompt — the AI can resolve pronouns ("them", "it") and relative references
  ("now for staging", "same but without -r").
- REPL prompt shows turn count: `yo [+3] ›` so users know context is active.
- `!context` / `!ctx` shortcut: inspect the full context window at any time.
- `!clear` shortcut: reset context for a fresh session.
- `--no-context` CLI flag: disable for a single session without changing config.
- Context is in-memory only — not persisted to disk (session-specific, privacy).

#### 📜 Shell history integration
- New module `history.rs`: appends confirmed commands to the shell history file.
- Format per shell:
  - **zsh**: `: <unix_timestamp>:0;<command>` (EXTENDED_HISTORY format)
  - **bash**: `<command>` one per line (plain format)
  - **fish**: `- cmd: <command>\n  when: <timestamp>` (YAML-like)
- Respects `$ZDOTDIR`, `$HISTFILE`, and XDG data directories.
- History appending enabled by default; can be disabled via `!api` or `--no-history`.
- Non-fatal: a failed write logs a warning but never interrupts the session.

#### 🧪 Dry-run mode
- `--dry` / `-d` CLI flag: suggest commands without ever executing them.
- Command box rendered in **yellow** with `[dry-run — not executed]` label.
- AI is still called normally — useful for previewing destructive operations.
- Context is updated in dry-run mode so follow-up prompts still work.
- Shell history is not appended in dry-run mode.
- Banner shows `DRY-RUN MODE` badge when active.

#### 🪝 Post-execution feedback loop
- After commands run, yo-rust asks "Did that work? [Y/n]".
- **Y / Enter** → record turn in context, append to shell history, continue.
- **N** → ask "What went wrong?" → build a refinement prompt including the original
  commands and the failure description → call AI → show refined suggestion → confirm again.
- Prompt wording adapts: if a command exited non-zero, the default shifts to N
  (explicit confirmation required that it worked anyway).
- Ctrl-D at the feedback prompt = "yes, done" (safe default, clean exit path).

#### 🐚 Precise shell detection — new module `shell.rs`
- Detects the full shell matrix: zsh, bash, fish, sh/dash, PowerShell 5, PowerShell 7,
  cmd.exe, Git Bash (MSYS2 on Windows).
- Detection uses: `$SHELL` (Unix), `$PSModulePath` (PowerShell presence + version),
  `$MSYSTEM` (Git Bash), `$COMSPEC` (cmd.exe fallback).
- Context string now includes `SHELL=<kind>` and `syntax=<family>`:
  - `syntax=posix` — POSIX sh, zsh, bash, fish, sh, Git Bash
  - `syntax=powershell` — PS5 or PS7 (model avoids `&&` for PS5, allows it for PS7)
  - `syntax=cmd` — cmd.exe (model uses `&` chaining, `%VAR%` expansion)
- System prompt updated with explicit syntax rules per family.
- Execution dispatched via `shell::run_in_shell()` — correct program and args per shell:
  - POSIX: `sh -c "<cmd>"`
  - PowerShell 5: `powershell -NoProfile -Command "<cmd>"`
  - PowerShell 7: `pwsh -NoProfile -Command "<cmd>"`
  - cmd.exe: `cmd /C "<cmd>"`
  - fish: `fish -c "<cmd>"`

#### 🪟 Windows support
- Command execution uses detected shell (PS5/PS7/cmd.exe/Git Bash) instead of
  a hardcoded `#[cfg(target_os = "windows")]` / `#[cfg(not)]` branch.
- Config path resolves to `%APPDATA%\yo-rust\config.json` via `dirs` crate.
- Context string falls back to `$COMSPEC` if `$SHELL` is not set (Windows native).
- Build guide added to README and INSTALL.md: winget + cargo, PowerShell installer path.

#### 🗂️ New shortcuts
- `!context` / `!ctx` — print the full conversation context (prompts + commands).
- `!clear` — reset context window to empty.

#### 📟 New CLI flags (`cli.rs` with clap)
- `--dry` / `-d` — dry-run mode for the session
- `--no-history` — disable history appending for the session
- `--no-context` — disable multi-turn context for the session
- `--help` / `--version` — standard clap output

### 🔧 Improvements

- **Config struct** extended with: `backend`, `ollama_url`, `history_enabled`,
  `context_size`.  All use `#[serde(default)]` — v1.x configs load without error.
- **Setup wizard** restructured: backend selection first, then OpenRouter or Ollama
  sub-wizard, then history preference, then context size.
- **Intro display** shows active backend, dry-run status, history and context state.
- **Help screen** shows session status (backend, dry-run, history, context size)
  and documents all new shortcuts and flags.
- **REPL prompt** shows context turn count: `yo [+3] ›`.
- **Ollama error messages** include model pull hint on 404.
- **HTTP client** now has explicit timeouts: 60 s (OpenRouter), 120 s (Ollama).

### 📦 New modules

| Module | Purpose |
|---|---|
| `shell.rs` | Shell kind detection and cross-platform execution dispatch |
| `context.rs` | Rolling conversation context window |
| `history.rs` | Shell history file appending (zsh/bash/fish) |
| `cli.rs` | clap-based CLI argument parsing |

### 📚 Documentation

- README fully rewritten for v2.0.0: Ollama section, dry-run section, Windows
  install guide, shell detection table, multi-turn context explanation, updated
  engineering notes.
- INSTALL.md: Windows installation options, updated troubleshooting table.
- CHANGELOG.md: this entry.

---

## [1.1.3] — 2026-03-22

### 🐛 Fixed
- **Critical: uninstall.sh prompt always fired "Cancelled"** when run via `curl | bash`.
  Root cause: `read -r reply` read from the pipe not the terminal.
  Fix: all prompts use `read -r reply </dev/tty`.
- Prompt shows `[Y/N]` (uppercase both) for main confirm; `[y/N]` only for optional steps.
- Removed Unicode characters from all shell scripts — pure ASCII for portability.
- `echo -e` replaced with `printf` throughout.
- `trap 'rm -rf "$TMP_DIR"' EXIT` added to `yo.sh` and `update.sh`.

---

## [1.1.2] — 2026-03-22

### ✨ New
- `update.sh` — dedicated update script with version detection and early-exit.
- `uninstall.sh` — full removal with prompts for config and alias cleanup.
- `yo.sh` improved — detects existing install, prints update/uninstall one-liners.
- `INSTALL.md` rewritten — single reference for all operations.

---

## [1.1.1] — 2026-03-22

### 🐛 Fixed
- Default model reverted to `openai/gpt-4o-mini` (free Llama tier hits rate limits).
- Model menu reordered; free Llama moved to position 5 with rate-limit note.

---

## [1.1.0] — 2026-03-22

### 📚 Improvements
- Deep source annotations across all four modules.
- `VERSION` const in `ui.rs` as single source of truth for banner version.
- System prompt tightened (POSIX sh explicit, avoid bash-isms).
- Help screen expanded with macOS/Linux config paths and ↑/↓ history note.

---

## [1.0.0] — 2026-03-22

### 🌟 Initial Release
- Core REPL loop via `yo`, `hi`, or `hello`
- ASCII robot banner + YO, RUST! block-letter logo
- OpenRouter API integration with strict JSON envelope
- Y/N confirmation before any command executes
- First-run interactive setup (API key + model)
- Context injection (OS, arch, CWD, shell)
- Regex-based intent detection for API config changes
- Shortcuts: `!help`, `!api`, `!exit`
- Shell aliases via installer
- One-command installer (`yo.sh`) with auto Rust install
- MIT License
