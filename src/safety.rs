// =============================================================================
//  safety.rs — local, model-independent risk assessment of suggested commands
//  https://github.com/paulfxyz/mang
//
//  WHY THIS MODULE EXISTS
//  ──────────────────────
//  Until v3.0.7 the only safety net was rule 5 of the system prompt, which
//  *asks* the model to add a comment to catastrophic commands.  The v3.0.7
//  audit benchmark showed that this is not enough: the default model happily
//  returned `kill -9`, `rm -rf /tmp/build/*` and `chmod -R a+w ~` with no
//  warning at all, and reasoning models sometimes return nothing at all.
//  A prompt is a request; this module is an enforcement point.
//
//  HOW IT WORKS
//  ────────────
//  `assess()` scans every suggested command against a table of patterns that
//  describe well-known foot-guns (recursive deletes, raw disk writes, piping
//  a download into a shell, forced git history rewrites, …).  The result is
//  a list of short human-readable reasons.  The REPL uses it in exactly one
//  way: when the list is non-empty it prints the reasons and flips the
//  confirmation default from [Y/n] (Enter runs it) to [y/N] (Enter cancels).
//
//  WHAT IT IS NOT
//  ──────────────
//  It is a heuristic, not a sandbox.  It cannot prove a command is safe, and
//  it deliberately errs on the side of false positives — a spurious warning
//  costs one keypress, a missed `rm -rf` costs a disk.  It never blocks a
//  command; the human always has the final word.
// =============================================================================

use regex::Regex;
use std::sync::OnceLock;

/// One row of the rule table: a compiled pattern and the reason shown to the user.
struct Rule {
    pattern: Regex,
    reason:  &'static str,
}

/// Source table.  Patterns are matched case-insensitively against each command.
///
/// Notes on the less obvious entries:
///   * `rm` — flagged only when it carries a recursive/force flag; a plain
///     `rm file.txt` is routine and would otherwise train users to ignore warnings.
///   * the `find` rule catches `-delete` and `-exec rm`, the idiomatic
///     "delete matching files" forms that never contain the literal `rm -rf`.
///   * the pipe-to-shell rule tolerates `sudo` and common shell names.
const RULE_TABLE: &[(&str, &str)] = &[
    (r"\brm\s+(-\w*[rf]\w*|--recursive|--force|--no-preserve-root)", "recursive or forced delete (rm)"),
    (r"\bfind\b.*(-delete\b|-exec\s+rm\b)",                           "bulk delete via find"),
    (r"\bdd\b.*\bof=",                                                "raw write with dd"),
    (r"\b(mkfs(\.\w+)?|newfs\w*|wipefs|shred)\b",                     "formats or wipes storage"),
    (r"\bdiskutil\s+(erase\w*|zero\w*|secure\w*|partition\w*|repartition\w*)", "erases or repartitions a disk"),
    (r">\s*/dev/(sd|nvme|disk|rdisk|hd|mmcblk)",                      "writes directly to a disk device"),
    (r"\b(chmod|chown|chgrp)\s+(-\w*R\w*|--recursive)",               "recursive permission or ownership change"),
    (r"\b(sudo|doas)\b",                                              "runs with elevated privileges"),
    (r"\b(curl|wget)\b[^|;&]*\|\s*(sudo\s+)?(ba|z|da|k)?sh\b",        "pipes a download straight into a shell"),
    (r":\(\)\s*\{",                                                   "fork bomb pattern"),
    (r"\b(kill\s+-9|kill\s+-KILL|killall|pkill)\b",                   "force-kills processes"),
    (r"\bgit\s+(push\b.*(--force|\s-f\b)|reset\s+--hard|clean\s+-\w*f)", "irreversible git operation"),
    (r"\b(drop\s+(table|database|schema)|truncate\s+table)\b",        "destructive SQL"),
    (r"\b(shutdown|reboot|halt|poweroff)\b",                          "shuts down or restarts the machine"),
    // ── Windows (cmd.exe / PowerShell) ───────────────────────────────────────
    (r"\b(rd|rmdir)\s+.*/s\b",                                        "recursive directory delete (Windows)"),
    (r"\bdel\s+.*/[sfq]\b",                                           "forced delete (Windows)"),
    (r"\bremove-item\b.*-(recurse|force)\b",                          "recursive or forced delete (PowerShell)"),
    (r"\b(format\s+[a-z]:|format-volume|clear-disk)\b",               "formats a drive (Windows)"),
];

