type Slice = super::Slice;

pub(super) const INDEX_TABLES_CHILD_OWNER_SPLIT: Slice = (
    "Runtime 15 M3 plan-status index-tables child-owner split",
    &[
        "runtime_15_plan_status_index_tables_child_owner_split_static_passed_cargo_deferred",
        "plan_status/index_tables.rs",
        "plan_status/index_tables/subplan_map.rs",
        "plan_status/index_tables/status_anchors.rs",
        "plan_status/index_tables/index_consistency.rs",
        "runtime_15_plan_status_index_tables_guard_child_owner_split",
    ],
);

pub(super) const INDEX_TABLES_PARENT_GUARD_FOLDER_BACKED_SPLIT: Slice = (
    "Runtime 15 M3 plan-status index-tables parent guard folder-backed split",
    &[
        "runtime_15_plan_status_index_tables_parent_guard_folder_backed_static_passed_cargo_deferred",
        "plan_status/index_tables.rs",
        "plan_status/index_tables/index_consistency.rs",
        "plan_status/index_tables/split_layout.rs",
        "plan_status/index_tables/status_anchors.rs",
        "plan_status/index_tables/subplan_map.rs",
        "runtime_15_plan_status_index_tables_parent_guard_is_folder_backed",
        "Cargo gate deferred",
    ],
);

pub(super) const INDEX_TABLES_SPLIT_LAYOUT_FOLDER_BACKED_SPLIT: Slice = (
    "Runtime 15 M3 plan-status index-tables split-layout guard folder-backed split",
    &[
        "runtime_15_plan_status_index_tables_split_layout_folder_backed_static_passed_cargo_deferred",
        "plan_status/index_tables.rs",
        "plan_status/index_tables/split_layout.rs",
        "plan_status/index_tables/split_layout/child_owner.rs",
        "plan_status/index_tables/split_layout/parent_guard.rs",
        "plan_status/index_tables/split_layout/split_guard.rs",
        "runtime_15_plan_status_index_tables_split_layout_is_folder_backed",
        "Cargo gate deferred",
    ],
);

pub(super) const INDEX_STATUS_ANCHORS_FOLDER_BACKED_SPLIT: Slice = (
    "Runtime 15 M3 plan-status index status-anchors folder-backed split",
    &[
        "runtime_15_plan_status_index_status_anchors_folder_backed_static_passed_cargo_deferred",
        "plan_status/index_tables/status_anchors.rs",
        "plan_status/index_tables/status_anchors/runtime03_module_doc.rs",
        "plan_status/index_tables/status_anchors/runtime07_scene_asset.rs",
        "plan_status/index_tables/status_anchors/runtime07_owner_budget.rs",
        "plan_status/index_tables/status_anchors/generated_status.rs",
        "plan_status/index_tables/status_anchors/runtime10_behavior.rs",
        "plan_status/index_tables/status_anchors/cargo_attempt.rs",
        "plan_status/index_tables/status_anchors/split_layout.rs",
        "runtime_15_plan_status_index_status_anchors_are_folder_backed",
        "Cargo gate deferred",
    ],
);

pub(super) use recent_and_reconciliation::{
    CHILD_MAP_SOURCE_RECONCILIATION, RECENT_STATIC_GUARDS_FOLDER_BACKED_SPLIT,
};
#[path = "plan_status_children/recent_and_reconciliation.rs"]
mod recent_and_reconciliation;
