use crate::cleanup::builders::Def;
use crate::cleanup::model::{Item, Status, Target, Tier};
use crate::cleanup::shell::{run_bash, sq};

/// Number of days a top-level `T/` entry must be untouched before it is
/// considered stale and eligible for removal.
pub(in crate::cleanup) const TEMP_STALE_DAYS: u32 = 3;

/// System temporary files — stale entries under the per-user Darwin temp dir
/// plus orphaned Chrome code-sign clones. The path is resolved via `getconf`
/// so the per-user `/private/var/folders/<hash>` segment is never hardcoded.
pub(in crate::cleanup) fn system_temp_target() -> Target {
    let temp_dir = run_bash("getconf DARWIN_USER_TEMP_DIR 2>/dev/null")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .map(|s| s.trim_end_matches('/').to_string());

    let Some(temp_dir) = temp_dir else {
        return Def {
            id: "system-temp",
            name: "System temporary files",
            tier: Tier::One,
            path: None,
            reason: "Stale temporary files left under the per-user system temp directory.",
            risk_note: "Temporary — regenerated on demand.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    };

    // X/ is a sibling of T/ (both under the per-user folder).
    let base = std::path::Path::new(&temp_dir)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let chrome_clones = format!("{base}/X/com.google.Chrome.code_sign_clone");

    let mut subitems = Vec::new();

    // Stale temp files — age-gated so temp files in use are never touched.
    subitems.push(Item {
        id: "stale-temp".to_string(),
        label: format!("Stale temp files (untouched >{TEMP_STALE_DAYS} days)"),
        // Empty path: only the stale subset is removed, so a du of T/ would mislead.
        path: String::new(),
        size_bytes: 0,
        size_human: String::new(),
        meta: Some("cdk-nextjs-archive · DockerDesktop temp · …".to_string()),
        requires_double_confirm: false,
        command: format!(
            "find {} -mindepth 1 -maxdepth 1 -mtime +{} -exec rm -rf {{}} + 2>/dev/null; true",
            sq(&temp_dir),
            TEMP_STALE_DAYS
        ),
    });

    // Orphaned Chrome code-sign clones — measurable; quit Chrome first.
    if std::path::Path::new(&chrome_clones).exists() {
        subitems.push(Item {
            id: "chrome-code-sign-clones".to_string(),
            label: "Chrome code-sign clones".to_string(),
            path: chrome_clones.clone(),
            size_bytes: 0,
            size_human: String::new(),
            meta: Some("Quits Google Chrome first".to_string()),
            requires_double_confirm: false,
            command: format!(
                "osascript -e 'quit app \"Google Chrome\"' 2>/dev/null; sleep 1; rm -rf {}",
                sq(&chrome_clones)
            ),
        });
    }

    Def {
        id: "system-temp",
        name: "System temporary files",
        tier: Tier::One,
        path: None,
        reason: "Stale temporary files under the per-user system temp directory \
                 (build archives, updater payloads, orphaned Chrome clones).",
        risk_note: "Temporary — regenerated on demand.",
        caveat: Some(
            "Only entries untouched for over 3 days are removed, so files in use \
             by running apps are left alone.",
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

/// QuickLook thumbnail cache — lives in the per-user Darwin cache dir.
pub(in crate::cleanup) fn quicklook_cache_target() -> Target {
    let cache_dir = run_bash("getconf DARWIN_USER_CACHE_DIR 2>/dev/null")
        .map(|s| s.trim().to_string())
        .ok()
        .filter(|s| !s.is_empty())
        .map(|d| format!("{d}com.apple.QuickLook.thumbnailcache"));
    Def {
        id: "quicklook-cache",
        name: "QuickLook thumbnails",
        tier: Tier::One,
        path: cache_dir,
        reason: "Cached Finder/QuickLook thumbnail previews.",
        risk_note: "Pure cache — regenerated automatically by the owning tool.",
        caveat: Some("Reset via `qlmanage -r cache`; thumbnails rebuild on demand."),
        requires_double_confirm: false,
        command: Some(
            "qlmanage -r cache >/dev/null 2>&1; echo 'QuickLook thumbnail cache reset'".to_string(),
        ),
        status: Status::Available,
        subitems: Vec::new(),
    }
    .into_target()
}
