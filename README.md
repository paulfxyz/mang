# 🤖 Yo, Rust!

<div align="center">

**Natural language → Terminal commands, powered by AI.**

*Type `yo` and talk to your terminal like a human being.*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Powered by OpenRouter](https://img.shields.io/badge/Powered%20by-OpenRouter-6c47ff?style=for-the-badge)](https://openrouter.ai)
[![Ollama](https://img.shields.io/badge/Supports-Ollama-black?style=for-the-badge)](https://ollama.ai)
[![Version](https://img.shields.io/badge/Version-2.0.0-brightgreen?style=for-the-badge)](CHANGELOG.md)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-blue?style=for-the-badge)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge)](https://github.com/paulfxyz/yo-rust/pulls)

</div>

---

```
  ╔══════════════════════════════════════════════════════════════════╗
  ║           ╷▲╷             ██╗   ██╗ ██████╗                     ║
  ║      ┌────┴─┴────┐        ╚██╗ ██╔╝██╔═══██╗                    ║
  ║      │ ╔═══╗╔═══╗│         ╚████╔╝ ██║   ██║                    ║
  ║      │ ║◈  ◈║◈  ◈║│          ╚██╔╝  ██║   ██║                    ║
  ║      │ ╚═══╝╚═══╝│           ██║   ╚██████╔╝                    ║
  ║ ┌──┐ │ ┌─────────┐ │ ┌──┐    ╚═╝    ╚═════╝                     ║
  ║ │░░├─┤ │ · · · · │ ├─░░│                                        ║
  ║ └──┘ │ ┌──┬──┬──┐ │ └──┘   ██████╗ ██╗   ██╗███████╗████████╗  ║
  ║      │ │▓▓│▓▓│▓▓│ │        ██╔══██╗██║   ██║██╔════╝╚══██╔══╝  ║
  ║      │ └──┴──┴──┘ │        ██████╔╝██║   ██║███████╗   ██║     ║
  ║      └─────┬─┬────┘        ██╔══██╗██║   ██║╚════██║   ██║     ║
  ║            │ │             ██║  ██║╚██████╔╝███████║   ██║     ║
  ║           ┌┘ └┐            ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝     ║
  ║          │░░││░░│           v2.0.0  ·  github.com/paulfxyz       ║
  ╚══════════════════════════════════════════════════════════════════╝
```

---

## 👨‍💻 Why this exists

I'm **Paul Fleury** — French internet entrepreneur based in Lisbon, managing infrastructure, DNS records, Docker stacks, servers, and all the operational chaos that comes with running multiple products.

And I kept hitting the same wall.

Not the hard problems — DNS propagation, TLS cert chains, Docker networking. The wall was the *boring* ones: the `find` flags I can never remember. The `rsync` syntax that doesn't accidentally wipe the destination. The `awk` one-liner to extract column 3 from a log file. The `openssl` command that decodes a certificate. Things I've typed a hundred times but never fully memorised because I don't type them every single day.

Every time: stop → open browser → search → skip ads → scan Stack Overflow → adapt → test. **Five minutes gone.** Ten times a day. An hour of archaeology.

I wanted to type `yo`, describe the thing, get the command, run it.

**No runtime dependencies.** One compiled Rust binary. One `curl | bash` to install. And now in v2.0.0 — a choice of AI backend, memory of what you just did, shell history integration, dry-run mode, and full Windows support.

> 💡 This project was designed and built in collaboration with **[Perplexity Computer](https://www.perplexity.ai)** — architecture, implementation, debugging, and documentation. A genuine example of human intent + AI execution.

---

## 🌟 What's new in v2.0.0

| Feature | Description |
|---|---|
| 🏠 **Ollama backend** | Run against a local model — no API key, no network, complete privacy |
| 🔁 **Multi-turn context** | Follow-up prompts work: "now do the same for staging" resolves correctly |
| 📜 **Shell history** | Confirmed commands are appended to your `~/.zsh_history` / `~/.bash_history` |
| 🧪 **Dry-run mode** | `yo --dry` — see what would run without executing anything |
| 🪝 **Post-execution feedback** | "Did that work?" loop — describe the problem and get a refined suggestion |
| 🪟 **Windows support** | Native cmd.exe, PowerShell 5, PowerShell 7, Git Bash all detected and handled |
| 🐚 **Precise shell detection** | Detects zsh, bash, fish, sh, PS5, PS7, cmd.exe, Git Bash — AI syntax matches |
| 🗂️ **!context / !clear** | Inspect or reset the AI's conversation memory at any time |

---

## 🌟 Feature overview

| Feature | Detail |
|---|---|
| 🗣️ Natural language input | Plain English → shell commands via any OpenRouter model or local Ollama |
| ✅ Explicit confirmation | Every suggestion requires `Y` before anything runs |
| ⚡ Single compiled binary | No Python, Node.js, or runtime required |
| 🔑 Local config | API key stored in your OS config directory only |
| 🤖 Fancy ASCII banner | Full robot illustration + block-letter logo on every launch |
| 🧠 Intent detection | "use ollama" / "change model" triggers reconfiguration, no API call |
| 📟 Shortcuts | `!help`, `!api`, `!context`, `!clear`, `!exit` |
| 🐚 Shell aliases | `yo`, `hi`, or `hello` all work |
| 🌍 Context-aware | OS, arch, CWD, and precise shell sent with every request |
| 🛡️ Safe by default | Temperature 0.2 — deterministic, conservative command suggestions |
| 💬 Explanations | Every suggestion includes a one-sentence description |
| 📜 Session history | ↑/↓ keys recall previous prompts within a session |

---

## 🚀 Install

```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
```

Works on macOS, Linux, and Windows (Git Bash / WSL2). Installs Rust automatically if needed.

**Update:**
```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/update.sh | bash
```

**Uninstall:**
```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/uninstall.sh | bash
```

> Full guide: **[INSTALL.md](INSTALL.md)**

---

## 🎬 See it in action

```
$ yo

  [banner]

  ◈  Backend: OpenRouter  model: openai/gpt-4o-mini
  ◈  Shell history: on
  ◈  Context: 5 turns

  yo ›  find all log files older than 7 days and delete them

  ◈  Finds .log files not modified in 7+ days and removes them.

  ┌────────────────────────────────────────────────────────┐
  │  $  find . -name "*.log" -mtime +7 -type f -delete    │
  └────────────────────────────────────────────────────────┘

  Run it? [Y/n] › Y

  ►  find . -name "*.log" -mtime +7 -type f -delete
  ✔  Done.

  Did that work? [Y/n] › Y
  ◈  Great! What else?

  yo [+1] ›  now do the same for the /tmp folder

  ◌  Thinking…

  ◈  Applies the same log cleanup to /tmp instead of the current directory.

  ┌──────────────────────────────────────────────────────────────┐
  │  $  find /tmp -name "*.log" -mtime +7 -type f -delete       │
  └──────────────────────────────────────────────────────────────┘

  Run it? [Y/n] ›
```

The `[+1]` after the prompt shows how many turns of context the AI has. Follow-up pronouns and references ("same for", "now also", "undo that") work because the prior turn is passed as conversation history.

---

## 🏠 Ollama — local, private, offline

Run yo-rust against a local model with no API key and no internet:

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model
ollama pull llama3.2

# Start yo-rust and select Ollama during setup
yo
```

During setup, choose backend `2) Ollama`. yo-rust will ask for the URL (default `http://localhost:11434`) and model name.

Or switch at any time:
```
yo ›  use ollama
```

### Ollama model recommendations

| Model | Pull command | Best for |
|---|---|---|
| `llama3.2` | `ollama pull llama3.2` | General purpose — best default |
| `mistral` | `ollama pull mistral` | Fast, good at commands and code |
| `codellama` | `ollama pull codellama` | Code-heavy sessions |
| `qwen2.5-coder` | `ollama pull qwen2.5-coder` | Programming tasks |

---

## 🧪 Dry-run mode

Preview exactly what yo-rust would do, with zero side effects:

```bash
yo --dry
```

In dry-run mode:
- The AI is called normally
- Commands are displayed in a **yellow** box with a `[dry-run]` label
- No command ever executes
- Context is still updated (follow-ups work)
- Shell history is not appended

Useful for: checking what a destructive command looks like before committing, demos, scripting around yo-rust output.

---

## 🐚 Shell detection — Windows & beyond

yo-rust detects your exact shell environment at launch and passes it to the AI as `SHELL=<kind> syntax=<family>`. The AI then generates commands in the correct syntax for your environment.

| Shell | Detected as | Syntax used |
|---|---|---|
| zsh | `SHELL=zsh syntax=posix` | POSIX + zsh idioms |
| bash | `SHELL=bash syntax=posix` | POSIX + bash idioms |
| fish | `SHELL=fish syntax=posix` | fish-compatible |
| sh / dash | `SHELL=sh syntax=posix` | Pure POSIX |
| PowerShell 5 | `SHELL=powershell5 syntax=powershell` | PS5 (no `&&`, uses `;`) |
| PowerShell 7 | `SHELL=powershell7 syntax=powershell` | PS7 (supports `&&`) |
| cmd.exe | `SHELL=cmd.exe syntax=cmd` | `&` chaining, `%VAR%` |
| Git Bash | `SHELL=gitbash syntax=posix` | POSIX via MSYS2 |

The AI knows that PowerShell 5 doesn't support `&&` (use `;` instead), that cmd.exe uses `%VARIABLE%` not `$VARIABLE`, and that Git Bash on Windows is POSIX despite being on a Windows host.

### Windows installation

**Option 1 — Git Bash (recommended):**
```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
```

**Option 2 — PowerShell (requires winget + Rust):**
```powershell
winget install Rustlang.Rust.MSVC
git clone https://github.com/paulfxyz/yo-rust
cd yo-rust
cargo build --release
copy target\release\yo.exe C:\Windows\System32\yo.exe
```

**Option 3 — WSL2:**
```bash
# Inside WSL2 terminal — identical to Linux install
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
```

---

## ⌨️ All shortcuts

| Input | What happens |
|---|---|
| `!help` / `!h` | Full help screen with examples and session status |
| `!api` | Reconfigure backend, API key, model, history, context size |
| `!context` / `!ctx` | Show what the AI currently remembers |
| `!clear` | Clear conversation context — start a fresh session |
| `!exit` / `!q` | Quit yo-rust |
| `Y` or Enter | Confirm and run |
| `N` | Skip — rephrase and try again |
| `↑` / `↓` | Recall previous prompts in this session |
| `Ctrl+D` | Exit at any time |

### CLI flags

```bash
yo --dry          # Dry-run: show commands, never execute
yo -d             # Short form of --dry
yo --no-history   # Disable shell history appending this session
yo --no-context   # Disable multi-turn context this session
yo --help         # Show all flags
yo --version      # Show version
```

### Natural language config

```
yo ›  use ollama             → triggers !api, selects Ollama backend
yo ›  change my API key      → triggers !api, asks for new key
yo ›  switch to claude       → triggers !api, asks for model
yo ›  change backend         → triggers !api, full reconfiguration
```

---

## 📁 Code structure

```
yo-rust/
├── src/
│   ├── main.rs      Entry point · REPL loop · execution · feedback loop
│   ├── ai.rs        OpenRouter + Ollama HTTP · JSON parsing · intent detection
│   ├── config.rs    Load/save config · interactive setup wizard (both backends)
│   ├── shell.rs     Shell detection matrix · OS-specific command execution
│   ├── context.rs   Multi-turn conversation window (rolling N-turn buffer)
│   ├── history.rs   Shell history appending (zsh, bash, fish formats)
│   ├── cli.rs       Command-line argument parsing (--dry, --no-history, etc.)
│   └── ui.rs        Banner · help · suggestion display · context summary
├── Cargo.toml       Rust manifest · annotated dependencies
├── yo.sh            Install (or reinstall) — detects existing version
├── update.sh        Update to latest — skips if already current
├── uninstall.sh     Full removal — binary, config (optional), aliases
├── README.md        You're reading it
├── INSTALL.md       Full install / update / uninstall reference
├── CHANGELOG.md     Version history
└── LICENSE          MIT
```

---

## 🧠 How it works

### Reliable structured output from an LLM

The hardest problem in yo-rust is not the HTTP call — it's getting the AI to produce **machine-parseable output, every time**.

We use a strict JSON envelope:
```json
{ "commands": ["cmd1", "cmd2"], "explanation": "one sentence" }
```

The system prompt states this schema twice and adds numbered rules. We also strip any accidental markdown fences before parsing. Both OpenRouter and Ollama backends go through the same parser.

### Shell detection — why it matters for Windows

Without knowing the shell, the AI must guess. A user in PowerShell 5 asking "list all processes" might get `ps aux` (Linux), which is an alias for `Get-Process` in PS but behaves completely differently. Or they might get `ls | grep nginx` which fails in PS5 because `ls` outputs objects, not text.

With `SHELL=powershell5 syntax=powershell` in the context, the AI produces `Get-Process | Where-Object {$_.Name -like "*nginx*"}` — correct, native PowerShell. With `SHELL=powershell7`, it can use `&&`. With `SHELL=gitbash`, it uses POSIX syntax despite being on Windows.

### Multi-turn context

Each confirmed prompt+command pair is recorded in a `ConversationContext` (rolling window, default 5 turns). On the next request, prior turns are injected as `user`/`assistant` message pairs before the current prompt. This lets the model resolve:
- Pronouns: "delete them" → refers to the files listed in the previous command
- Relative references: "now for staging" → uses the same pattern with a different path
- Negations: "without the -r flag" → modifies the previous command

The context window is bounded to prevent unbounded token growth. Oldest turns are evicted first.

### Post-execution feedback loop

After commands run, yo-rust asks "Did that work?". If not, it asks what went wrong, then constructs a refinement prompt:
```
I ran: <commands>
The problem was: <user description>
Please suggest a corrected command.
```
This is sent to the AI with the full conversation context, so the refined suggestion incorporates both the original intent and the failure mode.

### Shell history integration

Each shell has its own history format:
- **zsh**: `: <timestamp>:0;<command>` (EXTENDED_HISTORY format)
- **bash**: plain `<command>` one per line
- **fish**: `- cmd: <command>\n  when: <timestamp>` (YAML-like)

We append in the correct format for the detected shell after a confirmed execution. Writing to the file does not update the live shell's in-memory history buffer — a new terminal window (or `history -r` / `history -n`) picks up the entries.

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

## 🔬 Engineering notes & lessons learned

**JSON envelope beats freeform every time.** Even a `temperature=0` model will occasionally add prose around a bare command string. A JSON schema with a `commands` array and `explanation` field is parsed deterministically and has zero ambiguity.

**Shell detection is the highest-leverage Windows improvement.** The difference between `syntax=posix` and `syntax=powershell5` is the difference between getting `&&` (invalid in PS5) and `;` (works everywhere). One env var check, massive improvement in correctness.

**PowerShell 5 vs 7 actually matters.** PS7 supports `&&` for chaining. PS5 doesn't. Users of Windows PowerShell (the built-in one) hitting "unexpected token &&" are confused — detecting the version and adjusting the syntax hint eliminates this entirely.

**Multi-turn context is a sliding window problem.** We evict the oldest turn first (FIFO), not the least recently used, because chronological order matters for resolving follow-up references. The most recent turn is most relevant.

**Post-execution feedback closes the loop.** A command that exits non-zero is not always a failure (grep returns 1 for no matches). Asking "did that work?" rather than "command succeeded?" puts the user in control of what constitutes success.

**Dry-run is useful even for experts.** Before running any destructive find+delete, previewing the command in a yellow box with `[dry-run]` label builds confidence.

---

## 📝 Changelog

> Full history: **[CHANGELOG.md](CHANGELOG.md)**

### 🔖 v2.0.0 — 2026-03-22

- 🏠 Ollama backend — local inference, no API key, offline support
- 🔁 Multi-turn context — follow-up prompts resolve correctly
- 📜 Shell history — confirmed commands appended to zsh/bash/fish history
- 🧪 Dry-run mode — `--dry` flag, yellow command box, nothing executes
- 🪝 Post-execution feedback — "Did that work?" with refinement loop
- 🐚 Precise shell detection — zsh, bash, fish, sh, PS5, PS7, cmd.exe, Git Bash
- 🪟 Windows support — cmd.exe and PowerShell execution, PS5/PS7 syntax distinction
- 🗂️ `!context` / `!clear` shortcuts
- 📟 `--no-history` and `--no-context` CLI flags
- ⬆️ Context turn counter shown in REPL prompt (`yo [+3] ›`)
- 📦 New modules: `shell.rs`, `context.rs`, `history.rs`, `cli.rs`

### 🔖 v1.1.3 — 2026-03-22

- 🐛 Uninstall prompt fix (`/dev/tty`), pure ASCII scripts, `printf` everywhere

---

## 🤝 Contributing

```bash
git checkout -b feat/your-feature
git commit -m 'feat: describe your change'
git push origin feat/your-feature
# → open a Pull Request
```

Ideas still on the list:
- Shell history persistence for the yo-rust REPL itself (separate from shell history)
- `--stop-on-error` flag for multi-command sequences
- PowerShell native installer (`install.ps1`)
- Keychain/credential manager storage for the API key

---

## 📜 License

MIT — see [LICENSE](LICENSE).

---

## 👤 Author

Made with ❤️ by **Paul Fleury** — built in collaboration with **[Perplexity Computer](https://www.perplexity.ai)**.

- 🌐 **[paulfleury.com](https://paulfleury.com)**
- 🔗 **[linkedin.com/in/paulfxyz](https://www.linkedin.com/in/paulfxyz/)**
- 🐙 **[@paulfxyz](https://github.com/paulfxyz)**

---

<div align="center">

⭐ **If yo-rust saved you time, drop a star — it helps others find it.** ⭐

</div>
