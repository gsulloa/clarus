// ─────────────────────────────────────────────────────────────────
// TESTS  (composition only — catalog_defs does detection, never deletes)
// ─────────────────────────────────────────────────────────────────

use crate::cleanup::catalog::catalog_defs;
use crate::cleanup::disk::parse_human_size;
use crate::cleanup::model::{Status, Target, Tier};

fn find<'a>(targets: &'a [Target], id: &str) -> &'a Target {
    targets
        .iter()
        .find(|t| t.id == id)
        .unwrap_or_else(|| panic!("target `{id}` missing from catalog"))
}

#[test]
fn new_tier1_targets_present() {
    let targets = catalog_defs();
    for id in [
        "pnpm-store",
        "gradle-caches",
        "gradle-wrapper",
        "gradle-daemon",
        "webex-upgrades",
        "slack-cache",
        "claude-desktop-cache",
        "aws-toolkit-cache",
        "cursor-vsix-cache",
    ] {
        assert_eq!(find(&targets, id).tier, Tier::One, "{id} should be Tier 1");
    }
}

#[test]
fn new_tier2_container_targets_present() {
    let targets = catalog_defs();
    for id in ["pyenv", "ollama"] {
        let t = find(&targets, id);
        assert_eq!(t.tier, Tier::Two, "{id} should be Tier 2");
        // Container targets act per-subitem, so the target itself has no command.
        assert!(t.command.is_none(), "{id} should have no top-level command");
    }
}

#[test]
fn webex_keeps_newest_via_sed() {
    let targets = catalog_defs();
    let webex = find(&targets, "webex-upgrades");
    if let Some(cmd) = &webex.command {
        // BSD head lacks `-n -1`; keep-newest is done with `sed '$d'`.
        assert!(cmd.contains("sort -V"), "webex command should sort by version");
        assert!(cmd.contains("sed '$d'"), "webex command should drop the newest with sed");
        assert!(!cmd.contains("head -n -1"), "webex must not use non-portable head");
    }
}

#[test]
fn notion_is_the_only_tier3_notion() {
    let targets = catalog_defs();
    let notion_count = targets.iter().filter(|t| t.id == "notion").count();
    assert_eq!(notion_count, 1, "notion should appear exactly once (no duplicate)");
    assert_eq!(find(&targets, "notion").tier, Tier::Three);
}

#[test]
fn parse_human_size_handles_common_units() {
    assert_eq!(parse_human_size("6.6", "GB"), (6.6 * 1024f64.powi(3)) as u64);
    assert_eq!(parse_human_size("512", "MB"), 512 * 1024 * 1024);
    assert_eq!(parse_human_size("1", "TB"), 1024u64.pow(4));
    // Garbled input degrades to 0 rather than panicking.
    assert_eq!(parse_human_size("abc", "GB"), 0);
    assert_eq!(parse_human_size("4.1", "??"), 0);
}

#[test]
fn new_simple_tier1_targets_present() {
    let targets = catalog_defs();
    for id in [
        "docker-scout",
        "uv-cache",
        "puppeteer-cache",
        "node-gyp",
        "tableplus-cache",
        "user-logs",
        "quicklook-cache",
        "vscode-cache",
        "cursor-cache",
        "discord-cache",
        "notion-cache",
        "teams-cache",
        "postman-cache",
        "zoom-cache",
    ] {
        assert_eq!(find(&targets, id).tier, Tier::One, "{id} should be Tier 1");
    }
}

#[test]
fn new_tier2_simple_targets_present() {
    let targets = catalog_defs();
    for id in ["coresimulator-caches", "xcode-devicesupport", "trash"] {
        assert_eq!(find(&targets, id).tier, Tier::Two, "{id} should be Tier 2");
    }
    // Trash must not require an in-app double confirmation.
    assert!(!find(&targets, "trash").requires_double_confirm);
}

#[test]
fn new_pattern_containers_have_no_top_level_command() {
    let targets = catalog_defs();
    for (id, tier) in [
        ("shipit-updaters", Tier::One),
        ("electron-updaters", Tier::One),
        ("rustup", Tier::Two),
    ] {
        let t = find(&targets, id);
        assert_eq!(t.tier, tier, "{id} tier");
        assert!(t.command.is_none(), "{id} should act per-subitem only");
    }
}

#[test]
fn shipit_updaters_excludes_todesktop() {
    let targets = catalog_defs();
    let t = find(&targets, "shipit-updaters");
    let excluded = "com.todesktop.230313mzl4w4u92.ShipIt";
    for item in &t.subitems {
        assert_ne!(item.id, excluded, "shipit-updaters must not include the todesktop cache");
        assert!(
            !item.path.contains(excluded),
            "shipit-updaters subitem path must not be the todesktop cache"
        );
    }
}

#[test]
fn downloads_is_tier3_deletable() {
    let targets = catalog_defs();
    let d = find(&targets, "downloads");
    assert_eq!(d.tier, Tier::Three);
    assert!(
        d.requires_double_confirm,
        "downloads must require double confirmation"
    );
    // `~/Downloads` always exists on a real machine, but tests must not
    // depend on the machine's actual contents — only assert the status is
    // one of the expected outcomes and that any subitems are confirmed.
    assert!(matches!(d.status, Status::Available | Status::Empty | Status::NotInstalled));
    for item in &d.subitems {
        assert!(item.requires_double_confirm, "downloads subitem must require double confirmation");
    }
}

#[test]
fn every_tier3_target_is_deletable_and_confirmed() {
    let targets = catalog_defs();
    for id in [
        "postgres",
        "spark",
        "claude-vm",
        "utm",
        "whatsapp",
        "notion",
        "cursor",
        "chrome-profiles",
        "downloads",
    ] {
        let t = find(&targets, id);
        assert_eq!(t.tier, Tier::Three, "{id} should be Tier 3");
        assert!(t.requires_double_confirm, "{id} must require double confirmation");
        for item in &t.subitems {
            assert!(
                item.requires_double_confirm,
                "{id} subitem `{}` must require double confirmation",
                item.id
            );
        }
    }
}
