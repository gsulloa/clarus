use std::process::Command;

use super::shell::expand;

/// Size in bytes via `du -sk` (KB blocks → bytes). 0 if the path is missing.
pub(in crate::cleanup) fn du_bytes(path: &str) -> u64 {
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
pub(in crate::cleanup) fn size_human(bytes: u64) -> String {
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

/// Disk usage of the data volume, captured from a single `df` snapshot.
///
/// `used` is intentionally derived as `total - free`, NOT read from `df`'s
/// per-volume `Used` column. On APFS, `/System/Volumes/Data` is one volume in a
/// shared container: the `Used` column reflects only that volume's attributed
/// usage while `Size` is the whole container and `Avail` is container-wide free
/// space net of reserved overhead, so `Used + Avail != Size`. Deriving
/// `used = total - free` guarantees the three figures reconcile
/// (`used + free = total`) for the readout — do NOT "restore" the column-2 read.
#[derive(Debug, Clone)]
pub(in crate::cleanup) struct DiskUsage {
    pub(in crate::cleanup) free_gb: i64,
    pub(in crate::cleanup) free_human: String,
    pub(in crate::cleanup) total_gb: i64,
    pub(in crate::cleanup) total_human: String,
    pub(in crate::cleanup) used_gb: i64,
    pub(in crate::cleanup) used_human: String,
}

/// Read used/free/total for the data volume from a single `df -g` snapshot.
///
/// All figures come from one invocation so they are mutually consistent, and the
/// human strings are formatted from the same GiB integers so the displayed
/// values always sum (`used + free = total`). See `DiskUsage` for why `used` is
/// derived rather than read from `df`'s `Used` column.
pub(in crate::cleanup) fn disk_usage() -> DiskUsage {
    // Single snapshot. `-g` reports whole GiB blocks; column order on macOS is
    // 0=Filesystem, 1=Size, 2=Used, 3=Avail, 4=Capacity.
    let text = Command::new("df")
        .arg("-g")
        .arg("/System/Volumes/Data")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();

    let field_gb = |index: usize| {
        parse_df_field_at(&text, index)
            .and_then(|f| f.parse::<i64>().ok())
            .unwrap_or(0)
    };

    let total_gb = field_gb(1);
    let free_gb = field_gb(3);
    let used_gb = (total_gb - free_gb).max(0);

    DiskUsage {
        free_gb,
        free_human: gib_human(free_gb),
        total_gb,
        total_human: gib_human(total_gb),
        used_gb,
        used_human: gib_human(used_gb),
    }
}

/// Format a whole-GiB integer the way `df -h` would (Gi, then Ti past 1024 GiB),
/// so used/free/total read as a consistent set derived from the same snapshot.
pub(in crate::cleanup) fn gib_human(gib: i64) -> String {
    if gib >= 1024 {
        format!("{:.1}Ti", gib as f64 / 1024.0)
    } else {
        format!("{gib}Gi")
    }
}

/// `df` output: second line, Nth whitespace field.
/// Column order on macOS: 0=Filesystem, 1=Size, 2=Used, 3=Avail, 4=Capacity.
pub(in crate::cleanup) fn parse_df_field_at(text: &str, index: usize) -> Option<String> {
    text.lines()
        .nth(1)
        .and_then(|line| line.split_whitespace().nth(index))
        .map(|s| s.to_string())
}

pub(in crate::cleanup) fn path_exists(path: &str) -> bool {
    std::path::Path::new(&expand(path)).exists()
}

/// Parse a human-readable size (e.g. `4.7 GB`) into bytes, base 1024.
pub(in crate::cleanup) fn parse_human_size(value: &str, unit: &str) -> u64 {
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
