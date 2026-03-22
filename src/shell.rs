// =============================================================================
//  shell.rs — Shell environment detection and command execution
//  https://github.com/paulfxyz/yo-rust
//
//  OVERVIEW
//  ────────
//  This module solves a real pain point: the AI needs to know exactly what
//  shell syntax is valid in the user's environment.  "Shell" is not just
//  Unix vs Windows — it's a matrix:
//
//    Platform  Shell          Syntax family         Invoke via
//    ────────  ─────          ─────────────         ──────────
//    macOS     zsh            POSIX + zsh extras    sh -c / zsh -c
//    macOS     bash           POSIX + bash extras   sh -c / bash -c
//    macOS     fish           fish syntax           fish -c
//    Linux     bash           POSIX + bash extras   sh -c / bash -c
//    Linux     zsh            POSIX + zsh extras    sh -c / zsh -c
//    Linux     fish           fish syntax           fish -c
//    Linux     dash/sh        pure POSIX            sh -c
//    Windows   PowerShell     PS syntax             powershell -Command
//    Windows   pwsh (PS 7)    PS syntax             pwsh -Command
//    Windows   cmd.exe        cmd syntax            cmd /C
//    Windows   Git Bash       POSIX (via MSYS2)     sh -c (via $SHELL)
//    Windows   WSL            Linux bash/zsh        (Linux path above)
//
//  WHY THIS MATTERS
//  ────────────────
//  PowerShell is NOT compatible with POSIX sh syntax.  Key differences:
//    • Pipelines: PS uses different objects, not text streams
//    • Command chaining: `&&` only works in PS 7+; use `;` or `if` in PS 5
//    • Redirections: `>` works but `>>` and `2>` behave differently
//    • Quoting: PS uses backtick as escape, not backslash
//    • No `grep` (use `Select-String`), no `ls` (it's an alias for Get-ChildItem)
//    • Environment variables: `$env:VARIABLE` not `$VARIABLE`
//
//  Git Bash on Windows is a special case: it runs POSIX sh via MSYS2 and
//  sets $SHELL to something like /usr/bin/bash.  Commands must still be
//  POSIX-compatible but they run inside a Windows process tree.
//
//  DETECTION STRATEGY
//  ──────────────────
//  1. Check $SHELL on Unix (always set by login shell, contains full path)
//  2. On Windows, check $PSVersionTable presence (PowerShell session)
//     or $COMSPEC (usually C:\Windows\System32\cmd.exe)
//  3. Distinguish PS 5 (Windows PowerShell) from PS 7+ (pwsh) because
//     PS 7 supports `&&` while PS 5 does not
//  4. Fall back to the safest assumption for the platform if nothing matches
// =============================================================================

/// The detected shell environment — passed to the AI as context.
#[derive(Debug, Clone, PartialEq)]
pub enum ShellKind {
    /// zsh (most macOS users, many Linux power users)
    Zsh,
    /// bash (default on most Linux distros, older macOS)
    Bash,
    /// fish shell (friendly interactive shell)
    Fish,
    /// dash or plain /bin/sh (common on Ubuntu as /bin/sh, CI systems)
    Sh,
    /// Windows PowerShell 5.x (built into Windows, limited && support)
    PowerShell5,
    /// PowerShell 7+ / pwsh (cross-platform, full && support)
    PowerShell7,
    /// cmd.exe (legacy Windows shell)
    Cmd,
    /// Git Bash / MSYS2 on Windows (POSIX-compatible)
    GitBash,
    /// Unknown — fall back to safe POSIX assumptions
    Unknown,
}

