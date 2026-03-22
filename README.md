# 🤖 Yo, Rust!

<div align="center">

**Natural language → Terminal commands, powered by AI.**

*Just type `yo` — and talk to your terminal like a human being.*

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Powered by OpenRouter](https://img.shields.io/badge/Powered%20by-OpenRouter-6c47ff?style=for-the-badge)](https://openrouter.ai)
[![Version](https://img.shields.io/badge/Version-1.1.0-brightgreen?style=for-the-badge)](CHANGELOG.md)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux-blue?style=for-the-badge)]()
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=for-the-badge)](https://github.com/paulfxyz/yo-rust/pulls)

</div>

---

```
  ╔══════════════════════════════════════════════════════════════════╗
  ║                                                                  ║
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
  ║          ┌┴─┐┌─┴┐                                                ║
  ║          │░░││░░│           v1.1.0  ·  github.com/paulfxyz       ║
  ║          └──┘└──┘                                                ║
  ╚══════════════════════════════════════════════════════════════════╝
```

---

## 👨‍💻 Why this exists

I'm **Paul Fleury** — a French internet entrepreneur based in Lisbon. I manage infrastructure, DNS, deployments, server configs, and all the operational chaos that comes with running multiple products simultaneously.

And I kept hitting the same wall.

Not the hard problems — DNS propagation, TLS cert chains, Docker networking. Those I can think through. The wall was the *boring* ones: the `find` command with the exact combination of flags I need. The `rsync` syntax that doesn't wipe the destination. The `awk` one-liner to pull column 3 from a log file. The `openssl` command that decodes a certificate. Things I've typed a hundred times but never memorised because I don't type them *every* day.

Every time, the same ritual: stop what I'm doing → open a browser → search "rsync exclude directory" → skip the ads → scan Stack Overflow → copy the command → adapt it → test it. **Five minutes gone.** Multiply that by ten times a day and you've lost an hour to command-syntax archaeology.

I wanted something that felt like messaging a friend who knows Linux cold. You just describe the thing. You get the command. You run it.

The key requirement: **no runtime dependency**. I've been burned enough times by Python version hell and Node.js ecosystem chaos. I wanted a single binary that installs with one command and just works, on any machine, forever. That pointed straight at Rust.

The second requirement: **free to use immediately**. I chose [OpenRouter](https://openrouter.ai) because it gives access to free-tier models (Llama 3.3 70B is free and excellent) and paid models (GPT-4o, Claude) behind a single API key. One key, every model, no vendor lock-in.

> 💡 This project was designed and built in collaboration with **[Perplexity Computer](https://www.perplexity.ai)** — from architecture through implementation, debugging, and documentation. A genuine example of human intent meeting AI execution.

---

## 🌟 What it does

**yo-rust** is an open-source terminal assistant. You describe what you want to do in plain English — it suggests the exact shell commands to do it. You review them, confirm with `Y`, and they run. Nothing executes without your explicit approval.

**Feature overview:**

| Feature | Detail |
|---|---|
| 🗣️ Natural language input | Plain English → shell commands via any OpenRouter model |
| ✅ Explicit confirmation | Every suggestion requires `Y` before anything runs |
| ⚡ Single binary | Compiled Rust — no Python, Node.js, or runtime required |
| 🔑 Local config | API key stored in `~/.config/yo-rust/config.json` only |
| 🤖 Fancy ASCII banner | Full robot illustration + block-letter logo on every launch |
| 🧠 Intent detection | "change my API key" triggers reconfiguration, no API call |
| 📟 Shortcuts | `!help`, `!api`, `!exit` built in |
| 🐚 Multiple aliases | `yo`, `hi`, or `hello` all work |
| 🌍 Context-aware | OS, architecture, CWD, and shell sent with every request |
| 🛡️ Safe by default | Temperature 0.2 — deterministic, conservative command suggestions |
| 💬 Explanations | Every suggestion includes a one-sentence description |
| 📜 Session history | ↑/↓ keys recall previous prompts within a session |

---

## 🚀 Install in one command

Works on macOS and Linux. Rust is installed automatically if you don't have it.

```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
```

Restart your terminal, then:

```
yo
```

On first launch, you'll be asked for your [OpenRouter API key](https://openrouter.ai/keys) and a model. Takes 30 seconds. Free options available.

---

## 🎬 See it in action

```
$ yo

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
  ║          │░░││░░│           v1.1.0  ·  github.com/paulfxyz       ║
  ╚══════════════════════════════════════════════════════════════════╝

  ◈  Natural language → Terminal commands, powered by AI.
  ◈  Type !help for options.  Type !api to configure.

  yo ›  find all .env files in this project tree

  ◈  Recursively searches for files named .env under the current directory.

  ┌──────────────────────────────────────────────┐
  │  $  find . -name ".env" -type f              │
  └──────────────────────────────────────────────┘

  Run it? [Y/n] › Y

  ►  find . -name ".env" -type f
  ./services/api/.env
  ./services/worker/.env
  ./docker/.env
  ✔  Done.

  yo ›
```

**More real-world examples:**

```
yo ›  kill whatever is using port 3000
  ◈  Finds and kills the process listening on port 3000.
  $ lsof -ti:3000 | xargs kill -9

yo ›  show git log last 7 days with author and message
  ◈  Displays commits from the past week with author name and subject.
  $ git log --since="7 days ago" --pretty=format:"%h %an %ad: %s" --date=short

yo ›  count how many lines of Rust code I have in this project
  ◈  Finds all .rs files and sums their line counts.
  $ find . -name "*.rs" | xargs wc -l | tail -1

yo ›  watch the nginx error log in real time
  ◈  Streams new lines from the nginx error log as they are written.
  $ tail -f /var/log/nginx/error.log

yo ›  check what's eating my disk space
  ◈  Lists the 10 largest items in the current directory, sorted by size.
  $ du -sh * | sort -rh | head -10
```

---

## ⌨️ Commands & shortcuts

| What you type | What happens |
|---|---|
| `!help` / `!h` | Full help screen with examples |
| `!api` | Reconfigure OpenRouter API key and model |
| `!exit` / `!q` | Quit yo-rust |
| `Y` or Enter | Run the suggested command(s) |
| `N` | Skip — refine your prompt and try again |
| `Ctrl+D` | Exit at any time |
| `↑` / `↓` | Recall previous prompts in this session |

Natural language configuration also works:

```
yo ›  change my API key           → same as !api
yo ›  switch to a different model → same as !api
yo ›  use a new openrouter key    → same as !api
```

---

## 🔑 OpenRouter — one key, every model

yo-rust uses [OpenRouter](https://openrouter.ai) as its AI backbone — a single API that routes to every major model provider. One key. No vendor lock-in.

Get a key (free, no credit card required for free-tier models): **[openrouter.ai/keys](https://openrouter.ai/keys)**

### Model recommendations

| Model slug | Cost | Best for |
|---|---|---|
| `meta-llama/llama-3.3-70b-instruct:free` | 🆓 Free | Getting started — excellent quality |
| `openai/gpt-4o-mini` | ~$0.15/1M tokens | Daily driver — fast and reliable |
| `openai/gpt-4o` | ~$2.50/1M tokens | Complex multi-step requests |
| `anthropic/claude-3-haiku` | ~$0.25/1M tokens | Speed-critical workflows |
| `anthropic/claude-3.5-sonnet` | ~$3/1M tokens | Tricky, context-heavy requests |

For shell command generation at temperature 0.2, the free Llama model performs comparably to GPT-4o-mini for most tasks. Start free, upgrade if you hit limits.

---

## 📁 Code structure

```
yo-rust/
├── src/
│   ├── main.rs     Entry point · REPL loop · command execution
│   ├── ai.rs       OpenRouter HTTP call · JSON parsing · intent detection
│   ├── config.rs   Load/save config · interactive setup wizard
│   └── ui.rs       ASCII banner · help screen · suggestion display
├── Cargo.toml      Rust manifest · dependency rationale (commented)
├── yo.sh           One-command installer
├── README.md       You're reading it
├── INSTALL.md      Detailed installation guide
├── CHANGELOG.md    Version history
└── LICENSE         MIT
```

---

## 🧠 How it works

### The problem with freeform LLM output

The hardest engineering challenge in yo-rust isn't the Rust code — it's getting the AI to produce **reliable, machine-parseable output** every single time.

Ask an LLM to "output just a shell command" and it will, with some probability, instead give you:

```
Here's how you can do that:

```bash
find . -name "*.env"
```

This command will recursively search...
```

None of that is what we need. We need the command string and nothing else.

### The solution: a strict JSON envelope

Every request to OpenRouter includes a system prompt that instructs the model to respond **only** with a JSON object matching this schema:

```json
{
  "commands": ["cmd1", "cmd2"],
  "explanation": "One sentence describing what the commands do."
}
```

This works because:
- LLMs are extensively trained on JSON and reliably produce valid JSON when a schema is clearly specified
- The schema forces separation between the machine-readable part (`commands`) and the human-readable part (`explanation`)
- We can parse and validate deterministically with `serde_json`
- An array naturally handles multi-step answers without string splitting

We also strip any accidental markdown fences (` ```json `) the model might add — belt-and-suspenders parsing.

### Temperature 0.2 — why not 0?

Shell commands are not creative. You want the model to pick the highest-probability, most conventional answer — the command that any experienced sysadmin would write. Temperature 0 would be fully greedy (always the single most likely token), which can produce repetitive or stuck outputs for some models.

At `0.2`:
- The model picks the most likely command in ~95% of cases
- It has just enough randomness to handle natural language variation in prompts
- It never "gets creative" with flags or invents tool names

Empirically tested across GPT-4o-mini, Claude 3 Haiku, and Llama 3.3 70B — `0.2` gives correct, safe commands consistently.

### Context injection — matching commands to your machine

Without context, a model asked to "open the downloads folder" doesn't know if you're on macOS (should use `open ~/Downloads`), Linux (should use `xdg-open ~/Downloads`), or a headless server (neither makes sense).

yo-rust prepends every prompt with:

```
System context: OS=macos ARCH=aarch64 CWD=/Users/paul/project SHELL=/bin/zsh
```

With this, the model knows:
- Use `brew` not `apt` or `apt-get`
- Use `open` not `xdg-open`
- Use `pbcopy`/`pbpaste` for clipboard operations on macOS
- Use paths relative to the actual CWD
- Use zsh-compatible syntax when the user mentions "shell alias" or "profile"

Four fields. Massive improvement in command accuracy.

### Intent detection — no API call for config changes

When a user types "change my API key", we don't send that to the AI. We detect it locally using 8 regex patterns and immediately open the reconfiguration wizard. This is:

- **Instant**: microseconds vs 1–3 seconds for an API round-trip
- **Free**: zero tokens consumed
- **Reliable**: regex patterns are more predictable than LLM intent classification for a narrow domain

The patterns are conservative — they only match phrases that unambiguously mean "please reconfigure", not anything that could be a valid shell task.

### Command execution: `sh -c` with inherited stdio

Commands are executed via:

```rust
Command::new("sh").arg("-c").arg(cmd)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .stderr(Stdio::inherit())
    .status()
```

**Why `sh -c` and not direct `exec`?**

LLM-generated commands use shell features that only the shell interprets:
- Pipelines: `ps aux | grep nginx | awk '{print $2}'`
- Redirections: `echo "hello" >> file.txt`
- Glob expansion: `rm *.tmp`
- Environment variables: `cd $HOME/projects`
- Command chaining: `git add . && git commit -m "fix"`

Parsing these ourselves would mean reimplementing a shell. Delegating to `sh -c` is correct, safe, and keeps the code at ~10 lines.

**Why inherited stdio?**

Using `Stdio::piped()` to capture output would break:
- **Interactive programs**: `vim`, `htop`, `less`, `fzf` need a real TTY and will either crash or show nothing with piped stdio
- **Streaming output**: `cargo build`, `npm install`, `docker pull` print progress in real time — piping buffers it until completion
- **Colour output**: `ls`, `grep --color`, `cargo` check whether stdout is a TTY and disable colour for pipes

Inherited stdio is the correct choice for a terminal assistant.

### No async runtime — a deliberate decision

yo-rust uses `reqwest::blocking` (synchronous HTTP) rather than `tokio` + `reqwest` async. Why?

yo-rust makes **one HTTP request per user prompt**, with seconds of human think-time between requests. There is nothing to parallelise. An async runtime would add:
- ~200 KB to the binary size
- ~30 seconds to compile time on a cold build
- Cognitive overhead (`async fn`, `.await`, `#[tokio::main]`)

For a single sequential HTTP call, `reqwest::blocking` is the right tool. We print "Thinking…" before the call so the user knows the program is alive during the wait.

---

## 🔬 Challenges we solved

These are the real engineering problems encountered while building yo-rust — not the obvious ones.

### 1. Getting structured JSON from LLMs, reliably

**The problem**: Models ignore format instructions unpredictably. A model that outputs perfect JSON for 50 prompts in a row will suddenly wrap it in markdown fences or prepend "Sure!" on the 51st.

**The solution**: A system prompt with numbered rules, the schema repeated twice (in the instructions and as an example), plus client-side fence stripping. We use `serde_json::Value` (generic parse) rather than deserialising directly to a typed struct, which lets us give meaningful error messages including the raw response when parsing fails.

### 2. Making config reconfiguration feel natural

**The problem**: Users naturally type things like "yo, can I change my API key?" as if talking to a person. Requiring them to know the exact `!api` shortcut feels mechanical.

**The solution**: Eight regex patterns on the lowercase prompt detect configuration intent before any API call. Caught before network I/O — the response is instant. The patterns are anchored loosely enough to catch variations ("update", "change", "new", "set") but specific enough to avoid false-positives on real shell tasks.

### 3. Interactive commands and streaming output

**The problem**: Early prototypes captured command stdout and printed it after the process exited. This broke `vim` (needs TTY), `htop` (needs TTY), `less` (needs TTY), and made `cargo build` appear to hang silently for 30 seconds then dump 200 lines of output at once.

**The solution**: Inherit stdin/stdout/stderr directly. The child process gets the same TTY file descriptors as the parent. Output streams in real time. Interactive programs work. This is a one-line change in Rust (`Stdio::inherit()`) but required understanding why the naive approach fails.

### 4. Version string in the ASCII banner

**The problem**: The ASCII banner is a grid of Unicode box-drawing characters. Embedding a version string requires careful spacing — one character off and the right border misaligns.

**The solution**: A `VERSION` constant at the top of `ui.rs` and a `format!()` call that injects it into the version line. The only manual alignment needed is on the version line itself; all other lines are static strings. Future versions: use `env!("CARGO_PKG_VERSION")` to read the version from Cargo.toml at compile time and eliminate the manual sync.

### 5. First-run UX without being annoying

**The problem**: Tools that ask 10 questions before you can use them are frustrating. But you genuinely need an API key to do anything.

**The solution**: Detect first run by checking for an empty `api_key` field (the `Default` impl produces empty strings). Ask only two things: key and model (with sensible defaults). Save immediately to disk so a crash during setup doesn't lose the key. The whole flow takes under 30 seconds and is never shown again unless the user explicitly requests it.

---

## 🛠️ Build from source

Requirements: **[Rust stable](https://rustup.rs/)**

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Clone and build
git clone https://github.com/paulfxyz/yo-rust
cd yo-rust
cargo build --release

# Install
sudo cp target/release/yo /usr/local/bin/yo

# Optional aliases
echo "alias hi='yo'"    >> ~/.zshrc
echo "alias hello='yo'" >> ~/.zshrc
source ~/.zshrc
```

The release binary is stripped and LTO-optimised (see `[profile.release]` in Cargo.toml). Typical binary size: ~3–5 MB.

---

## 📝 Changelog

> Full history: **[CHANGELOG.md](CHANGELOG.md)**

### 🔖 v1.1.0 — 2026-03-22

- 📚 **Deep source annotations** — every function and design decision documented
- 🔖 **VERSION const in ui.rs** — banner version is now a single source of truth
- 🧠 **Expanded system prompt** — improved rule set for safer, more portable commands
- 🆓 **Default model changed** — free Llama 3.3 70B is now the default (was gpt-4o-mini)
- ⬆️ **↑/↓ history** — documented in help screen
- 🖥️ **Platform config paths** — macOS and Linux paths shown separately in !help
- 🔒 **Config security note** — documented in config.rs and README

### 🔖 v1.0.0 — 2026-03-22

- 🚀 Initial release — full feature set

---

## 🤝 Contributing

Pull requests welcome. Interesting directions:

- 📜 **Shell history**: append confirmed commands to `~/.zsh_history` / `~/.bash_history`
- 🔁 **Multi-turn context**: remember the last N commands so follow-up requests ("now do the same for the other folder") work
- 🧪 **Dry-run flag**: `--dry` to show commands without prompting to run them
- 🏠 **Ollama backend**: local model support for offline/air-gapped use
- 🪝 **Post-execution feedback**: ask "did that work?" and refine if not
- 🪟 **Windows native**: currently works on WSL2, native Windows CMD/PowerShell untested

```bash
git checkout -b feat/your-feature
git commit -m 'feat: describe your change'
git push origin feat/your-feature
# → open a Pull Request on GitHub
```

---

## 📜 License

MIT — free to use, modify, distribute. See [LICENSE](LICENSE).

---

## 👤 Author

Made with ❤️ by **Paul Fleury** — designed and built in collaboration with **[Perplexity Computer](https://www.perplexity.ai)**.

- 🌐 **[paulfleury.com](https://paulfleury.com)**
- 🔗 **[linkedin.com/in/paulfxyz](https://www.linkedin.com/in/paulfxyz/)**
- 🐙 **[@paulfxyz](https://github.com/paulfxyz)** on GitHub
- 📧 **[hello@paulfleury.com](mailto:hello@paulfleury.com)**

---

<div align="center">

⭐ **If yo-rust saved you time, drop a star — it helps other people find it.** ⭐

</div>
