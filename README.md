# mang.sh 句芒 🥷

<div align="center">

**The spirit messenger between you and your shell.**

*Describe what you need. Gou Mang delivers the command. You confirm. It executes.*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Powered by OpenRouter](https://img.shields.io/badge/Powered%20by-OpenRouter-6c47ff?style=for-the-badge)](https://openrouter.ai)
[![Ollama](https://img.shields.io/badge/Supports-Ollama-black?style=for-the-badge)](https://ollama.ai)
[![Version](https://img.shields.io/badge/Version-3.0.2-brightgreen?style=for-the-badge)](CHANGELOG.md)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-blue?style=for-the-badge)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge)](https://github.com/paulfxyz/mang-sh/pulls)

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
  ║   v3.0.2  ·  mang.sh  ·  github.com/paulfxyz  ║
  ╚═══════════════════════════════════════════════╝
```

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

Persisted to `~/.config/mang-sh/shortcuts.json` across sessions.

---

## ⌨️ All shortcuts

| Input | What happens |
|---|---|
| `!prompt` / `!p` | Advanced Prompt Mode — up to 3 AI questions to clarify a vague request |
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
│   ├── main.rs        Entry point · REPL loop · execution · telemetry handles
│   ├── ai.rs          OpenRouter + Ollama HTTP · JSON envelope · intent detection
│   ├── config.rs      Load/save config · interactive setup wizard
│   ├── shell.rs       Shell detection matrix · cross-platform dispatch
│   ├── context.rs     Multi-turn conversation window (rolling N-turn buffer)
│   ├── history.rs     Shell history appending (zsh/bash/fish native formats)
│   ├── shortcuts.rs   Named command shortcuts (save, run, forget, persist)
│   ├── updater.rs     Background version check · !update handler
│   ├── telemetry.rs   JSONBin.io submission · thread handle management
│   ├── feedback.rs    !feedback / !fb subcommand handler and UI wizards
│   ├── cli.rs         clap argument parsing (--dry, --no-history, etc.)
│   └── ui.rs          Banner · help · suggestion display · context summary
├── Cargo.toml         Rust manifest · annotated dependencies
├── yo.sh              Install (macOS/Linux/Git Bash)  →  curl https://mang.sh/install | bash
├── update.sh          Update (macOS/Linux/Git Bash)   →  curl https://mang.sh/update | bash
├── uninstall.sh       Uninstall (macOS/Linux)
├── install.ps1        Install (Windows PowerShell 5+/7+)
├── update.ps1         Update (Windows PowerShell)
├── uninstall.ps1      Uninstall (Windows PowerShell)
├── README.md          You're reading it
├── INSTALL.md         Full install / update / uninstall reference
├── CHANGELOG.md       Complete version history
└── LICENSE            MIT
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

---

## 🔬 Lessons learned — building mang.sh from scratch


### On prompt engineering for tool-use UIs

**When the tool can't understand you, the tool should ask — not just fail.**
The original empty-suggestion path printed a message and returned to the blank
prompt.  Users were left wondering: was the prompt too vague? Wrong phrasing?
The AI broken?  Adding the wizard converted a dead end into a guided recovery.

**"Prompt coaching" is a different mode than "command generation".**
The same AI backend can play two completely different roles depending on the
system prompt.  Command generation uses `temperature=0.2`, strict JSON schema,
and deterministic phrasing.  Prompt coaching uses `temperature=0.5`, plain
prose, and a conversational tone.  Separating these into `suggest_commands()`
vs `suggest_raw()` keeps both clean.

**Synthesis beats summarisation.**  Rather than asking the AI to summarise
collected Q&A context into a refined prompt (extra round-trip, non-deterministic),
we simply concatenate the original prompt + user answers with `". "` and pass
that compound sentence to `suggest_commands()`.  The downstream AI handles
disambiguation naturally from rich context.

### On LLM prompt engineering

**State your output format twice.** LLMs attenuate instructions that appeared many tokens ago. Stating the JSON schema once at the top and again in a concrete example dramatically improves compliance, especially on smaller models.

**Structured output is load-bearing, not cosmetic.** The JSON envelope is the foundation everything else depends on. Without it, command extraction is unreliable and the entire pipeline breaks. Invest in the system prompt first.

**Temperature 0.2 > temperature 0 for tool use.** Full greedy decoding (temp=0) produces too-literal, sometimes stuck outputs. 0.2 handles paraphrasing and unusual prompts without breaking format compliance.

**Context window position matters.** The most recent message gets the most attention. Important constraints (like the JSON schema) are best restated near the end of the prompt for smaller models.

**Platform context beats tool inventories.** `OS=macos SHELL=zsh syntax=posix` is more useful than telling the model "you have access to brew, apt, etc." Let the model infer the tools from the environment.

### On Rust for CLI tools

**`cargo check` is your fastest feedback loop.** Type-checks the entire project in seconds without building. Use it constantly. `cargo build --release` only when you need the binary.

**`#[serde(default)]` on every config field.** This is the key to forward-compatible config files. Every new field added to a config struct must have `#[serde(default)]` so existing configs load without error. Forgetting this breaks every existing user's installation on the next update.

**`Box<dyn Error>` early, custom errors later.** For a project of this size, `Box<dyn std::error::Error>` as a return type lets you use `?` on any error type without defining a custom enum. The right starting point. Add custom error types when you need to match on variants.

**`Stdio::inherit()` for any program that runs other CLIs.** Capturing stdout breaks interactive programs (`vim`, `htop`), buffers streaming output (`cargo build`), and disables colour in colour-aware tools (`ls`, `grep`). Inherit stdio — always.

**Store `JoinHandle`s; never detach.** A dropped `JoinHandle` detaches the thread. When the process exits, detached threads are killed. For any background work that must complete (network requests, file writes), store the handle and join at every exit point.

### On Windows support

**`curl` is not curl in PowerShell.** PowerShell has a built-in alias `curl → Invoke-WebRequest` that doesn't accept `-fsSL`. The correct idiom is `iwr -useb <url> | iex`. Document this prominently. Provide a native PowerShell installer.

**PowerShell 5 vs 7 is a real distinction.** PS5 (built into Windows) doesn't support `&&`. PS7 does. Detecting the version and adjusting the syntax hint in the AI context prevents a class of confusing errors for Windows users.

**`$ErrorActionPreference = "Stop"` + `2>&1` + native commands = disaster.** This triple combination kills scripts on `cargo`'s normal progress output. The fix is to drop all three: keep `$ErrorActionPreference = "Continue"`, don't redirect stderr, check `$LASTEXITCODE`.

### On telemetry and data pipelines

**Fire-and-forget is wrong for anything that must complete.** The first version detached every telemetry thread. Every entry was silently lost because the process exited faster than the HTTP POST completed. Store handles, join at exit.

**Silent error swallowing makes debugging impossible.** `Err(_) => {}` everywhere means you can't tell if entries are being sent and rejected, or never sent. Add a debug mode (`MANGDEBUG=1`) that prints payloads and responses to stderr.

**Write-only API keys are the correct security model.** A key that can only create new bins cannot read, update, or delete anything — safe to ship in the binary. The worst-case abuse: someone adds junk entries. Noise, not a breach.

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
| Version | `"v3.0.2"` |
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
# → open a Pull Request at github.com/paulfxyz/mang-sh
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
