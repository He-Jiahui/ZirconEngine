type Slice = super::ExpectedStatusOutputSlice;

#[path = "runtime_index_anchors/cargo_attempt.rs"]
mod cargo_attempt;
#[path = "runtime_index_anchors/index_baseline.rs"]
mod index_baseline;
#[path = "runtime_index_anchors/plan_status_children.rs"]
mod plan_status_children;
#[path = "runtime_index_anchors/runtime_status_anchors.rs"]
mod runtime_status_anchors;
#[path = "runtime_index_anchors/support_inventory.rs"]
mod support_inventory;

pub(super) const EXPECTED_STATUS_OUTPUT_SLICES: &[Slice] = &[
    index_baseline::SUBPLAN_MAP_SYNC,
    index_baseline::PROBLEM_ROW_PARSER_SYNC,
    runtime_status_anchors::RUNTIME_03_MODULE_DOC_STATUS_INDEX_ANCHOR_SYNC,
    runtime_status_anchors::RUNTIME_07_SCENE_ASSET_STATUS_ANCHOR_SYNC,
    runtime_status_anchors::RUNTIME_07_OWNER_BUDGET_STATUS_ANCHOR_SYNC,
    runtime_status_anchors::RUNTIME_02_GENERATED_STATUS_ANCHOR_SYNC,
    runtime_status_anchors::RUNTIME_10_BEHAVIOR_STATUS_ANCHOR_SYNC,
    cargo_attempt::RUNTIME_CARGO_ATTEMPT_STATUS_ANCHOR_SYNC,
    plan_status_children::INDEX_TABLES_CHILD_OWNER_SPLIT,
    plan_status_children::INDEX_TABLES_PARENT_GUARD_FOLDER_BACKED_SPLIT,
    plan_status_children::INDEX_TABLES_SPLIT_LAYOUT_FOLDER_BACKED_SPLIT,
    plan_status_children::INDEX_STATUS_ANCHORS_FOLDER_BACKED_SPLIT,
    plan_status_children::RECENT_STATIC_GUARDS_FOLDER_BACKED_SPLIT,
    plan_status_children::CHILD_MAP_SOURCE_RECONCILIATION,
    support_inventory::CLOSEOUT_GUARDS_FOLDER_BACKED_SPLIT,
    support_inventory::SUPPORT_HELPERS_FOLDER_BACKED_SPLIT,
    support_inventory::SUPPORT_INVENTORY_REVIEW_SYNC,
];
