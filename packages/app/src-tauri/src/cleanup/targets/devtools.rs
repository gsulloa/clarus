use crate::cleanup::builders::Def;
use crate::cleanup::disk::path_exists;
use crate::cleanup::model::{Item, Status, Target, Tier};
use crate::cleanup::shell::{has_tool, home, run_bash};

pub(in crate::cleanup) fn ios_unavailable_target() -> Target {
    let has_xcrun = has_tool("xcrun");
    Def {
        id: "ios-sim-unavailable",
        name: "iOS simulators (unavailable)",
        tier: Tier::Two,
        path: Some("~/Library/Developer/CoreSimulator/Devices".to_string()),
        reason: "Simulator devices marked unavailable (e.g. from removed runtimes).",
        risk_note: "Regenerable — recreate simulators from Xcode.",
        caveat: None,
        requires_double_confirm: false,
        command: if has_xcrun {
            Some("xcrun simctl delete unavailable".to_string())
        } else {
            None
        },
        status: if has_xcrun {
            Status::Available
        } else {
            Status::NotInstalled
        },
        subitems: Vec::new(),
    }
    .into_target()
}

/// Old iOS runtimes — keep the newest per platform (matches the script's jq).
pub(in crate::cleanup) fn ios_runtimes_target() -> Target {
    let (status, subitems) = if !has_tool("xcrun") {
        (Status::NotInstalled, Vec::new())
    } else if !has_tool("jq") {
        (Status::ToolMissing, Vec::new())
    } else {
        let pipeline = ".runtimes | group_by(.name | split(\" \") | .[0:2] | join(\" \")) | .[] | sort_by(.version) | reverse | .[1:][] | select(.isAvailable == true) | .identifier";
        let ids = run_bash(&format!("xcrun simctl list runtimes -j | jq -r '{pipeline}'"))
            .unwrap_or_default();
        let mut subitems = Vec::new();
        for id in ids.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let name = run_bash(&format!(
                "xcrun simctl list runtimes -j | jq -r '.runtimes[] | select(.identifier == \"{id}\") | .name'"
            ))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| id.to_string());
            subitems.push(Item {
                id: id.to_string(),
                label: if name.is_empty() { id.to_string() } else { name },
                path: String::new(),
                size_bytes: 0,
                size_human: String::new(),
                meta: Some(id.to_string()),
                requires_double_confirm: false,
                command: format!("xcrun simctl runtime delete '{id}'"),
            });
        }
        (
            if subitems.is_empty() {
                Status::Empty
            } else {
                Status::Available
            },
            subitems,
        )
    };

    Def {
        id: "ios-runtimes",
        name: "Old iOS runtimes",
        tier: Tier::Two,
        path: None,
        reason: "Older simulator runtimes; the newest per platform is kept.",
        risk_note: "Regenerable — re-download runtimes from Xcode if needed.",
        caveat: if matches!(status, Status::ToolMissing) {
            Some("jq is not installed; runtime pruning is unavailable.")
        } else {
            None
        },
        requires_double_confirm: false,
        command: None,
        status,
        subitems,
    }
    .into_target()
}

/// Android SDK system-images — each image individually.
pub(in crate::cleanup) fn android_images_target() -> Target {
    let base = format!("{}/Library/Android/sdk/system-images", home());
    if !path_exists(&base) {
        return Def {
            id: "android-images",
            name: "Android SDK system-images",
            tier: Tier::Two,
            path: None,
            reason: "Emulator system images downloaded by the Android SDK.",
            risk_note: "Regenerable — re-download from the SDK manager.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    // Enumerate api-level/*/* two levels deep (matches `find -maxdepth 2 -mindepth 2`).
    let mut subitems = Vec::new();
    if let Ok(level1) = std::fs::read_dir(&base) {
        let mut l1: Vec<_> = level1.flatten().filter(|e| e.path().is_dir()).map(|e| e.path()).collect();
        l1.sort();
        for api in l1 {
            if let Ok(level2) = std::fs::read_dir(&api) {
                let mut l2: Vec<_> = level2.flatten().filter(|e| e.path().is_dir()).map(|e| e.path()).collect();
                l2.sort();
                for img in l2 {
                    let img_str = img.to_string_lossy().to_string();
                    let label = img_str.strip_prefix(&format!("{base}/")).unwrap_or(&img_str).to_string();
                    subitems.push(Item {
                        id: label.replace('/', "_"),
                        label,
                        path: img_str.clone(),
                        size_bytes: 0,
                        size_human: String::new(),
                        meta: None,
                        requires_double_confirm: false,
                        command: format!("rm -rf '{img_str}'"),
                    });
                }
            }
        }
    }

    Def {
        id: "android-images",
        name: "Android SDK system-images",
        tier: Tier::Two,
        path: None,
        reason: "Emulator system images downloaded by the Android SDK.",
        risk_note: "Regenerable — re-download from the SDK manager.",
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
