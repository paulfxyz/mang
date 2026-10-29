// =============================================================================
//  updater.rs — Background update check & !update shortcut
//  https://github.com/paulfxyz/mang
//
//  OVERVIEW
//  ────────
//  On every launch, mang.sh silently checks GitHub for a newer version.
//  The check runs in a background thread so startup latency is zero.
//  If a newer version is found, the user sees a one-liner notification
//  AFTER the banner (before the first prompt) and is offered Y/N to update.
//
//  The check is rate-limited: we record the last check time in the config
//  directory and skip the network call if the check was done within 24 hours.
//  This avoids hammering GitHub on every single `yo` invocation.
//
//  WHAT HAPPENS ON Y
//  ──────────────────
//  mang.sh shells out to:
//    curl -fsSL https://raw.githubusercontent.com/paulfxyz/mang/main/update.sh | bash
//  on Unix, or the equivalent iwr/iex on Windows PowerShell.
//  The update script rebuilds and replaces the binary in-place, then exits.
//  mang.sh itself exits after launching the update so the user restarts fresh.
//
//  SHORTCUTS
//  ─────────
//  !update / !upd    — check for updates right now and offer to install
//  !check            — same as !update
//
//  RATE LIMITING
//  ─────────────
//  We store the last-check timestamp in:
//    ~/.config/mang/last_update_check
//  (a simple Unix timestamp file — one integer, no JSON parsing needed)
//  If the file is fresh (< 24 hours old), the background check is skipped.
//  The !update / !check shortcuts always force a fresh check regardless.
// =============================================================================

use colored::Colorize;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const CHECK_INTERVAL_SECS: u64 = 86_400; // 24 hours
const RAW_BASE: &str = "https://raw.githubusercontent.com/paulfxyz/mang/main";

/// Result of an update check.
#[derive(Debug)]
pub enum UpdateStatus {
    /// A newer version is available — contains the new version string
    UpdateAvailable(String),
    /// Already on the latest version
    UpToDate,
    /// Could not reach GitHub (offline, rate limit, etc.) — silently ignored
    Unavailable,
}

// =============================================================================
//  check_for_update — fetch latest version from Cargo.toml on GitHub
// =============================================================================

/// Fetch the latest version from GitHub and compare to the running version.
///
/// This function makes a single HTTP GET to Cargo.toml on the main branch.
/// It is designed to be called from a background thread — any error produces
/// `Unavailable`, never a panic or user-visible error.
///
/// `force` — if true, bypass the 24-hour rate limit and always check.
pub fn check_for_update(force: bool) -> UpdateStatus {
    // Rate-limit check (skipped when force=true)
    if !force && !should_check() {
        return UpdateStatus::UpToDate; // treated as "nothing new" for display
    }

    let current = env!("CARGO_PKG_VERSION");

    let client = match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(6))
        .build()
    {
        Ok(c) => c,
        Err(_) => return UpdateStatus::Unavailable,
    };

    let resp = match client.get("https://raw.githubusercontent.com/paulfxyz/mang/main/Cargo.toml").send() {
        Ok(r) => r,
        Err(_) => return UpdateStatus::Unavailable,
    };

    let body = match resp.text() {
        Ok(t) => t,
        Err(_) => return UpdateStatus::Unavailable,
    };

    // Extract: version = "X.Y.Z"
    let latest = body
        .lines()
        .find(|l| l.trim_start().starts_with("version"))
        .and_then(|l| l.split('"').nth(1))
        .map(str::to_string);

    // Record that we checked (update the timestamp file)
    record_check_time();

    match latest {
        // v3.0.7: compare numerically.  The old `!=` also fired when the
        // local build was NEWER than main (a dev build, or main briefly
        // behind a release) and offered a "downgrade" as an update.
        Some(v) if is_newer(&v, current) => UpdateStatus::UpdateAvailable(v),
        Some(_) => UpdateStatus::UpToDate,
        None    => UpdateStatus::Unavailable,
    }
}

// =============================================================================
//  should_check — rate-limit: only check once per 24 hours
// =============================================================================

fn should_check() -> bool {
    let path = last_check_path();
    let now = unix_now();

    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(last) = content.trim().parse::<u64>() {
            return now.saturating_sub(last) >= CHECK_INTERVAL_SECS;
        }
    }
    true // no file or unparseable → check now
}

fn record_check_time() {
    let path = last_check_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, unix_now().to_string());
}

fn last_check_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("mang")
        .join("last_update_check")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

// =============================================================================
//  print_update_notice — shown after the banner when an update is available
// =============================================================================

pub fn print_update_notice(new_version: &str) {
    println!();
    println!(
        "  {}  {}  {}  {}",
        "◈".yellow().bold(),
        "Update available:".white().bold(),
        format!("v{new_version}").cyan().bold(),
        "— type !update to install".dimmed()
    );
    println!();
}

// =============================================================================
//  run_update — shell out to the update script
// =============================================================================

/// Invoke the platform-appropriate update script and exit mang.sh.
///
/// On Unix: pipes update.sh through bash via `sh -c`.
/// On Windows PowerShell: uses iwr | iex.
///
/// Returns an error string if the launch itself fails.
pub fn run_update() -> Result<(), String> {
    println!();
    println!("{}", "  ◌  Launching updater…".dimmed());
    println!();

    #[cfg(target_os = "windows")]
    {
        use crate::shell::ShellKind;
        let cmd = format!(
            "iwr -useb {RAW_BASE}/update.ps1 | iex"
        );
        let (prog, args) = ShellKind::detect().executor();
        let status = std::process::Command::new(prog)
            .args(args)
            .arg(&cmd)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();
        match status {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Could not launch updater: {e}")),
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let cmd = format!("curl -fsSL {RAW_BASE}/update.sh | bash");
        let status = std::process::Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status();
        match status {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Could not launch updater: {e}")),
        }
    }
}

// =============================================================================
//  Version comparison (v3.0.7)
// =============================================================================

/// Parse "MAJOR.MINOR.PATCH" (a leading `v` and any `-pre` suffix are ignored).
fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let core = v.trim().trim_start_matches('v').split(['-', '+']).next()?;
    let mut it = core.split('.').map(|p| p.parse::<u64>().ok());
    Some((it.next()??, it.next().flatten().unwrap_or(0), it.next().flatten().unwrap_or(0)))
}

/// True when `latest` is strictly newer than `current`.
/// Unparseable versions never trigger an update prompt.
fn is_newer(latest: &str, current: &str) -> bool {
    match (parse_version(latest), parse_version(current)) {
        (Some(l), Some(c)) => l > c,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_versions_are_detected() {
        assert!(is_newer("3.0.7", "3.0.6"));
        assert!(is_newer("3.1.0", "3.0.9"));
        assert!(is_newer("4.0.0", "3.9.9"));
        assert!(is_newer("3.0.10", "3.0.9")); // numeric, not lexicographic
    }

    #[test]
    fn same_or_older_is_not_an_update() {
        assert!(!is_newer("3.0.6", "3.0.6"));
        assert!(!is_newer("3.0.5", "3.0.6"));
        assert!(!is_newer("garbage", "3.0.6"));
        assert!(!is_newer("3.0.7", "garbage"));
    }

    #[test]
    fn v_prefix_and_prerelease_suffix_are_tolerated() {
        assert_eq!(parse_version("v3.0.6"), Some((3, 0, 6)));
        assert_eq!(parse_version("3.0.6-beta.1"), Some((3, 0, 6)));
    }
}
