use super::super::support::assert_contains_all;

#[path = "split_layout/child_owner.rs"]
mod child_owner;
#[path = "split_layout/parent_guard.rs"]
mod parent_guard;
#[path = "split_layout/split_guard.rs"]
mod split_guard;

const TEST_ATTRIBUTE: &str = concat!("#[", "test", "]");

const STATUS: &str =
    "runtime_15_plan_status_index_tables_parent_guard_folder_backed_static_passed_cargo_deferred";
const FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_plan_status_index_tables_parent_guard_folder_backed_static_passed_cargo_deferred";
const SLICE: &str = "Runtime 15 M3 plan-status index-tables parent guard folder-backed split";
const GUARD: &str = "runtime_15_plan_status_index_tables_parent_guard_is_folder_backed";

const SPLIT_LAYOUT_STATUS: &str =
    "runtime_15_plan_status_index_tables_split_layout_folder_backed_static_passed_cargo_deferred";
const SPLIT_LAYOUT_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_plan_status_index_tables_split_layout_folder_backed_static_passed_cargo_deferred";
const SPLIT_LAYOUT_SLICE: &str =
    "Runtime 15 M3 plan-status index-tables split-layout guard folder-backed split";
const SPLIT_LAYOUT_GUARD: &str =
    "runtime_15_plan_status_index_tables_split_layout_is_folder_backed";

const PARENT_PATH: &str = "plan_status/index_tables.rs";
const CHILD_PATHS: &[&str] = &[
    "plan_status/index_tables/index_consistency.rs",
    "plan_status/index_tables/split_layout.rs",
    "plan_status/index_tables/status_anchors.rs",
    "plan_status/index_tables/subplan_map.rs",
];
const SPLIT_LAYOUT_CHILD_PATHS: &[&str] = &[
    "plan_status/index_tables/split_layout/child_owner.rs",
    "plan_status/index_tables/split_layout/parent_guard.rs",
    "plan_status/index_tables/split_layout/split_guard.rs",
];
