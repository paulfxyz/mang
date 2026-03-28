# 📦 Installation Guide — mang.sh 句芒

> **Quick reference**
> ```bash
> # Install
> curl -fsSL https://mang.sh/install | bash        # macOS / Linux
> iwr -useb https://mang.sh/install.ps1 | iex       # Windows PowerShell
>
> # Update
> curl -fsSL https://mang.sh/update | bash
> iwr -useb https://mang.sh/update.ps1 | iex
>
> # Uninstall
> curl -fsSL https://mang.sh/uninstall | bash
> iwr -useb https://mang.sh/uninstall.ps1 | iex
> ```

---

## macOS / Linux — Install

### Option A — One command (recommended)

```bash
curl -fsSL https://mang.sh/install | bash
```

Installs Rust automatically if you don't have it. Reload your shell after:

```bash
source ~/.zshrc    # zsh
source ~/.bashrc   # bash
```

Then type `yo` to summon the spirit messenger.

---

### Option B — Manual build from source

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Clone and build
git clone https://github.com/paulfxyz/mang-sh
cd mang-sh
cargo build --release

# Install
sudo cp target/release/yo /usr/local/bin/yo

# Optional aliases
echo "alias hi='yo'"    >> ~/.zshrc
echo "alias hello='yo'" >> ~/.zshrc
```

---

## Windows — Install

> ⚠️ **Important:** On Windows, `curl` in PowerShell is an alias for `Invoke-WebRequest`. It does **not** accept `-fsSL` flags. The Unix command `curl -fsSL ... | bash` will fail in PowerShell. Use the options below.

### Option A — PowerShell native (recommended)

```powershell
iwr -useb https://mang.sh/install.ps1 | iex
```

Works in PowerShell 5 and 7. No Git Bash, no WSL needed. Installs Rust automatically, builds mang.sh, sets up PATH and `yo`/`hi`/`hello` aliases in `$PROFILE`.

If you get an execution policy error:
```powershell
Set-ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### Option B — Git Bash

Install [Git for Windows](https://git-scm.com/download/win), open Git Bash:

```bash
curl -fsSL https://mang.sh/install | bash
```

### Option C — WSL2

Inside a WSL2 terminal — identical to Linux:

```bash
curl -fsSL https://mang.sh/install | bash
```

---

## First launch

On first run, mang.sh asks for:

1. **AI Backend** — OpenRouter (cloud) or Ollama (local, private)
2. **API key** (OpenRouter only) — get one at [openrouter.ai/keys](https://openrouter.ai/keys)
3. **Model** — press Enter for `openai/gpt-4o-mini` (default) or pick from the list
4. **Shell history** — whether to append confirmed commands to your history file
5. **Context size** — how many prior turns to remember for follow-up prompts (default 5)

Config saved to:
- macOS: `~/Library/Application Support/mang-sh/config.json`
- Linux: `~/.config/mang/config.json`
- Windows: `%APPDATA%\mang-sh\config.json`

---

## Ollama (local, private, offline)

```bash
# Install Ollama
curl -fsSL https://ollama.ai/install.sh | sh

# Pull a model
ollama pull llama3.2

# Launch mang.sh — choose Ollama during setup
yo
```

Or switch from within a session: `yo ›  use ollama`

---

## Update

```bash
curl -fsSL https://mang.sh/update | bash             # macOS/Linux
iwr -useb https://mang.sh/update.ps1 | iex            # Windows PS
```

Or from within mang.sh: `!update`

Detects installed version, checks latest, skips if current, replaces binary in-place. Config never touched.

---

## Uninstall

```bash
curl -fsSL https://mang.sh/uninstall | bash           # macOS/Linux
iwr -useb https://mang.sh/uninstall.ps1 | iex         # Windows PS
```

Removes binary, asks before deleting config, cleans alias block from shell rc.

### Manual uninstall

```bash
sudo rm -f /usr/local/bin/yo
rm -f ~/.local/bin/yo

# Config (optional — contains your API key)
rm -rf ~/.config/mang
rm -rf ~/Library/Application\ Support/mang   # macOS

# Legacy yo-rust config (if you used mang.sh before v3.0.0)
rm -rf ~/.config/yo-rust
rm -rf ~/Library/Application\ Support/yo-rust   # macOS

# Aliases — open ~/.zshrc or ~/.bashrc and delete the mang.sh aliases block
```

To also remove Rust: `rustup self uninstall`

---

## Troubleshooting

| Problem | Solution |
|---|---|
| Upgrading from yo-rust (pre-v3.0.0) | The uninstall script automatically finds and removes the old `~/.config/yo-rust` config. Or delete it manually: `rm -rf ~/.config/yo-rust` |
| `curl -fsSL` fails in PowerShell | Use `iwr -useb https://mang.sh/install.ps1 \| iex` — PowerShell's `curl` is `Invoke-WebRequest` and doesn't accept `-fsSL` |
| `yo: command not found` | Run `source ~/.zshrc`. Check `/usr/local/bin` is in `$PATH`. |
| `OpenRouter returned 401` | API key invalid — type `!api` inside mang.sh to update it |
| Build failed `error[E0...]` | `rustup update stable` |
| `hi` / `hello` not working | `source ~/.zshrc` |
| Model returns no commands | Try `!api` to switch models. Free-tier models may hit rate limits |
| Stuck on "Thinking..." | Check connection. For Ollama: is `ollama serve` running? |
| Wrong syntax on Windows | mang.sh auto-detects PS5/PS7/cmd. If wrong, type `!api` |

---

## Platform support

| Platform | Status |
|---|---|
| macOS — Apple Silicon (arm64) | ✅ Fully supported |
| macOS — Intel (x86_64) | ✅ Fully supported |
| Linux — x86_64 | ✅ Fully supported |
| Linux — ARM / Raspberry Pi | ✅ Fully supported |
| Windows — Git Bash | ✅ Fully supported |
| Windows — WSL2 | ✅ Fully supported |
| Windows — PowerShell 5 | ✅ Supported (auto-detected, syntax adapted) |
| Windows — PowerShell 7 | ✅ Supported (auto-detected) |
| Windows — cmd.exe | ✅ Supported (auto-detected) |
