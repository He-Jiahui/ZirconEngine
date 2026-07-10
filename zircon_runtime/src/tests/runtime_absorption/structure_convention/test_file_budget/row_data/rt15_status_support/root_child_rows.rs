use super::*;

#[path = "root_child_rows/guard_children.rs"]
mod guard_children;

pub(super) use guard_children::STATUS_SUPPORT_ROW_DATA_GUARD_CHILDREN;

pub(super) const EXPECTED_SLICE_MAP_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "base_maps",
        EXPECTED_SLICE_BASE_MAPS_PATH,
        "Runtime 15 M3 status output expected-slice maps split",
    ),
    (
        "top_level_support",
        EXPECTED_SLICE_TOP_LEVEL_SUPPORT_PATH,
        "Runtime 15 M3 status output expected-slice top-level map support child-owner split",
    ),
    (
        "route_metadata",
        EXPECTED_SLICE_ROUTE_METADATA_PATH,
        "Runtime 15 M3 naming-boundary expected-slice guard body folder-backed split",
    ),
    (
        "structure_support",
        EXPECTED_SLICE_STRUCTURE_SUPPORT_PATH,
        "Runtime 15 M3 structure-support expected-slice map child-owner split",
    ),
    (
        "status_support_maps",
        EXPECTED_SLICE_STATUS_SUPPORT_MAPS_PATH,
        "Runtime 15 M3 status-support expected-slice map child split",
    ),
    (
        "review_guard_structure",
        EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_PATH,
        "Runtime 15 M3 review-guard expected-slice structure guard child-module split",
    ),
    (
        "warning_cleanup",
        EXPECTED_SLICE_WARNING_CLEANUP_PATH,
        "Runtime 15 M3 structure-convention warning cleanup",
    ),
];

pub(super) const EXPECTED_SLICE_ROUTE_METADATA_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_metadata/naming_boundary_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_metadata/child_owner_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_metadata/child_owner_budget_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/route_metadata/row_data_owner_rows.rs",
];

pub(super) const EXPECTED_SLICE_STATUS_SUPPORT_MAPS_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_metadata_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/guard_body_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/runtime_index_anchor_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/expected_slice_route_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/route_input_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/route_guard_rows/row_data_owner.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/expected_slice_map_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/status_support_maps/row_data_owner_rows.rs",
];

pub(super) const EXPECTED_SLICE_REVIEW_GUARD_STRUCTURE_NESTED_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/typed_error_rows/route_metadata_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/typed_error_rows/guard_body_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/root_route_rows/route_metadata_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/root_route_rows/route_mount_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/root_route_rows/status_mirror_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/source_inventory_rows/root_source_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/source_inventory_rows/route_source_rows.rs",
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/source_inventory_rows/structure_path_rows.rs",
];

pub(super) const ROW_DATA_AND_BUDGET_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "test_file_budget",
        STATUS_SUPPORT_ROW_DATA_TEST_FILE_BUDGET_PATH,
        "Runtime 15 M3 test file budget root-layout child split",
    ),
    (
        "runtime_row_data",
        STATUS_SUPPORT_ROW_DATA_RUNTIME_ROW_DATA_PATH,
        "Runtime 15 M3 status output Runtime 15 row data split",
    ),
    (
        "hub_editor_support",
        STATUS_SUPPORT_ROW_DATA_HUB_EDITOR_SUPPORT_PATH,
        "Runtime 15 M3 support Hub project-actions tests child-owner split",
    ),
    (
        "render_shader_support",
        STATUS_SUPPORT_ROW_DATA_RENDER_SHADER_SUPPORT_PATH,
        "Runtime 15 M3 render shader template assembly guard support child-owner split",
    ),
    (
        "m3_m4_row_data",
        STATUS_SUPPORT_ROW_DATA_M3_M4_ROW_DATA_PATH,
        "Runtime 15 M3 Runtime 15 M4 row-data guard folder-backed split",
    ),
];

pub(super) const RUNTIME_INDEX_ANCHOR_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "index_baseline",
        STATUS_SUPPORT_RUNTIME_INDEX_ANCHOR_BASELINE_PATH,
        "Runtime 15 M3 runtime index subplan map 01-15 sync",
    ),
    (
        "runtime_status_anchors",
        STATUS_SUPPORT_RUNTIME_STATUS_ANCHORS_PATH,
        "Runtime 15 M3 Runtime 07 scene asset status anchor sync",
    ),
    (
        "cargo_attempt",
        STATUS_SUPPORT_RUNTIME_CARGO_ATTEMPT_PATH,
        "Runtime 15 M3 Runtime Cargo attempt status anchor sync",
    ),
    (
        "plan_status_children",
        STATUS_SUPPORT_RUNTIME_PLAN_STATUS_CHILDREN_PATH,
        "Runtime 15 M3 plan-status index-tables child-owner split",
    ),
    (
        "support_inventory",
        STATUS_SUPPORT_RUNTIME_SUPPORT_INVENTORY_PATH,
        "Runtime 15 M3 plan-status support inventory review sync",
    ),
];