impl ShellKind {
    /// Detect the shell from environment variables.
    ///
    /// This is called once per session and cached in the detection result.
    /// We read several env vars in order of reliability:
    ///
    ///   $SHELL        — set by login shell on Unix, contains full path
    ///   $PSVersionTable — always set inside a PowerShell session
    ///   $ComSpec      — Windows default: C:\Windows\System32\cmd.exe
    ///   $MSYSTEM      — set by Git Bash / MSYS2 (MINGW64, MSYS, etc.)
    pub fn detect() -> Self {
        // ── Unix path ─────────────────────────────────────────────────────────
        if let Ok(shell) = std::env::var("SHELL") {
            let s = shell.to_lowercase();
            // Match on the basename portion — paths vary (/bin/zsh, /usr/local/bin/zsh, etc.)
            if s.contains("zsh")  { return Self::Zsh;  }
            if s.contains("fish") { return Self::Fish; }
            if s.contains("bash") {
                // Git Bash on Windows also sets $SHELL to something containing "bash"
                // Distinguish via $MSYSTEM which Git Bash always sets
                if std::env::var("MSYSTEM").is_ok() {
                    return Self::GitBash;
                }
                return Self::Bash;
            }
            if s.contains("dash") || s.ends_with("/sh") {
                return Self::Sh;
            }
            // Any other value in $SHELL — treat as generic POSIX sh
            return Self::Sh;
        }

        // ── Windows path ──────────────────────────────────────────────────────
        // PSVersionTable is an automatic variable inside every PS session.
        // We can't read it directly from an env var, but PS sets $PSModulePath
        // and a few others that distinguish it from cmd.
        if let Ok(ps_module) = std::env::var("PSModulePath") {
            // Distinguish PS 7 (pwsh) from PS 5 (powershell.exe) by checking
            // for the presence of "pwsh" or "PowerShell/7" in the module path
            if ps_module.contains("PowerShell\\7") || ps_module.contains("PowerShell/7") {
                return Self::PowerShell7;
            }
            return Self::PowerShell5;
        }

        // $COMSPEC is set in every cmd.exe session (and often inherited)
        if std::env::var("COMSPEC").is_ok() && cfg!(target_os = "windows") {
            return Self::Cmd;
        }

        Self::Unknown
    }

    /// A short, human-readable label for context injection into the AI prompt.
    /// The AI uses this to choose appropriate syntax.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Zsh           => "zsh",
            Self::Bash          => "bash",
            Self::Fish          => "fish",
            Self::Sh            => "sh",
            Self::PowerShell5   => "powershell5",
            Self::PowerShell7   => "powershell7",
            Self::Cmd           => "cmd.exe",
            Self::GitBash       => "gitbash (POSIX on Windows)",
            Self::Unknown       => "sh",
        }
    }

    /// Whether this shell uses POSIX-compatible syntax.
    #[allow(dead_code)]
    /// Used to select the correct executor in execute_commands().
    pub fn is_posix(&self) -> bool {
        matches!(self, Self::Zsh | Self::Bash | Self::Fish | Self::Sh | Self::GitBash | Self::Unknown)
    }

    /// Whether this shell is some flavour of PowerShell.
    pub fn is_powershell(&self) -> bool {
        matches!(self, Self::PowerShell5 | Self::PowerShell7)
    }

    /// The executable name to use when spawning a child process.
    /// Returns (program, args_prefix) — args_prefix is prepended before the command string.
    pub fn executor(&self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::PowerShell7   => ("pwsh",         &["-NoProfile", "-Command"]),
            Self::PowerShell5   => ("powershell",   &["-NoProfile", "-Command"]),
            Self::Cmd           => ("cmd",           &["/C"]),
            Self::Fish          => ("fish",          &["-c"]),
            // Everything else (zsh, bash, sh, git bash, unknown) goes through sh -c.
            // On macOS/Linux sh is always available at /bin/sh.
            // On Git Bash, sh is available via MSYS2.
            _                   => ("sh",            &["-c"]),
        }
    }
}

/// Execute a single shell command string in the detected shell environment.
///
/// Returns `true` if the process exited with status 0.
///
/// The shell is detected fresh for each invocation — this is fast (env var
/// reads, no syscalls) and avoids threading issues with a global.
pub fn run_in_shell(cmd: &str) -> std::io::Result<std::process::ExitStatus> {
    let shell = ShellKind::detect();
    let (program, prefix_args) = shell.executor();

    let mut command = std::process::Command::new(program);
    for arg in prefix_args {
        command.arg(arg);
    }
    command.arg(cmd);

    command
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status()
}
