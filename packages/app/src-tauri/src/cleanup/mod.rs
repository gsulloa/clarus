//! Disk cleanup catalog — a faithful port of `~/disk-cleanup.sh`.
//!
//! The catalog lists a fixed set of known macOS caches and regenerable data,
//! grouped into three tiers. Detection (paths / tool availability / sub-item
//! enumeration) is fast and deterministic; measuring size (`du`) is slow, so a
//! scan enumerates the catalog first, then fills sizes concurrently, emitting a
//! `cleanup://target` event per target as it finishes.
//!
//! Cleanup commands are copied verbatim from the script so the app does exactly
//! the same thing. Commands run through a login shell (`bash -lc`) so tools such
//! as `brew`, `yarn`, `nvm`, and `docker` resolve on PATH the way they do in the
//! user's terminal.

use std::process::Command;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

// ─────────────────────────────────────────────────────────────────
// TYPES  (serde camelCase — the frontend mirrors this exactly)
// ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Tier {
    One,
    Two,
    Three,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Available,
    Empty,
    ToolMissing,
    NotInstalled,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    id: String,
    label: String,
    path: String,
    size_bytes: u64,
    size_human: String,
    meta: Option<String>,
    requires_double_confirm: bool,
    command: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    id: String,
    name: String,
    tier: Tier,
    path: Option<String>,
    size_bytes: u64,
    size_human: String,
    status: Status,
    reason: String,
    risk_note: String,
    caveat: Option<String>,
    requires_double_confirm: bool,
    command: Option<String>,
    subitems: Vec<Item>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupScan {
    free_before_gb: i64,
    free_before_human: String,
    targets: Vec<Target>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    ok: bool,
    message: Option<String>,
    free_gb: i64,
    free_human: String,
    freed_gb: i64,
}

// ─────────────────────────────────────────────────────────────────
// HELPERS
// ─────────────────────────────────────────────────────────────────

fn home() -> String {
    std::env::var("HOME").unwrap_or_default()
}

/// Expand a leading `~` to $HOME.
fn expand(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        format!("{}/{}", home(), rest)
    } else if path == "~" {
        home()
    } else {
        path.to_string()
    }
}

