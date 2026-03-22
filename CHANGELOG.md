# 📝 Changelog — Yo, Rust!

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versioning follows [Semantic Versioning](https://semver.org/).

---

## [1.1.0] — 2026-03-22

### 📚 Documentation & Code Quality

- **Deep source annotations** across all four modules (`main.rs`, `ai.rs`, `config.rs`, `ui.rs`)
  — every function, type, and design decision is now documented with the *why*, not just the *what*.
  Comments explain tradeoffs (blocking vs async, `sh -c` vs direct exec, regex vs LLM intent),
  performance characteristics, and future improvement paths.
- **`Cargo.toml` fully annotated** — every dependency includes a comment explaining what it does,
  why it was chosen over alternatives, and which features are enabled and why.

### 🎨 UI & UX

- **VERSION const** introduced in `ui.rs` — the banner version string is now a single source of
  truth. Changing the version only requires updating `Cargo.toml` and `VERSION` in `ui.rs`.
- **Help screen expanded** — shows macOS and Linux config paths separately, documents ↑/↓ history
  navigation, adds 2 new prompt examples (watch log, count code lines).
- **Suggestion box** minimum width increased (46 chars) and right-padding improved for better
  visual alignment across commands of varying length.

### 🧠 AI & Prompting

- **System prompt tightened** — Rule 4 now explicitly says "POSIX sh-compatible" and
  "avoid bash-isms" to reduce shell-specific syntax that breaks on `/bin/sh`.
- **Default model changed** to `meta-llama/llama-3.3-70b-instruct:free` — free-tier, no credit
  card required, excellent quality for shell command generation.
- **Model selection menu reordered** — free tier listed first to reduce friction for new users.

### 🔒 Security & Config

- **Security notes added** to `config.rs` documenting the plaintext storage tradeoff and the
  future keychain integration path.
- **Config path comment** explains the fallback chain (`dirs::config_dir()` → `"."`) and why
  atomic writes are not used for this file size.

---

## [1.0.0] — 2026-03-22

### 🌟 Initial Release

- 🚀 **Core REPL loop** — interactive terminal session launched by `yo`, `hi`, or `hello`
- 🤖 **ASCII banner** — split-panel robot illustration + block-letter YO, RUST! logo on every launch
- 🔑 **First-run setup** — prompts for OpenRouter API key and model on first launch; never asks again
- 🧠 **Natural language → shell commands** — structured JSON envelope prompt forces reliable,
  parseable output from any OpenRouter model
- ✅ **Y/N confirmation** — no command runs without explicit user approval; bare Enter = Y
- 💬 **AI explanation** — every suggestion includes a one-sentence plain-English description
- 🔁 **Intent detection** — 8 regex patterns detect "change my API key / switch model" phrases
  before any API call, triggering reconfiguration instantly
- ⌨️ **Shortcuts** — `!help` / `!h`, `!api`, `!exit` / `!q`, `Ctrl+D`
- 🌍 **Context injection** — OS, arch, CWD, and shell sent with every request for accurate,
  platform-appropriate command suggestions
- 🛡️ **Temperature 0.2** — deterministic, conservative outputs; tested across GPT-4o-mini,
  Claude 3 Haiku, and Llama 3.3 70B
- 🐚 **Shell aliases** — `hi` and `hello` added to `.zshrc` / `.bashrc` by the installer
- 📦 **One-command installer** (`yo.sh`) — auto-installs Rust via rustup, clones, builds
  release binary, installs to `/usr/local/bin/yo` or `~/.local/bin/yo`
- 📜 **In-session history** — rustyline provides ↑/↓ recall of previous prompts
- 📚 **Documentation** — `README.md`, `INSTALL.md`, `CHANGELOG.md`, MIT `LICENSE`

---

*Future releases will be tracked here.*
