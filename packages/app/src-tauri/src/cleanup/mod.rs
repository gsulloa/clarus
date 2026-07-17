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

mod builders;
mod catalog;
mod commands;
mod disk;
mod measure;
mod model;
mod shell;
mod targets;

#[cfg(test)]
mod tests;

// A glob re-export (rather than naming the four command fns) is required:
// `#[tauri::command]` also emits hidden `__cmd__*` / `__tauri_command_name_*`
// helper items alongside each function, and `tauri::generate_handler!` in
// lib.rs resolves those at `cleanup::__cmd__<fn>` — they must be re-exported
// too, or `cleanup::scan_cleanup_targets` etc. fail to resolve from lib.rs.
pub use commands::*;
