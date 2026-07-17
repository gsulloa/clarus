use crate::cleanup::builders::Def;
use crate::cleanup::disk::path_exists;
use crate::cleanup::model::{Item, Status, Target, Tier};
use crate::cleanup::shell::{home, run_bash};

/// Returns true if `path` is a git repo or worktree (has a `.git` entry).
pub(in crate::cleanup) fn is_git_dir(path: &std::path::Path) -> bool {
    path.join(".git").exists()
}

/// Returns true if `path` is a project container: no own `.git`, but at least
/// one immediate subdir that does have `.git`.
pub(in crate::cleanup) fn is_project_container(path: &std::path::Path) -> bool {
    if is_git_dir(path) {
        return false;
    }
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .any(|e| e.path().is_dir() && is_git_dir(&e.path()))
        })
        .unwrap_or(false)
}

/// Enumerate individual workspace paths under `dir`.
/// Project containers (no own .git but children have .git) are expanded one
/// level; everything else is added directly.
pub(in crate::cleanup) fn enumerate_workspaces(dir: &str) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    let mut top: Vec<_> = entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.path())
        .collect();
    top.sort();
    for entry in top {
        if is_project_container(&entry) {
            let Ok(children) = std::fs::read_dir(&entry) else {
                continue;
            };
            let mut ws: Vec<_> = children
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.path())
                .collect();
            ws.sort();
            out.extend(ws);
        } else {
            out.push(entry);
        }
    }
    out
}

/// Build label and id for a workspace path rooted at `workspaces_dir`.
pub(in crate::cleanup) fn workspace_label_id(
    ws: &std::path::Path,
    workspaces_dir: &str,
) -> (String, String) {
    let name = ws
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| ws.to_string_lossy().to_string());
    let parent = ws
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let workspaces_leaf = std::path::Path::new(workspaces_dir)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspaces".to_string());
    let label = if parent == workspaces_leaf {
        name.clone()
    } else {
        format!("{parent}/{name}")
    };
    // Use __ separator to avoid collisions between project containers whose
    // workspaces share the same name (e.g. backend/kingston-v2 vs clarus/kingston-v2).
    let id = format!("{parent}__{name}");
    (label, id)
}