/// Compile the rule table exactly once per process.
/// `OnceLock` keeps this lazy (zero cost on startup) and thread-safe.
fn rules() -> &'static [Rule] {
    static RULES: OnceLock<Vec<Rule>> = OnceLock::new();
    RULES.get_or_init(|| {
        RULE_TABLE
            .iter()
            .filter_map(|(src, reason)| {
                // (?i) → case-insensitive.  A pattern that fails to compile is
                // skipped rather than panicking; the unit tests below make
                // sure that never happens in a released build.
                Regex::new(&format!("(?i){src}"))
                    .ok()
                    .map(|pattern| Rule { pattern, reason })
            })
            .collect()
    })
}

/// Return the de-duplicated reasons why `commands` deserve extra caution.
///
/// An empty vector means "nothing known-risky was spotted" — *not* "safe".
pub fn assess(commands: &[String]) -> Vec<&'static str> {
    let mut reasons: Vec<&'static str> = Vec::new();
    for cmd in commands {
        for rule in rules() {
            if rule.pattern.is_match(cmd) && !reasons.contains(&rule.reason) {
                reasons.push(rule.reason);
            }
        }
    }
    reasons
}

// =============================================================================
//  Tests — the first unit tests in the project (added by the v3.0.7 audit)
// =============================================================================
#[cfg(test)]
mod tests {
    use super::*;

    fn flagged(cmd: &str) -> bool {
        !assess(&[cmd.to_string()]).is_empty()
    }

    #[test]
    fn every_rule_compiles() {
        // If a pattern is invalid, `rules()` silently drops it — catch that here.
        assert_eq!(rules().len(), RULE_TABLE.len());
    }

    #[test]
    fn flags_the_classics() {
        for cmd in [
            "rm -rf /tmp/build/*",
            "rm -fr ~/old",
            "rm --recursive dir",
            "sudo apt update",
            "dd if=/dev/zero of=/dev/disk2",
            "mkfs.ext4 /dev/sdb1",
            "chmod -R 777 .",
            "curl -fsSL https://x.sh | sudo bash",
            "wget -qO- https://x.sh | sh",
            "kill -9 1234",
            "pkill node",
            "git push --force origin main",
            "git reset --hard HEAD~3",
            "find . -name '*.log' -delete",
            "find . -type f -exec rm {} +",
            ":(){ :|:& };:",
            "DROP TABLE users;",
            "Remove-Item C:\\tmp -Recurse -Force",
            "rd /s /q build",
            "diskutil eraseDisk APFS X disk3",
        ] {
            assert!(flagged(cmd), "expected a warning for: {cmd}");
        }
    }

    #[test]
    fn leaves_routine_commands_alone() {
        for cmd in [
            "ls -laS",
            "rm notes.txt",
            "git status",
            "git push origin main",
            "tar -czf logs.tar.gz logs",
            "find . -type f | head -n 5",
            "lsof -ti tcp:3000 | xargs kill",
            "brew install node",
            "curl -fsSL https://example.com/data.json -o data.json",
            "Get-Process | Sort-Object WorkingSet64 -Descending | Select-Object -First 5",
            "chmod +x script.sh",
        ] {
            assert!(!flagged(cmd), "unexpected warning for: {cmd}");
        }
    }

    #[test]
    fn reasons_are_deduplicated_across_commands() {
        let cmds = vec!["rm -rf a".to_string(), "rm -rf b".to_string()];
        assert_eq!(assess(&cmds).len(), 1);
    }
}
