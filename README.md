# mang.sh 句芒 🥷

<div align="center">

**The spirit messenger between you and your shell.**

*Describe what you need. Gou Mang delivers the command. You confirm. It executes.*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Powered by OpenRouter](https://img.shields.io/badge/Powered%20by-OpenRouter-6c47ff?style=for-the-badge)](https://openrouter.ai)
[![Ollama](https://img.shields.io/badge/Supports-Ollama-black?style=for-the-badge)](https://ollama.ai)
[![Version](https://img.shields.io/badge/Version-3.0.4-brightgreen?style=for-the-badge)](CHANGELOG.md)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-blue?style=for-the-badge)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge)](https://github.com/paulfxyz/mang/pulls)

<a href="https://paulfleury.com/github/mang-sh.jpeg">
  <img src="https://paulfleury.com/github/mang-sh.jpeg" alt="The All Seeing Eye — domain monitor dashboard" width="700" />
</a>

*Click image to view full resolution*

</div>

---

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
  ║   v3.0.4  ·  mang.sh  ·  github.com/paulfxyz  ║
  ╚═══════════════════════════════════════════════╝
```

---

> This README is a deeply technical reference — it documents not just *how* to use mang.sh, but *why* every architectural decision was made, every bug that was hit, and every lesson learned building it across 13 versions. If you're building a similar tool or want to understand how a production-grade AI CLI is architected in Rust, read on.

---

## Table of Contents

1. [Gou Mang — the myth](#-gou-mang-the-spirit-messenger)
2. [Why this exists](#-why-this-exists)
3. [Why Rust](#-why-rust--a-serious-answer)
4. [Feature overview](#-feature-overview)
5. [Install](#-install)
6. [See it in action](#-see-it-in-action)
7. [Ollama — local, private, offline](#-ollama--local-private-offline)
8. [Named shortcuts](#-named-shortcuts--instant-command-replay)
9. [All shortcuts](#️-all-shortcuts)
10. [Code structure](#-code-structure)
11. [Architecture deep dive](#-architecture--how-it-works)
12. [Bugs worth documenting](#-bugs--the-ones-worth-documenting)
13. [Building with Perplexity Computer](#-building-with-perplexity-computer)
14. [Lessons learned](#-lessons-learned--building-mangsh-from-scratch)
15. [Community telemetry](#-community-data--telemetry)
16. [Model recommendations](#-openrouter-model-recommendations)
17. [Changelog](#-changelog)
18. [Contributing](#-contributing)
19. [Author](#-author)

---

## 句芒 — Gou Mang, the Spirit Messenger

In ancient Chinese mythology, **Gou Mang (句芒)** served as the divine messenger between the Emperor of Heaven and the mortal world. He carried intent across the boundary between realms — translating the will of heaven into action on earth.

mang.sh is named in his honour.

You type what you want in plain English. Gou Mang translates that intent into the exact shell command your machine understands. You confirm — he executes. No ceremony. No Stack Overflow. No translation tax.

The command to invoke him is `yo` — casual, direct, immediate. The way you'd ask a friend. No incantation required.

> *"Gou Mang carried messages between the Emperor of Heaven and the mortal world. Now he carries yours — between human intent and machine command."*

---

## 👨‍💻 Why this exists

I'm **Paul Fleury** — French internet entrepreneur based in Lisbon. I run [Openline](https://openline.ai) and manage infrastructure across multiple products: DNS, Docker stacks, reverse proxies, SSL certs, CI/CD pipelines, API integrations, cron jobs. The full sysadmin surface.

And I kept hitting the same wall — not the hard problems, but the *tedious* ones.

The `find` flags for deleting files older than 7 days. The `rsync` invocation that syncs without wiping the destination. The `awk` one-liner for column 3 of a log. The `openssl` command that decodes a cert. The `lsof` incantation to kill port 3000. Things I've done dozens of times but never fully memorised because I don't type them *every single day*.

Every time: stop → open browser → Google it → skip ads → scan Stack Overflow → adapt the 2016 answer → test it. **Five minutes gone.** Ten times a day. That's an hour of command-syntax archaeology, daily.

I wanted something that felt like messaging a developer friend who knows Linux cold. Describe the thing. Get the command. Run it.

Three constraints I set from day one:

1. **No runtime.** One compiled binary. Works on any machine, forever, without installation ceremonies.
2. **One command to install.** `curl | bash`. Even Rust installs automatically.
3. **Any AI model.** OpenRouter for cloud (GPT-4o, Claude, Llama). Ollama for offline, air-gapped, private.

Rust answered all three. This document explains why.

> 💡 Designed and built in collaboration with **[Perplexity Computer](https://www.perplexity.ai)** — architecture through implementation, debugging, and documentation. Human intent + AI execution.

---

## 🦀 Why Rust — a serious answer

### The binary distribution argument

A Python CLI needs Python, the right version, the right packages, the right virtualenv. A Node.js tool needs Node, npm, and potentially hundreds of packages in `node_modules`. These aren't theoretical concerns — they cause real failures in production. The wrong Python version, a missing package, a broken lockfile.

A compiled Rust binary is a single self-contained executable. Copy it to any machine with the same OS and architecture — it works. No interpreter, no runtime, no dependencies. `yo` on macOS or Linux. `yo.exe` on Windows. That's it.

For a tool you want to install once and trust forever, this is non-negotiable.

### The performance argument

Rust compiles to native machine code via LLVM — the same optimisation infrastructure used by C and C++. For mang.sh, the bottleneck is always the AI network call (500ms–3s). The binary itself starts in under 10ms. No interpreter warmup, no GC pause, no JIT compilation lag. The prompt appears instantly.

### The memory safety argument

Rust's ownership system enforces memory safety at compile time, without a garbage collector. For mang.sh this matters concretely: we spawn background threads for telemetry submissions. In C++, passing data to a thread while the main thread continues is a minefield. In Rust, the compiler refuses to compile code that would create a data race. The telemetry thread's data is *moved*, not shared — enforced at compile time, not runtime.

### The type system argument

Rust's type system encodes invariants at compile time:
- `Option<T>` forces handling the "absent" case — no null pointer exceptions
- `Result<T, E>` forces handling errors — no uncaught exceptions
- Exhaustive pattern matching: add a new enum variant and the compiler tells you every match that doesn't handle it

In mang.sh, the `ShellKind` enum covers zsh, bash, fish, sh, PowerShell 5, PowerShell 7, cmd.exe, and Git Bash. Add a new shell — the compiler flags every unhandled match. In Python, that's a runtime bug shipped to users.

### The `cargo` ecosystem

`cargo` is one of the best build tools ever designed:
- `cargo check` type-checks in seconds without building
- `cargo clippy` catches logic errors and anti-patterns beyond the type checker
- `Cargo.toml` is a clean, readable manifest with exact version locking
- Cross-compilation built in

The entire mang.sh project builds with `cargo build --release`. No Makefile, no CMakeLists.txt, no Gradle.

### Rust is not just for systems programming

The misconception: Rust is for operating systems, game engines, embedded firmware. The reality: Rust is excellent for CLI tools and developer utilities. Compile times are longer than Python (improving rapidly). The learning curve is steeper. The result: a binary that ships, works everywhere, starts instantly, and cannot crash due to memory errors or type errors. For a tool that runs shell commands on your machine, that's the right trade-off.

---

## 🌟 Feature overview

| Feature | Detail |
|---|---|
| 🗣️ Natural language | Plain English → shell commands via any OpenRouter model or local Ollama |
| ✅ Always confirms | Every suggestion requires `Y` before anything runs |
| ⚡ Single binary | No Python, Node.js, or runtime — one file, works everywhere |
| 🔑 Local config | API key stored in your OS config directory only |
| 句芒 Spirit banner | Block-letter `MANG` / `.sh` with Gou Mang (句芒) subtitle on every launch |
| 🧠 Intent detection | "use ollama" / "change model" triggers reconfiguration without API call |
| 📟 Rich shortcuts | `!help`, `!api`, `!feedback`, `!shortcuts`, `!context`, `!update`, `!exit` |
| 🧭 Prompt wizard | `!prompt` / `!p` — guided 3-question mode when you're stuck or request is vague |
| 🪪 Credits screen | `!credits` / `!cr` — author info, project links, build stack |
| 🐚 Three aliases | `yo`, `hi`, `hello` — all invoke the spirit messenger |
| 🌍 Context-aware | OS, arch, CWD, and precise shell sent with every request |
| 🛡️ Safe prompting | Temperature 0.2 — deterministic, conservative suggestions |
| 💬 Explanations | Every suggestion includes a plain-English description |
| 🏠 Ollama support | Local AI — no API key, no internet, complete privacy |
| 🔁 Multi-turn context | Follow-up prompts: "now do the same for /tmp" works |
| 📂 Shell history | Confirmed commands appended to `~/.zsh_history` / `~/.bash_history` |
| 🧪 Dry-run | `yo --dry` — preview every command before any execution |
| 🔄 Refinement | N on a suggestion → describe what to change → Mang adjusts |
| 🪝 Feedback loop | "Did that work?" with AI refinement if it didn't |
| 🪟 Windows native | PS5, PS7, cmd.exe, Git Bash — detected, correct syntax generated |
| 💾 Named shortcuts | `!save`, `!forget`, instant replay with `!<name>` |
| 🔔 Update check | Background version check on every launch, `!update` to install |
| 📊 Telemetry | Opt-in community data sharing via JSONBin.io |

---

## 🚀 Install

**macOS / Linux:**
```bash
curl -fsSL https://mang.sh/install | bash
```

**Windows — PowerShell** (native, no Git Bash needed):
```powershell
iwr -useb https://mang.sh/install.ps1 | iex
```

**Windows — Git Bash or WSL2:**
```bash
curl -fsSL https://mang.sh/install | bash
```

> ⚠️ On Windows, `curl` in PowerShell is an alias for `Invoke-WebRequest` and does **not** accept `-fsSL`. Always use `iwr` or open Git Bash.

**Update:**
```bash
curl -fsSL https://mang.sh/update | bash          # macOS/Linux
iwr -useb https://mang.sh/update.ps1 | iex         # Windows PS
```

**Uninstall:**
```bash
curl -fsSL https://mang.sh/uninstall | bash        # macOS/Linux
iwr -useb https://mang.sh/uninstall.ps1 | iex      # Windows PS
```

> Full guide: **[INSTALL.md](INSTALL.md)**

---

## 🎬 See it in action

```
$ yo

  [banner — Gou Mang's tree + MANG.SH logotype]

  ◈  Backend: OpenRouter  model: openai/gpt-4o-mini
  ◈  Context: 5 turns

  yo ›  find all log files older than 7 days and delete them

  ◈  Finds .log files not modified in 7+ days and removes them.

  ┌──────────────────────────────────────────────────────────┐
  │  $  find . -name "*.log" -mtime +7 -type f -delete      │
  └──────────────────────────────────────────────────────────┘

  Run it? [Y/n] › N

  ◈  Let's refine — what should be different?
  ◈  (Describe the change, or press Enter / !skip to cancel)

  yo ›  only in the /var/log folder, not here

  ◌  Thinking…

  ◈  Applies the same log cleanup but restricted to /var/log.

  ┌──────────────────────────────────────────────────────────┐
  │  $  find /var/log -name "*.log" -mtime +7 -type f -delete│
  └──────────────────────────────────────────────────────────┘

  Run it? [Y/n] › Y

  ►  find /var/log -name "*.log" -mtime +7 -type f -delete
  ✔  Done.

  Did that work? [Y/n] › Y
  ◈  Great! What else?

  yo [+1] ›  now do the same for /tmp

  ◌  Thinking…

  ◈  Same cleanup applied to /tmp.
  ┌──────────────────────────────────────────────────────────┐
  │  $  find /tmp -name "*.log" -mtime +7 -type f -delete   │
  └──────────────────────────────────────────────────────────┘
```

Notice: pressing **N** opens a refinement loop — describe what to change, Mang adjusts. The `[+1]` in the prompt shows how many prior turns Mang remembers.

---

## 🏠 Ollama — local, private, offline

Run mang.sh entirely on your own machine, zero network traffic:

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model (one-time, ~2GB)
ollama pull llama3.2

# Launch — choose Ollama during setup
yo
```

Or switch mid-session:
```
yo ›  use ollama
```

| Model | Pull | Best for |
|---|---|---|
| `llama3.2` | `ollama pull llama3.2` | General — best default |
| `mistral` | `ollama pull mistral` | Fast, great at commands |
| `codellama` | `ollama pull codellama` | Code-heavy sessions |

---

## 💾 Named shortcuts — instant command replay

```
yo ›  docker restart myapp && docker logs --tail 50 myapp
  ✔  Done.
  Did that work? [Y/n] › Y

yo ›  !save restartapp

# Any time later — no AI, no confirmation:
yo ›  !restartapp
  ◈  Running shortcut !restartapp
  ►  docker restart myapp && docker logs --tail 50 myapp
```

| Command | Action |
|---|---|
| `!save <name>` | Save last confirmed commands as `!name` |
| `!<name>` | Run instantly — no AI, no Y/N |
| `!forget <name>` | Remove a shortcut |
| `!shortcuts` | List all saved shortcuts |

Persisted to `~/.config/mang/shortcuts.json` across sessions.

---

## ⌨️ All shortcuts

| Input | What happens |
|---|---|
| `!prompt` / `!p` | Advanced Prompt Mode — up to 3 AI questions to clarify a vague request |
| `!credits` / `!cr` | About mang.sh — author, links, build stack, mythology |
| `!help` / `!h` | Full help screen |
| `!update` / `!check` | Check for a new version, offer to install |
| `!api` | Reconfigure backend, model, API key, history, context |
| `!feedback` / `!fb` | Telemetry status, opt-in/out, personal JSONBin |
| `!shortcuts` / `!sc` | List all saved shortcuts |
| `!save <name>` | Save last commands as `!<name>` |
| `!forget <name>` | Remove a shortcut |
| `!<name>` | Run a shortcut instantly |
| `!context` / `!ctx` | Show Mang's current memory |
| `!clear` | Clear conversation context |
| `!exit` / `!q` | Dismiss Mang |
| `Y` / Enter | Confirm and run |
| `N` | Refine — describe what to change |
| `↑` / `↓` | Recall previous prompts |
| `Ctrl+D` | Exit at any time |

### CLI flags

```bash
yo --dry          # Dry-run: show commands, never execute
yo -d             # Short form
yo --no-history   # Disable shell history appending this session
yo --no-context   # Disable multi-turn context this session
yo --help         # Show all flags
yo --version      # Show version
```

---

## 📁 Code structure

```
mang-sh/
├── src/
│   ├── main.rs          Entry point, REPL loop, execution, telemetry handle tracking
│   │                    All 4 exit paths join pending_telemetry before returning
│   ├── ai.rs            OpenRouter + Ollama HTTP calls, JSON envelope, intent detection
│   │                    suggest_commands() → Suggestion  |  suggest_raw() → String
│   ├── prompt_wizard.rs Advanced Prompt Mode — 3-question Socratic dialogue
│   │                    coach_prompt() → suggest_raw() → synthesise() → suggest_commands()
│   ├── config.rs        Load/save ~/.config/mang/config.json, interactive setup
│   ├── shell.rs         ShellKind enum (8 variants), detection matrix, syntax hint
│   ├── context.rs       ConversationContext — rolling window of N prior turns
│   ├── history.rs       Shell history appending — zsh EXTENDED_HISTORY, bash, fish
│   ├── shortcuts.rs     ShortcutStore — save/run/forget, persisted to shortcuts.json
│   ├── updater.rs       Background version check (rate-limited 24h), !update handler
│   ├── telemetry.rs     JSONBin.io background POST, JoinHandle tracking, MANGDEBUG
│   ├── feedback.rs      !feedback subcommands — setup, on/off, personal, test, about
│   ├── cli.rs           clap Args — --dry/-d, --no-history, --no-context
│   └── ui.rs            Banner, help, credits, suggestion display, context summary
├── Cargo.toml           Manifest — all deps annotated with purpose
├── yo.sh                Unix installer — Rust auto-install, binary build, aliases
├── update.sh            Unix updater — in-place binary replacement
├── uninstall.sh         Unix uninstaller — binary + config + aliases, /dev/tty Y/N
├── install.ps1          Windows PowerShell installer (PS5 + PS7 compatible)
├── update.ps1           Windows updater
├── uninstall.ps1        Windows uninstaller
├── README.md            You're reading it
├── INSTALL.md           Full install/update/uninstall reference
├── CHANGELOG.md         Complete version history (all 13 versions)
└── LICENSE              MIT
```

---

## 🧠 Architecture — how it works

### The hardest problem: reliable structured output from an LLM

Every LLM wants to be conversational. Ask for "a shell command to list files" and you'll get prose, markdown fences, explanations, caveats. None of that is machine-parseable.

mang.sh forces the model to respond exclusively with this JSON schema:

```json
{
  "commands": ["cmd1", "cmd2"],
  "explanation": "One plain-English sentence."
}
```

The system prompt states this schema twice (once in rules, once as an example) and includes numbered constraints. We also strip accidental markdown fences before parsing. Both OpenRouter and Ollama backends go through the same parser. This single design decision — structured output over freeform — is what makes every other feature possible.

### Temperature 0.2 — why not 0?

Shell commands aren't creative. Temperature 0.2 is low enough that the model picks the conventional, widely-understood command form rather than an exotic variant. It's high enough to handle natural language variation without getting stuck. Tested across GPT-4o-mini, Claude 3 Haiku, Claude 3.5 Sonnet, and Llama 3.2 — 0.2 produces correct, safe commands in over 95% of cases.

### Context injection — why it matters

Without context, a model asked "open the downloads folder" has to guess the platform. With `OS=macos ARCH=aarch64 CWD=/Users/paul SHELL=zsh syntax=posix` prepended to every request, the model knows: use `open`, use `brew`, use `pbcopy`, use arm64 binary paths, use POSIX syntax. Four fields. Measurable improvement in correctness.

The `syntax=posix` / `syntax=powershell` / `syntax=cmd` hint is the highest-leverage addition: it explicitly tells the model which shell syntax family to use, eliminating the most common cross-platform errors. PowerShell 5 doesn't support `&&` — the model knows this and uses `;` instead.

### Multi-turn context — how "now do the same for X" works

Each confirmed prompt+command pair is injected as prior `user`/`assistant` message pairs before the new request:

```json
[
  { "role": "system",    "content": "<system prompt>" },
  { "role": "user",      "content": "find log files older than 7 days" },
  { "role": "assistant", "content": "{\"commands\": [\"find . -name '*.log' -mtime +7\"]}" },
  { "role": "user",      "content": "now do the same for /tmp" }
]
```

The window is bounded (default 5 turns, configurable) to prevent unbounded token growth. Oldest turns evicted first.

### The N = refinement design

Previously, pressing N returned you to a blank prompt. You had to retype your entire request. Now N opens an inline refinement loop: the user describes what to change, mang.sh constructs a context-aware prompt including the original request and previous suggestion, and the AI produces an adjusted command. The loop continues until Y or explicit cancel (`!skip`, blank Enter, Ctrl-D).

This transforms mang.sh from a one-shot tool into a conversational assistant that learns from your feedback without requiring you to repeat yourself.

### Shell history — native formats

Each shell expects its own history format:
- **zsh**: `: <unix_timestamp>:0;<command>` (EXTENDED_HISTORY)
- **bash**: plain `<command>` one per line
- **fish**: `- cmd: <command>\n  when: <timestamp>` (YAML-like)

We detect which format from `$SHELL`, `$ZDOTDIR`, and `$HISTFILE`. Writing to the file doesn't update the live shell's in-memory buffer — `history -r` or a new terminal window picks up the entries.

### The thread-join pattern for telemetry

Background threads are killed when the process exits. If you confirm a command and immediately type `!exit`, a naive fire-and-forget thread never completes its HTTP POST. mang.sh stores every `JoinHandle<()>` returned by `submit_background()` in a `Vec<JoinHandle<()>>`. Every exit path (Ctrl-D, Ctrl-C, `!exit`, input error) calls `h.join()` on all handles before returning. The network requests complete before the process terminates.

### Windows PowerShell — the `cargo stderr` trap

The most counterintuitive Windows bug: `$ErrorActionPreference = "Stop"` + `2>&1` kills the build script when `cargo` writes progress to stderr — even on success. `cargo` writes *all* progress output to stderr, not stdout. Every "Compiling foo v1.0" line becomes an `ErrorRecord` that triggers the Stop preference. Fix: remove `$ErrorActionPreference = "Stop"`, remove `2>&1`, let cargo output flow freely, check `$LASTEXITCODE` after.


### The REPL loop — design decisions

The main loop in `main.rs` is a blocking `loop {}` with a `rustyline` readline call at the top. Here's why every major decision was made the way it was:

**Why synchronous?** One AI call per user turn. The user is the bottleneck — they type, they read, they decide. Adding `tokio` for async would cost ~200 KB of binary size, 30 additional seconds of first compile time, and produce zero improvement in responsiveness from the user's perspective. Async is for servers. This is a CLI tool.

**Why `rustyline` and not raw `stdin.read_line()`?** Raw `stdin.read_line()` gives you no arrow key editing, no `Ctrl-W` word delete, no `↑`/`↓` session history. A REPL without readline feels broken. `rustyline` is the `readline` equivalent for Rust — battle-tested, used by `evcxr` (the Rust REPL), supports all expected key bindings, and integrates in five lines. The `add_history_entry()` call keeps in-session history separate from the shell's own history file (which `history.rs` handles as a distinct concern).

**Why is the context window bounded?** The `ConversationContext` holds at most N prior `(prompt, commands)` pairs. N is configurable (default 5). Without a bound, a long session would inject an unbounded token payload into every AI request — costs money, slows responses, and eventually exceeds context window limits on smaller models. Five turns is enough to resolve any pronoun or relative reference a user would realistically make in a single task.

**The four exit paths problem.** The main REPL has exactly four ways to exit:
1. `Ctrl-D` → `ReadlineError::Eof`
2. `Ctrl-C` → `ReadlineError::Interrupted`
3. `!exit` / `!q` → explicit shortcut
4. readline error → other `ReadlineError` variant

Every single one must call `h.join()` on all pending telemetry handles before returning. This was not the case in v2.3.0 — detached threads were killed on process exit, silently losing every telemetry entry. The fix was a `Vec<JoinHandle<()>>` that accumulates handles and is joined at all four exit points. Finding all four exit points is the kind of audit that's easy to miss on the first pass.

### The system prompt — the most important "code" in the project

The system prompt in `ai.rs::SYSTEM_PROMPT` is a constant string, but it's where most of the product design lives. Some specific decisions:

**Why are the rules numbered?** Because LLMs follow numbered lists better than prose paragraphs. The numbered format also makes it easy to reference specific rules when debugging model behaviour.

**Why state the JSON schema twice?** Rule 2 states the schema abstractly. The example restates it concretely. On smaller models like Llama 3.2 7B, having the schema appear only once produced ~60% JSON compliance. Adding a concrete example brought it to ~92%. The redundancy is intentional.

**Why rule 7 (empty commands fallback)?** Without an explicit escape hatch for non-shell requests, the model either invents nonsensical shell commands or produces malformed JSON. Rule 7 gives it a clean output: `{"commands": [], "explanation": "I cannot express this as a shell command."}`. The empty commands array triggers the prompt wizard (v3.0.2+) rather than showing a dead-end error.

**Why temperature 0.2 and not 0?** Full greedy decoding (temperature=0) creates a failure mode where ambiguous prompts get stuck producing the same wrong command repeatedly. Temperature 0.2 introduces just enough variability to break out of local optima while keeping the output firmly in the "conventional and correct" zone. Tested across GPT-4o-mini, Claude 3 Haiku, Claude 3.5 Sonnet, and Llama 3.2 — 0.2 is consistently correct.

### The JSON parsing pipeline — belt and suspenders

Despite the system prompt, some models (particularly smaller Ollama models) occasionally wrap their output in markdown fences. The `parse_suggestion()` function strips these before attempting JSON parse:

```rust
let cleaned = raw
    .trim()
    .trim_start_matches("```json")
    .trim_start_matches("```")
    .trim_end_matches("```")
    .trim();
```

Order matters: try the longer prefix first. This is belt-and-suspenders engineering — the system prompt says "no markdown fences" and we strip them anyway, because the cost of the strip is zero and the cost of a parse failure is a broken user experience.

We parse into `serde_json::Value` rather than a typed struct for three reasons:
1. Missing fields return `None` instead of panicking
2. The raw response is embedded in the error message for debugging
3. The `explanation` field is truly optional without `Option<String>` ceremony

### Shell detection — the full matrix

`shell.rs` implements a `ShellKind` enum with 8 variants:

```
ShellKind::Zsh         — $SHELL ends in "zsh" OR $ZSH_VERSION is set
ShellKind::Bash        — $SHELL ends in "bash" AND not Git Bash
ShellKind::Fish        — $SHELL ends in "fish" OR $FISH_VERSION is set
ShellKind::Sh          — $SHELL ends in "/sh" (non-bash POSIX shells)
ShellKind::GitBash     — Windows + MINGW or MSYS in $MSYSTEM
ShellKind::PowerShell5 — Windows + PSVersion.Major == 5
ShellKind::PowerShell7 — Windows + PSVersion.Major >= 7
ShellKind::Cmd         — Windows + none of the above (cmd.exe fallback)
ShellKind::Unknown     — last resort
```

The `syntax=` hint derived from this detection is the single most impactful addition to the AI context. Before it existed, Windows users got POSIX-syntax commands — which worked in Git Bash but failed in cmd.exe and PowerShell. The `syntax=cmd` and `syntax=powershell` hints tell the AI exactly which syntax family to target.

**Why PowerShell 5 and 7 are distinct:** PS5 doesn't support `&&` for command chaining. PS7 does. Version-specific detection means PS5 users get `;`-separated commands and PS7 users get `&&` chains — both correct for their environment.

**The Git Bash trap:** On Windows, Git Bash sets `$SHELL=/usr/bin/bash` — identical to native bash on Linux. Without the `$MSYSTEM` environment variable check (which Git Bash sets to `MINGW64`), mang.sh would detect Git Bash as plain bash and generate commands that fail in the user's primary Windows shell.

### The prompt wizard architecture — v3.0.2

The wizard in `prompt_wizard.rs` is built around one key insight: **the AI needs to ask questions, not generate commands.** These are fundamentally different tasks requiring different system prompts, temperatures, and output contracts.

```
User's vague prompt
        ↓
   coach_prompt()         ← "prompt coach" system message, temperature=0.5
        ↓
   ai::suggest_raw()      ← freeform call, no JSON schema, max_tokens=256
        ↓
   One plain question     ← prose, no JSON required
        ↓
   User answers (up to 3×)
        ↓
   synthesise()           ← pure string join, NO extra AI call
        ↓
   suggest_commands()     ← normal pipeline with enriched compound prompt
```

**Why `suggest_raw()` instead of reusing `suggest_commands()`?** The JSON schema forces the AI to produce `{"commands": [...], "explanation": "..."}`. When you ask an AI to generate a clarifying question through this schema, it tries to fit the question into `explanation` and produces nonsense in `commands`. A separate function with a separate contract — "just return plain text, one short question" — is the correct solution.

**Why is synthesis a string join and not another AI call?** The first design passed all Q&A back to the AI to synthesise a clean prompt. This added one more network round-trip (500ms–3s), was non-deterministic, and occasionally produced worse prompts than naive concatenation. Joining the original prompt and user answers with `". "` feeds a richer context string to `suggest_commands()` and lets the downstream AI handle disambiguation. Deterministic, instant, offline.

### Named shortcuts — the `ShortcutStore` design

```rust
pub struct ShortcutStore {
    pub shortcuts: HashMap<String, Vec<String>>,
}
```

A flat `HashMap<String, Vec<String>>`. Serialised to `~/.config/mang/shortcuts.json`. Three decisions:

**Names stored without `!`.** The `!` is invocation syntax, not the name. `"restartapp": ["docker restart myapp"]` is readable and manually editable. `"!restartapp"` is noisier.

**Normalised to lowercase.** `!RestartApp` and `!restartapp` are the same shortcut. Normalised at storage time.

**No confirmation on replay.** The user saved this intentionally. The confirmation gate exists for AI-generated suggestions because they're unknown — a saved shortcut is known by definition.

---

## 🐛 Bugs — the ones worth documenting

These are the non-obvious bugs in mang.sh's history that were instructive or affected real users.

### Bug 1: The detached-thread telemetry silence (v2.3.0)

**Symptom:** The JSONBin collection was empty. Every telemetry entry was silently lost. `!feedback test` (which uses the synchronous path) worked fine, making it look like a server-side issue.

**Cause:** `submit_background()` returns a `JoinHandle<()>`. The calling code in `main.rs` dropped it immediately — it was never stored. In Rust, dropping a `JoinHandle` detaches the thread. When `main()` returns, the OS kills all detached threads, and the HTTP POST never completes.

**The insidious part:** `!feedback test` uses `submit_sync()` — a blocking call. The sync path worked. The async path silently failed. These are different code paths with different failure modes.

**Fix:** `Vec<JoinHandle<()>>` in `main()`, populated on every `submit_background()` call, drained at all four exit paths.

**Lesson:** A dropped `JoinHandle` is not an error — Rust lets you do this deliberately. But an implicit drop that happens because you forgot to store the handle is a silent race condition. The `#[must_use]` attribute on `JoinHandle` produces a compiler warning if you discard it — but only if clippy is run with `-D warnings`.

### Bug 2: The uninstall script stdin trap (v1.1.2)

**Symptom:** Running `curl -fsSL https://mang.sh/uninstall | bash` always printed "Cancelled" regardless of user input.

**Cause:** When a script runs via `curl | bash`, its stdin is the pipe (the script source). `read -r reply` reads from stdin — which is the pipe — returns EOF immediately when the script content is exhausted, and stores an empty string. The empty string triggers the `[Y/n]` default of "n" → "Cancelled".

**Fix:** `read -r reply </dev/tty` — reads from the terminal directly, bypassing the pipe.

**Why this is easy to miss:** In development, you test the script by running it directly (`bash uninstall.sh`). Stdin is the terminal. Everything works. The pipe-stdin issue only manifests when the script is piped from curl, exactly the production usage pattern.

### Bug 3: Shell colour variable escaping (v2.3.4)

**Symptom:** update.sh printed raw escape sequences: `[0;36m` instead of rendering cyan.

**Root cause:**
```bash
CYN='[0;36m'   # WRONG: single quotes = literal characters, no escape processing
CYN=$'[0;36m'  # CORRECT: ANSI-C quoting = ESC byte stored in variable
```

Single-quoted strings in bash are completely literal. No escape sequences are processed. The variable `CYN` contained the string `[0;36m` — ten ASCII characters — not an ESC byte followed by `[0;36m`.

`$''` uses ANSI-C quoting, which processes `` as octal escape → ESC byte (0x1B) at assignment time.

**Why subtle:** `echo -e '[0;36m'` renders correctly because `-e` processes escape sequences. But `printf "${CYN}text${RST}"` without `-e` does not re-process the variable content. The distinction between "the variable contains an escape sequence as text" vs "the variable IS the escape byte" is not obvious from syntax inspection.

### Bug 4: Windows PS5 cargo stderr trap (v2.2.0)

**Symptom:** The install.ps1 script terminated during `cargo build` with a PowerShell `TerminatingError` even though the build would have succeeded. First reported by Wayne.

**Root cause:** Three settings in combination:
1. `$ErrorActionPreference = "Stop"` — terminate on any error
2. `2>&1` — redirect stderr to stdout
3. `cargo` writes ALL progress to stderr

Under `$ErrorActionPreference = "Stop"`, any object written to the error stream terminates the script. `2>&1` causes PowerShell to wrap stderr output in `ErrorRecord` objects. Every "Compiling foo v1.0.0" progress line from cargo became a terminating error.

**Fix:** Remove all three. Drop `$ErrorActionPreference = "Stop"`, remove `2>&1`, check `$LASTEXITCODE` after the build.

**The lesson:** `$ErrorActionPreference = "Stop"` is correct for scripts that only call PowerShell cmdlets. It fails for scripts that invoke native executables (cargo, git, npm, docker) that legitimately write to stderr during normal operation.

### Bug 5: The config forward-compatibility trap (anticipated and prevented)

**Not a shipped bug — a design decision that prevents one.**

Every time a field is added to `Config`, existing users have a JSON config without that field. Without `#[serde(default)]`, `serde_json` fails to deserialise on the next update with "missing field" — breaking every existing installation.

The fix: all fields annotated with `#[serde(default)]` from the start:

```rust
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)] pub api_key: String,
    #[serde(default)] pub model: String,
    #[serde(default)] pub backend: String,
    #[serde(default)] pub telemetry_share_central: bool,
    #[serde(default)] pub sessions_since_telemetry_prompt: u32,
    // ... every field
}
```

**The rule:** annotate config fields with `#[serde(default)]` at the point of addition. Retroactive annotation requires a re-release. If you ship without it, the next `!update` sends every existing user into a broken state with no automatic recovery path.

---

## 🤝 Building with Perplexity Computer

mang.sh was designed and built entirely in collaboration with **[Perplexity Computer](https://www.perplexity.ai)** — from initial architecture through every feature, bug fix, and documentation pass.

### What the collaboration looked like

**Paul provided:**
- Product judgment — what the tool should feel like, what features matter, when something is good enough
- Domain knowledge — how CLI tools actually behave, what shell quirks exist, what users realistically do
- Quality bar — what "done" means, when to push for more depth, when to ship
- The bug reports — actual observed failures that drove fixes

**Perplexity Computer provided:**
- Implementation speed — a working Rust module in minutes rather than hours
- Rust idiom knowledge — knowing `#[serde(default)]` exists, that `dirs` resolves config paths correctly, that `rustyline` is the right readline library, that `reqwest::blocking` is the right call for a CLI
- Debugging breadth — holding the entire codebase in context when diagnosing a multi-file bug
- Documentation completeness — writing this README in full

**The result:** 13 versions from v1.0.0 to v3.0.3 in a matter of days. The code compiles, has zero clippy warnings, handles edge cases properly. The architecture decisions reflect real trade-offs between binary size, compile time, correctness, and maintainability.

### What AI assistance actually looks like

It looks like a very good senior engineer who responds instantly. You describe what you want — "add a background version check that doesn't block the startup prompt" — and you get back a full implementation: a thread, a rate-limiting file, a join handle stored in the right Vec, the UI message integrated into the REPL flow.

Then you test it. You find edge cases: "what happens if the network is unavailable?" "What happens if the rate-limit file is corrupted?" You describe the gap. You get a fix.

**What it does NOT look like:** blindly accepting every output. Every suggestion went through `cargo check`, `cargo clippy`, and manual review. The detached-thread telemetry bug was not caught by the AI — it was caught by noticing the JSONBin collection was empty after real use. The uninstall stdin bug was caught by a user report. AI-generated code must be compiled, run, and used.

### Why this credit is in the README

Because honesty about how software is built matters. Open-source lives and dies by trust. The architecture decisions are human. The product taste is human. The implementation was AI-assisted, and that assistance was substantial. Presenting this as entirely human-written would be a quiet lie.

The model: AI as force multiplier. The human makes the meaningful decisions. The AI removes the mechanical barriers between intention and implementation.

---
## 🔬 Lessons learned — building mang.sh from scratch

This section documents real technical decisions, dead ends, and the reasoning behind them. It's meant to be genuinely useful if you're building something similar — not a highlight reel.

### On architecture: the REPL loop

**A blocking single-threaded REPL is the right model for a CLI tool.**
The entire mang.sh main loop is synchronous. There is exactly one network call per turn and it blocks. The alternative — async with tokio — would add ~200 KB to the binary, 30 seconds to the compile time, and zero user-visible benefit at this call frequency. One request per second doesn't need an async runtime. `async`/`await` is for servers handling hundreds of concurrent connections, not for CLI tools where the user is the bottleneck.

**`rustyline` gives you readline for free — use it.**
The REPL prompt needs arrow-key editing, `Ctrl-W` word delete, and in-session history (`↑`/`↓`). Rolling your own input handling is a week of work. `rustyline` is battle-tested, supports all of these, and integrates in five lines. The add_history_entry call keeps session history without touching the shell's history file (which is what `history.rs` is for — a separate, intentional concern).

**The context window is a rolling buffer, not a log.**
Multi-turn context (follow-up prompts like "now do the same for /tmp") works by injecting prior prompt/command pairs as `user`/`assistant` turns in the messages array. The window is bounded — oldest turns evicted first — to prevent unbounded token growth. Default is 5 turns. The key insight: the AI doesn't need the full session history, just the last few turns to resolve pronouns and relative references.

### On prompt engineering for tool-use UIs

**State the output format twice. Really.**
LLMs attenuate instructions that appeared many tokens ago. Stating the JSON schema once at the top and repeating it in a concrete example dramatically improves compliance, especially on smaller models (tested on Llama 3.2 7B and Mistral 7B where compliance dropped from ~90% to ~60% with single-statement format rules).

**Structured output is load-bearing, not cosmetic.**
The JSON envelope (`{ "commands": [...], "explanation": "..." }`) is the foundation everything else depends on. Without it, you have to parse freeform text — which means regex, which means edge cases, which means failures. Invest in the system prompt first. The entire suggestion pipeline, command display, execution, history recording, and telemetry all hang off clean parsing.

**Temperature 0.2 is the sweet spot for command generation.**
Full greedy decoding (`temperature=0`) produces too-literal outputs and gets stuck on ambiguous prompts. Temperature 0.5+ introduces hallucinated flags. 0.2 picks the most-probable conventional command form while handling natural language variation. Tested across GPT-4o-mini, Claude 3 Haiku, Llama 3.2, Mistral — 0.2 is consistently correct.

**Platform context beats tool inventories.**
`OS=macos SHELL=zsh syntax=posix` in every prompt is worth more than a paragraph listing available tools. The model infers `brew`, `open`, `pbcopy`, `arm64` paths from the OS. The `syntax=posix` / `syntax=powershell` / `syntax=cmd` field is the single highest-leverage addition — it tells the model exactly which shell syntax family to use, eliminating the most common cross-platform errors (PowerShell 5 doesn't support `&&`; cmd.exe uses `%VAR%` not `$VAR`).

**When the tool can't understand you, the tool should ask — not just fail.**
The original empty-suggestion path printed a message and returned to a blank prompt. Users were left stranded. The wizard (v3.0.2) converted a dead end into a guided recovery. The prompt coach uses `temperature=0.5` for natural-sounding questions. Synthesis is pure string concatenation (no extra AI call) — the downstream `suggest_commands()` handles disambiguation from rich compound context.

**"Prompt coaching" and "command generation" are different modes.**
Same backend, completely different system prompts. Command generation: `temperature=0.2`, strict JSON, deterministic. Coaching: `temperature=0.5`, plain prose, conversational. Separating them into `suggest_commands()` vs `suggest_raw()` keeps both contracts clean and makes each easy to reason about.

### On LLM prompt engineering

**State your output format twice.** LLMs attenuate instructions that appeared many tokens ago. Stating the JSON schema once at the top and again in a concrete example dramatically improves compliance, especially on smaller models.

**Structured output is load-bearing, not cosmetic.** The JSON envelope is the foundation everything else depends on. Without it, command extraction is unreliable and the entire pipeline breaks. Invest in the system prompt first.

**Temperature 0.2 > temperature 0 for tool use.** Full greedy decoding (temp=0) produces too-literal, sometimes stuck outputs. 0.2 handles paraphrasing and unusual prompts without breaking format compliance.

**Context window position matters.** The most recent message gets the most attention. Important constraints (like the JSON schema) are best restated near the end of the prompt for smaller models.

**Platform context beats tool inventories.** `OS=macos SHELL=zsh syntax=posix` is more useful than telling the model "you have access to brew, apt, etc." Let the model infer the tools from the environment.

### On Rust for CLI tools

**`cargo check` is your fastest feedback loop.**
Type-checks the entire project in seconds without a full build. Use it after every change. `cargo build --release` only when you need the actual binary. The full release build takes 60–90 seconds on first run; `cargo check` takes under 2 seconds. That's the difference between a tight iteration loop and a frustrating one.

**`cargo clippy` catches what the type checker misses.**
Clippy is not just style — it catches logic bugs. `is_some_and()` vs `map_or()`, `is_empty()` vs `len() == 0`, iterator chains that allocate unnecessarily. Running with `-D warnings` (deny all warnings as errors) enforces a clean codebase across every commit. mang.sh has maintained zero clippy warnings since v2.3.3.

**`#[serde(default)]` on every config field — without exception.**
This is the key to forward-compatible config files. Every new field added to `Config` must have `#[serde(default)]` so existing users' JSON configs load without error after an update. Forgetting this once breaks every existing installation on the next `!update`. The lesson: annotate new fields defensively at the point of addition, not retroactively when the bug surfaces in prod.

**`Box<dyn Error>` early, custom error types later.**
For a project of this size, `Box<dyn std::error::Error>` as a return type lets you use `?` on any error type — `reqwest`, `serde_json`, `std::io`, all work without defining a custom enum. The right starting point. Add typed errors only when you need to pattern-match on variants (e.g. network timeout vs auth failure). Don't prematurely optimise the error path.

**`Stdio::inherit()` for any child process — always.**
Capturing stdout/stderr from a child process breaks: interactive programs (`vim`, `htop`, `ssh`), streaming output (`cargo build`, `docker logs`), colour-aware tools (`ls --color`, `grep --color`), and any program that detects whether it's writing to a TTY. The correct approach: `Stdio::inherit()` for all three streams, let the child own the terminal directly.

**Store `JoinHandle`s; never detach background threads.**
A dropped `JoinHandle` silently detaches the thread. When `main()` returns, all detached threads are killed immediately — before any pending I/O completes. mang.sh v2.3.1 shipped with this bug: every telemetry entry was silently lost because the HTTP POST thread was killed on process exit. Fix: `Vec<JoinHandle<()>>` stored in `main()`, joined at every exit path (Ctrl-D, Ctrl-C, `!exit`, readline error — all four).

**Blocking reqwest is the right call for a CLI tool.**
`reqwest::blocking` is synchronous, has a clean API, handles redirects and TLS, and doesn't require `tokio`. For a tool that makes one HTTP request per user turn — where the user is already staring at a "Thinking…" spinner — async adds compile overhead and binary size for zero UX benefit. Reserve `async`/`await` for servers and high-concurrency situations.

**The `dirs` crate for config paths — always use it.**
Hard-coding `~/.config/` breaks on macOS (Application Support), Windows (AppData\Roaming), and any system with a non-standard `$XDG_CONFIG_HOME`. The `dirs` crate resolves the OS-correct path automatically and uses the Cargo package name (`mang`) as the subdirectory. Changing the package name in `Cargo.toml` automatically moves the config directory — seamless rebrand.

**`regex::Regex` should be compiled once, not per call.**
The intent detection patterns in `ai.rs` compile a `Regex` from a string on every `intent_is_api_change()` call. For a REPL that runs this on every input, this is a small but unnecessary allocation. Production pattern: use `once_cell::sync::Lazy<Regex>` or `regex::RegexSet` compiled at startup. The current implementation is correct and cheap enough at this scale — but worth knowing for higher-frequency applications.

### On cross-platform shell detection

**Shell detection is harder than it looks.**
`$SHELL` gives you `/bin/zsh` or `/usr/bin/bash` on Unix — helpful but not precise. On Windows it's empty. In a Git Bash session on Windows, `$SHELL` is `/usr/bin/bash` but you're actually inside Git Bash running on Win32. The correct approach: check multiple env vars (`$SHELL`, `$ZSH_VERSION`, `$BASH_VERSION`, `$FISH_VERSION`), check the parent process name on Windows, and maintain an explicit `ShellKind` enum covering zsh, bash, fish, sh, PowerShell 5, PowerShell 7, cmd.exe, Git Bash, and Unknown. The shell matrix (8 variants × 3 OS families) sounds complex but each variant is a simple string match.

**The `syntax=` hint is the highest-leverage AI context field.**
Telling the AI `syntax=powershell5` (not `syntax=powershell`) means it knows to avoid `&&` (unsupported in PS5) and use `;` or `-and` instead. `syntax=cmd` means `%VAR%` paths and `&` chaining instead of POSIX. Without this, the AI generates technically correct commands for the wrong shell — they look right but fail silently or with confusing errors.

### On Windows support

**`curl` is not curl in PowerShell.**
PowerShell has a built-in alias `curl → Invoke-WebRequest` that silently ignores `-fsSL`. The Unix idiom `curl -fsSL https://... | bash` fails in PowerShell with no useful error message. The correct idiom: `iwr -useb <url> | iex`. Document this at every install instruction and provide a separate `.ps1` installer that doesn't use curl at all.

**PowerShell 5 vs 7 is a real distinction.**
PS5 (built into every Windows installation) doesn't support `&&` for command chaining. PS7 does. Detecting the version and emitting `syntax=powershell5` vs `syntax=powershell7` to the AI context prevents an entire class of confusing "command not found" errors for Windows users. They both look like "PowerShell" but behave differently.

**`$ErrorActionPreference = "Stop"` + `2>&1` + native commands = script death.**
This triple combination in an installer script kills `cargo build` — cargo writes all progress output to stderr, which PowerShell's Stop preference interprets as a terminating error even when the build succeeds. Fix: remove `$ErrorActionPreference = "Stop"`, remove `2>&1`, let cargo output flow freely, and check `$LASTEXITCODE` after. v2.2.0 fixed this after a bug report from a Windows user whose PS5.1 install script was killing on "Compiling foo v1.0.0".

### On telemetry and data pipelines

**Fire-and-forget is wrong for anything that must complete.**
The v2.3.0 implementation used `std::thread::spawn()` and immediately dropped the `JoinHandle`. The thread was detached. The process exited. The HTTP POST was killed at the OS level before any bytes left the machine. Every single telemetry entry was silently lost for an entire release. The fix is mechanical but critical: store every `JoinHandle` in a `Vec`, join all of them at every exit path. There are exactly four exit paths in mang.sh's main loop (Ctrl-D, Ctrl-C, readline error, `!exit`) — every one must join.

**Silent error swallowing makes debugging impossible.**
`Err(_) => {}` is a legitimate production choice for non-critical background work. But during development it's a blackout. `MANGDEBUG=1 yo` enables a mode that prints the full JSON payload and HTTP response to stderr before submitting. This is how we diagnosed the dropped-thread bug — without it, the only symptom was "no data in the collection". Always add an escape hatch that makes the invisible visible.

**Write-only API keys are the correct security model for embedded credentials.**
The telemetry write key is embedded in the mang.sh binary. It can only create new JSONBin bins (POST /b) — it cannot read, update, or delete anything. The worst-case scenario if the key leaks: someone adds junk entries to a collection that one person reads once a week. Compare to: a read key that exposes every user's command history. The security model maps directly to the threat model. A write-only key ships. A read key never ships.

**Opt-in with periodic gentle reminders is better than opt-out.**
Community telemetry is off by default in mang.sh. A counter in config triggers a reminder every 10 sessions. Users who want to contribute opt in with `!feedback on`. This approach: collects meaningful signal from genuinely interested users (not passive non-rejection), generates no resentment, and keeps the README's "never shared" claims credible. The alternative (opt-out) increases the data volume but degrades the signal quality and trust.


### On installer design

**`curl | bash` is controversial but practical.**
Security-conscious developers object to piping untrusted scripts to a shell. The correct response: serve the script over HTTPS (TLS validates the server identity), pin the SHA-256 if paranoid, and always provide a manual install option (clone + `cargo build --release`). The `curl | bash` pattern is the dominant convention for developer tools precisely because it works on every Unix system with zero prerequisites — which matters enormously for a tool that installs Rust as part of its own installation.

**`/dev/tty` is required for interactive prompts in piped scripts.**
When a script runs via `curl | bash`, its stdin is the pipe (the script content), not the terminal. Any `read` call reads from the pipe, not from the user's keyboard — it returns EOF immediately and the script proceeds with an empty answer. This caused the v1.1.2 uninstall script to always report "Cancelled" regardless of what the user typed. Fix: `read -r reply </dev/tty` forces the read from the actual terminal. This is not widely documented and has caused bugs in many prominent open-source installers.

**Shell colour codes must use ANSI-C quoting.**
`CYN='[0;36m'` stores a literal backslash + 0 + 3 + 3 — not an escape byte. The terminal sees `[0;36m` as printable text and renders the raw escape sequence instead of colour. The correct form: `CYN=$'[0;36m'` (ANSI-C quoting) stores the actual ESC byte at assignment time. v2.3.4 fixed this after a bug report about the update script printing literal `` sequences instead of colours.

**Idempotent installers are non-negotiable.**
Running the installer twice must produce the same result as running it once. This means: detect existing binary and reinstall in place, detect existing aliases and skip writing them again, never corrupt the user's shell rc. The mang.sh installer checks for an existing `yo` binary at the start and uses its directory as the install target for reinstalls — preserving any non-standard install locations.

### On AI-assisted development

**AI pair programming is a force multiplier, not a replacement.**
mang.sh was built entirely in collaboration with Perplexity Computer (Claude Sonnet + GPT-4o). The architecture decisions, design trade-offs, and final judgements are human. The implementation speed — 13 versions from v1.0.0 to v3.0.3 in a few days — would have taken weeks solo. The right mental model: AI handles the "how to write this in idiomatic Rust" while the human handles "what should this actually do and why".

**AI-generated code must be compiled and tested, not trusted.**
Every AI suggestion for this project went through `cargo check`, `cargo clippy`, and manual review before committing. Several suggestions had subtle bugs: dropped `JoinHandle`s (the telemetry issue), wrong PowerShell quoting, incorrect serde attribute placement. The AI is a skilled colleague who works fast and occasionally makes mistakes — treat it accordingly.

**Document the "why" aggressively.**
AI-generated code is often correct but context-free. Adding the comment blocks explaining *why* a design decision was made — not just what it does — is entirely the human's responsibility. Future maintainers (including future-you working with AI) need the reasoning, not just the implementation.


---

## 📊 Community data & telemetry

mang.sh can optionally share anonymised data to improve the AI system prompt. Reviewed weekly by Paul Fleury.

**Shared (opt-in, default OFF):**

| Field | Example |
|---|---|
| Prompt | `"find log files older than 7 days"` |
| Commands | `["find . -name '*.log' -mtime +7"]` |
| Model | `"openai/gpt-4o-mini"` |
| OS + shell | `"macos"` + `"zsh"` |
| Worked | `true` |
| Version | `"v3.0.4"` |
| Timestamp | `"2026-03-23T12:00:00Z"` |

**Never shared:** API keys, file paths, CWD, command output, username, hostname.

Enable: `!feedback on` · Disable: `!feedback off` · Full settings: `!feedback` · Live test: `!feedback test`

Debug mode: `MANGDEBUG=1 yo`

---

## 🔑 OpenRouter model recommendations

| Model | Cost | Best for |
|---|---|---|
| `openai/gpt-4o-mini` | ~$0.15/1M tokens | ★ Default — fast, reliable |
| `openai/gpt-4o` | ~$2.50/1M tokens | Complex multi-step requests |
| `anthropic/claude-3.5-sonnet` | ~$3/1M tokens | Tricky, context-heavy tasks |
| `anthropic/claude-3-haiku` | ~$0.25/1M tokens | Speed-critical workflows |
| `meta-llama/llama-3.3-70b-instruct:free` | Free | Getting started |

Get a key: **[openrouter.ai/keys](https://openrouter.ai/keys)**

---

## 📝 Changelog

> Full history: **[CHANGELOG.md](CHANGELOG.md)**

### 🔖 v3.0.3 — 2026-03-25
- 🪪 **Credits screen** (`!credits` / `!cr`) — author info, project links, build stack, mythology
- 📚 README: deep technical expansion — REPL architecture, module reference, design rationale, toolchain notes

### 🔖 v3.0.2 — 2026-03-25
- 🧭 **Advanced Prompt Mode** (`!prompt` / `!p`) — up to 3 AI-generated clarifying questions when you're stuck; auto-triggers when the AI returns no commands

### 🔖 v3.0.1 — 2026-03-23
- 🎨 Redesigned banner: clean block-letter `MANG` (cyan) + `.sh` (bold white), `句芒 · Gou Mang · Spirit Messenger` header, minimal dim frame
- 🔧 Uninstall script: auto-removes legacy `yo-rust` config directories and aliases

### 🔖 v3.0.0 — 2026-03-23
- 🏛️ Rebranded from **Yo, Rust!** to **mang.sh** — Gou Mang (句芒), spirit messenger
- 🌐 New home: **[mang.sh](https://mang.sh)** · install: `curl -fsSL https://mang.sh/install | bash`
- 📊 JSONBin collection renamed to `mang-sh-telemetry`
- 🔍 Zero remaining references to the old name anywhere in the codebase

### 🔖 v2.3.5 — 2026-03-23
- 🔔 Background update check on every launch · `!update` / `!check` shortcuts
- 🔄 N on suggestion = iterative refinement tunnel

### 🔖 v2.3.4 — 2026-03-22
- 🐛 Shell colour variables fixed (`$'\033'` ANSI-C quoting)

---

## 🤝 Contributing

```bash
git checkout -b feat/your-feature
git commit -m 'feat: describe your change'
git push origin feat/your-feature
# → open a Pull Request at github.com/paulfxyz/mang
```

Ideas on the list:
- `--stop-on-error` flag for multi-command sequences
- Keychain/credential manager storage for the API key
- `yo --version` checking against `mang.sh/version` (lightweight text endpoint)

---

## 📜 License

MIT — see [LICENSE](LICENSE).

---

## 👤 Author

Made with ❤️ by **Paul Fleury** — built in collaboration with **[Perplexity Computer](https://www.perplexity.ai)**.

- 🌐 **[paulfleury.com](https://paulfleury.com)**
- 🔗 **[linkedin.com/in/paulfxyz](https://www.linkedin.com/in/paulfxyz/)**
- 🐙 **[@paulfxyz](https://github.com/paulfxyz)**
- 📧 **[hello@paulfleury.com](mailto:hello@paulfleury.com)**

---

<div align="center">

**句芒** *— the messenger between heaven and earth, between intent and command.*

⭐ **If mang.sh saved you time, drop a star — it helps others find it.** ⭐

</div>