/// Conductor workspaces — each workspace individually, double-confirm if dirty.
pub(in crate::cleanup) fn conductor_target() -> Target {
    let dir = format!("{}/conductor/workspaces", home());
    if !path_exists(&dir) {
        return Def {
            id: "conductor",
            name: "Conductor workspaces",
            tier: Tier::Two,
            path: None,
            reason: "Per-project git worktrees created by Conductor.",
            risk_note: "Deleting a workspace removes its files permanently.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    let workspaces = enumerate_workspaces(&dir);
    let mut subitems = Vec::new();

    for ws in &workspaces {
        let ws_str = ws.to_string_lossy().to_string();
        let (label, id) = workspace_label_id(ws, &dir);
        let branch = run_bash(&format!("git -C '{ws_str}' branch --show-current 2>/dev/null"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let status_out =
            run_bash(&format!("git -C '{ws_str}' status --porcelain 2>/dev/null"))
                .unwrap_or_default();
        let dirty = !status_out.trim().is_empty();
        let meta = format!(
            "{} · {}",
            if branch.is_empty() { "no-git".to_string() } else { branch },
            if dirty { "uncommitted changes" } else { "clean" }
        );
        subitems.push(Item {
            id,
            label,
            path: ws_str.clone(),
            size_bytes: 0,
            size_human: String::new(),
            meta: Some(meta),
            requires_double_confirm: dirty,
            command: format!("rm -rf '{ws_str}'"),
        });
    }

    Def {
        id: "conductor",
        name: "Conductor workspaces",
        tier: Tier::Two,
        path: None,
        reason: "Per-project git worktrees created by Conductor.",
        risk_note: "Deleting a workspace removes its files permanently.",
        caveat: Some("Workspaces with uncommitted changes require a second confirmation."),
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

/// Regenerable artifact directories that live inside workspaces.
pub(in crate::cleanup) const ARTIFACT_DIRS: &[&str] = &[
    "node_modules",
    ".next",
    "dist",
    "cdk.out",
    ".turbo",
    "target",
    "__pycache__",
    ".venv",
    "venv",
    "build",
    ".cache",
    ".parcel-cache",
];

/// Shell command that deletes all artifact dirs inside `ws_str` without
/// descending into already-matched dirs.
pub(in crate::cleanup) fn artifact_clean_cmd(ws_str: &str) -> String {
    let name_expr: String = ARTIFACT_DIRS
        .iter()
        .enumerate()
        .map(|(i, d)| {
            if i == 0 {
                format!("-name '{d}'")
            } else {
                format!(" -o -name '{d}'")
            }
        })
        .collect();
    format!(
        "find '{ws_str}' -maxdepth 6 -type d \\( {name_expr} \\) -prune -exec rm -rf {{}} + 2>/dev/null; true"
    )
}

/// Conductor — regenerable artifacts inside each workspace (node_modules, .next, dist, …).
pub(in crate::cleanup) fn conductor_artifacts_target() -> Target {
    let dir = format!("{}/conductor/workspaces", home());
    if !path_exists(&dir) {
        return Def {
            id: "conductor-artifacts",
            name: "Conductor — regenerable artifacts",
            tier: Tier::One,
            path: None,
            reason: "Build outputs and dependency dirs inside Conductor workspaces.",
            risk_note: "Fully regenerable — reinstall/rebuild to recreate.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    let workspaces = enumerate_workspaces(&dir);
    let mut subitems = Vec::new();

    for ws in &workspaces {
        let ws_str = ws.to_string_lossy().to_string();
        let (label, name_id) = workspace_label_id(ws, &dir);
        subitems.push(Item {
            id: format!("artifacts__{name_id}"),
            label,
            // Empty path so the scanner doesn't re-du the whole workspace;
            // showing workspace total as "artifact size" would be misleading.
            path: String::new(),
            size_bytes: 0,
            size_human: String::new(),
            meta: Some("node_modules · .next · dist · target · cdk.out · …".to_string()),
            requires_double_confirm: false,
            command: artifact_clean_cmd(&ws_str),
        });
    }

    Def {
        id: "conductor-artifacts",
        name: "Conductor — regenerable artifacts",
        tier: Tier::One,
        path: None,
        reason: "Build outputs and dependency dirs inside Conductor workspaces \
                 (node_modules, .next, dist, target, cdk.out, …).",
        risk_note: "Fully regenerable — reinstall/rebuild to recreate.",
        caveat: Some("Freed space shown in the disk meter after cleaning."),
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

// ── Project build artifacts (repos under conventional dev roots) ──

/// Conventional directory names (relative to $HOME) where developers keep repos.
/// Probed for existence — we never assume a specific personal path is present.
pub(in crate::cleanup) const DEV_ROOT_CANDIDATES: &[&str] = &[
    "dev", "Developer", "Projects", "src", "code", "repos", "work", "git", "workspace",
];

/// Existing dev-root directories: {home}/{candidate} that actually exist.
pub(in crate::cleanup) fn existing_dev_roots() -> Vec<std::path::PathBuf> {
    let home = home();
    let mut roots = Vec::new();
    for name in DEV_ROOT_CANDIDATES {
        let p = std::path::PathBuf::from(format!("{home}/{name}"));
        if p.is_dir() {
            roots.push(p);
        }
    }
    roots
}

/// Recursively find git repositories (dirs containing a `.git` entry) under
/// `root`, up to `max_depth` levels deep. Descent stops at the first `.git`
/// hit (nested submodules are not listed separately). Symlinks are never
/// followed, and anything under ~/conductor/workspaces is skipped (already
/// covered by the Conductor artifact target).
pub(in crate::cleanup) fn enumerate_git_repos(
    root: &std::path::Path,
    max_depth: usize,
) -> Vec<std::path::PathBuf> {
    let conductor = format!("{}/conductor/workspaces", home());
    let mut out = Vec::new();
    collect_git_repos(root, max_depth, &conductor, &mut out);
    out.sort();
    out
}

pub(in crate::cleanup) fn collect_git_repos(
    dir: &std::path::Path,
    depth_left: usize,
    conductor_excl: &str,
    out: &mut Vec<std::path::PathBuf>,
) {
    // Skip the Conductor workspaces subtree — handled by conductor-artifacts.
    if dir.to_string_lossy().starts_with(conductor_excl) {
        return;
    }
    // A directory containing `.git` is a repo; record it and don't descend.
    if dir.join(".git").exists() {
        out.push(dir.to_path_buf());
        return;
    }
    if depth_left == 0 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut children: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        // Directories only, never following symlinks.
        .filter(|p| {
            std::fs::symlink_metadata(p)
                .map(|m| m.file_type().is_dir())
                .unwrap_or(false)
        })
        .collect();
    children.sort();
    for child in children {
        collect_git_repos(&child, depth_left - 1, conductor_excl, out);
    }
}

/// Project build artifacts — regenerable dirs (node_modules, .next, dist, …)
/// inside git repos discovered under conventional dev roots.
pub(in crate::cleanup) fn project_artifacts_target() -> Target {
    let roots = existing_dev_roots();
    if roots.is_empty() {
        return Def {
            id: "project-artifacts",
            name: "Project build artifacts",
            tier: Tier::One,
            path: None,
            reason: "Build outputs and dependency dirs inside your project repos.",
            risk_note: "Fully regenerable — reinstall/rebuild to recreate.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    // Discover repos under each root (bounded depth 3), de-duplicated.
    let mut repos: Vec<std::path::PathBuf> = Vec::new();
    for root in &roots {
        for repo in enumerate_git_repos(root, 3) {
            if !repos.contains(&repo) {
                repos.push(repo);
            }
        }
    }
    repos.sort();

    let mut subitems = Vec::new();
    for repo in &repos {
        let repo_str = repo.to_string_lossy().to_string();
        let name = repo
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| repo_str.clone());
        let parent = repo
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let label = if parent.is_empty() {
            name.clone()
        } else {
            format!("{parent}/{name}")
        };
        // Full-path-derived id guarantees uniqueness across roots.
        let id = format!(
            "project__{}",
            repo_str
                .trim_start_matches('/')
                .replace(|c: char| !c.is_alphanumeric(), "_")
        );
        subitems.push(Item {
            id,
            label,
            // Empty path: don't du the whole repo (would misrepresent artifact size).
            path: String::new(),
            size_bytes: 0,
            size_human: String::new(),
            meta: Some("node_modules · .next · dist · target · cdk.out · …".to_string()),
            requires_double_confirm: false,
            command: artifact_clean_cmd(&repo_str),
        });
    }

    Def {
        id: "project-artifacts",
        name: "Project build artifacts",
        tier: Tier::One,
        path: None,
        reason: "Build outputs and dependency dirs inside your project repos \
                 (node_modules, .next, dist, target, cdk.out, …).",
        risk_note: "Fully regenerable — reinstall/rebuild to recreate.",
        caveat: Some(
            "Found in git repos under ~/dev, ~/Projects, ~/src, …; Conductor \
             workspaces are handled separately.",
        ),
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
