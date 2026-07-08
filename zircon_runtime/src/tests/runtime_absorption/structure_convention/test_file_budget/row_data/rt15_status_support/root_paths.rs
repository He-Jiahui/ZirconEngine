#[path = "root_paths/review_guard_foundation.rs"]
mod review_guard_foundation;
#[path = "root_paths/structure_support.rs"]
mod structure_support;
#[path = "root_paths/top_level_support.rs"]
mod top_level_support;

pub(super) use review_guard_foundation::*;
pub(super) use structure_support::{
    EXPECTED_SLICE_STRUCTURE_SUPPORT_CHILDREN, EXPECTED_SLICE_STRUCTURE_SUPPORT_NESTED_CHILDREN,
};
pub(super) use top_level_support::EXPECTED_SLICE_TOP_LEVEL_SUPPORT_CHILDREN;

pub(super) const STATUS_OUTPUT_ROW_DATA_PARENT_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/status_output_row_data.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_status_support_row_data.rs";
pub(super) const ROOT_PATHS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/root_paths.rs";
pub(super) const ROOT_STATUSES_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/root_statuses.rs";
pub(super) const ROOT_CHILD_ROWS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/root_child_rows.rs";
pub(super) const ROOT_GUARD_CHILDREN_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/root_child_rows/guard_children.rs";
pub(super) const ROOT_OWNER_PATHS_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/root_owner_paths.rs";
pub(super) const ROOT_INVENTORY_GUARD_PATH: &str =
    "tests/runtime_absorption/structure_convention/test_file_budget/row_data/rt15_status_support/root_inventory.rs";
pub(super) const TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data.rs";
pub(super) const RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15.rs";
pub(super) const RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs";
pub(super) const STATUS_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs";
pub(super) const STATUS_SUPPORT_ANCHOR_MIRROR_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/anchor_mirror.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_AND_BUDGET_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_TEST_FILE_BUDGET_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/test_file_budget.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_RUNTIME_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/runtime_row_data.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_ANCHOR_MIRROR_ROW_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/anchor_mirror_row.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_HUB_EDITOR_SUPPORT_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/hub_editor_support.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_RENDER_SHADER_SUPPORT_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/render_shader_support.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_M3_M4_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget/m3_m4_row_data.rs";
pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_MAPS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs";
pub(super) const EXPECTED_SLICE_BASE_MAPS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/base_maps.rs";
pub(super) const EXPECTED_SLICE_TOP_LEVEL_SUPPORT_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/top_level_support.rs";
pub(super) const EXPECTED_SLICE_ROUTE_METADATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_metadata.rs";
pub(super) const EXPECTED_SLICE_STRUCTURE_SUPPORT_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support.rs";
pub(super) const EXPECTED_SLICE_STATUS_SUPPORT_MAPS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps.rs";
pub(super) const EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure.rs";
pub(super) const EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/structure_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/typed_error_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/root_route_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/guard_body_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/source_inventory_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/row_data_owner_rows.rs",
];
pub(super) const EXPECTED_SLICE_WARNING_CLEANUP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/warning_cleanup.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_INDEX_ANCHORS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_INDEX_ANCHOR_BASELINE_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/index_baseline.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_STATUS_ANCHORS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/runtime_status_anchors.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_CARGO_ATTEMPT_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/cargo_attempt.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_PLAN_STATUS_CHILDREN_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/plan_status_children.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_SUPPORT_INVENTORY_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors/support_inventory.rs";
pub(super) const STATUS_SUPPORT_PRIORITY_PLAN_DOCS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs";
pub(super) const PRIORITY_PLAN_DOCS_INTEGRITY_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs";
pub(super) const PRIORITY_PLAN_DOCS_OWNER_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs";
pub(super) const PRIORITY_PLAN_DOCS_FOLLOWUPS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs";
pub(super) const PRIORITY_PLAN_DOCS_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs";
pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/row_data_maps/root_runtime_maps.rs";
pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/row_data_maps/root_runtime_maps.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_INDEX_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs";
pub(super) const STATUS_SUPPORT_RUNTIME_INDEX_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/runtime_index_anchor_maps.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/status_row_data_support_maps.rs";
pub(super) const STATUS_SUPPORT_ROW_DATA_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/status_row_data_support_maps.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_RUNTIME_ROW_DATA_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data.rs";
pub(super) const PRODUCTION_GUARD_SUPPORT_STATUS_SUPPORT_PRIORITY_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/runtime_row_data/status_support_priority_rows.rs";
