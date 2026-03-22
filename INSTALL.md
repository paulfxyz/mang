# Installation Guide — Yo, Rust!

> **Quick reference**
> ```bash
> # Install
> curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
>
> # Update
> curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/update.sh | bash
>
> # Uninstall
> curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/uninstall.sh | bash
> ```

---

## macOS / Linux — Install

### Option A — One-command (recommended)

Installs Rust automatically if you don't have it.

```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
```

Reload your shell:
```bash
source ~/.zshrc    # zsh
source ~/.bashrc   # bash
```

Then: `yo`

---

### Option B — Manual build

```bash
# Install Rust
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
```

---

## Windows — Install

### Option A — Git Bash (recommended, easiest)

Install [Git for Windows](https://git-scm.com/download/win) which includes Git Bash,
then open Git Bash and run:

```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
```

yo-rust will detect Git Bash and generate POSIX-compatible commands.

---

### Option B — WSL2

Inside a WSL2 terminal (Ubuntu, Debian, etc.) — identical to Linux:

```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/yo.sh | bash
```

yo-rust detects your WSL shell (bash/zsh) and generates Linux commands.

---

### Option C — PowerShell + Manual build

Requires Rust for Windows and Git. Run in PowerShell:

```powershell
# Install Rust (if not already installed)
winget install Rustlang.Rust.MSVC

# Install Git (if not already installed)
winget install Git.Git

# Clone and build
git clone https://github.com/paulfxyz/yo-rust
cd yo-rust
cargo build --release

# Install (choose a directory in your PATH)
copy target\release\yo.exe C:\Windows\System32\yo.exe
```

yo-rust auto-detects PowerShell 5 vs PowerShell 7 and generates the correct syntax for each.

---

## First launch

On first run, yo-rust asks:

1. **AI Backend** — OpenRouter (cloud, any model) or Ollama (local, private)
2. **API key** (OpenRouter only) — get one at [openrouter.ai/keys](https://openrouter.ai/keys)
3. **Model** — pick from the list or paste any slug
4. **Shell history** — whether to append confirmed commands to your history file
5. **Context size** — how many recent turns to remember for follow-up prompts (default: 5)

Config is saved to:
- macOS: `~/Library/Application Support/yo-rust/config.json`
- Linux: `~/.config/yo-rust/config.json`
- Windows: `%APPDATA%\yo-rust\config.json`

---

## Ollama setup

To use Ollama (local, private, offline):

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh     # macOS / Linux
# Windows: download from https://ollama.ai/download

# Pull a model
ollama pull llama3.2       # recommended general-purpose
ollama pull mistral        # fast, good at commands
ollama pull codellama      # code-focused sessions

# Launch yo-rust
yo
# Choose backend: 2) Ollama
```

Or switch to Ollama from within a session:
```
yo ›  use ollama
```

---

## Update

```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/update.sh | bash
```

Detects your installed version, checks latest on GitHub, skips if already current.
Never touches your config or aliases.

---

## Uninstall

```bash
curl -fsSL https://raw.githubusercontent.com/paulfxyz/yo-rust/main/uninstall.sh | bash
```

Removes binary, optionally removes config (asks first), cleans alias block from shell rc.
All prompts read from `/dev/tty` — works correctly whether piped or run directly.

### Manual uninstall

```bash
# Remove binary
sudo rm -f /usr/local/bin/yo
rm -f ~/.local/bin/yo              # if installed to user-local fallback

# Remove config (optional — contains your API key)
rm -rf ~/.config/yo-rust           # Linux
rm -rf ~/Library/Application\ Support/yo-rust   # macOS

# Remove aliases
# Open ~/.zshrc or ~/.bashrc and delete the yo-rust aliases block
```

To also remove Rust:
```bash
rustup self uninstall
```

---

## Troubleshooting

| Problem | Solution |
|---|---|
| `yo: command not found` | Run `source ~/.zshrc`. Check `/usr/local/bin` is in `$PATH`. |
| `OpenRouter returned 401` | API key invalid. Type `!api` inside yo-rust to update it. |
| `Build failed: error[E0...]` | Run `rustup update stable` to update your toolchain. |
| `hi` / `hello` not working | Run `source ~/.zshrc` to reload aliases. |
| Model returns no commands | Try `!api` to switch to a different model. Free-tier models may hit rate limits. |
| Stuck on "Thinking..." > 30 s | Check connection. For Ollama: is `ollama serve` running? |
| Ollama 404 or connection refused | Run `ollama serve` in another terminal. Check your `ollama_url` in config. |
| Wrong shell syntax on Windows | yo-rust auto-detects PS5/PS7/cmd. If wrong, type `!api` to reconfigure. |
| Uninstall prompt accepts nothing | Script reads from `/dev/tty`. Run in an interactive terminal, not inside another pipe. |

---

## Platform support

| Platform | Status |
|---|---|
| macOS — Apple Silicon (arm64) | Fully supported |
| macOS — Intel (x86_64) | Fully supported |
| Linux — x86_64 | Fully supported |
| Linux — ARM / Raspberry Pi | Fully supported |
| Windows — Git Bash | Fully supported |
| Windows — WSL2 | Fully supported |
| Windows — PowerShell 5 | Supported (auto-detected, syntax adapted) |
| Windows — PowerShell 7 | Supported (auto-detected, syntax adapted) |
| Windows — cmd.exe | Supported (auto-detected) |
| Windows — native build (no Git Bash / WSL) | Manual build required |
