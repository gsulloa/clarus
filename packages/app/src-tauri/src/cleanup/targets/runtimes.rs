use crate::cleanup::builders::Def;
use crate::cleanup::disk::{parse_human_size, path_exists};
use crate::cleanup::model::{Item, Status, Target, Tier};
use crate::cleanup::shell::{expand, has_tool, home, run_bash};

/// Old nvm Node versions — only when >3 installed, keeping current + latest LTS.
pub(in crate::cleanup) fn nvm_target() -> Target {
    let nvm_dir = std::env::var("NVM_DIR").unwrap_or_else(|_| format!("{}/.nvm", home()));
    let versions_dir = format!("{nvm_dir}/versions/node");

    if !path_exists(&versions_dir) {
        return Def {
            id: "nvm",
            name: "nvm — old Node versions",
            tier: Tier::Two,
            path: None,
            reason: "Node versions installed via nvm.",
            risk_note: "Regenerable — reinstall with `nvm install` if needed.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    // Sorted (version order) list of installed versions.
    let listing = run_bash(&format!("ls '{versions_dir}' 2>/dev/null | sort -V")).unwrap_or_default();
    let versions: Vec<String> = listing
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    if versions.len() <= 3 {
        return Def {
            id: "nvm",
            name: "nvm — old Node versions",
            tier: Tier::Two,
            path: None,
            reason: "Node versions installed via nvm (≤3 installed — nothing to prune).",
            risk_note: "Regenerable — reinstall with `nvm install` if needed.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::Empty,
            subitems: Vec::new(),
        }
        .into_target();
    }

    let current = run_bash(&format!("source '{nvm_dir}/nvm.sh' 2>/dev/null; nvm current 2>/dev/null"))
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    // Latest v24.x, else the newest installed.
    let lts = versions
        .iter()
        .filter(|v| v.starts_with("v24."))
        .last()
        .or_else(|| versions.last())
        .cloned()
        .unwrap_or_default();

    let mut subitems = Vec::new();
    for v in &versions {
        if *v == current || *v == lts {
            continue;
        }
        let path = format!("{versions_dir}/{v}");
        subitems.push(Item {
            id: v.clone(),
            label: v.clone(),
            path: path.clone(),
            size_bytes: 0,
            size_human: String::new(),
            meta: None,
            requires_double_confirm: false,
            command: format!(
                "source '{nvm_dir}/nvm.sh' 2>/dev/null; nvm uninstall '{v}' 2>&1 || rm -rf '{path}'"
            ),
        });
    }

    Def {
        id: "nvm",
        name: "nvm — old Node versions",
        tier: Tier::Two,
        path: None,
        reason: format!(
            "Old Node versions (keeping current {} and LTS {}).",
            if current.is_empty() { "?" } else { &current },
            if lts.is_empty() { "?" } else { &lts }
        )
        .leak(),
        risk_note: "Regenerable — reinstall with `nvm install` if needed.",
        caveat: None,
        requires_double_confirm: false,
        command: None,
        status: if subitems.is_empty() {
            Status::Empty
        } else {
            Status::Available
        },
        subitems,
    }
    .into_target()
}

/// pnpm content-addressable store — `pnpm store prune` when pnpm is present.
pub(in crate::cleanup) fn pnpm_store_target() -> Target {
    if !has_tool("pnpm") {
        return Def {
            id: "pnpm-store",
            name: "pnpm store",
            tier: Tier::One,
            path: None,
            reason: "pnpm's content-addressable package store.",
            risk_note: "Pure cache — regenerated automatically by the owning tool.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::ToolMissing,
            subitems: Vec::new(),
        }
        .into_target();
    }

    let store_path = run_bash("pnpm store path 2>/dev/null")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| expand("~/Library/pnpm"));

    Def {
        id: "pnpm-store",
        name: "pnpm store",
        tier: Tier::One,
        path: Some(store_path),
        reason: "pnpm's content-addressable package store.",
        risk_note: "Pure cache — regenerated automatically by the owning tool.",
        caveat: Some("`pnpm store prune` removes only orphaned packages, so freed space may be less than the reported store size."),
        requires_double_confirm: false,
        command: Some("pnpm store prune".to_string()),
        status: Status::Available,
        subitems: Vec::new(),
    }
    .into_target()
}

/// Old pyenv Python versions — keep the active version, offer the rest.
pub(in crate::cleanup) fn pyenv_target() -> Target {
    let pyenv_root = std::env::var("PYENV_ROOT").unwrap_or_else(|_| format!("{}/.pyenv", home()));
    let versions_dir = format!("{pyenv_root}/versions");

    if !path_exists(&versions_dir) {
        return Def {
            id: "pyenv",
            name: "pyenv — old Python versions",
            tier: Tier::Two,
            path: None,
            reason: "Python versions installed via pyenv.",
            risk_note: "Regenerable — reinstall with `pyenv install` if needed.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    let listing = run_bash(&format!("ls '{versions_dir}' 2>/dev/null | sort -V")).unwrap_or_default();
    let versions: Vec<String> = listing
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    // Active version (falls back to the newest installed if pyenv can't report).
    let active = run_bash("pyenv version-name 2>/dev/null")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty() && s != "system")
        .or_else(|| versions.last().cloned())
        .unwrap_or_default();

    let mut subitems = Vec::new();
    for v in &versions {
        if *v == active {
            continue;
        }
        let path = format!("{versions_dir}/{v}");
        subitems.push(Item {
            id: v.clone(),
            label: v.clone(),
            path: path.clone(),
            size_bytes: 0,
            size_human: String::new(),
            meta: None,
            requires_double_confirm: false,
            command: format!("pyenv uninstall -f '{v}' 2>&1 || rm -rf '{path}'"),
        });
    }

    Def {
        id: "pyenv",
        name: "pyenv — old Python versions",
        tier: Tier::Two,
        path: None,
        reason: format!(
            "Old Python versions (keeping active {}).",
            if active.is_empty() { "?" } else { &active }
        )
        .leak(),
        risk_note: "Regenerable — reinstall with `pyenv install` if needed.",
        caveat: None,
        requires_double_confirm: false,
        command: None,
        status: if subitems.is_empty() {
            Status::Empty
        } else {
            Status::Available
        },
        subitems,
    }
    .into_target()
}

/// rustup toolchains — keep the active/default toolchain, offer the rest.
pub(in crate::cleanup) fn rustup_target() -> Target {
    if !has_tool("rustup") {
        return Def {
            id: "rustup",
            name: "rustup — old toolchains",
            tier: Tier::Two,
            path: None,
            reason: "Rust toolchains installed via rustup.",
            risk_note: "Regenerable — reinstall with `rustup toolchain install` if needed.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::ToolMissing,
            subitems: Vec::new(),
        }
        .into_target();
    }
    let listing = run_bash("rustup toolchain list 2>/dev/null").unwrap_or_default();
    let toolchains_dir = format!("{}/.rustup/toolchains", home());
    let mut subitems = Vec::new();
    for line in listing.lines().map(str::trim).filter(|l| !l.is_empty()) {
        // Lines like "stable-... (default)", "... (active, default)", or "1.75.0-...".
        // The toolchain name is the first token; markers follow in parentheses and
        // never appear in a toolchain name, so a bare-word check is safe.
        let is_active =
            line.contains("default") || line.contains("active") || line.contains("override");
        if is_active {
            continue;
        }
        let name = line.split_whitespace().next().unwrap_or(line).to_string();
        let path = format!("{toolchains_dir}/{name}");
        subitems.push(Item {
            id: name.clone(),
            label: name.clone(),
            path,
            size_bytes: 0,
            size_human: String::new(),
            meta: None,
            requires_double_confirm: false,
            command: format!("rustup toolchain uninstall '{name}'"),
        });
    }
    Def {
        id: "rustup",
        name: "rustup — old toolchains",
        tier: Tier::Two,
        path: None,
        reason: "Old Rust toolchains (the active/default toolchain is kept).",
        risk_note: "Regenerable — reinstall with `rustup toolchain install` if needed.",
        caveat: None,
        requires_double_confirm: false,
        command: None,
        status: if subitems.is_empty() {
            Status::Empty
        } else {
            Status::Available
        },
        subitems,
    }
    .into_target()
}

/// Ollama models — one deletable subitem per model from `ollama list`.
pub(in crate::cleanup) fn ollama_target() -> Target {
    if !has_tool("ollama") {
        return Def {
            id: "ollama",
            name: "Ollama models",
            tier: Tier::Two,
            path: None,
            reason: "Local models pulled by Ollama.",
            risk_note: "Regenerable — re-pull with `ollama pull` if needed.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    let listing = run_bash("ollama list 2>/dev/null").unwrap_or_default();
    let mut subitems = Vec::new();
    for line in listing.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("NAME") {
            continue;
        }
        let tokens: Vec<&str> = trimmed.split_whitespace().collect();
        let Some(name) = tokens.first() else {
            continue;
        };
        // Columns: NAME  ID  SIZE_value  SIZE_unit  MODIFIED...
        let size_bytes = if tokens.len() >= 4 {
            parse_human_size(tokens[2], tokens[3])
        } else {
            0
        };
        subitems.push(Item {
            id: (*name).to_string(),
            label: (*name).to_string(),
            path: String::new(),
            size_bytes,
            size_human: String::new(),
            meta: None,
            requires_double_confirm: false,
            command: format!("ollama rm '{name}'"),
        });
    }

    Def {
        id: "ollama",
        name: "Ollama models",
        tier: Tier::Two,
        path: None,
        reason: "Local models pulled by Ollama.",
        risk_note: "Regenerable — re-pull with `ollama pull` if needed.",
        caveat: None,
        requires_double_confirm: false,
        command: None,
        status: if subitems.is_empty() {
            Status::Empty
        } else {
            Status::Available
        },
        subitems,
    }
    .into_target()
}
