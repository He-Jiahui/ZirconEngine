use super::*;

#[path = "evidence_anchors/budgets.rs"]
mod budgets;
#[path = "evidence_anchors/delegation.rs"]
mod delegation;
#[path = "evidence_anchors/root_child_rows.rs"]
mod root_child_rows;
#[path = "evidence_anchors/root_inventory.rs"]
mod root_inventory;
#[path = "evidence_anchors/root_owner_paths.rs"]
mod root_owner_paths;
#[path = "evidence_anchors/root_paths.rs"]
mod root_paths;
#[path = "evidence_anchors/root_statuses.rs"]
mod root_statuses;
#[path = "evidence_anchors/status_mirrors.rs"]
mod status_mirrors;
#[path = "evidence_anchors/variable_evidence.rs"]
mod variable_evidence;

pub(super) use root_child_rows::*;
pub(super) use root_owner_paths::*;
pub(super) use root_paths::*;
pub(super) use root_statuses::*;

pub(super) fn evidence_anchors_guard_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in EVIDENCE_ANCHORS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}
