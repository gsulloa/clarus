// ─────────────────────────────────────────────────────────────────
// TYPES  (serde camelCase — the frontend mirrors this exactly)
// ─────────────────────────────────────────────────────────────────

use serde::Serialize;

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
    pub(in crate::cleanup) id: String,
    pub(in crate::cleanup) label: String,
    pub(in crate::cleanup) path: String,
    pub(in crate::cleanup) size_bytes: u64,
    pub(in crate::cleanup) size_human: String,
    pub(in crate::cleanup) meta: Option<String>,
    pub(in crate::cleanup) requires_double_confirm: bool,
    pub(in crate::cleanup) command: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub(in crate::cleanup) id: String,
    pub(in crate::cleanup) name: String,
    pub(in crate::cleanup) tier: Tier,
    pub(in crate::cleanup) path: Option<String>,
    pub(in crate::cleanup) size_bytes: u64,
    pub(in crate::cleanup) size_human: String,
    pub(in crate::cleanup) status: Status,
    pub(in crate::cleanup) reason: String,
    pub(in crate::cleanup) risk_note: String,
    pub(in crate::cleanup) caveat: Option<String>,
    pub(in crate::cleanup) requires_double_confirm: bool,
    pub(in crate::cleanup) command: Option<String>,
    pub(in crate::cleanup) subitems: Vec<Item>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupScan {
    pub(in crate::cleanup) free_before_gb: i64,
    pub(in crate::cleanup) free_before_human: String,
    pub(in crate::cleanup) total_before_gb: i64,
    pub(in crate::cleanup) total_before_human: String,
    pub(in crate::cleanup) used_before_gb: i64,
    pub(in crate::cleanup) used_before_human: String,
    pub(in crate::cleanup) targets: Vec<Target>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanResult {
    pub(in crate::cleanup) ok: bool,
    pub(in crate::cleanup) message: Option<String>,
    pub(in crate::cleanup) free_gb: i64,
    pub(in crate::cleanup) free_human: String,
    pub(in crate::cleanup) freed_gb: i64,
    pub(in crate::cleanup) total_gb: i64,
    pub(in crate::cleanup) total_human: String,
    pub(in crate::cleanup) used_gb: i64,
    pub(in crate::cleanup) used_human: String,
}
