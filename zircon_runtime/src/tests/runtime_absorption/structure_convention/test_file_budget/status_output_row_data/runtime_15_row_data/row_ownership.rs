use super::*;

#[test]
fn runtime_15_status_output_runtime_15_row_data_is_child_owner() {
    let parent = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_foundation =
        read_runtime_src(RUNTIME_15_FOUNDATION_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m2 = read_runtime_src(RUNTIME_15_M2_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3 = read_runtime_src(RUNTIME_15_M3_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15_m3_status_support = read_runtime_src(RUNTIME_15_M3_STATUS_SUPPORT_ROW_DATA_PATH);
    let runtime_15_m4 = read_runtime_src(RUNTIME_15_M4_EXPECTED_STATUS_ROW_DATA_PATH);

    assert_contains_all(
        "status row data parent keeps only group aggregation",
        &parent,
        &[
            "#[path = \"expected_status_row_data/runtime_15.rs\"]",
            "mod runtime_15;",
            "pub(super) const EXPECTED_STATUS_OUTPUT_SLICE_GROUPS",
            "runtime_15::RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "runtime_15::RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    for moved_owner in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
    ] {
        assert!(
            !parent.contains(moved_owner),
            "expected_status_row_data.rs should delegate Runtime 15 row literals instead of keeping {moved_owner}"
        );
    }

    assert_contains_all(
        "Runtime 15 status row child owns Runtime 15 row groups",
        &runtime_15,
        &[
            "#[path = \"runtime_15/foundation.rs\"]",
            "mod foundation;",
            "#[path = \"runtime_15/m2.rs\"]",
            "mod m2;",
            "#[path = \"runtime_15/m3.rs\"]",
            "mod m3;",
            "#[path = \"runtime_15/m4.rs\"]",
            "mod m4;",
            "pub(super) const RUNTIME_15_FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "foundation::FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M2_EXPECTED_STATUS_OUTPUT_SLICES",
            "m2::EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M4_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const RUNTIME_15_M3_FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
            "Runtime 15 F12 offscreen target texture owner cleanup",
        ],
    );
    for moved_row in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 M3 graphics dead-code guard module split",
        ROW_DATA_SPLIT_STATUS_NAME,
        "Runtime 15 M3 status output expected-slice maps split",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
        "runtime_15_scene_world_project_io_mesh_is_child_owner",
    ] {
        assert!(
            !runtime_15.contains(moved_row),
            "expected_status_row_data/runtime_15.rs should delegate moved row literals instead of keeping {moved_row}"
        );
    }

    assert_contains_all(
        "Runtime 15 foundation row-data child delegates foundation row literals",
        &runtime_15_foundation,
        &[
            "#[path = \"foundation/core_rows.rs\"]",
            "mod core_rows;",
            "#[path = \"foundation/typed_error_runtime_rows.rs\"]",
            "mod typed_error_runtime_rows;",
            "#[path = \"foundation/typed_error_plugin_rows.rs\"]",
            "mod typed_error_plugin_rows;",
            "#[path = \"foundation/typed_error_scene_asset_rows.rs\"]",
            "mod typed_error_scene_asset_rows;",
            "pub(super) const FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const FOUNDATION_TYPED_ERROR_RUNTIME_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const FOUNDATION_TYPED_ERROR_PLUGIN_EXPECTED_STATUS_OUTPUT_SLICES",
            "pub(super) const FOUNDATION_TYPED_ERROR_SCENE_ASSET_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );

    assert_contains_all(
        "Runtime 15 M3 status support rows keep historical Runtime 15 row-data split",
        &runtime_15_m3_status_support,
        &[
            ROW_DATA_SPLIT_STATUS_NAME,
            ROW_DATA_SPLIT_STATUS_ID,
            "plan_status/status_output_tables/expected_status_row_data.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            ROW_DATA_SPLIT_GUARD_NAME,
        ],
    );

    for (label, source) in [
        (
            "plan_status/status_output_tables/expected_status_row_data.rs",
            parent.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
            runtime_15.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
            runtime_15_foundation.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m2.rs",
            runtime_15_m2.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
            runtime_15_m3.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
            runtime_15_m3_status_support.as_str(),
        ),
        (
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m4.rs",
            runtime_15_m4.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{label} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
