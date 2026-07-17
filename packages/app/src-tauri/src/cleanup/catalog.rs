use super::builders::*;
use super::model::{Status, Target, Tier};
use super::shell::{has_tool, run_bash};
use super::targets::*;

/// Build the full catalog with detection done but sizes unmeasured.
pub(in crate::cleanup) fn catalog_defs() -> Vec<Target> {
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
        .unwrap_or_else(|_| crate::cleanup::shell::expand("~/.npm"));
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
            .unwrap_or_else(|_| crate::cleanup::shell::expand("~/Library/Caches/Homebrew"));
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

    targets.push(tier1(
        "docker-scout",
        "Docker Scout cache",
        "~/.docker/scout",
        "Docker Scout's local CVE and image-analysis database.",
        None,
        "rm -rf ~/.docker/scout".to_string(),
    ));

    targets.push(tier1(
        "uv-cache",
        "uv cache",
        "~/.cache/uv",
        "The uv Python package manager's download and build cache.",
        None,
        "uv cache clean 2>/dev/null || rm -rf ~/.cache/uv/*".to_string(),
    ));

    targets.push(tier1(
        "puppeteer-cache",
        "Puppeteer browsers",
        "~/.cache/puppeteer",
        "Chromium/Chrome builds downloaded by Puppeteer.",
        Some("Re-downloaded next time Puppeteer installs a browser."),
        "rm -rf ~/.cache/puppeteer/*".to_string(),
    ));

    targets.push(tier1(
        "node-gyp",
        "node-gyp cache",
        "~/Library/Caches/node-gyp",
        "Cached Node headers used to compile native addons.",
        None,
        "rm -rf ~/Library/Caches/node-gyp/*".to_string(),
    ));

    targets.push(tier1(
        "tableplus-cache",
        "TablePlus cache",
        "~/Library/Caches/com.tinyapp.TablePlus",
        "TablePlus's on-disk cache.",
        None,
        "rm -rf ~/Library/Caches/com.tinyapp.TablePlus/*".to_string(),
    ));

    targets.push(tier1(
        "user-logs",
        "User application logs",
        "~/Library/Logs",
        "Diagnostic logs written by user applications.",
        Some("Apps recreate logs as they run; only past logs are removed."),
        "rm -rf ~/Library/Logs/*".to_string(),
    ));

    targets.push(quicklook_cache_target());
    targets.push(system_temp_target());

    targets.push(tier1(
        "vscode-cache",
        "VS Code cache",
        "~/Library/Application Support/Code/Cache",
        "VS Code's HTTP and compiled-data caches.",
        Some("Also clears VS Code's CachedData."),
        "rm -rf ~/Library/Application\\ Support/Code/Cache ~/Library/Application\\ Support/Code/CachedData"
            .to_string(),
    ));

    targets.push(tier1(
        "cursor-cache",
        "Cursor cache",
        "~/Library/Application Support/Cursor/Cache",
        "Cursor's HTTP and compiled-data caches.",
        Some("Also clears Cursor's CachedData; leaves your settings intact."),
        "rm -rf ~/Library/Application\\ Support/Cursor/Cache ~/Library/Application\\ Support/Cursor/CachedData"
            .to_string(),
    ));

    targets.push(electron_cache_target(
        "discord-cache",
        "Discord cache",
        "Discord's HTTP/GPU/service-worker caches.",
        "~/Library/Application Support/discord",
    ));

    targets.push(electron_cache_target(
        "notion-cache",
        "Notion cache",
        "Notion's HTTP/GPU/service-worker caches (leaves your Notion data intact).",
        "~/Library/Application Support/Notion",
    ));

    targets.push(cache_or_missing(
        "teams-cache",
        "Microsoft Teams cache",
        "Microsoft Teams' sandbox cache.",
        Some("Clears the app's Caches directory; leaves your data and login intact."),
        "~/Library/Containers/com.microsoft.teams2/Data/Library/Caches",
        "rm -rf ~/Library/Containers/com.microsoft.teams2/Data/Library/Caches/*".to_string(),
    ));

    targets.push(electron_cache_target(
        "postman-cache",
        "Postman cache",
        "Postman's HTTP/GPU/service-worker caches.",
        "~/Library/Application Support/Postman",
    ));

    targets.push(cache_or_missing(
        "zoom-cache",
        "Zoom cache",
        "Zoom's on-disk cache.",
        None,
        "~/Library/Caches/us.zoom.xos",
        "rm -rf ~/Library/Caches/us.zoom.xos/*".to_string(),
    ));

    targets.push(shipit_updaters_target());
    targets.push(electron_updaters_target());

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
    targets.push(project_artifacts_target());
    targets.push(conductor_target());
    targets.push(android_images_target());

    targets.push(
        Def {
            id: "coresimulator-caches",
            name: "CoreSimulator caches",
            tier: Tier::Two,
            path: Some("~/Library/Developer/CoreSimulator/Caches".to_string()),
            reason: "Cached simulator runtime data and dyld caches.",
            risk_note: "Regenerable — recreated by Xcode/simulators as needed.",
            caveat: None,
            requires_double_confirm: false,
            command: Some("rm -rf ~/Library/Developer/CoreSimulator/Caches/*".to_string()),
            status: Status::Available,
            subitems: Vec::new(),
        }
        .into_target(),
    );

    targets.push(
        Def {
            id: "xcode-devicesupport",
            name: "Xcode device support",
            tier: Tier::Two,
            path: Some("~/Library/Developer/Xcode/iOS DeviceSupport".to_string()),
            reason: "Symbol data cached per connected iOS/watchOS/tvOS device+OS version.",
            risk_note: "Regenerable — Xcode recreates it the next time you attach a device.",
            caveat: Some("Also clears watchOS and tvOS device support."),
            requires_double_confirm: false,
            command: Some(
                "rm -rf ~/Library/Developer/Xcode/iOS\\ DeviceSupport/* ~/Library/Developer/Xcode/watchOS\\ DeviceSupport/* ~/Library/Developer/Xcode/tvOS\\ DeviceSupport/*"
                    .to_string(),
            ),
            status: Status::Available,
            subitems: Vec::new(),
        }
        .into_target(),
    );

    targets.push(
        Def {
            id: "trash",
            name: "Trash",
            tier: Tier::Two,
            path: Some("~/.Trash".to_string()),
            reason: "Files you have moved to the Trash but not yet emptied.",
            risk_note: "Permanently deletes everything currently in the Trash.",
            caveat: Some("Also empties trashes on mounted volumes."),
            requires_double_confirm: false,
            command: Some(
                "rm -rf ~/.Trash/* ~/.Trash/.[!.]* 2>/dev/null; rm -rf /Volumes/*/.Trashes/$(id -u)/* 2>/dev/null; true"
                    .to_string(),
            ),
            status: Status::Available,
            subitems: Vec::new(),
        }
        .into_target(),
    );

    targets.push(rustup_target());

    // ── TIER 3 — persistent personal data (deletable behind double-confirm) ──
    targets.push(tier3_simple(
        "postgres",
        "PostgreSQL databases",
        "~/Library/Application Support/Postgres/var-16/base",
    ));
    targets.push(tier3_simple(
        "spark",
        "Spark Desktop emails",
        "~/Library/Application Support/Spark Desktop/core-data",
    ));
    targets.push(tier3_simple(
        "whatsapp",
        "WhatsApp data",
        "~/Library/Group Containers/group.net.whatsapp.WhatsApp.shared",
    ));
    targets.push(tier3_simple("notion", "Notion", "~/Library/Application Support/Notion"));
    targets.push(tier3_simple("cursor", "Cursor editor", "~/Library/Application Support/Cursor"));

    targets.push(tier3_collection(
        "downloads",
        "Downloads",
        Some("~/Downloads".to_string()),
        "~/Downloads",
        |_| true,
    ));
    targets.push(tier3_collection(
        "claude-vm",
        "Claude VM bundles",
        Some("~/Library/Application Support/Claude/vm_bundles".to_string()),
        "~/Library/Application Support/Claude/vm_bundles",
        |_| true,
    ));
    targets.push(tier3_collection(
        "utm",
        "UTM Virtual Machines",
        Some("~/Library/Containers/com.utmapp.UTM/Data".to_string()),
        "~/Library/Containers/com.utmapp.UTM/Data/Documents",
        |name| name.ends_with(".utm"),
    ));
    targets.push(tier3_collection(
        "chrome-profiles",
        "Google Chrome profiles",
        Some("~/Library/Application Support/Google".to_string()),
        "~/Library/Application Support/Google/Chrome",
        |name| {
            name == "Default"
                || name == "Guest Profile"
                || name == "System Profile"
                || name.starts_with("Profile ")
        },
    ));

    targets
}
