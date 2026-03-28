# 📝 Changelog — mang.sh 句芒

Format: [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) · Versioning: [SemVer](https://semver.org/)

---

## [3.0.4] — 2026-03-28

### Changed — Repository rename: `mang-sh` → `mang`

This release renames everything from `mang-sh` to `mang`.

**What changed:**
- GitHub repository: `paulfxyz/mang-sh` → `paulfxyz/mang`
  (old URL redirects automatically via GitHub's permanent redirect)
- Cargo package name: `mang-sh` → `mang`
- Config directory: `~/.config/mang-sh/` → `~/.config/mang/`
  (macOS: `~/Library/Application Support/mang-sh/` → `.../mang/`)
- Windows install directory: `%LOCALAPPDATA%\mang-sh\bin` → `%LOCALAPPDATA%\mang\bin`
- Shell alias marker: `# mang-sh aliases` → `# mang aliases`
- All installer scripts, scripts, source headers updated

**What did NOT change:**
- The binary name is still `yo` — `yo`, `hi`, `hello` all work
- The website URL is still `https://mang.sh`
- Install command is still `curl -fsSL https://mang.sh/install | bash`
- All features, shortcuts, AI backends, telemetry — unchanged

**Existing users — automatic migration:**
On first launch after updating, `config.rs` silently copies
`~/.config/mang-sh/` to `~/.config/mang/` if the new directory doesn't
exist yet.  Your API key, model choice, shortcuts, and telemetry settings
are preserved.  The old directory is kept as a backup.

---

## [3.0.3] — 2026-03-25

### Added

#### `!credits` / `!cr` — About screen

New shortcut that displays a formatted credits screen showing:

- **AUTHOR** — Paul Fleury: name, role, location, website, email, GitHub,
  LinkedIn, Twitter handle
- **PROJECT** — mang.sh 句芒, version, license, website, source, install command
- **BUILT WITH** — Rust, OpenRouter, Ollama, Perplexity Computer (AI pair programming)
- **THE NAME** — Gōu Mḁng mythology and the bridge metaphor

The screen is static (no config, no network) and uses the same box-drawing
style as the help and context summary screens.

The Perplexity Computer credit is intentional — this project was built in
genuine collaboration with AI pair programming.  The architecture decisions
are human; the implementation speed was only possible with AI assistance.
Honesty about that is a feature, not a caveat.

**Changes:**
- `src/ui.rs`: `print_credits()` function added; VERSION bumped to `v3.0.3`
- `src/main.rs`: `!credits` / `!cr` handled before shortcut dispatch
- `src/shortcuts.rs`: `!credits` / `!cr` excluded from named-shortcut parsing

#### README: major deep-dive expansion (690 → 948 lines)

The README now serves as a comprehensive technical reference, inspired by the documentation depth of [mercury-sh](https://github.com/paulfxyz/mercury-sh). New content across:

- **Architecture notes** — blocking REPL design rationale, rustyline usage,
  context window as a rolling buffer (not a log)
- **On Rust for CLI tools** — `cargo clippy -D warnings`, `#[serde(default)]`
  forward-compat pattern, blocking reqwest vs async, `dirs` crate rationale,
  `Regex` compile-once pattern
- **On cross-platform shell detection** — `ShellKind` matrix (8 variants),
  `syntax=` hint as highest-leverage context field
- **On Windows support** — expanded with PS5/PS7 detection story, the
  `$ErrorActionPreference + 2>&1 + native commands` triple-failure explanation
- **On installer design** — `curl | bash` security context, `/dev/tty` for
  piped script prompts (the root cause of the v1.1.2 uninstall bug), ANSI-C
  quoting (`$'\033'` vs `'\033'`), idempotent installer design
- **On AI-assisted development** — force multiplier framing, "compile and test,
  don't trust", documenting the "why" is human work
- **Module reference table** — all 14 source files with their exact
  responsibility boundaries
- **On telemetry** — fire-and-forget root cause analysis, `MANGDEBUG=1`
  design rationale, write-only key security model, opt-in vs opt-out
- **Table of Contents** — 19-section navigation
- **Architecture Deep Dive** — REPL design decisions, system prompt design
  rationale, JSON parsing pipeline, shell detection matrix, wizard architecture,
  `ShortcutStore` design
- **Bugs worth documenting** — 5 named bugs with root cause analysis and lessons:
  detached-thread telemetry silence, uninstall stdin trap, shell colour escaping,
  Windows PS5 `cargo stderr` trap, config forward-compatibility design
- **Building with Perplexity Computer** — honest account of the AI collaboration
  model: what the AI provided, what the human provided, and why this credit
  is in the README

---

## [3.0.2] — 2026-03-25

### Added

#### Advanced Prompt Mode (`!prompt` / `!p`)

When you're stuck — prompt too vague, AI returned nothing, or you just want
a more guided experience — Advanced Prompt Mode runs a short Socratic dialogue
to help you build the right request.

**How it works:**

Up to 3 rounds of AI-generated clarifying questions.  Each question is targeted
at the most ambiguous part of what you said.  Answer as many or as few as you
want — press Enter or type `!skip` to fire with what's been gathered so far.
Your answers are synthesised into one precise prompt that goes through the
normal `suggest_commands()` pipeline.  Nothing executes without the usual Y
confirmation.

**Trigger paths:**

| How | When |
|---|---|
| Type `!prompt` or `!p` | Any time — starts from scratch with an open first question |
| Automatic | When the AI returns no commands (couldn't interpret your request) |

**Example session:**

```
yo ›  do the docker thing

  ◌  Thinking…
  ✗  Gou Mang couldn't pin that down. Let's clarify.

  ╔═══════════════════════════════════════════════╗
  ║  ✦  Advanced Prompt Mode                      ║
  ║     I'll ask up to 3 questions to nail it.    ║
  ╚═══════════════════════════════════════════════╝

  ✦  (1/3)  What do you want Docker to do — start, stop,
             restart, view logs, or something else?

  yo ›  restart my app container

  ✦  (2/3)  What's the container name or ID?

  yo ›  myapp

  ◌  Building your command…
  ◈  Refined: do the docker thing. restart my app container. myapp.

  ◌  Thinking…

  ◈  Restarts the container named myapp.
  ┌────────────────────────────────────────────┐
  │  $  docker restart myapp                  │
  └────────────────────────────────────────────┘

  Run it? [Y/n] ›
```

**New module: `src/prompt_wizard.rs`**

The wizard is a dedicated module (`MAX_ROUNDS = 3`) with clean separation of:
- `run()` — orchestrates the dialogue loop
- `coach_prompt()` — builds the AI prompt for each clarifying question
- `synthesise()` — pure string synthesis (no AI call, always deterministic)
- UI helpers — `print_wizard_header()`, `print_wizard_question()`

**New function: `ai::suggest_raw()`**

The wizard needs freeform text (clarifying questions), not the strict JSON schema
used by `suggest_commands()`.  `suggest_raw()` calls the same backends but with
a permissive system prompt and returns the raw content string.  Temperature 0.5
(vs 0.2 for commands) produces more natural-sounding questions.

#### Lessons learned

**The "prompt coach" framing matters more than the schema.**
First attempt tried parsing the AI's clarifying question as a `Suggestion` JSON
blob and using the `explanation` field as the question text.  This was fragile —
small models sometimes wrap freeform text in JSON, sometimes don't.  The fix:
a separate `suggest_raw()` function with a completely different system prompt
that explicitly says "just ask one short question, no JSON, no markdown".

**Synthesise, don't summarise.**
The wizard doesn't ask the AI to summarise the collected context.  It just joins
the original prompt + all user answers with `". "` and sends that compound
sentence to `suggest_commands()`.  Deterministic, fast, and the downstream AI
handles disambiguation well from rich context.  An AI-assisted synthesis step
would add a network round-trip for no meaningful quality improvement.

**Auto-trigger needs a graceful escape.**
When the wizard auto-triggers on an empty suggestion, the user might not want
it — they might want to retype from scratch.  The escape is immediate: pressing
Enter on the first wizard question abandons cleanly.  No extra keystrokes, no
`Ctrl-C` required.

---

## [3.0.1] — 2026-03-23

### Changed

#### Banner redesign
The launch banner has been completely redesigned. The previous tree-motif layout
(ASCII art cosmic tree on the left, MANG/SH block letters on the right) is
replaced with a focused two-row block-letter design:

```
  ╔═══════════════════════════════════════════════╗
  ║                                               ║
  ║   句芒   ·   Gou Mang   ·   Spirit Messenger  ║
  ║                                               ║
  ║   ███╗   ███╗ █████╗ ███╗  ██╗ ██████╗        ║
  ║   ████╗ ████║██╔══██╗████╗ ██║██╔════╝        ║
  ║   ██╔████╔██║███████║██╔██╗██║██║  ███╗       ║
  ║   ██║╚██╔╝██║██╔══██║██║╚████║██║   ██║       ║
  ║   ██║ ╚═╝ ██║██║  ██║██║ ╚███║╚██████╔╝       ║
  ║   ╚═╝     ╚═╝╚═╝  ╚═╝╚═╝  ╚══╝ ╚═════╝        ║
  ║                                               ║
  ║   ██████╗ ██╗  ██╗                            ║
  ║   ██╔════╝██║  ██║                            ║
  ║   ███████╗███████║                            ║
  ║   ╚════██║██╔══██║                            ║
  ║   ███████║██║  ██║                            ║
  ║   ╚══════╝╚═╝  ╚═╝                            ║
  ║                                               ║
  ║   v3.0.1  ·  mang.sh  ·  github.com/paulfxyz  ║
  ╚═══════════════════════════════════════════════╝
```

Colour scheme:
- **Cyan** — header line (Chinese glyphs + subtitle), `MANG` block letters
- **Bold white** — `.sh` block letters
- **Dimmed** — outer box frame, footer metadata

The Chinese characters `句芒` appear first on the header line — the name in
its original form, before the romanisation. The tool is named after Gou Mang;
showing his name in Chinese is the correct presentation.

#### Uninstall script — legacy yo-rust cleanup
`uninstall.sh` now removes legacy `yo-rust` configuration directories left
behind from versions before the v3.0.0 rebrand:
- macOS: `~/Library/Application Support/yo-rust`
- Linux: `~/.config/yo-rust`

Also cleans `yo-rust aliases` marker lines from shell rc files, in addition
to the existing `mang.sh aliases` cleanup.

Config directory path corrected from `~/.config/mang.sh` to `~/.config/mang-sh`
(the correct XDG path the `dirs` crate uses on Linux).

---

## [3.0.0] — 2026-03-23

### 🏛️ Rebrand — Yo, Rust! → mang.sh (句芒)

This is a breaking rename, not a breaking code change. All features are identical
to v2.3.5. The binary is still invoked as `yo`. The config directory moves from
`~/.config/yo-rust/` to `~/.config/mang-sh/` (handled automatically by the
`dirs` crate using the new Cargo.toml package name `mang-sh`).

**The name change:**

The project started as *Yo, Rust!* — a developer pun. `yo` is the command you
type. Rust is the language underneath. Put them together: Yo, Rust! A coder
shouting at their toolchain.

It was a fine name for a side project. It was a bad name for a tool that deserves
to be taken seriously.

**Gou Mang (句芒):**

In ancient Chinese mythology, Gou Mang serves as the divine messenger between
the Emperor of Heaven and the mortal world. He carries intent across the boundary
between realms — translating the will of heaven into action on earth.

mang.sh does exactly this. You speak in human language — imprecise, contextual,
full of implicit assumptions. The shell speaks in machine language — exact syntax,
specific flags, precise operators. Gou Mang bridges the gap.

The command stays `yo` — a casual, direct summons. No ceremony. The god comes
when called. That's the right tone for a developer tool.

**What changed:**

- Package renamed `mang-sh` in `Cargo.toml`
- Binary still named `yo` (no change to how you invoke it)
- New homepage: `https://mang.sh`
- Install: `curl -fsSL https://mang.sh/install | bash`
- New banner: Gou Mang's cosmic tree + MANG.SH block-letter logotype
- JSONBin collection renamed to `mang-sh-telemetry`
- `MANGDEBUG=1` replaces `YODEBUG=1` for telemetry debugging
- All installer scripts updated with mang.sh branding and URLs
- README completely rewritten with Gou Mang mythology, deeper engineering
  context, and full lessons learned
- INSTALL.md and CHANGELOG.md completely rewritten
- Zero remaining references to the old name anywhere in the codebase

---

## [2.3.5] — 2026-03-23

### ✨ Background update check on every launch

On launch, a background thread silently fetches `Cargo.toml` from GitHub to
check for a newer version. The thread runs concurrently with the banner —
zero startup latency. If a newer version is found:

```
  ◈  Update available: v2.3.6 — type !update to install
```

Rate-limited to once per 24 hours via `~/.config/mang-sh/last_update_check`.

New shortcuts: `!update` / `!upd` / `!check` — force-checks and offers Y/N
to install. On Y, shells out to the update script and exits for a clean restart.

New module: `src/updater.rs`

### ✨ N on a suggestion = iterative refinement tunnel

Pressing N no longer abandons the session — it opens an inline refinement loop:

```
  Run it? [Y/n] › N

  ◈  Let's refine — what should be different?
  yo ›  use zip instead of tar.gz

  ◌  Thinking…

  [refined suggestion with zip]

  Run it? [Y/n] ›
```

The refinement prompt includes the original request AND the previous suggestion,
so the AI understands exactly what to change. Loop continues until Y or cancel
(blank Enter, `!skip`, Ctrl-D).

---

## [2.3.4] — 2026-03-22

### 🐛 Shell script colour variables fixed

Root cause: colour variables were single-quoted — `CYN='\\033[0;36m'` — storing
a literal backslash-033 instead of an ESC byte. `printf` printed the raw escape
sequence instead of rendering colour.

Fix: ANSI-C quoting — `CYN=$'\033[0;36m'` — stores the actual ESC byte at
assignment time. Applied to all three Unix scripts.

---

## [2.3.3] — 2026-03-22

### 🔍 Code audit — zero clippy warnings

- `telemetry.rs`: Fixed `posted_any` logic bug (debug path consumed response
  body before success check), `is_some_and()` replacing `map_or()`,
  `is_multiple_of()` replacing manual modulo in `is_leap()`
- `main.rs`: Fixed duplicate step numbering, fixed `Err(e)` readline exit path
  not joining telemetry handles
- `ui.rs`: Three `print_literal` clippy warnings resolved

---

## [2.3.2] — 2026-03-22

### 🐛 Telemetry entries not appearing in JSONBin

Three bugs causing empty collection:

1. **Detached thread race**: `submit_background()` now returns `JoinHandle`.
   Main loop stores all handles, joins at every exit point (Ctrl-D, Ctrl-C,
   `!exit`, input error). Without this, process exits before HTTP POST completes.
2. **`YODEBUG=1` mode** added: prints JSON payload and HTTP response to stderr.
3. **Success flag logic**: debug path was consuming the response body before the
   `is_success()` check, so `posted_any` was never set in debug mode.

### ✨ `!feedback test`

Sends a live entry synchronously and shows the result immediately. Useful for
verifying the pipeline before relying on it.

---

## [2.3.1] — 2026-03-22

### ✨ `!feedback` / `!fb` shortcut

Full subcommand UI: `!feedback`, `!feedback setup`, `!feedback on/off`,
`!feedback personal` (with live connectivity test), `!feedback clear`,
`!feedback about`, `!feedback test`.

JSONBin.io collection `mang-sh-telemetry` live and accepting entries.

---

## [2.3.0] — 2026-03-22

### ✨ Community telemetry via JSONBin.io

Opt-in anonymous sharing of prompt/command pairs via JSONBin.io.
Write-only Access Key embedded in binary (Bins Create permission only).
Personal JSONBin support for private command history.

---

## [2.2.0] — 2026-03-22

### 🐛 Windows PS5.1 TerminatingError on `cargo build`

Root cause: `$ErrorActionPreference = "Stop"` + `Set-StrictMode` + `2>&1`
caused `cargo`'s normal stderr progress output to trigger `TerminatingError`.

Fix: removed all three. Let cargo output flow to terminal. Check `$LASTEXITCODE`.

### ✨ Named command shortcuts

`!save <name>` · `!<name>` (instant replay) · `!forget <name>` · `!shortcuts`

---

## [2.1.0] — 2026-03-22

### ✨ Native PowerShell installer

`install.ps1`, `update.ps1`, `uninstall.ps1` — no Git Bash or WSL required.
Fixes the `curl -fsSL ... | bash` failure in Windows PowerShell where `curl`
is an alias for `Invoke-WebRequest`.

---

## [2.0.0] — 2026-03-22 · Major version milestone

- 🏠 **Ollama backend** — local inference, no API key, offline
- 🔁 **Multi-turn context** — follow-up prompts resolve correctly
- 📜 **Shell history** — zsh/bash/fish native format appending
- 🧪 **Dry-run** — `yo --dry` with yellow command box
- 🪝 **Post-execution feedback** — "Did that work?" refinement loop
- 🐚 **Shell detection** — zsh, bash, fish, sh, PS5, PS7, cmd.exe, Git Bash
- 🪟 **Windows** — cmd.exe and PowerShell execution, PS5/PS7 syntax
- 🗂️ `!context` / `!clear` · `--no-history` · `--no-context` flags

---

## [1.1.3] — 2026-03-22

### 🐛 Uninstall script prompt fix

Root cause: `read -r reply` read from the pipe (script content) not the
terminal when run via `curl | bash`. Fix: `read -r reply </dev/tty`.
Also: `echo -e` → `printf`, pure ASCII in shell scripts, `trap` for cleanup.

---

## [1.0.0] — 2026-03-22 · Initial release

- Core REPL via `yo`, `hi`, or `hello`
- OpenRouter API with JSON envelope
- Y/N confirmation, first-run setup, context injection
- Regex intent detection, `!help`, `!api`, `!exit`
- One-command installer with auto Rust install
- MIT License
