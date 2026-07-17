use crate::cleanup::builders::{caches_dir_subitems, Def};
use crate::cleanup::disk::path_exists;
use crate::cleanup::model::{Status, Target, Tier};

/// All Squirrel/ShipIt updater caches EXCEPT the ToDesktop one (which has its
/// own `shipit` entry) — one deletable subitem per app.
pub(in crate::cleanup) fn shipit_updaters_target() -> Target {
    let excluded = "com.todesktop.230313mzl4w4u92.ShipIt";
    let subitems = caches_dir_subitems(|name| name.ends_with(".ShipIt") && name != excluded);
    Def {
        id: "shipit-updaters",
        name: "App updater caches (ShipIt)",
        tier: Tier::One,
        path: None,
        reason: "Leftover installer payloads from Squirrel/ShipIt app auto-updaters.",
        risk_note: "Pure cache — regenerated automatically by the owning tool.",
        caveat: Some("Excludes the ToDesktop ShipIt cache, which has its own entry."),
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

/// electron-updater download caches (`*updater*`, `@*updater*`) — one subitem each.
pub(in crate::cleanup) fn electron_updaters_target() -> Target {
    let subitems = caches_dir_subitems(|name| name.to_lowercase().contains("updater"));
    Def {
        id: "electron-updaters",
        name: "App updater downloads (electron-updater)",
        tier: Tier::One,
        path: None,
        reason: "Downloaded update payloads kept by electron-updater apps.",
        risk_note: "Pure cache — regenerated automatically by the owning tool.",
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

/// Cisco Webex old upgrade payloads — keep the newest version directory.
pub(in crate::cleanup) fn webex_upgrades_target() -> Target {
    let dir = "~/Library/Application Support/Cisco Spark/Webexteams_upgrades_arm";
    if !path_exists(dir) {
        return Def {
            id: "webex-upgrades",
            name: "Cisco Webex old upgrades",
            tier: Tier::One,
            path: None,
            reason: "Old Webex upgrade payloads; the newest version is kept.",
            risk_note: "Pure cache — regenerated automatically by the owning tool.",
            caveat: None,
            requires_double_confirm: false,
            command: None,
            status: Status::NotInstalled,
            subitems: Vec::new(),
        }
        .into_target();
    }

    // Keep the newest (highest-versioned) directory, delete the rest.
    // `sed '$d'` drops the last line after `sort -V` (BSD head lacks `-n -1`).
    let command = "ls -d ~/Library/Application\\ Support/Cisco\\ Spark/Webexteams_upgrades_arm/*/ 2>/dev/null | sort -V | sed '$d' | tr '\\n' '\\0' | xargs -0 rm -rf";

    Def {
        id: "webex-upgrades",
        name: "Cisco Webex old upgrades",
        tier: Tier::One,
        path: Some(dir.to_string()),
        reason: "Old Webex upgrade payloads; the newest version is kept.",
        risk_note: "Pure cache — regenerated automatically by the owning tool.",
        caveat: Some("The newest version directory is kept, so its size is included in the reported total but will not be freed."),
        requires_double_confirm: false,
        command: Some(command.to_string()),
        status: Status::Available,
        subitems: Vec::new(),
    }
    .into_target()
}
