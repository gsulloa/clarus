// ─────────────────────────────────────────────────────────────────
// CATALOG DEFINITIONS  (fast — no `du`; reused by scan and clean)
// ─────────────────────────────────────────────────────────────────

use super::disk::path_exists;
use super::model::{Item, Status, Target, Tier};
use super::shell::{expand, home, sq};

/// A small builder to reduce boilerplate for the many simple targets.
pub(in crate::cleanup) struct Def {
    pub(in crate::cleanup) id: &'static str,
    pub(in crate::cleanup) name: &'static str,
    pub(in crate::cleanup) tier: Tier,
    pub(in crate::cleanup) path: Option<String>,
    pub(in crate::cleanup) reason: &'static str,
    pub(in crate::cleanup) risk_note: &'static str,
    pub(in crate::cleanup) caveat: Option<&'static str>,
    pub(in crate::cleanup) requires_double_confirm: bool,
    pub(in crate::cleanup) command: Option<String>,
    pub(in crate::cleanup) status: Status,
    pub(in crate::cleanup) subitems: Vec<Item>,
}

impl Def {
    pub(in crate::cleanup) fn into_target(self) -> Target {
        Target {
            id: self.id.to_string(),
            name: self.name.to_string(),
            tier: self.tier,
            path: self.path,
            size_bytes: 0,
            size_human: String::new(),
            status: self.status,
            reason: self.reason.to_string(),
            risk_note: self.risk_note.to_string(),
            caveat: self.caveat.map(|c| c.to_string()),
            requires_double_confirm: self.requires_double_confirm,
            command: self.command,
            subitems: self.subitems,
        }
    }
}

pub(in crate::cleanup) fn tier1(
    id: &'static str,
    name: &'static str,
    path: &str,
    reason: &'static str,
    caveat: Option<&'static str>,
    command: String,
) -> Target {
    Def {
        id,
        name,
        tier: Tier::One,
        path: Some(path.to_string()),
        reason,
        risk_note: "Pure cache — regenerated automatically by the owning tool.",
        caveat,
        requires_double_confirm: false,
        command: Some(command),
        status: Status::Available,
        subitems: Vec::new(),
    }
    .into_target()
}

/// A single-command Tier 3 target: irreplaceable personal data, deletable only
/// behind the double-confirm. Clears the path's contents (including dotfiles),
/// tolerating an empty or absent directory.
pub(in crate::cleanup) fn tier3_simple(id: &'static str, name: &'static str, path: &str) -> Target {
    let q = sq(&expand(path));
    let command = format!("rm -rf {q}/* {q}/.[!.]* 2>/dev/null; true");
    Def {
        id,
        name,
        tier: Tier::Three,
        path: Some(path.to_string()),
        reason: "Persistent personal data.",
        risk_note: "Permanent and irreplaceable — deleting this cannot be undone.",
        caveat: None,
        requires_double_confirm: true,
        command: Some(command),
        status: Status::Available,
        subitems: Vec::new(),
    }
    .into_target()
}

