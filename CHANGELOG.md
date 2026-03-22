# 📝 Changelog — Yo, Rust!

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) · Versioning: [SemVer](https://semver.org/)

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
