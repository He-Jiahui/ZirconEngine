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

pub(super) const RECENT_STATIC_GUARDS_FOLDER_BACKED_SPLIT: Slice = (
    "Runtime 15 M3 plan-status recent static guards folder-backed split",
    &[
        "runtime_15_plan_status_recent_static_guards_folder_backed_static_passed_cargo_deferred",
        "plan_status/recent_static_guards.rs",
        "plan_status/recent_static_guards/document_sources.rs",
        "plan_status/recent_static_guards/runtime_01_to_04.rs",
        "plan_status/recent_static_guards/runtime_05_to_08.rs",
        "plan_status/recent_static_guards/runtime_09_to_12.rs",
        "plan_status/recent_static_guards/runtime_13_14_review_index.rs",
        "plan_status/recent_static_guards/split_layout.rs",
        "runtime_15_plan_status_recent_static_guards_are_folder_backed",
        "Cargo gate deferred",
    ],
);

pub(super) const CHILD_MAP_SOURCE_RECONCILIATION: Slice = (
    "Runtime 15 M3 plan-status child-map source reconciliation",
    &[
        "runtime_15_plan_status_child_map_source_reconciliation_static_passed_cargo_deferred",
        "plan_status/index_tables/subplan_map.rs",
        "plan_status/index_tables/split_layout/child_owner.rs",
        "plan_status/index_tables/split_layout/parent_guard.rs",
        "plan_status/index_tables/split_layout/split_guard.rs",
        "plan_status/index_tables/status_anchors/cargo_attempt.rs",
        "plan_status/index_tables/status_anchors/split_layout.rs",
        "plan_status/closeout/split_layout.rs",
        "plan_status/recent_static_guards/split_layout.rs",
        "docs/engine-architecture/runtime-architecture-review-m0.md",
        "plan_status_harness 48/48",
        "Cargo gate deferred",
    ],
);
