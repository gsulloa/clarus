// ─────────────────────────────────────────────────────────────────
// TAURI COMMANDS
// ─────────────────────────────────────────────────────────────────

use tauri::{AppHandle, Emitter};

use super::catalog::catalog_defs;
use super::disk::disk_usage;
use super::measure::measure;
use super::model::{CleanResult, CleanupScan};
use super::shell::run_bash;

#[tauri::command]
pub async fn scan_cleanup_targets(app: AppHandle) -> Result<CleanupScan, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<CleanupScan, String> {
        let usage = disk_usage();

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
            free_before_gb: usage.free_gb,
            free_before_human: usage.free_human,
            total_before_gb: usage.total_gb,
            total_before_human: usage.total_human,
            used_before_gb: usage.used_gb,
            used_before_human: usage.used_human,
            targets,
        })
    })
    .await
    .map_err(|e| format!("scan task failed: {e}"))?
}

fn clean_result(run: Result<String, String>, free_before: i64) -> CleanResult {
    let usage = disk_usage();
    let freed_gb = usage.free_gb - free_before;
    match run {
        Ok(msg) => CleanResult {
            ok: true,
            message: Some(msg),
            free_gb: usage.free_gb,
            free_human: usage.free_human,
            freed_gb,
            total_gb: usage.total_gb,
            total_human: usage.total_human,
            used_gb: usage.used_gb,
            used_human: usage.used_human,
        },
        Err(msg) => CleanResult {
            ok: false,
            message: Some(msg),
            free_gb: usage.free_gb,
            free_human: usage.free_human,
            freed_gb,
            total_gb: usage.total_gb,
            total_human: usage.total_human,
            used_gb: usage.used_gb,
            used_human: usage.used_human,
        },
    }
}

#[tauri::command]
pub async fn clean_target(id: String, confirmed: bool) -> Result<CleanResult, String> {
    tauri::async_runtime::spawn_blocking(move || -> Result<CleanResult, String> {
        let free_before = disk_usage().free_gb;
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
        let free_before = disk_usage().free_gb;
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
    let usage = disk_usage();
    Ok(CleanResult {
        ok: true,
        message: None,
        free_gb: usage.free_gb,
        free_human: usage.free_human,
        freed_gb: 0,
        total_gb: usage.total_gb,
        total_human: usage.total_human,
        used_gb: usage.used_gb,
        used_human: usage.used_human,
    })
}
