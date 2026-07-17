// ─────────────────────────────────────────────────────────────────
// SIZE MEASUREMENT
// ─────────────────────────────────────────────────────────────────

use super::disk::{du_bytes, size_human};
use super::model::{Status, Target};

/// Fill sizes for a target and its subitems, then finalize status.
pub(in crate::cleanup) fn measure(target: &mut Target) {
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
