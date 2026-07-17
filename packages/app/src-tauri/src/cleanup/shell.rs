// ─────────────────────────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────────────────────────

use std::process::Command;

pub(in crate::cleanup) fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

/// Expand a leading `~` to $HOME.
pub(in crate::cleanup) fn expand(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        format!("{}/{}", home(), rest)
    } else if path == "~" {
        home()
    } else {
        path.to_string()
    }
}

/// Single-quote a path for safe use in a shell command.
pub(in crate::cleanup) fn sq(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Run a command through a login shell, capturing stdout+stderr.
pub(in crate::cleanup) fn run_bash(cmd: &str) -> Result<String, String> {
    let output = Command::new("bash")
        .arg("-lc")
        .arg(cmd)
        .output()
        .map_err(|e| format!("Failed to spawn shell: {e}"))?;

    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    if output.status.success() {
        Ok(text)
    } else {
        Err(if text.trim().is_empty() {
            format!("Command exited with status {}", output.status)
        } else {
            text
        })
    }
}

/// `command -v <tool>` — true if the tool resolves on PATH.
pub(in crate::cleanup) fn has_tool(tool: &str) -> bool {
    Command::new("bash")
        .arg("-lc")
        .arg(format!("command -v {tool} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
