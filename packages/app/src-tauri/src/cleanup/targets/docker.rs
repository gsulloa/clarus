use crate::cleanup::builders::Def;
use crate::cleanup::disk::path_exists;
use crate::cleanup::model::{Status, Target, Tier};
use crate::cleanup::shell::{has_tool, home};

pub(in crate::cleanup) fn docker_raw_path() -> String {
    format!(
        "{}/Library/Containers/com.docker.docker/Data/vms/0/data/Docker.raw",
        home()
    )
}

pub(in crate::cleanup) fn docker_installed() -> bool {
    path_exists(&docker_raw_path())
        || path_exists("~/Library/Containers/com.docker.docker")
        || has_tool("docker")
}

pub(in crate::cleanup) fn docker_prune_target() -> Target {
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

pub(in crate::cleanup) fn docker_raw_target() -> Target {
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