/// A Tier 3 collection target: enumerate the members of `enum_dir` (kept when
/// `keep` returns true) as individually deletable double-confirm subitems, plus
/// a top-level group command that removes them all (the "Clean all" button).
/// Reports NotInstalled when `enum_dir` is absent, Empty when it has no members.
/// `display_path` is what the UI shows (may differ from `enum_dir`).
pub(in crate::cleanup) fn tier3_collection(
    id: &'static str,
    name: &'static str,
    display_path: Option<String>,
    enum_dir: &str,
    keep: impl Fn(&str) -> bool,
) -> Target {
    let expanded_dir = expand(enum_dir);
    if !std::path::Path::new(&expanded_dir).exists() {
        return Def {
            id,
            name,
            tier: Tier::Three,
            path: display_path,
            reason: "Persistent personal data.",
            risk_note: "Permanent and irreplaceable — deleting this cannot be undone.",
            caveat: None,
            requires_double_confirm: true,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    let mut members: Vec<std::path::PathBuf> = std::fs::read_dir(&expanded_dir)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .map(|n| keep(&n.to_string_lossy()))
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    members.sort();

    let mut subitems = Vec::new();
    for member in &members {
        let member_str = member.to_string_lossy().to_string();
        let label = member
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| member_str.clone());
        subitems.push(Item {
            id: label.replace('/', "_"),
            label,
            path: member_str.clone(),
            size_bytes: 0,
            size_human: String::new(),
            meta: None,
            requires_double_confirm: true,
            command: format!("rm -rf {}", sq(&member_str)),
        });
    }

    // Group command deletes exactly the enumerated members.
    let group_command = if subitems.is_empty() {
        None
    } else {
        let joined: String = members
            .iter()
            .map(|m| sq(&m.to_string_lossy()))
            .collect::<Vec<_>>()
            .join(" ");
        Some(format!("rm -rf {joined} 2>/dev/null; true"))
    };

    Def {
        id,
        name,
        tier: Tier::Three,
        path: display_path,
        reason: "Persistent personal data.",
        risk_note: "Permanent and irreplaceable — deleting this cannot be undone.",
        caveat: None,
        requires_double_confirm: true,
        command: group_command,
        status: if subitems.is_empty() {
            Status::Empty
        } else {
            Status::Available
        },
        subitems,
    }
    .into_target()
}

// ── Container / special target builders ──────────────────────────

/// A single cache directory that may be absent (reports NotInstalled then).
/// `path` is measured directly; `command` clears it. `path`/`command` are given
/// unescaped and escaped respectively by the caller as needed.
pub(in crate::cleanup) fn cache_or_missing(
    id: &'static str,
    name: &'static str,
    reason: &'static str,
    caveat: Option<&'static str>,
    path: &str,
    command: String,
) -> Target {
    if !path_exists(path) {
        return Def {
            id,
            name,
            tier: Tier::One,
            path: None,
            reason,
            risk_note: "Pure cache — regenerated automatically by the owning tool.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }
    Def {
        id,
        name,
        tier: Tier::One,
        path: Some(path.to_string()),
        reason,
        risk_note: "Pure cache — regenerated automatically by the owning tool.",
        caveat,
        requires_double_confirm: false,
        command: Some(command),
        status: Status::Available,
        subitems: Vec::new(),
    }
    .into_target()
}

/// An Electron app's HTTP/GPU/service-worker caches under `base` (an Application
/// Support directory). Reports NotInstalled when `base` is absent, so persistent
/// user data is never touched. Measures the `Cache` subdir (mirrors slack-cache).
pub(in crate::cleanup) fn electron_cache_target(
    id: &'static str,
    name: &'static str,
    reason: &'static str,
    base: &str,
) -> Target {
    if !path_exists(base) {
        return Def {
            id,
            name,
            tier: Tier::One,
            path: None,
            reason,
            risk_note: "Pure cache — regenerated automatically by the owning tool.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }
    let esc = base.replace(' ', "\\ ");
    let command = format!(
        "rm -rf {esc}/Cache {esc}/Code\\ Cache {esc}/GPUCache {esc}/Service\\ Worker/CacheStorage"
    );
    Def {
        id,
        name,
        tier: Tier::One,
        path: Some(format!("{base}/Cache")),
        reason,
        risk_note: "Pure cache — regenerated automatically by the owning tool.",
        caveat: Some(
            "Clears HTTP, code, GPU and service-worker caches; leaves your data and login intact.",
        ),
        requires_double_confirm: false,
        command: Some(command),
        status: Status::Available,
        subitems: Vec::new(),
    }
    .into_target()
}

/// Build container subitems for every dir under ~/Library/Caches matching a
/// predicate on the directory name. Each subitem clears its own contents.
pub(in crate::cleanup) fn caches_dir_subitems(pred: impl Fn(&str) -> bool) -> Vec<Item> {
    let caches = format!("{}/Library/Caches", home());
    let mut dirs: Vec<std::path::PathBuf> = std::fs::read_dir(&caches)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.is_dir())
                .filter(|p| {
                    p.file_name()
                        .map(|n| pred(&n.to_string_lossy()))
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    let mut subitems = Vec::new();
    for dir in &dirs {
        let dir_str = dir.to_string_lossy().to_string();
        let name = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| dir_str.clone());
        subitems.push(Item {
            id: name.clone(),
            label: name.clone(),
            path: dir_str.clone(),
            size_bytes: 0,
            size_human: String::new(),
            meta: None,
            requires_double_confirm: false,
            command: format!("rm -rf '{dir_str}'/*"),
        });
    }
    subitems
}