/// Run a command through a login shell, capturing stdout+stderr.
fn run_bash(cmd: &str) -> Result<String, String> {
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
fn has_tool(tool: &str) -> bool {
    Command::new("bash")
        .arg("-lc")
        .arg(format!("command -v {tool} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Size in bytes via `du -sk` (KB blocks → bytes). 0 if the path is missing.
fn du_bytes(path: &str) -> u64 {
    let expanded = expand(path);
    let output = match Command::new("du").arg("-sk").arg(&expanded).output() {
        Ok(o) => o,
        Err(_) => return 0,
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .next()
        .and_then(|kb| kb.parse::<u64>().ok())
        .map(|kb| kb * 1024)
        .unwrap_or(0)
}

/// Mirror of `src/format.ts` formatBytes.
fn size_human(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut index = (bytes as f64).log(1024.0).floor() as usize;
    if index >= units.len() {
        index = units.len() - 1;
    }
    let value = bytes as f64 / 1024f64.powi(index as i32);
    let decimals = if value >= 10.0 || index == 0 { 0 } else { 1 };
    format!("{value:.decimals$} {}", units[index])
}

/// Free space on the data volume, GB, matching the script's `df_free`.
fn disk_free_gb() -> i64 {
    let output = match Command::new("df")
        .arg("-g")
        .arg("/System/Volumes/Data")
        .output()
    {
        Ok(o) => o,
        Err(_) => return 0,
    };
    parse_df_field(&String::from_utf8_lossy(&output.stdout))
        .and_then(|f| f.parse::<i64>().ok())
        .unwrap_or(0)
}

/// Free space human string, matching the script's `df_free_human`.
fn disk_free_human() -> String {
    let output = match Command::new("df")
        .arg("-h")
        .arg("/System/Volumes/Data")
        .output()
    {
        Ok(o) => o,
        Err(_) => return "?".to_string(),
    };
    parse_df_field(&String::from_utf8_lossy(&output.stdout)).unwrap_or_else(|| "?".to_string())
}

/// `df` output: second line, 4th field (Avail).
fn parse_df_field(text: &str) -> Option<String> {
    text.lines()
        .nth(1)
        .and_then(|line| line.split_whitespace().nth(3))
        .map(|s| s.to_string())
}

fn path_exists(path: &str) -> bool {
    std::path::Path::new(&expand(path)).exists()
}

// ─────────────────────────────────────────────────────────────────
// CATALOG DEFINITIONS  (fast — no `du`; reused by scan and clean)
// ─────────────────────────────────────────────────────────────────

/// A small builder to reduce boilerplate for the many simple targets.
struct Def {
    id: &'static str,
    name: &'static str,
    tier: Tier,
    path: Option<String>,
    reason: &'static str,
    risk_note: &'static str,
    caveat: Option<&'static str>,
    requires_double_confirm: bool,
    command: Option<String>,
    status: Status,
    subitems: Vec<Item>,
}

impl Def {
    fn into_target(self) -> Target {
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

fn tier1(
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

/// Build the full catalog with detection done but sizes unmeasured.
pub fn catalog_defs() -> Vec<Target> {
    let mut targets: Vec<Target> = Vec::new();

    // ── TIER 1 — pure caches ─────────────────────────────────────
    targets.push(tier1(
        "yarn",
        "Yarn cache",
        "~/Library/Caches/Yarn",
        "Yarn's package download cache.",
        None,
        "yarn cache clean 2>/dev/null || rm -rf ~/Library/Caches/Yarn/*".to_string(),
    ));

    // npm cache dir is resolved via `npm config get cache`.
    let npm_dir = run_bash("npm config get cache 2>/dev/null || echo ~/.npm")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| expand("~/.npm"));
    targets.push(tier1(
        "npm",
        "npm cache",
        &npm_dir,
        "npm's package download cache.",
        None,
        "npm cache clean --force".to_string(),
    ));

    targets.push(tier1(
        "pip",
        "pip cache",
        "~/Library/Caches/pip",
        "pip's wheel/download cache.",
        None,
        "rm -rf ~/Library/Caches/pip/*".to_string(),
    ));

    // Homebrew — only if brew is installed.
    if has_tool("brew") {
        let brew_cache = run_bash("brew --cache 2>/dev/null")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| expand("~/Library/Caches/Homebrew"));
        targets.push(tier1(
            "homebrew",
            "Homebrew cache",
            &brew_cache,
            "Downloaded bottles and old versions kept by Homebrew.",
            None,
            "brew cleanup -s 2>/dev/null; rm -rf \"$(brew --cache)\"".to_string(),
        ));
    } else {
        targets.push(
            Def {
                id: "homebrew",
                name: "Homebrew cache",
                tier: Tier::One,
                path: None,
                reason: "Downloaded bottles and old versions kept by Homebrew.",
                risk_note: "Pure cache — regenerated automatically by the owning tool.",
                caveat: None,
                requires_double_confirm: false,
                command: None,
                status: Status::NotInstalled,
                subitems: Vec::new(),
            }
            .into_target(),
        );
    }

    targets.push(tier1(
        "shipit",
        "ShipIt installer cache",
        "~/Library/Caches/com.todesktop.230313mzl4w4u92.ShipIt",
        "Leftover installer payloads from a ToDesktop app updater.",
        None,
        "rm -rf ~/Library/Caches/com.todesktop.230313mzl4w4u92.ShipIt/*".to_string(),
    ));

    targets.push(tier1(
        "playwright",
        "Playwright cache",
        "~/Library/Caches/ms-playwright",
        "Downloaded Playwright browser binaries.",
        Some("Re-downloaded next time you run `playwright install`."),
        "rm -rf ~/Library/Caches/ms-playwright/*".to_string(),
    ));

    targets.push(tier1(
        "spotify",
        "Spotify cache",
        "~/Library/Caches/com.spotify.client",
        "Spotify's local media cache.",
        Some("Quits Spotify if it is running."),
        "osascript -e 'quit app \"Spotify\"' 2>/dev/null; sleep 1; rm -rf ~/Library/Caches/com.spotify.client/*"
            .to_string(),
    ));

    targets.push(tier1(
        "chrome",
        "Google Chrome cache",
        "~/Library/Caches/Google/Chrome",
        "Chrome's on-disk web cache.",
        Some("Quits Google Chrome if it is running."),
        "osascript -e 'quit app \"Google Chrome\"' 2>/dev/null; sleep 1; rm -rf ~/Library/Caches/Google/Chrome/*"
            .to_string(),
    ));

    targets.push(tier1(
        "bun",
        "Bun cache",
        "~/.bun/install/cache",
        "Bun's package install cache.",
        None,
        "bun pm cache rm 2>/dev/null || rm -rf ~/.bun/install/cache/*".to_string(),
    ));

    targets.push(pnpm_store_target());

    targets.push(tier1(
        "gradle-caches",
        "Gradle caches",
        "~/.gradle/caches",
        "Gradle's downloaded dependencies and build caches.",
        Some("Stops the Gradle daemon first to release locks."),
        "gradle --stop 2>/dev/null; rm -rf ~/.gradle/caches".to_string(),
    ));

    targets.push(tier1(
        "gradle-wrapper",
        "Gradle wrapper dists",
        "~/.gradle/wrapper/dists",
        "Gradle wrapper distributions downloaded per project.",
        None,
        "rm -rf ~/.gradle/wrapper/dists".to_string(),
    ));

    targets.push(tier1(
        "gradle-daemon",
        "Gradle daemon logs",
        "~/.gradle/daemon",
        "Gradle daemon logs and registry files.",
        None,
        "rm -rf ~/.gradle/daemon".to_string(),
    ));

    targets.push(webex_upgrades_target());

    targets.push(tier1(
        "slack-cache",
        "Slack cache",
        "~/Library/Application Support/Slack/Cache",
        "Slack's on-disk web and service-worker caches.",
        Some("Also clears Slack's Service Worker cache."),
        "rm -rf ~/Library/Application\\ Support/Slack/Cache ~/Library/Application\\ Support/Slack/Service\\ Worker"
            .to_string(),
    ));

    targets.push(tier1(
        "claude-desktop-cache",
        "Claude Desktop cache",
        "~/Library/Application Support/Claude/Cache",
        "Claude Desktop's web and code caches.",
        Some("Also clears Claude Desktop's Code Cache."),
        "rm -rf ~/Library/Application\\ Support/Claude/Cache ~/Library/Application\\ Support/Claude/Code\\ Cache"
            .to_string(),
    ));

    targets.push(tier1(
        "aws-toolkit-cache",
        "AWS Toolkit cache",
        "~/Library/Caches/aws",
        "Cached data from the AWS Toolkit / AWS CLI.",
        None,
        "rm -rf ~/Library/Caches/aws".to_string(),
    ));

    targets.push(tier1(
        "cursor-vsix-cache",
        "Cursor cached VSIXs",
        "~/Library/Application Support/Cursor/CachedExtensionVSIXs",
        "Cached extension installer packages kept by Cursor.",
        None,
        "rm -rf ~/Library/Application\\ Support/Cursor/CachedExtensionVSIXs".to_string(),
    ));

    // ── TIER 2 — regenerables ────────────────────────────────────
    targets.push(docker_prune_target());
    targets.push(docker_raw_target());
    targets.push(ios_unavailable_target());
    targets.push(ios_runtimes_target());
    targets.push(nvm_target());
    targets.push(pyenv_target());
    targets.push(ollama_target());

    targets.push(
        Def {
            id: "xcode-archives",
            name: "Xcode Archives",
            tier: Tier::Two,
            path: Some("~/Library/Developer/Xcode/Archives".to_string()),
            reason: "Archived app builds from Xcode.",
            risk_note: "Regenerable — rebuild/re-archive from Xcode if needed.",
            caveat: None,
            requires_double_confirm: false,
            command: Some("rm -rf ~/Library/Developer/Xcode/Archives/*".to_string()),
            status: Status::Available,
            subitems: Vec::new(),
        }
        .into_target(),
    );

    targets.push(
        Def {
            id: "xcode-deriveddata",
            name: "Xcode DerivedData",
            tier: Tier::Two,
            path: Some("~/Library/Developer/Xcode/DerivedData".to_string()),
            reason: "Xcode build intermediates and indexes.",
            risk_note: "Regenerable — Xcode rebuilds this on next build.",
            caveat: None,
            requires_double_confirm: false,
            command: Some("rm -rf ~/Library/Developer/Xcode/DerivedData/*".to_string()),
            status: Status::Available,
            subitems: Vec::new(),
        }
        .into_target(),
    );

    // Cargo — smart clean if cargo-cache is present, else manual registry cache.
    let cargo_cmd = if has_tool("cargo-cache") {
        "cargo cache --autoclean".to_string()
    } else {
        "rm -rf ~/.cargo/registry/cache/*".to_string()
    };
    targets.push(
        Def {
            id: "cargo",
            name: "Cargo cache",
            tier: Tier::Two,
            path: Some("~/.cargo/registry".to_string()),
            reason: "Cargo's registry sources and download cache.",
            risk_note: "Regenerable — Cargo re-fetches crates on next build.",
            caveat: if has_tool("cargo-cache") {
                None
            } else {
                Some("cargo-cache not installed; cleans ~/.cargo/registry/cache manually.")
            },
            requires_double_confirm: false,
            command: Some(cargo_cmd),
            status: Status::Available,
            subitems: Vec::new(),
        }
        .into_target(),
    );

    targets.push(conductor_artifacts_target());
    targets.push(conductor_target());
    targets.push(android_images_target());

    // ── TIER 3 — persistent data (informational only) ────────────
    for (id, name, path) in [
        (
            "postgres",
            "PostgreSQL databases",
            "~/Library/Application Support/Postgres/var-16/base",
        ),
        (
            "spark",
            "Spark Desktop emails",
            "~/Library/Application Support/Spark Desktop/core-data",
        ),
        (
            "claude-vm",
            "Claude VM bundles",
            "~/Library/Application Support/Claude/vm_bundles",
        ),
        (
            "utm",
            "UTM Virtual Machines",
            "~/Library/Containers/com.utmapp.UTM/Data",
        ),
        (
            "whatsapp",
            "WhatsApp data",
            "~/Library/Group Containers/group.net.whatsapp.WhatsApp.shared",
        ),
        ("notion", "Notion", "~/Library/Application Support/Notion"),
        ("cursor", "Cursor editor", "~/Library/Application Support/Cursor"),
        (
            "chrome-profiles",
            "Google Chrome profiles",
            "~/Library/Application Support/Google",
        ),
    ] {
        targets.push(
            Def {
                id: Box::leak(id.to_string().into_boxed_str()),
                name: Box::leak(name.to_string().into_boxed_str()),
                tier: Tier::Three,
                path: Some(path.to_string()),
                reason: "Persistent user data — shown for awareness only.",
                risk_note: "Requires a manual decision. Clarus never deletes Tier 3 data.",
                caveat: None,
                requires_double_confirm: false,
                command: None,
                status: Status::Available,
                subitems: Vec::new(),
            }
            .into_target(),
        );
    }

    targets
}

// ── Container / special target builders ──────────────────────────

fn docker_raw_path() -> String {
    format!(
        "{}/Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw",
        home()
    )
}

fn docker_installed() -> bool {
    path_exists(&docker_raw_path())
        || path_exists("~/Library/Containers/com.docker.docker")
        || has_tool("docker")
}

fn docker_prune_target() -> Target {
    let installed = docker_installed();
    // Auto-start Docker (≤90s), then run the full prune sequence.
    let command = "if ! docker info >/dev/null 2>&1; then \
open -a Docker 2>/dev/null; t=0; \
while ! docker info >/dev/null 2>&1; do sleep 3; t=$((t+3)); \
if [ \"$t\" -ge 90 ]; then echo 'Docker did not start within 90s'; exit 1; fi; done; \
fi; \
docker builder prune -af 2>/dev/null; \
docker image prune -af 2>/dev/null; \
docker container prune -f 2>/dev/null; \
docker volume prune -af 2>/dev/null; \
docker system prune -af --volumes 2>/dev/null; \
echo 'Docker prune completed'"
        .to_string();

    Def {
        id: "docker-prune",
        name: "Docker prune",
        tier: Tier::Two,
        path: Some(docker_raw_path()),
        reason: "Dangling images, stopped containers, unused volumes and build cache.",
        risk_note: "Removes unused Docker resources; running containers are untouched.",
        caveat: Some("Starts Docker if it is not running (waits up to 90s)."),
        requires_double_confirm: false,
        command: if installed { Some(command) } else { None },
        status: if installed {
            Status::Available
        } else {
            Status::NotInstalled
        },
        subitems: Vec::new(),
    }
    .into_target()
}

fn docker_raw_target() -> Target {
    let installed = docker_installed();
    let raw = docker_raw_path();
    let command = format!(
        "osascript -e 'quit app \"Docker\"' 2>/dev/null; sleep 3; rm -f '{raw}'; open -a Docker 2>/dev/null"
    );
    Def {
        id: "docker-raw",
        name: "Docker.raw regeneration",
        tier: Tier::Two,
        path: Some(raw),
        reason: "The Docker VM disk image. Regenerating reclaims physical space it no longer uses.",
        risk_note: "Destroys ALL remaining Docker images and volumes.",
        caveat: Some("Quits Docker, deletes Docker.raw, then reopens Docker."),
        requires_double_confirm: true,
        command: if installed { Some(command) } else { None },
        status: if installed {
            Status::Available
        } else {
            Status::NotInstalled
        },
        subitems: Vec::new(),
    }
    .into_target()
}

fn ios_unavailable_target() -> Target {
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
fn ios_runtimes_target() -> Target {
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

/// Old nvm Node versions — only when >3 installed, keeping current + latest LTS.
fn nvm_target() -> Target {
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
fn pnpm_store_target() -> Target {
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

/// Cisco Webex old upgrade payloads — keep the newest version directory.
fn webex_upgrades_target() -> Target {
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

/// Old pyenv Python versions — keep the active version, offer the rest.
fn pyenv_target() -> Target {
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

/// Parse a human-readable size (e.g. `4.7 GB`) into bytes, base 1024.
fn parse_human_size(value: &str, unit: &str) -> u64 {
    let Ok(num) = value.parse::<f64>() else {
        return 0;
    };
    let multiplier = match unit.to_ascii_uppercase().as_str() {
        "B" => 1.0,
        "KB" => 1024.0,
        "MB" => 1024f64.powi(2),
        "GB" => 1024f64.powi(3),
        "TB" => 1024f64.powi(4),
        _ => return 0,
    };
    (num * multiplier) as u64
}

/// Ollama models — one deletable subitem per model from `ollama list`.
fn ollama_target() -> Target {
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

/// Returns true if `path` is a git repo or worktree (has a `.git` entry).
fn is_git_dir(path: &std::path::Path) -> bool {
    path.join(".git").exists()
}

/// Returns true if `path` is a project container: no own `.git`, but at least
/// one immediate subdir that does have `.git`.
fn is_project_container(path: &std::path::Path) -> bool {
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
fn enumerate_workspaces(dir: &str) -> Vec<std::path::PathBuf> {
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
fn workspace_label_id(ws: &std::path::Path, workspaces_dir: &str) -> (String, String) {
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
fn conductor_target() -> Target {
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
const ARTIFACT_DIRS: &[&str] = &[
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
fn artifact_clean_cmd(ws_str: &str) -> String {
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
fn conductor_artifacts_target() -> Target {
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

/// Android SDK system-images — each image individually.
fn android_images_target() -> Target {
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

// ─────────────────────────────────────────────────────────────────
// SIZE MEASUREMENT
// ─────────────────────────────────────────────────────────────────

/// Fill sizes for a target and its subitems, then finalize status.
fn measure(target: &mut Target) {
    for item in target.subitems.iter_mut() {
        if !item.path.is_empty() {
            item.size_bytes = du_bytes(&item.path);
        }
        item.size_human = size_human(item.size_bytes);
    }

    if !target.subitems.is_empty() {
        target.size_bytes = target.subitems.iter().map(|i| i.size_bytes).sum();
    } else if let Some(path) = &target.path {
        target.size_bytes = du_bytes(path);
    }
    target.size_human = size_human(target.size_bytes);

    // A pure-cache/simple target with a command but nothing on disk is "empty".
    if matches!(target.status, Status::Available)
        && target.subitems.is_empty()
        && target.command.is_some()
        && target.size_bytes == 0
    {
        target.status = Status::Empty;
    }
}

// ─────────────────────────────────────────────────────────────────
// TAURI COMMANDS
// ─────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn scan_cleanup_targets(app: AppHandle) -> Result<CleanupScan, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<CleanupScan, String> {
        let free_before_gb = disk_free_gb();
        let free_before_human = disk_free_human();

        let mut targets = catalog_defs();

        // Enumeration phase: emit the full unmeasured catalog (with total) up
        // front so the UI can render skeleton rows and a determinate bar before
        // any slow `du` runs.
        let _ = app.emit("cleanup://catalog", &targets);

        // Measuring phase: fill sizes concurrently; emit an event per target as
        // it completes.
        std::thread::scope(|scope| {
            let app_ref = &app;
            for target in targets.iter_mut() {
                scope.spawn(move || {
                    measure(target);
                    let _ = app_ref.emit("cleanup://target", &*target);
                });
            }
        });

        Ok(CleanupScan {
            free_before_gb,
            free_before_human,
            targets,
        })
    })
    .await
    .map_err(|e| format!("scan task failed: {e}"))?
}

fn clean_result(run: Result<String, String>, free_before: i64) -> CleanResult {
    let free_gb = disk_free_gb();
    let free_human = disk_free_human();
    match run {
        Ok(msg) => CleanResult {
            ok: true,
            message: Some(msg),
            free_gb,
            free_human,
            freed_gb: free_gb - free_before,
        },
        Err(msg) => CleanResult {
            ok: false,
            message: Some(msg),
            free_gb,
            free_human,
            freed_gb: free_gb - free_before,
        },
    }
}

#[tauri::command]
pub async fn clean_target(id: String, confirmed: bool) -> Result<CleanResult, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<CleanResult, String> {
        let free_before = disk_free_gb();
        let catalog = catalog_defs();
        let target = catalog
            .iter()
            .find(|t| t.id == id)
            .ok_or_else(|| format!("Unknown target: {id}"))?;

        if target.requires_double_confirm && !confirmed {
            return Err("This target requires explicit confirmation.".to_string());
        }
        let command = target
            .command
            .as_ref()
            .ok_or_else(|| "This target has no cleanup action.".to_string())?;

        Ok(clean_result(run_bash(command), free_before))
    })
    .await
    .map_err(|e| format!("cleanup task failed: {e}"))?
}

#[tauri::command]
pub async fn clean_item(
    target_id: String,
    item_id: String,
    confirmed: bool,
) -> Result<CleanResult, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<CleanResult, String> {
        let free_before = disk_free_gb();
        let catalog = catalog_defs();
        let target = catalog
            .iter()
            .find(|t| t.id == target_id)
            .ok_or_else(|| format!("Unknown target: {target_id}"))?;
        let item = target
            .subitems
            .iter()
            .find(|i| i.id == item_id)
            .ok_or_else(|| format!("Unknown item: {item_id}"))?;

        if item.requires_double_confirm && !confirmed {
            return Err("This item requires explicit confirmation.".to_string());
        }

        Ok(clean_result(run_bash(&item.command), free_before))
    })
    .await
    .map_err(|e| format!("cleanup task failed: {e}"))?
}

#[tauri::command]
pub fn disk_free() -> Result<CleanResult, String> {
    let free_gb = disk_free_gb();
    Ok(CleanResult {
        ok: true,
        message: None,
        free_gb,
        free_human: disk_free_human(),
        freed_gb: 0,
    })
}

// ─────────────────────────────────────────────────────────────────
// TESTS  (composition only — catalog_defs does detection, never deletes)
// ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

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
}
