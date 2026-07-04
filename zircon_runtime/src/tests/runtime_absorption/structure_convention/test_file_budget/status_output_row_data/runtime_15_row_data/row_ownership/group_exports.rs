use super::*;

#[test]
fn runtime_15_row_data_group_exports_are_child_owned() {
    let parent = read_runtime_src(TOP_LEVEL_EXPECTED_STATUS_ROW_DATA_PATH);
    let runtime_15 = read_runtime_src(RUNTIME_15_EXPECTED_STATUS_ROW_DATA_PATH);
    let f12_resource = read_runtime_src(RUNTIME_15_F12_RESOURCE_EXPECTED_STATUS_ROW_DATA_PATH);

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
            "#[path = \"runtime_15/f12_resource.rs\"]",
            "mod f12_resource;",
            "f12_resource::EXPECTED_STATUS_OUTPUT_SLICES",
            "#[path = \"runtime_15/foundation.rs\"]",
            "mod foundation;",
            "#[path = \"runtime_15/m2.rs\"]",
            "mod m2;",
            "#[path = \"runtime_15/m3.rs\"]",
            "mod m3;",
            "#[path = \"runtime_15/m4.rs\"]",
            "mod m4;",
            "foundation::FOUNDATION_EXPECTED_STATUS_OUTPUT_SLICES",
            "m2::EXPECTED_STATUS_OUTPUT_SLICES",
            "m3::FOUNDATION_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
        ],
    );
    assert_contains_all(
        "Runtime 15 F12 child owns F12 row literals",
        &f12_resource,
        &["Runtime 15 F12 offscreen target texture owner cleanup"],
    );

    for moved_row in [
        "Runtime 15 F9 runtime prelude required type coverage",
        "Runtime 15 M3 graphics dead-code guard module split",
        ROW_DATA_SPLIT_STATUS_NAME,
        "Runtime 15 M3 status output expected-slice maps split",
        "Runtime 15 M4 scene world project I/O mesh owner split",
        "runtime_15_scene_world_project_io_mesh_owner_split_static_passed_cargo_timeout_no_result",
        "runtime_15_scene_world_project_io_mesh_is_child_owner",
        "Runtime 15 F12 offscreen target texture owner cleanup",
    ] {
        assert!(
            !runtime_15.contains(moved_row),
            "expected_status_row_data/runtime_15.rs should delegate moved row literals instead of keeping {moved_row}"
        );
    }
}
